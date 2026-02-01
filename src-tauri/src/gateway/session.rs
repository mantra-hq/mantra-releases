//! MCP Session 管理模块
//!
//! Story 11.14: MCP Streamable HTTP 规范合规 - Task 2
//!
//! 实现 MCP Streamable HTTP 规范要求的 MCP-Session-Id Header 会话管理。
//!
//! ## 规范要求
//! - 服务端可在初始化响应中返回 `MCP-Session-Id` Header
//! - 客户端后续请求必须包含此 Header
//! - Session ID 必须是可见 ASCII 字符 (0x21-0x7E)
//! - 无效或已过期的 Session ID 返回 HTTP 404 Not Found
//! - 缺少 Session ID（非初始化请求）返回 HTTP 400 Bad Request

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header::HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::state::SessionProjectContext;

/// MCP-Session-Id Header 名称
pub const MCP_SESSION_ID_HEADER: &str = "mcp-session-id";

/// 默认会话过期时间（分钟）
pub const DEFAULT_SESSION_TIMEOUT_MINUTES: i64 = 30;

/// MCP 会话信息
///
/// 扩展自原有的 ClientSession，添加 MCP Streamable HTTP 规范要求的字段
#[derive(Debug, Clone)]
pub struct McpSession {
    /// MCP-Session-Id Header 值（UUID v4 格式）
    pub session_id: String,
    /// 内部会话 ID（用于兼容旧的 SSE session）
    pub internal_id: String,
    /// 协商的协议版本
    pub protocol_version: String,
    /// 会话创建时间
    pub created_at: DateTime<Utc>,
    /// 最后活跃时间
    pub last_active: DateTime<Utc>,
    /// 工作目录
    pub work_dir: Option<PathBuf>,
    /// 项目上下文
    pub project_context: Option<SessionProjectContext>,
    /// 是否已初始化（收到 initialize 请求）
    pub initialized: bool,
    /// 会话过期时间（分钟）
    pub timeout_minutes: i64,
}

impl McpSession {
    /// 创建新的 MCP 会话
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            internal_id: Uuid::new_v4().to_string(),
            protocol_version: "2025-03-26".to_string(),
            created_at: now,
            last_active: now,
            work_dir: None,
            project_context: None,
            initialized: false,
            timeout_minutes: DEFAULT_SESSION_TIMEOUT_MINUTES,
        }
    }

    /// 创建带自定义超时的 MCP 会话
    pub fn with_timeout(timeout_minutes: i64) -> Self {
        let mut session = Self::new();
        session.timeout_minutes = timeout_minutes;
        session
    }

    /// 更新最后活跃时间
    pub fn touch(&mut self) {
        self.last_active = Utc::now();
    }

    /// 标记为已初始化
    pub fn mark_initialized(&mut self) {
        self.initialized = true;
        self.touch();
    }

    /// 设置协议版本
    pub fn set_protocol_version(&mut self, version: String) {
        self.protocol_version = version;
    }

    /// 设置工作目录
    pub fn set_work_dir(&mut self, work_dir: PathBuf) {
        self.work_dir = Some(work_dir);
    }

    /// 设置项目上下文
    pub fn set_project_context(&mut self, context: SessionProjectContext) {
        self.project_context = Some(context);
    }

    /// 检查会话是否已过期
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expiry = self.last_active + Duration::minutes(self.timeout_minutes);
        now > expiry
    }

    /// 获取有效的项目上下文
    pub fn get_effective_project(&self) -> Option<&SessionProjectContext> {
        self.project_context.as_ref()
    }
}

impl Default for McpSession {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP 会话存储
///
/// 管理所有活跃的 MCP 会话
#[derive(Debug, Default)]
pub struct McpSessionStore {
    /// 会话映射 (session_id -> McpSession)
    sessions: HashMap<String, McpSession>,
    /// 默认会话超时时间（分钟）
    default_timeout_minutes: i64,
}

impl McpSessionStore {
    /// 创建新的会话存储
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            default_timeout_minutes: DEFAULT_SESSION_TIMEOUT_MINUTES,
        }
    }

    /// 创建带自定义超时的会话存储
    pub fn with_timeout(timeout_minutes: i64) -> Self {
        Self {
            sessions: HashMap::new(),
            default_timeout_minutes: timeout_minutes,
        }
    }

    /// 创建新会话
    pub fn create_session(&mut self) -> &McpSession {
        let session = McpSession::with_timeout(self.default_timeout_minutes);
        let session_id = session.session_id.clone();
        self.sessions.insert(session_id.clone(), session);
        self.sessions.get(&session_id).unwrap()
    }

    /// 获取会话
    pub fn get_session(&self, session_id: &str) -> Option<&McpSession> {
        self.sessions.get(session_id).filter(|s| !s.is_expired())
    }

    /// 获取可变会话
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut McpSession> {
        self.sessions.get_mut(session_id).filter(|s| !s.is_expired())
    }

    /// 移除会话
    pub fn remove_session(&mut self, session_id: &str) -> Option<McpSession> {
        self.sessions.remove(session_id)
    }

    /// 检查会话是否存在且有效
    pub fn is_session_valid(&self, session_id: &str) -> bool {
        self.get_session(session_id).is_some()
    }

    /// 获取活跃会话数
    pub fn active_count(&self) -> usize {
        self.sessions.values().filter(|s| !s.is_expired()).count()
    }

    /// 清理过期会话
    pub fn cleanup_expired(&mut self) -> usize {
        let expired: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        let count = expired.len();
        for id in expired {
            self.sessions.remove(&id);
        }
        count
    }
}

/// 线程安全的会话存储包装
pub type SharedMcpSessionStore = Arc<RwLock<McpSessionStore>>;

/// Session 验证错误响应
/// 当前预留供 session_middleware 使用
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct SessionErrorResponse {
    pub jsonrpc: &'static str,
    pub id: Option<()>,
    pub error: SessionError,
}

/// Session 验证错误对象
#[derive(Debug, Serialize)]
pub struct SessionError {
    pub code: i32,
    pub message: String,
}

impl SessionErrorResponse {
    /// 创建 Session Not Found 错误（HTTP 404）
    pub fn not_found(session_id: &str) -> Self {
        Self {
            jsonrpc: "2.0",
            id: None,
            error: SessionError {
                code: -32002,
                message: format!("Session not found or expired: {}. Please reinitialize.", session_id),
            },
        }
    }

    /// 创建 Missing Session ID 错误（HTTP 400）
    pub fn missing_session_id() -> Self {
        Self {
            jsonrpc: "2.0",
            id: None,
            error: SessionError {
                code: -32003,
                message: "Missing MCP-Session-Id header".to_string(),
            },
        }
    }
}

/// 从请求中提取 MCP-Session-Id Header
pub fn extract_session_id(request: &Request<Body>) -> Option<String> {
    request
        .headers()
        .get(MCP_SESSION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// 创建 MCP-Session-Id Header
pub fn create_session_id_header(session_id: &str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static(MCP_SESSION_ID_HEADER),
        HeaderValue::from_str(session_id).unwrap_or_else(|_| HeaderValue::from_static("")),
    )
}

/// Session 验证中间件
///
/// MCP Streamable HTTP 规范要求：
/// - 后续请求（非初始化）必须包含 MCP-Session-Id Header
/// - 缺少 Session ID 返回 HTTP 400 Bad Request
/// - 无效或已过期的 Session ID 返回 HTTP 404 Not Found
///
/// 注意：此中间件应放在 auth 中间件之后，origin 中间件之后
/// 当前 Gateway 路由尚未接入此中间件，待 Streamable HTTP 全面上线后启用
#[allow(dead_code)]
pub async fn session_middleware(
    State(session_store): State<SharedMcpSessionStore>,
    request: Request,
    next: Next,
) -> Response {
    // 提取请求路径和方法
    let path = request.uri().path();
    let method = request.method().clone();

    // 检查是否是需要 Session ID 的请求
    // POST /mcp 需要 Session ID（除了 initialize 请求，但 initialize 在 handler 中处理）
    // GET /mcp 不需要 Session ID（建立新的 SSE 流）
    // DELETE /mcp 需要 Session ID
    let requires_session = path == "/mcp" && (method == "POST" || method == "DELETE");

    if !requires_session {
        // 不需要 Session ID 验证，继续处理
        return next.run(request).await;
    }

    // 提取 Session ID
    let session_id = extract_session_id(&request);

    match session_id {
        None => {
            // 检查是否是 initialize 请求（通过请求体）
            // 由于我们无法在中间件中读取请求体（会消耗掉），
            // 我们允许没有 Session ID 的 POST 请求通过，
            // 让 handler 处理 initialize 的特殊情况
            if method == "POST" {
                return next.run(request).await;
            }
            // 其他情况返回 400
            missing_session_response()
        }
        Some(sid) => {
            // 验证 Session ID
            let store = session_store.read().await;
            if store.is_session_valid(&sid) {
                drop(store);
                // Session 有效，更新活跃时间
                {
                    let mut store = session_store.write().await;
                    if let Some(session) = store.get_session_mut(&sid) {
                        session.touch();
                    }
                }
                next.run(request).await
            } else {
                // Session 无效或已过期
                not_found_response(&sid)
            }
        }
    }
}

/// 生成 400 Bad Request 响应（缺少 Session ID）
#[allow(dead_code)]
fn missing_session_response() -> Response {
    let response = SessionErrorResponse::missing_session_id();
    (StatusCode::BAD_REQUEST, Json(response)).into_response()
}

/// 生成 404 Not Found 响应（Session 无效或已过期）
#[allow(dead_code)]
fn not_found_response(session_id: &str) -> Response {
    let response = SessionErrorResponse::not_found(session_id);
    (StatusCode::NOT_FOUND, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== McpSession 测试 =====

    #[test]
    fn test_mcp_session_new() {
        let session = McpSession::new();
        assert!(!session.session_id.is_empty());
        assert!(!session.internal_id.is_empty());
        assert_eq!(session.protocol_version, "2025-03-26");
        assert!(!session.initialized);
        assert!(session.work_dir.is_none());
        assert!(session.project_context.is_none());
    }

    #[test]
    fn test_mcp_session_uuid_format() {
        let session = McpSession::new();
        // UUID v4 格式：xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        assert_eq!(session.session_id.len(), 36);
        assert!(Uuid::parse_str(&session.session_id).is_ok());
    }

    #[test]
    fn test_mcp_session_touch() {
        let mut session = McpSession::new();
        let initial_time = session.last_active;

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.touch();
        assert!(session.last_active > initial_time);
    }

    #[test]
    fn test_mcp_session_mark_initialized() {
        let mut session = McpSession::new();
        assert!(!session.initialized);

        session.mark_initialized();
        assert!(session.initialized);
    }

    #[test]
    fn test_mcp_session_expiry() {
        let mut session = McpSession::with_timeout(0); // 0 分钟超时
        session.last_active = Utc::now() - Duration::minutes(1);

        assert!(session.is_expired());
    }

    #[test]
    fn test_mcp_session_not_expired() {
        let session = McpSession::new();
        assert!(!session.is_expired());
    }

    #[test]
    fn test_mcp_session_with_timeout() {
        let session = McpSession::with_timeout(60);
        assert_eq!(session.timeout_minutes, 60);
    }

    // ===== McpSessionStore 测试 =====

    #[test]
    fn test_session_store_create() {
        let mut store = McpSessionStore::new();
        let session = store.create_session();

        assert!(!session.session_id.is_empty());
        assert_eq!(store.active_count(), 1);
    }

    #[test]
    fn test_session_store_get() {
        let mut store = McpSessionStore::new();
        let session = store.create_session();
        let session_id = session.session_id.clone();

        let retrieved = store.get_session(&session_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session_id);
    }

    #[test]
    fn test_session_store_get_expired() {
        let mut store = McpSessionStore::with_timeout(0);
        let session = store.create_session();
        let session_id = session.session_id.clone();

        // 手动设置过期
        if let Some(s) = store.sessions.get_mut(&session_id) {
            s.last_active = Utc::now() - Duration::minutes(1);
        }

        // 过期会话应该返回 None
        assert!(store.get_session(&session_id).is_none());
    }

    #[test]
    fn test_session_store_remove() {
        let mut store = McpSessionStore::new();
        let session = store.create_session();
        let session_id = session.session_id.clone();

        let removed = store.remove_session(&session_id);
        assert!(removed.is_some());
        assert!(store.get_session(&session_id).is_none());
        assert_eq!(store.active_count(), 0);
    }

    #[test]
    fn test_session_store_is_valid() {
        let mut store = McpSessionStore::new();
        let session = store.create_session();
        let session_id = session.session_id.clone();

        assert!(store.is_session_valid(&session_id));
        assert!(!store.is_session_valid("invalid-session-id"));
    }

    #[test]
    fn test_session_store_cleanup_expired() {
        let mut store = McpSessionStore::new(); // 使用默认的 30 分钟超时

        // 创建两个会话
        let id1 = store.create_session().session_id.clone();
        let id2 = store.create_session().session_id.clone();

        // 手动将第一个会话的 timeout 设为 0 并标记为过期
        if let Some(s) = store.sessions.get_mut(&id1) {
            s.timeout_minutes = 0;
            s.last_active = Utc::now() - Duration::minutes(1);
        }

        // 清理过期
        let cleaned = store.cleanup_expired();
        assert_eq!(cleaned, 1);

        // 只有 session2 应该存在
        assert!(store.sessions.get(&id1).is_none());
        assert!(store.sessions.get(&id2).is_some());
    }

    // ===== SessionErrorResponse 测试 =====

    #[test]
    fn test_session_error_not_found() {
        let response = SessionErrorResponse::not_found("test-session-id");
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.id.is_none());
        assert_eq!(response.error.code, -32002);
        assert!(response.error.message.contains("test-session-id"));
        assert!(response.error.message.contains("reinitialize"));
    }

    #[test]
    fn test_session_error_missing() {
        let response = SessionErrorResponse::missing_session_id();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.id.is_none());
        assert_eq!(response.error.code, -32003);
        assert!(response.error.message.contains("Missing"));
    }

    #[test]
    fn test_session_error_serialization() {
        let response = SessionErrorResponse::not_found("abc-123");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":null"));
        assert!(json.contains("-32002"));
    }

    // ===== Helper 函数测试 =====

    #[test]
    fn test_create_session_id_header() {
        let (name, value) = create_session_id_header("test-session-123");
        assert_eq!(name.as_str(), "mcp-session-id");
        assert_eq!(value.to_str().unwrap(), "test-session-123");
    }

    #[test]
    fn test_extract_session_id() {
        use axum::http::Request as HttpRequest;

        let request = HttpRequest::builder()
            .header(MCP_SESSION_ID_HEADER, "my-session-id")
            .body(Body::empty())
            .unwrap();

        let session_id = extract_session_id(&request);
        assert_eq!(session_id, Some("my-session-id".to_string()));
    }

    #[test]
    fn test_extract_session_id_missing() {
        use axum::http::Request as HttpRequest;

        let request = HttpRequest::builder().body(Body::empty()).unwrap();

        let session_id = extract_session_id(&request);
        assert!(session_id.is_none());
    }
}
