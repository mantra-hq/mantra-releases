//! HTTP 路由处理器
//!
//! 实现 /api/privacy/check 等 API 端点
//! Story 3.11: 新增 /api/privacy/check-files 端点，用于 PreToolUse 文件内容检测
//! Story 3.12: 支持 PreToolUse Hook 格式

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::path::Path;

use crate::sanitizer::{InterceptionRecord, InterceptionSource, PrivacyScanner, ScanMatch, Severity, UserAction};
use crate::storage::Database;

/// 共享状态
pub struct AppState {
    pub scanner: PrivacyScanner,
    #[allow(dead_code)] // 保留用于后续功能扩展
    pub config_dir: std::path::PathBuf,
    pub db: Option<Arc<std::sync::Mutex<Database>>>,
}

/// 隐私检查请求 - 支持两种格式
///
/// 1. UserPromptSubmit 格式 (旧):
/// ```json
/// { "prompt": "...", "context": {...} }
/// ```
///
/// 2. PreToolUse 格式 (新):
/// ```json
/// { "hook_event": "PreToolUse", "tool_name": "...", "tool_input": {...}, "context": {...} }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct PrivacyCheckRequest {
    /// 待检查的 prompt 内容 (UserPromptSubmit 格式)
    pub prompt: Option<String>,
    /// Hook 事件类型 (PreToolUse 格式)
    pub hook_event: Option<String>,
    /// 工具名称 (PreToolUse 格式)
    pub tool_name: Option<String>,
    /// 工具输入参数 (PreToolUse 格式)
    pub tool_input: Option<Value>,
    /// 可选的上下文信息
    #[serde(default)]
    pub context: Option<PrivacyCheckContext>,
}

/// 上下文信息
#[derive(Debug, Clone, Deserialize)]
pub struct PrivacyCheckContext {
    /// 工具名称 (如 "claude-code")
    pub tool: Option<String>,
    /// 时间戳
    #[allow(dead_code)] // API 预留字段
    pub timestamp: Option<String>,
}

/// 隐私检查响应
#[derive(Debug, Clone, Serialize)]
pub struct PrivacyCheckResponse {
    /// 动作：allow 或 block
    pub action: String,
    /// 匹配的敏感信息（仅当 action=block 时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<MatchInfo>>,
    /// 提示消息（仅当 action=block 时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// 匹配信息
#[derive(Debug, Clone, Serialize)]
pub struct MatchInfo {
    /// 规则 ID
    pub rule_id: String,
    /// 严重程度
    pub severity: String,
    /// 预览（脱敏后的内容）
    pub preview: String,
}

impl From<&ScanMatch> for MatchInfo {
    fn from(m: &ScanMatch) -> Self {
        // 生成脱敏预览：显示前4个和后4个字符
        let preview = if m.matched_text.len() <= 8 {
            "*".repeat(m.matched_text.len())
        } else {
            let start = &m.matched_text[..4];
            let end = &m.matched_text[m.matched_text.len()-4..];
            format!("{}****{}", start, end)
        };

        Self {
            rule_id: m.rule_id.clone(),
            severity: match m.severity {
                Severity::Critical => "critical".to_string(),
                Severity::Warning => "warning".to_string(),
                Severity::Info => "info".to_string(),
            },
            preview,
        }
    }
}

/// 网络命令正则表达式模式
/// Story 3.12 AC3: Bash 命令过滤
///
/// ⚠️ 同步维护提醒：此列表需要与 TypeScript 端保持一致
/// 参见: packages/privacy-hook/src/hook-handler.ts - NETWORK_COMMAND_PATTERNS
static NETWORK_COMMAND_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // HTTP 客户端
        Regex::new(r"^curl\b").unwrap(),
        Regex::new(r"^wget\b").unwrap(),
        Regex::new(r"^http\b").unwrap(),
        Regex::new(r"^httpie\b").unwrap(),
        // Git 网络操作
        Regex::new(r"^git\s+push\b").unwrap(),
        Regex::new(r"^git\s+remote\b").unwrap(),
        // SSH 相关
        Regex::new(r"^ssh\b").unwrap(),
        Regex::new(r"^scp\b").unwrap(),
        Regex::new(r"^rsync\b").unwrap(),
        // Docker 网络操作
        Regex::new(r"^docker\s+push\b").unwrap(),
        Regex::new(r"^docker\s+login\b").unwrap(),
        // 包发布
        Regex::new(r"^npm\s+publish\b").unwrap(),
        Regex::new(r"^pnpm\s+publish\b").unwrap(),
        Regex::new(r"^yarn\s+publish\b").unwrap(),
        // GitHub CLI
        Regex::new(r"^gh\s+api\b").unwrap(),
        Regex::new(r"^gh\s+pr\b").unwrap(),
        Regex::new(r"^gh\s+issue\b").unwrap(),
    ]
});

/// 检查 Bash 命令是否为网络相关命令
/// Story 3.12 AC3
///
/// 使用正则表达式精确匹配命令开头，与 TypeScript 端实现保持一致
fn is_network_command(command: &str) -> bool {
    let trimmed = command.trim();
    NETWORK_COMMAND_PATTERNS.iter().any(|pattern| pattern.is_match(trimmed))
}

/// 从 PreToolUse 工具调用中提取检测目标
/// Story 3.12 AC2, AC4
///
/// 根据工具类型提取需要进行隐私扫描的内容:
/// - WebFetch: url + prompt
/// - WebSearch: query
/// - Bash: command (仅网络命令)
/// - Task: prompt
/// - mcp__*: 所有字符串值
fn extract_check_targets_for_tool(tool_name: &str, tool_input: &Value) -> Vec<String> {
    let mut targets = Vec::new();

    match tool_name {
        "WebFetch" => {
            if let Some(url) = tool_input.get("url").and_then(|v| v.as_str()) {
                targets.push(url.to_string());
            }
            if let Some(prompt) = tool_input.get("prompt").and_then(|v| v.as_str()) {
                targets.push(prompt.to_string());
            }
        }
        "WebSearch" => {
            if let Some(query) = tool_input.get("query").and_then(|v| v.as_str()) {
                targets.push(query.to_string());
            }
        }
        "Bash" => {
            if let Some(command) = tool_input.get("command").and_then(|v| v.as_str()) {
                // 仅检测网络相关命令
                if is_network_command(command) {
                    targets.push(command.to_string());
                }
            }
        }
        "Task" => {
            if let Some(prompt) = tool_input.get("prompt").and_then(|v| v.as_str()) {
                targets.push(prompt.to_string());
            }
        }
        // MCP 工具: mcp__* 开头
        _ if tool_name.starts_with("mcp__") => {
            // 提取所有字符串值
            extract_all_string_values(tool_input, &mut targets);
        }
        _ => {
            // 其他工具暂不检测
        }
    }

    targets
}

/// 递归提取 JSON 中的所有字符串值
fn extract_all_string_values(value: &Value, targets: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            if !s.is_empty() {
                targets.push(s.clone());
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                extract_all_string_values(v, targets);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                extract_all_string_values(v, targets);
            }
        }
        _ => {}
    }
}

/// POST /api/privacy/check
///
/// 检查 prompt 中是否包含敏感信息
/// Story 3.12: 支持 PreToolUse 和 UserPromptSubmit 两种格式
pub async fn privacy_check(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PrivacyCheckRequest>,
) -> impl IntoResponse {
    // 根据请求格式提取检测目标
    let (check_targets, hook_event, tool_name_for_log) = if request.hook_event.as_deref() == Some("PreToolUse") {
        // PreToolUse 格式
        let tool_name = request.tool_name.as_deref().unwrap_or("unknown");
        let targets = if let Some(ref tool_input) = request.tool_input {
            extract_check_targets_for_tool(tool_name, tool_input)
        } else {
            Vec::new()
        };
        (targets, "PreToolUse", tool_name.to_string())
    } else {
        // UserPromptSubmit 格式 (默认)
        let targets = request.prompt.as_ref().map_or_else(Vec::new, |p| vec![p.clone()]);
        let tool_name = request.context.as_ref()
            .and_then(|c| c.tool.clone())
            .unwrap_or_else(|| "unknown".to_string());
        (targets, "UserPromptSubmit", tool_name)
    };

    // 如果没有检测目标（如非网络 Bash 命令），直接放行
    if check_targets.is_empty() {
        return (StatusCode::OK, Json(PrivacyCheckResponse {
            action: "allow".to_string(),
            matches: None,
            message: None,
        }));
    }

    // 合并所有检测目标进行扫描
    let combined_text = check_targets.join("\n");
    let result = state.scanner.scan(&combined_text);

    if !result.matches.is_empty() {
        // 有敏感信息，返回 block
        let matches: Vec<MatchInfo> = result.matches.iter().map(MatchInfo::from).collect();
        let count = matches.len();
        let message = format!("检测到 {} 处敏感信息 ({})", count, hook_event);

        // Story 3.11 Task 3: 记录拦截事件到数据库
        if let Some(db) = &state.db {
            let source = if tool_name_for_log == "claude-code" {
                InterceptionSource::ClaudeCodeHook { session_id: None }
            } else {
                InterceptionSource::ExternalHook { tool_name: tool_name_for_log.clone() }
            };

            // 创建拦截记录
            let record = InterceptionRecord::new(
                source,
                result.matches.clone(),
                UserAction::Cancelled, // Hook 拦截时用户操作为取消
                compute_hash(&combined_text),
                None, // Hook 无法获取项目名
            );

            // 保存到数据库（忽略错误，不影响响应）
            if let Ok(db_guard) = db.lock() {
                if let Err(e) = db_guard.save_interception_record(&record) {
                    eprintln!("[Mantra] Failed to save interception record: {}", e);
                }
            }
        }

        let response = PrivacyCheckResponse {
            action: "block".to_string(),
            matches: Some(matches),
            message: Some(message),
        };

        (StatusCode::OK, Json(response))
    } else {
        // 无敏感信息，返回 allow
        let response = PrivacyCheckResponse {
            action: "allow".to_string(),
            matches: None,
            message: None,
        };

        (StatusCode::OK, Json(response))
    }
}

/// 计算文本哈希（用于去重）
fn compute_hash(text: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

// ============================================================================
// Story 3.11: /api/privacy/check-files 端点
// PreToolUse Hook 文件内容检测 - "抢先一步"策略
// ============================================================================

/// 文件检查请求 (Story 3.11 AC5)
#[derive(Debug, Clone, Deserialize)]
pub struct CheckFilesRequest {
    /// 要检测的文件路径列表
    pub file_paths: Vec<String>,
    /// 触发的工具名 (Read/Grep/Bash/Edit)
    pub tool_name: String,
    /// 可选的上下文信息
    #[serde(default)]
    #[allow(dead_code)] // API 预留字段
    pub context: Option<CheckFilesContext>,
}

/// 文件检查上下文
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)] // API 预留字段，用于后续功能扩展
pub struct CheckFilesContext {
    /// 会话 ID
    pub session_id: Option<String>,
    /// 当前工作目录
    pub cwd: Option<String>,
}

/// 文件检查响应 (Story 3.11 AC9)
#[derive(Debug, Clone, Serialize)]
pub struct CheckFilesResponse {
    /// 动作：allow 或 block
    pub action: String,
    /// 检测到的敏感信息列表
    pub findings: Vec<Finding>,
    /// 提示消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// 单个敏感信息发现 (Story 3.11 AC9)
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// 文件路径
    pub file_path: String,
    /// 行号 (1-based)
    pub line_number: usize,
    /// 规则 ID
    pub rule_id: String,
    /// 脱敏后的匹配预览
    pub preview: String,
    /// 严重程度
    pub severity: String,
}

impl Finding {
    /// 从 ScanMatch 和文件路径创建 Finding
    fn from_scan_match(scan_match: &ScanMatch, file_path: &str) -> Self {
        // 生成脱敏预览：显示前4个和后4个字符（与 MatchInfo 格式保持一致）
        let preview = if scan_match.matched_text.len() <= 8 {
            "*".repeat(scan_match.matched_text.len())
        } else {
            let start = &scan_match.matched_text[..4];
            let end = &scan_match.matched_text[scan_match.matched_text.len()-4..];
            format!("{}****{}", start, end)  // 统一使用 4 个星号
        };

        Self {
            file_path: file_path.to_string(),
            line_number: scan_match.line,
            rule_id: scan_match.rule_id.clone(),
            preview,
            severity: match scan_match.severity {
                Severity::Critical => "critical".to_string(),
                Severity::Warning => "warning".to_string(),
                Severity::Info => "info".to_string(),
            },
        }
    }
}

/// 单个文件最大大小限制 (10MB)
/// Story 3.11 Task A2.3: 防止读取超大文件
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// 文件读取错误类型
#[derive(Debug)]
enum FileReadError {
    NotFound,
    TooLarge(u64),
    PermissionDenied,
    IsDirectory,
    Other(String),
}

/// 安全读取文件内容
/// Story 3.11 Task A2: 文件读取与安全检查
fn read_file_safe(path: &Path) -> Result<String, FileReadError> {
    use std::fs;
    use std::io::Read;
    
    // 检查文件是否存在
    if !path.exists() {
        return Err(FileReadError::NotFound);
    }
    
    // 检查是否为目录
    if path.is_dir() {
        return Err(FileReadError::IsDirectory);
    }
    
    // 获取文件元数据
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(FileReadError::PermissionDenied);
            }
            return Err(FileReadError::Other(e.to_string()));
        }
    };
    
    // 检查文件大小
    let file_size = metadata.len();
    if file_size > MAX_FILE_SIZE {
        return Err(FileReadError::TooLarge(file_size));
    }
    
    // 读取文件内容
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(FileReadError::PermissionDenied);
            }
            return Err(FileReadError::Other(e.to_string()));
        }
    };
    
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => Ok(content),
        Err(e) => {
            // 可能是二进制文件，跳过
            if e.kind() == std::io::ErrorKind::InvalidData {
                // 返回空字符串，让扫描器正常处理
                Ok(String::new())
            } else {
                Err(FileReadError::Other(e.to_string()))
            }
        }
    }
}

/// POST /api/privacy/check-files
///
/// Story 3.11: PreToolUse Hook 文件内容检测
/// 在 AI 助理读取文件前，自动检测目标文件内容中的敏感数据
pub async fn check_files(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CheckFilesRequest>,
) -> impl IntoResponse {
    // 如果没有文件路径，直接放行
    if request.file_paths.is_empty() {
        return (StatusCode::OK, Json(CheckFilesResponse {
            action: "allow".to_string(),
            findings: vec![],
            message: None,
        }));
    }
    
    let mut all_findings: Vec<Finding> = Vec::new();
    
    // 逐个文件读取并检测
    for file_path_str in &request.file_paths {
        let file_path = Path::new(file_path_str);
        
        // 安全读取文件
        match read_file_safe(file_path) {
            Ok(content) => {
                if content.is_empty() {
                    continue;
                }
                
                // 使用扫描器检测敏感数据
                let scan_result = state.scanner.scan(&content);
                
                // 转换为 Finding
                for scan_match in &scan_result.matches {
                    all_findings.push(Finding::from_scan_match(scan_match, file_path_str));
                }
            }
            Err(e) => {
                // 记录跳过原因，但不阻止操作
                match e {
                    FileReadError::NotFound => {
                        eprintln!("[Mantra] File not found, skipping: {}", file_path_str);
                    }
                    FileReadError::TooLarge(size) => {
                        eprintln!("[Mantra] File too large ({} bytes), skipping: {}", size, file_path_str);
                    }
                    FileReadError::PermissionDenied => {
                        eprintln!("[Mantra] Permission denied, skipping: {}", file_path_str);
                    }
                    FileReadError::IsDirectory => {
                        eprintln!("[Mantra] Path is directory, skipping: {}", file_path_str);
                    }
                    FileReadError::Other(msg) => {
                        eprintln!("[Mantra] Error reading file {}: {}", file_path_str, msg);
                    }
                }
            }
        }
    }
    
    // 判断是否需要阻止
    if all_findings.is_empty() {
        return (StatusCode::OK, Json(CheckFilesResponse {
            action: "allow".to_string(),
            findings: vec![],
            message: None,
        }));
    }
    
    // 有敏感数据，需要阻止
    let critical_count = all_findings.iter().filter(|f| f.severity == "critical").count();
    let warning_count = all_findings.iter().filter(|f| f.severity == "warning").count();
    let total_count = all_findings.len();
    
    let message = format!(
        "检测到 {} 处敏感数据 ({} Critical, {} Warning)",
        total_count, critical_count, warning_count
    );
    
    // Story 3.11 Task A3: 记录拦截事件到数据库
    if let Some(db) = &state.db {
        // 创建 ScanMatch 列表用于记录
        // 注意: 由于 Finding 结构只保留脱敏后的 preview，无法获取原始 matched_text
        // 因此 matched_text 和 masked_text 都使用 preview 值
        // 这是设计权衡：保护隐私 vs 完整记录
        let scan_matches: Vec<ScanMatch> = all_findings.iter().map(|f| {
            ScanMatch {
                rule_id: f.rule_id.clone(),
                sensitive_type: crate::sanitizer::SensitiveType::ApiKey, // 简化处理
                severity: match f.severity.as_str() {
                    "critical" => Severity::Critical,
                    "warning" => Severity::Warning,
                    _ => Severity::Info,
                },
                line: f.line_number,
                column: 1,
                matched_text: format!("[REDACTED:{}]", f.rule_id), // 不记录原始敏感数据
                masked_text: f.preview.clone(),
                context: format!("File: {}", f.file_path),
            }
        }).collect();
        
        // 确定来源类型
        let source = InterceptionSource::PreToolUseFileCheck {
            tool_name: request.tool_name.clone(),
            file_paths: request.file_paths.clone(),
        };
        
        // 创建拦截记录
        let record = InterceptionRecord::new(
            source,
            scan_matches,
            UserAction::Cancelled,
            compute_hash(&request.file_paths.join(",")),
            None,
        );
        
        // 保存到数据库
        if let Ok(db_guard) = db.lock() {
            if let Err(e) = db_guard.save_interception_record(&record) {
                eprintln!("[Mantra] Failed to save interception record: {}", e);
            }
        }
    }
    
    (StatusCode::OK, Json(CheckFilesResponse {
        action: "block".to_string(),
        findings: all_findings,
        message: Some(message),
    }))
}

/// GET /api/health
///
/// 健康检查端点，用于检测客户端是否在运行
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ok",
        "service": "mantra-client"
    })))
}

/// 处理无效 JSON 请求
#[allow(dead_code)] // 预留用于自定义错误处理
pub async fn handle_json_error(
    err: axum::extract::rejection::JsonRejection,
) -> impl IntoResponse {
    let message = match err {
        axum::extract::rejection::JsonRejection::JsonDataError(_) => {
            "Invalid JSON data"
        }
        axum::extract::rejection::JsonRejection::JsonSyntaxError(_) => {
            "Invalid JSON syntax"
        }
        axum::extract::rejection::JsonRejection::MissingJsonContentType(_) => {
            "Missing Content-Type: application/json header"
        }
        _ => "JSON parsing error",
    };

    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": message
        })),
    )
}
