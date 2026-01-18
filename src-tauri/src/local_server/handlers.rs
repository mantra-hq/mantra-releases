//! HTTP 路由处理器
//!
//! 实现 /api/privacy/check 等 API 端点

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
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

/// 隐私检查请求
#[derive(Debug, Clone, Deserialize)]
pub struct PrivacyCheckRequest {
    /// 待检查的 prompt 内容
    pub prompt: String,
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

/// POST /api/privacy/check
///
/// 检查 prompt 中是否包含敏感信息
pub async fn privacy_check(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PrivacyCheckRequest>,
) -> impl IntoResponse {
    // 使用 PrivacyScanner 扫描文本
    let result = state.scanner.scan(&request.prompt);

    if !result.matches.is_empty() {
        // 有敏感信息，返回 block
        let matches: Vec<MatchInfo> = result.matches.iter().map(MatchInfo::from).collect();
        let count = matches.len();
        let message = format!("检测到 {} 处敏感信息", count);

        // Story 3.11 Task 3: 记录拦截事件到数据库
        if let Some(db) = &state.db {
            // 确定来源类型
            let tool_name = request.context
                .as_ref()
                .and_then(|c| c.tool.clone())
                .unwrap_or_else(|| "unknown".to_string());
            
            let source = if tool_name == "claude-code" {
                InterceptionSource::ClaudeCodeHook { session_id: None }
            } else {
                InterceptionSource::ExternalHook { tool_name }
            };

            // 创建拦截记录
            let record = InterceptionRecord::new(
                source,
                result.matches.clone(),
                UserAction::Cancelled, // Hook 拦截时用户操作为取消
                compute_hash(&request.prompt),
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
