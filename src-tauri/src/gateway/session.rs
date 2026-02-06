//! MCP Session 管理模块
//!
//! Story 11.14: MCP Streamable HTTP 规范合规 - Task 2
//! Story 11.26: MCP Roots 机制 - roots capability 支持
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
/// Story 11.26: 添加 roots capability 支持
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
    /// Client 是否支持 roots capability (Story 11.26)
    pub supports_roots: bool,
    /// Client 是否支持 roots.listChanged 通知 (Story 11.26)
    pub roots_list_changed: bool,
    /// 待响应的 roots/list 请求 ID (Story 11.26)
    pub pending_roots_request_id: Option<String>,
    /// 已解析的 roots 路径列表 (Story 11.26)
    pub roots_paths: Vec<PathBuf>,
    /// roots 请求是否超时 (Story 11.26)
    pub roots_request_timed_out: bool,
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
            supports_roots: false,
            roots_list_changed: false,
            pending_roots_request_id: None,
            roots_paths: Vec::new(),
            roots_request_timed_out: false,
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

    /// 设置 roots capability 信息
    ///
    /// Story 11.26: MCP Roots 机制
    pub fn set_roots_capability(&mut self, supports_roots: bool, list_changed: bool) {
        self.supports_roots = supports_roots;
        self.roots_list_changed = list_changed;
    }

    /// 设置已解析的 roots 路径列表
    ///
    /// Story 11.26: MCP Roots 机制
    pub fn set_roots_paths(&mut self, paths: Vec<PathBuf>) {
        self.roots_paths = paths;
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
mod tests;

// ============================================================
// Story 11.26: Server-to-Client 通信机制
// ============================================================

use tokio::sync::{mpsc, oneshot, Mutex};

/// 待处理的服务端请求
///
/// Story 11.26: Task 3.4
#[derive(Debug)]
#[allow(dead_code)]
pub struct PendingServerRequest {
    /// 请求 ID
    pub request_id: String,
    /// 方法名称
    pub method: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// Server-to-Client 通信通道
///
/// Story 11.26: Task 3
/// 每个 MCP 会话对应一个通道实例
#[allow(dead_code)]
pub struct SessionChannel {
    /// SSE 事件发送器 (Server → Client via SSE)
    pub sse_tx: mpsc::Sender<String>,
    /// 会话 ID
    pub session_id: String,
}

/// Server-to-Client 通信管理器
///
/// Story 11.26: Task 3
/// 管理所有会话的 SSE 事件通道和待处理请求
pub struct ServerToClientManager {
    /// 会话的 SSE 事件发送器映射 (session_id -> Sender)
    channels: Mutex<HashMap<String, mpsc::Sender<String>>>,
    /// 待处理的服务端请求映射 (request_id -> oneshot::Sender)
    pending_requests: Mutex<HashMap<String, (String, oneshot::Sender<serde_json::Value>)>>,
}

impl ServerToClientManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            channels: Mutex::new(HashMap::new()),
            pending_requests: Mutex::new(HashMap::new()),
        }
    }

    /// 注册会话的 SSE 通道
    ///
    /// 返回接收器供 SSE 流使用
    pub async fn register_channel(&self, session_id: &str, buffer_size: usize) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel(buffer_size);
        let mut channels = self.channels.lock().await;
        channels.insert(session_id.to_string(), tx);
        rx
    }

    /// 注销会话的 SSE 通道
    pub async fn unregister_channel(&self, session_id: &str) {
        let mut channels = self.channels.lock().await;
        channels.remove(session_id);
    }

    /// 向会话发送 SSE 事件
    ///
    /// Story 11.26: AC6 - 通过 SSE 流发送 JSON-RPC 请求
    pub async fn send_to_client(&self, session_id: &str, message: String) -> Result<(), String> {
        let channels = self.channels.lock().await;
        if let Some(tx) = channels.get(session_id) {
            tx.send(message).await.map_err(|e| format!("Failed to send SSE event: {}", e))
        } else {
            Err(format!("No SSE channel for session: {}", session_id))
        }
    }

    /// 发送 JSON-RPC 请求并等待响应
    ///
    /// Story 11.26: Task 3.3 - 管理待处理请求 ID 和响应 channel 映射
    /// Story 11.26: AC7 - 超时处理
    pub async fn send_request_and_wait(
        &self,
        session_id: &str,
        request_id: &str,
        request_json: String,
        timeout_secs: u64,
    ) -> Result<serde_json::Value, String> {
        // 1. 创建 oneshot channel 用于接收响应
        let (tx, rx) = oneshot::channel();

        // 2. 注册 pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request_id.to_string(), (session_id.to_string(), tx));
        }

        // 3. 发送请求到 SSE 流
        if let Err(e) = self.send_to_client(session_id, request_json).await {
            // 发送失败，清理 pending request
            let mut pending = self.pending_requests.lock().await;
            pending.remove(request_id);
            return Err(e);
        }

        // 4. 等待响应（带超时）
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            rx,
        ).await;

        // 5. 清理 pending request（无论成功与否）
        {
            let mut pending = self.pending_requests.lock().await;
            pending.remove(request_id);
        }

        match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err("Response channel closed".to_string()),
            Err(_) => Err(format!("Request timed out after {}s", timeout_secs)),
        }
    }

    /// 处理来自 Client 的响应
    ///
    /// Story 11.26: Task 3.5 - 修改 POST handler 处理 Client 的响应
    ///
    /// 返回 true 如果成功匹配了 pending request
    pub async fn handle_client_response(&self, request_id: &str, response: serde_json::Value) -> bool {
        let mut pending = self.pending_requests.lock().await;
        if let Some((_, tx)) = pending.remove(request_id) {
            // 发送响应到等待的 future
            let _ = tx.send(response);
            true
        } else {
            false
        }
    }

    /// 检查是否有待处理的请求
    #[allow(dead_code)]
    pub async fn has_pending_request(&self, request_id: &str) -> bool {
        let pending = self.pending_requests.lock().await;
        pending.contains_key(request_id)
    }

    /// 获取待处理请求数量（调试用）
    #[allow(dead_code)]
    pub async fn pending_count(&self) -> usize {
        let pending = self.pending_requests.lock().await;
        pending.len()
    }

    /// 注册待处理请求（用于 SSE 流内同步发送）
    ///
    /// Story 11.26 Fix: 在 SSE 流创建过程中注册 pending request
    /// 返回 oneshot::Receiver 供流内等待响应
    pub async fn register_pending_request(
        &self,
        session_id: &str,
        request_id: &str,
    ) -> oneshot::Receiver<serde_json::Value> {
        let (tx, rx) = oneshot::channel();
        let mut pending = self.pending_requests.lock().await;
        pending.insert(request_id.to_string(), (session_id.to_string(), tx));
        rx
    }

    /// 取消待处理请求（超时或错误时清理）
    ///
    /// Story 11.26 Fix: 清理未完成的 pending request
    pub async fn cancel_pending_request(&self, request_id: &str) {
        let mut pending = self.pending_requests.lock().await;
        pending.remove(request_id);
    }
}

impl Default for ServerToClientManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 线程安全的 Server-to-Client 管理器包装
pub type SharedServerToClientManager = Arc<ServerToClientManager>;
