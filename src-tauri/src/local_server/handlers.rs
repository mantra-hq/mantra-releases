//! HTTP 路由处理器
//!
//! 实现 /api/privacy/check 等 API 端点
//! Story 3.12: 支持 PreToolUse Hook 格式

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

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

/// 需要检测的网络命令前缀
/// Story 3.12 AC3: Bash 命令过滤
const NETWORK_COMMANDS: &[&str] = &[
    "curl", "wget", "http", "httpie",
    "git push", "git remote",
    "ssh", "scp", "rsync",
    "docker push", "docker login",
    "npm publish", "pnpm publish", "yarn publish",
    "gh api", "gh pr", "gh issue",
];

/// 检查 Bash 命令是否为网络相关命令
/// Story 3.12 AC3
fn is_network_command(command: &str) -> bool {
    let trimmed = command.trim();
    NETWORK_COMMANDS.iter().any(|prefix| {
        trimmed.starts_with(prefix) ||
        trimmed.contains(&format!(" {} ", prefix)) ||
        trimmed.contains(&format!("&& {}", prefix)) ||
        trimmed.contains(&format!("; {}", prefix))
    })
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
