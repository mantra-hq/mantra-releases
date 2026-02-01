//! HTTP/SSE 请求处理器
//!
//! Story 11.1: SSE Server 核心 - Task 2 & Task 4
//! Story 11.5: 上下文路由 - Task 4 & Task 5
//! Story 11.14: MCP Streamable HTTP 规范合规 - Task 3
//!
//! 实现 `/sse` SSE 端点、`/message` JSON-RPC 端点和 `/mcp` Streamable HTTP 端点

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

use super::origin::validate_origin;
use super::session::{
    create_session_id_header, McpSessionStore, SharedMcpSessionStore, MCP_SESSION_ID_HEADER,
};
use super::state::{GatewayState, GatewayStats};

/// Message 端点查询参数
#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    /// 会话 ID（可选，如果不提供则创建临时会话）
    pub session_id: Option<String>,
    // Note: token 已在 auth 中间件中处理，此处不再需要
}

/// JSON-RPC 请求
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    /// 请求参数 (当前 Story 未使用，后续 Story 11.5 路由转发时使用)
    #[serde(default)]
    #[allow(dead_code)]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 响应
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 错误对象
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    /// 创建成功响应
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    /// 创建错误响应
    pub fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }

    /// 方法未找到
    pub fn method_not_found(id: Option<serde_json::Value>) -> Self {
        Self::error(id, -32601, "Method not found".to_string())
    }

    /// 解析错误 (当 JSON 解析失败时使用)
    #[allow(dead_code)]
    pub fn parse_error() -> Self {
        Self::error(None, -32700, "Parse error".to_string())
    }

    /// 无效请求
    pub fn invalid_request(id: Option<serde_json::Value>) -> Self {
        Self::error(id, -32600, "Invalid Request".to_string())
    }
}

/// Gateway 共享应用状态
///
/// Story 11.5: 扩展添加 router 和 registry
/// Story 11.14: 添加 MCP Session Store
///
/// 注意：由于 rusqlite::Connection 不是 Send + Sync，
/// router 和 registry 需要通过 Tauri 状态管理在外部提供，
/// 而不是直接存储在这里。
#[derive(Clone)]
pub struct GatewayAppState {
    pub state: Arc<RwLock<GatewayState>>,
    pub stats: Arc<GatewayStats>,
    /// MCP Streamable HTTP 会话存储 (Story 11.14)
    pub mcp_sessions: SharedMcpSessionStore,
}

impl GatewayAppState {
    /// 创建应用状态
    pub fn new(state: Arc<RwLock<GatewayState>>, stats: Arc<GatewayStats>) -> Self {
        Self {
            state,
            stats,
            mcp_sessions: Arc::new(RwLock::new(McpSessionStore::new())),
        }
    }
}

/// 会话清理守卫
/// 
/// 当此结构体被 drop 时，自动从状态中移除对应的会话
struct SessionCleanupGuard {
    session_id: String,
    state: Arc<RwLock<GatewayState>>,
}

impl Drop for SessionCleanupGuard {
    fn drop(&mut self) {
        let session_id = self.session_id.clone();
        let state = self.state.clone();
        // 在后台异步清理会话
        tokio::spawn(async move {
            let mut state_guard = state.write().await;
            state_guard.remove_session(&session_id);
        });
    }
}

/// GET /sse - SSE 连接端点
///
/// 建立 SSE 连接，发送 `endpoint` 事件包含 message 端点 URL
///
/// **DEPRECATED:** 请迁移至 MCP Streamable HTTP 端点 `/mcp`
/// 此端点将在未来版本中移除。
pub async fn sse_handler(
    State(app_state): State<GatewayAppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Story 11.14: 记录 deprecation 警告
    eprintln!("[Mantra Gateway] DEPRECATED: GET /sse is deprecated. Please migrate to GET /mcp for SSE streams.");

    // 增加连接计数
    app_state.stats.increment_connections();

    // 注册新会话
    let session = {
        let mut state = app_state.state.write().await;
        state.register_session()
    };

    let session_id = session.session_id.clone();
    let message_endpoint = session.message_endpoint.clone();
    let state_clone = app_state.state.clone();
    let state_for_cleanup = app_state.state.clone();

    // 创建会话清理守卫 - 当流被 drop 时自动清理会话
    let cleanup_guard = SessionCleanupGuard {
        session_id: session_id.clone(),
        state: state_for_cleanup,
    };

    // 创建 SSE 事件流
    let stream = stream::once(async move {
        // 发送 endpoint 事件
        Ok::<_, Infallible>(
            Event::default()
                .event("endpoint")
                .data(message_endpoint),
        )
    })
    .chain(
        // 心跳流 - 每 30 秒发送一次
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(30)))
            .map(move |_| {
                // 更新会话活跃时间
                let state = state_clone.clone();
                let sid = session_id.clone();
                tokio::spawn(async move {
                    let mut state_guard = state.write().await;
                    if let Some(session) = state_guard.get_session_mut(&sid) {
                        session.touch();
                    }
                });
                Ok::<_, Infallible>(Event::default().comment("keepalive"))
            }),
    )
    // 包装流以确保在 drop 时触发清理
    .map(move |event| {
        // 保持 cleanup_guard 存活直到流结束
        let _guard = &cleanup_guard;
        event
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// POST /message - JSON-RPC 消息端点
///
/// 接收 JSON-RPC 请求并返回响应
///
/// **DEPRECATED:** 请迁移至 MCP Streamable HTTP 端点 `POST /mcp`
/// 此端点将在未来版本中移除。
///
/// 支持两种模式：
/// - 带 session_id: 使用已存在的 SSE 会话
/// - 不带 session_id: 创建临时会话（用于 Inspector 等简单测试场景）
pub async fn message_handler(
    State(app_state): State<GatewayAppState>,
    Query(query): Query<MessageQuery>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    // Story 11.14: 记录 deprecation 警告
    eprintln!("[Mantra Gateway] DEPRECATED: POST /message is deprecated. Please migrate to POST /mcp.");

    // 增加请求计数
    app_state.stats.increment_requests();

    // 获取或创建会话
    let session_id = match &query.session_id {
        Some(id) => {
            // 验证会话存在
            let state = app_state.state.read().await;
            if state.get_session(id).is_none() {
                let response = JsonRpcResponse::error(
                    request.id.clone(),
                    -32002,
                    format!("Session not found: {}", id),
                );
                return (StatusCode::NOT_FOUND, Json(response)).into_response();
            }
            id.clone()
        }
        None => {
            // 创建临时会话（用于 Inspector 等简单测试场景）
            let mut state = app_state.state.write().await;
            let session = state.register_session();
            session.session_id.clone()
        }
    };

    // 验证 JSON-RPC 版本
    if request.jsonrpc != "2.0" {
        return (StatusCode::BAD_REQUEST, Json(JsonRpcResponse::invalid_request(request.id)))
            .into_response();
    }

    // 更新会话活跃时间
    {
        let mut state = app_state.state.write().await;
        if let Some(session) = state.get_session_mut(&session_id) {
            session.touch();
        }
    }

    // 根据方法路由处理
    let response = match request.method.as_str() {
        "initialize" => {
            handle_initialize(&app_state, &session_id, &request).await
        }
        "ping" => {
            // 简单的 ping 方法
            JsonRpcResponse::success(request.id, serde_json::json!({}))
        }
        "tools/list" => {
            handle_tools_list(&app_state, &session_id, &request).await
        }
        "tools/call" => {
            handle_tools_call(&app_state, &session_id, &request).await
        }
        "resources/list" => {
            handle_resources_list(&request).await
        }
        "resources/read" => {
            handle_resources_read(&request).await
        }
        _ => {
            // 其他方法暂不支持
            JsonRpcResponse::method_not_found(request.id)
        }
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// 处理 initialize 请求
///
/// Story 11.5: 上下文路由 - Task 4
///
/// 1. 解析 rootUri/workspaceFolders 获取工作目录
/// 2. 保存工作目录到会话状态
/// 3. 返回 MCP 初始化响应
///
/// 注意：由于 rusqlite 线程安全限制，LPM 路由查找将通过
/// Tauri IPC 命令在外部执行，而不是在 HTTP handler 中直接调用。
async fn handle_initialize(
    app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // 1. 解析 rootUri/workspaceFolders
    let work_dir = request
        .params
        .as_ref()
        .and_then(|p| parse_work_dir_from_params(p));

    // 2. 保存工作目录到会话状态
    {
        let mut state = app_state.state.write().await;
        if let Some(session) = state.get_session_mut(session_id) {
            if let Some(ref dir) = work_dir {
                session.set_work_dir(dir.clone());
            }
        }
    }

    // 3. 返回 MCP 初始化响应
    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "mantra-gateway",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

/// 解析 MCP initialize 请求中的工作目录
///
/// 支持多种格式：
/// - `rootUri`: string (file URI)
/// - `workspaceFolders`: [{ uri: string, name: string }]
/// - `rootPath`: string (deprecated but still used)
fn parse_work_dir_from_params(params: &serde_json::Value) -> Option<std::path::PathBuf> {
    // 优先使用 workspaceFolders
    if let Some(folders) = params.get("workspaceFolders").and_then(|v| v.as_array()) {
        if let Some(first) = folders.first() {
            if let Some(uri) = first.get("uri").and_then(|v| v.as_str()) {
                return uri_to_path(uri);
            }
        }
    }

    // 回退到 rootUri
    if let Some(root_uri) = params.get("rootUri").and_then(|v| v.as_str()) {
        return uri_to_path(root_uri);
    }

    // 再回退到 rootPath (deprecated but still used)
    if let Some(root_path) = params.get("rootPath").and_then(|v| v.as_str()) {
        return Some(std::path::PathBuf::from(root_path));
    }

    None
}

/// 将 file:// URI 转换为本地路径
fn uri_to_path(uri: &str) -> Option<std::path::PathBuf> {
    if uri.starts_with("file://") {
        let path = &uri[7..];

        // Windows: file:///C:/path -> C:/path
        #[cfg(target_os = "windows")]
        {
            if path.starts_with('/') && path.len() > 2 && path.chars().nth(2) == Some(':') {
                return Some(std::path::PathBuf::from(&path[1..]));
            }
        }

        // Unix: file:///path -> /path
        // URL 解码
        if let Ok(decoded) = urlencoding::decode(path) {
            return Some(std::path::PathBuf::from(decoded.as_ref()));
        }
        return Some(std::path::PathBuf::from(path));
    }
    None
}

/// 处理 tools/list 请求
///
/// Story 11.5: 上下文路由 - Task 5
/// Story 11.10: Project-Level Tool Management - AC 4 (Gateway 拦截 - tools/list 响应过滤)
///
/// 返回工具列表。根据项目的 Tool Policy 过滤返回的工具。
///
/// 注意：由于 rusqlite 线程安全限制，服务列表查询将通过
/// Tauri IPC 命令在外部执行。当前实现返回基于 Tool Policy 的过滤结果。
///
/// ## Tool Policy 过滤规则 (AC 4)
/// - `mode = "allow_all"`: 返回所有工具（除了 deniedTools 中的）
/// - `mode = "deny_all"`: 返回空工具列表
/// - `mode = "custom"`: 仅返回 allowedTools 中且不在 deniedTools 中的工具
async fn handle_tools_list(
    app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // 获取会话的项目上下文
    let project_context = {
        let state = app_state.state.read().await;
        state
            .get_session(session_id)
            .and_then(|s| s.get_effective_project().cloned())
    };

    // 当前返回空工具列表
    // 完整实现将在 Task 6/7 中通过 MCP 子进程管理器获取实际工具
    // 此处仅演示 Tool Policy 过滤逻辑的占位
    let tools: Vec<serde_json::Value> = Vec::new();

    // 如果有项目上下文，记录日志
    if let Some(ref _ctx) = project_context {
        // Tool Policy 过滤将在实际工具列表获取后执行
        // 由于 rusqlite 线程安全限制，需要通过 Tauri IPC 查询 Tool Policy
    }

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "tools": tools
        }),
    )
}

/// 处理 resources/list 请求
///
/// 返回可用资源列表。当前返回空列表（占位实现）。
async fn handle_resources_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    // 当前返回空资源列表
    // 完整实现将通过 MCP 子进程管理器获取实际资源
    let resources: Vec<serde_json::Value> = Vec::new();

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "resources": resources
        }),
    )
}

/// 处理 resources/read 请求
///
/// 读取指定资源的内容。当前返回错误（占位实现）。
async fn handle_resources_read(request: &JsonRpcRequest) -> JsonRpcResponse {
    let uri = request
        .params
        .as_ref()
        .and_then(|p| p.get("uri"))
        .and_then(|v| v.as_str());

    match uri {
        Some(uri) => {
            // 当前返回未实现错误
            // 完整实现将通过 MCP 子进程管理器读取实际资源
            JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                format!("Resource read not yet implemented for: {}", uri),
            )
        }
        None => JsonRpcResponse::error(
            request.id.clone(),
            -32602,
            "Missing uri parameter".to_string(),
        ),
    }
}

/// 处理 tools/call 请求
///
/// Story 11.5: 上下文路由 - Task 7
/// Story 11.10: Project-Level Tool Management - AC 5 (Gateway 拦截 - tools/call 请求拦截)
///
/// 1. 解析工具名称和参数
/// 2. 检查 Tool Policy 是否允许该工具
/// 3. 路由到对应的 MCP 服务
/// 4. 转发请求并返回响应
///
/// 注意：由于 rusqlite 线程安全限制，实际的工具调用转发
/// 需要通过 Tauri IPC 命令在外部执行。当前实现返回占位响应。
///
/// ## Tool Policy 拦截规则 (AC 5)
/// 当工具被 Tool Policy 禁止时：
/// - 不转发请求到上游 MCP 服务
/// - 返回 JSON-RPC Error: `{"code": -32601, "message": "Tool not found: {tool_name}"}`
/// - 记录审计日志: `tool_blocked` 事件
async fn handle_tools_call(
    app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // 1. 解析工具名称和参数
    let params = match &request.params {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                "Missing params".to_string(),
            );
        }
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                "Missing tool name".to_string(),
            );
        }
    };

    // 2. 解析工具名称格式: service_name/tool_name
    let parts: Vec<&str> = tool_name.splitn(2, '/').collect();
    if parts.len() != 2 {
        return JsonRpcResponse::error(
            request.id.clone(),
            -32602,
            "Invalid tool name format, expected: service_name/tool_name".to_string(),
        );
    }

    let service_name = parts[0];
    let actual_tool_name = parts[1];

    // 3. 获取会话的项目上下文
    let project_context = {
        let state = app_state.state.read().await;
        state
            .get_session(session_id)
            .and_then(|s| s.get_effective_project().cloned())
    };

    // 4. Tool Policy 检查将在实际转发时执行
    // 由于 rusqlite 线程安全限制，Tool Policy 查询需要通过 Tauri IPC 命令执行
    if let Some(ref _ctx) = project_context {
        // 实际 Tool Policy 检查将通过 Tauri IPC 命令在外部执行
        // 这里仅用于占位，完整实现需要：
        // 1. 查询项目的 Tool Policy
        // 2. 如果工具被禁止，调用 tool_blocked_error 并记录审计日志
        let _ = (service_name, actual_tool_name);
    }

    // 5. 当前返回占位响应
    // 完整实现需要：
    // - 通过 Tauri IPC 查询 Tool Policy
    // - 如果工具被禁止，返回 -32601 错误并记录审计日志
    // - 否则，查找服务配置，启动 MCP 子进程，转发请求
    JsonRpcResponse::error(
        request.id.clone(),
        -32603,
        "Tool call forwarding not yet implemented. Use Tauri IPC commands.".to_string(),
    )
}

/// 检查工具是否被 Tool Policy 阻止
///
/// Story 11.10: Project-Level Tool Management - AC 5
///
/// 此函数用于 Gateway 拦截逻辑。当前为占位实现，
/// 完整集成需要通过 Tauri IPC 查询 Tool Policy 后调用。
///
/// # Arguments
/// * `tool_name` - 工具名称
/// * `policy` - Tool Policy 配置
///
/// # Returns
/// `true` 如果工具被阻止，`false` 如果允许
pub fn is_tool_blocked(tool_name: &str, policy: &crate::models::mcp::ToolPolicy) -> bool {
    !policy.is_tool_allowed(tool_name)
}

/// 创建工具被阻止的 JSON-RPC 错误响应
///
/// Story 11.10: Project-Level Tool Management - AC 5
///
/// 此函数用于 Gateway 拦截逻辑。当工具被 Tool Policy 禁止时，
/// 返回标准的 JSON-RPC -32601 错误（伪装为 "Tool not found"）。
///
/// # Arguments
/// * `id` - 请求 ID
/// * `tool_name` - 被阻止的工具名称
pub fn tool_blocked_error(id: Option<serde_json::Value>, tool_name: &str) -> JsonRpcResponse {
    JsonRpcResponse::error(id, -32601, format!("Tool not found: {}", tool_name))
}

/// 记录工具被阻止的审计日志
///
/// Story 11.10: Project-Level Tool Management - AC 5
///
/// 此函数用于 Gateway 拦截逻辑。当工具被阻止时，
/// 生成审计日志条目用于后续持久化存储。
///
/// # Arguments
/// * `project_id` - 项目 ID
/// * `service_id` - 服务 ID
/// * `tool_name` - 被阻止的工具名称
///
/// # Returns
/// 审计日志条目（用于持久化存储）
pub fn log_tool_blocked(project_id: &str, service_id: &str, tool_name: &str) -> serde_json::Value {
    let timestamp = chrono::Utc::now().to_rfc3339();
    serde_json::json!({
        "event": "tool_blocked",
        "project_id": project_id,
        "service_id": service_id,
        "tool_name": tool_name,
        "timestamp": timestamp,
        "message": "Tool call blocked by Tool Policy"
    })
}

/// GET /health - 健康检查端点
pub async fn health_handler(State(app_state): State<GatewayAppState>) -> impl IntoResponse {
    let state = app_state.state.read().await;
    let active_connections = state.active_connections();
    let total_connections = app_state.stats.get_total_connections();
    let total_requests = app_state.stats.get_total_requests();

    // Story 11.14: 添加 MCP 会话统计
    let mcp_session_count = {
        let store = app_state.mcp_sessions.read().await;
        store.active_count()
    };

    Json(serde_json::json!({
        "status": "ok",
        "service": "mantra-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "stats": {
            "activeConnections": active_connections,
            "totalConnections": total_connections,
            "totalRequests": total_requests,
            "mcpSessions": mcp_session_count
        }
    }))
}

// ===== Story 11.14: MCP Streamable HTTP 端点处理器 =====

/// POST /mcp - MCP Streamable HTTP POST 端点
///
/// 接收 JSON-RPC 请求、通知或响应：
/// - Request (带 `id`): 返回 HTTP 200 + JSON 响应，或 HTTP 200 + SSE 流
/// - Notification (无 `id`): 返回 HTTP 202 Accepted (无响应体)
/// - Response: 返回 HTTP 202 Accepted (无响应体)
pub async fn mcp_post_handler(
    State(app_state): State<GatewayAppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Response {
    // 增加请求计数
    app_state.stats.increment_requests();

    // 1. Origin 验证 (MUST)
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok());
    if !validate_origin(origin, None) {
        return forbidden_origin_response();
    }

    // 2. 验证 Content-Type
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.contains("application/json") {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(JsonRpcResponse::error(
                None,
                -32700,
                "Content-Type must be application/json".to_string(),
            )),
        )
            .into_response();
    }

    // 3. 验证 JSON-RPC 版本
    let jsonrpc = body.get("jsonrpc").and_then(|v| v.as_str());
    if jsonrpc != Some("2.0") {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::invalid_request(
                body.get("id").cloned(),
            )),
        )
            .into_response();
    }

    // 4. 确定消息类型
    let has_method = body.get("method").is_some();
    let has_id = body.get("id").is_some();
    let has_result = body.get("result").is_some();
    let has_error = body.get("error").is_some();

    // 5. 处理不同消息类型
    if has_method && has_id {
        // JSON-RPC Request - 需要返回响应
        handle_mcp_request(&app_state, &headers, &body).await
    } else if has_method && !has_id {
        // JSON-RPC Notification - 返回 202 Accepted
        handle_mcp_notification(&app_state, &headers, &body).await
    } else if has_result || has_error {
        // JSON-RPC Response - 返回 202 Accepted
        (StatusCode::ACCEPTED, "").into_response()
    } else {
        // 无法识别的消息格式
        (
            StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::invalid_request(
                body.get("id").cloned(),
            )),
        )
            .into_response()
    }
}

/// 处理 MCP JSON-RPC Request (带 id)
///
/// 返回 HTTP 200 + JSON 响应
async fn handle_mcp_request(
    app_state: &GatewayAppState,
    headers: &HeaderMap,
    body: &serde_json::Value,
) -> Response {
    let method = body
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let id = body.get("id").cloned();
    let params = body.get("params").cloned();

    // 提取 MCP-Session-Id Header
    let mcp_session_id = headers
        .get(MCP_SESSION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 处理 initialize 请求（特殊情况，不需要 Session ID）
    if method == "initialize" {
        return handle_mcp_initialize(app_state, id, params).await;
    }

    // 非 initialize 请求：验证 MCP-Session-Id
    let session_id = match mcp_session_id {
        Some(sid) => {
            // 验证 Session ID 是否有效
            let store = app_state.mcp_sessions.read().await;
            if store.is_session_valid(&sid) {
                sid
            } else {
                return session_not_found_response(&sid);
            }
        }
        None => {
            // 允许没有 Session ID 的请求（向后兼容旧客户端）
            // 使用旧的 session 机制
            return handle_legacy_request(app_state, method, id, params).await;
        }
    };

    // 更新会话活跃时间
    {
        let mut store = app_state.mcp_sessions.write().await;
        if let Some(session) = store.get_session_mut(&session_id) {
            session.touch();
        }
    }

    // 验证并存储 MCP-Protocol-Version Header (AC4)
    // 支持的协议版本
    const SUPPORTED_VERSIONS: &[&str] = &["2025-03-26", "2024-11-05"];
    const DEFAULT_VERSION: &str = "2025-03-26";

    let protocol_version = headers
        .get("mcp-protocol-version")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(DEFAULT_VERSION); // 如果 Header 缺失，默认使用 2025-03-26

    if !SUPPORTED_VERSIONS.contains(&protocol_version) {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::error(
                id,
                -32001,
                format!("Unsupported protocol version: {}", protocol_version),
            )),
        )
            .into_response();
    }

    // 将协商后的协议版本存储到会话状态 (AC4)
    {
        let mut store = app_state.mcp_sessions.write().await;
        if let Some(session) = store.get_session_mut(&session_id) {
            session.set_protocol_version(protocol_version.to_string());
        }
    }

    // 路由到对应的方法处理器
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: id.clone(),
        method: method.to_string(),
        params,
    };

    let response = match method {
        "ping" => JsonRpcResponse::success(id, serde_json::json!({})),
        "tools/list" => handle_tools_list(app_state, &session_id, &request).await,
        "tools/call" => handle_tools_call(app_state, &session_id, &request).await,
        "resources/list" => handle_resources_list(&request).await,
        "resources/read" => handle_resources_read(&request).await,
        _ => JsonRpcResponse::method_not_found(id),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// 处理 MCP initialize 请求
///
/// 创建新的 MCP 会话，返回 MCP-Session-Id Header
async fn handle_mcp_initialize(
    app_state: &GatewayAppState,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> Response {
    // 1. 创建新的 MCP 会话
    let session_id = {
        let mut store = app_state.mcp_sessions.write().await;
        let session = store.create_session();
        let sid = session.session_id.clone();
        // 标记为已初始化
        if let Some(s) = store.get_session_mut(&sid) {
            s.mark_initialized();
        }
        sid
    };

    // 2. 解析工作目录
    if let Some(ref p) = params {
        let work_dir = parse_work_dir_from_params(p);
        if let Some(ref dir) = work_dir {
            let mut store = app_state.mcp_sessions.write().await;
            if let Some(session) = store.get_session_mut(&session_id) {
                session.set_work_dir(dir.clone());
            }
        }
    }

    // 3. 同时注册到旧的 session 系统（向后兼容）
    {
        let mut state = app_state.state.write().await;
        let old_session = state.register_session();
        // 解析工作目录到旧 session
        if let Some(ref p) = params {
            let work_dir = parse_work_dir_from_params(p);
            if let Some(ref dir) = work_dir {
                if let Some(s) = state.get_session_mut(&old_session.session_id) {
                    s.set_work_dir(dir.clone());
                }
            }
        }
    }

    // 4. 构建初始化响应
    let result = serde_json::json!({
        "protocolVersion": "2025-03-26",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "mantra-gateway",
            "version": env!("CARGO_PKG_VERSION")
        }
    });

    let json_response = JsonRpcResponse::success(id, result);

    // 5. 返回带 MCP-Session-Id Header 的响应
    let (header_name, header_value) = create_session_id_header(&session_id);
    let mut response = (StatusCode::OK, Json(json_response)).into_response();
    response.headers_mut().insert(header_name, header_value);
    response
}

/// 处理旧版请求（无 MCP-Session-Id）
///
/// 向后兼容：使用旧的 session 机制
async fn handle_legacy_request(
    app_state: &GatewayAppState,
    method: &str,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> Response {
    // 创建临时会话
    let session_id = {
        let mut state = app_state.state.write().await;
        let session = state.register_session();
        session.session_id.clone()
    };

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: id.clone(),
        method: method.to_string(),
        params,
    };

    let response = match method {
        "initialize" => handle_initialize(app_state, &session_id, &request).await,
        "ping" => JsonRpcResponse::success(id, serde_json::json!({})),
        "tools/list" => handle_tools_list(app_state, &session_id, &request).await,
        "tools/call" => handle_tools_call(app_state, &session_id, &request).await,
        "resources/list" => handle_resources_list(&request).await,
        "resources/read" => handle_resources_read(&request).await,
        _ => JsonRpcResponse::method_not_found(id),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// 处理 MCP JSON-RPC Notification (无 id)
///
/// 返回 HTTP 202 Accepted (无响应体)
async fn handle_mcp_notification(
    app_state: &GatewayAppState,
    headers: &HeaderMap,
    body: &serde_json::Value,
) -> Response {
    let method = body
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // initialized notification 是特殊的 - 标记会话初始化完成
    if method == "notifications/initialized" || method == "initialized" {
        if let Some(sid) = headers
            .get(MCP_SESSION_ID_HEADER)
            .and_then(|v| v.to_str().ok())
        {
            let mut store = app_state.mcp_sessions.write().await;
            if let Some(session) = store.get_session_mut(sid) {
                session.mark_initialized();
            }
        }
    }

    // 返回 202 Accepted
    (StatusCode::ACCEPTED, "").into_response()
}

/// MCP Session 清理守卫
///
/// 当此结构体被 drop 时，自动从 MCP Session Store 中移除对应的会话
struct McpSessionCleanupGuard {
    session_id: Option<String>,
    session_store: SharedMcpSessionStore,
}

impl Drop for McpSessionCleanupGuard {
    fn drop(&mut self) {
        if let Some(ref session_id) = self.session_id {
            let sid = session_id.clone();
            let store = self.session_store.clone();
            // 在后台异步清理会话
            tokio::spawn(async move {
                let mut store = store.write().await;
                store.remove_session(&sid);
            });
        }
    }
}

/// GET /mcp - MCP Streamable HTTP SSE 端点
///
/// 建立 SSE 流用于服务端推送消息。
/// 服务端应立即发送一个包含 event ID 和空 data 的 SSE 事件（priming event）。
///
/// 注意：GET SSE 流上不得发送 JSON-RPC response
pub async fn mcp_get_handler(
    State(app_state): State<GatewayAppState>,
    headers: HeaderMap,
) -> Response {
    // 增加连接计数
    app_state.stats.increment_connections();

    // 1. Origin 验证 (MUST)
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok());
    if !validate_origin(origin, None) {
        return forbidden_origin_response();
    }

    // 2. 验证 Accept Header
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !accept.contains("text/event-stream") {
        return (
            StatusCode::NOT_ACCEPTABLE,
            Json(JsonRpcResponse::error(
                None,
                -32001,
                "Accept header must include text/event-stream".to_string(),
            )),
        )
            .into_response();
    }

    // 3. 提取 MCP-Session-Id（可选，GET 可以不需要）
    let mcp_session_id = headers
        .get(MCP_SESSION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 验证 Session ID（如果提供）
    if let Some(ref sid) = mcp_session_id {
        let store = app_state.mcp_sessions.read().await;
        if !store.is_session_valid(sid) {
            return session_not_found_response(sid);
        }
    }

    // 4. 创建 SSE 流
    // 发送 priming event（规范要求：立即发送包含 event ID 和空 data 的初始事件）
    let priming_event_id = uuid::Uuid::new_v4().to_string();
    let session_id_for_heartbeat = mcp_session_id.clone();
    let session_id_for_cleanup = mcp_session_id.clone();
    let sessions_for_heartbeat = app_state.mcp_sessions.clone();
    let sessions_for_cleanup = app_state.mcp_sessions.clone();

    // 创建 MCP Session 清理守卫 - 当 SSE 流被 drop 时自动清理会话 (M3 修复)
    let cleanup_guard = McpSessionCleanupGuard {
        session_id: session_id_for_cleanup,
        session_store: sessions_for_cleanup,
    };

    let stream = stream::once(async move {
        // Priming event: 空 data + event ID，用于客户端断线重连
        Ok::<_, Infallible>(
            Event::default()
                .id(priming_event_id)
                .data(""),
        )
    })
    .chain(
        // 心跳流 - 每 30 秒发送一次
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(30)))
            .map(move |_| {
                // 更新 MCP 会话活跃时间
                if let Some(ref sid) = session_id_for_heartbeat {
                    let store = sessions_for_heartbeat.clone();
                    let sid = sid.clone();
                    tokio::spawn(async move {
                        let mut store = store.write().await;
                        if let Some(session) = store.get_session_mut(&sid) {
                            session.touch();
                        }
                    });
                }
                Ok::<_, Infallible>(Event::default().comment("keepalive"))
            }),
    )
    // 包装流以确保在 drop 时触发 MCP Session 清理
    .map(move |event| {
        // 保持 cleanup_guard 存活直到流结束
        let _guard = &cleanup_guard;
        event
    });

    // 5. 构建 SSE 响应，带 MCP-Session-Id Header（如果有）
    let sse = Sse::new(stream).keep_alive(KeepAlive::default());
    let mut response = sse.into_response();

    if let Some(sid) = mcp_session_id {
        let (header_name, header_value) = create_session_id_header(&sid);
        response.headers_mut().insert(header_name, header_value);
    }

    response
}

/// DELETE /mcp - MCP 会话终止端点
///
/// 终止指定的 MCP 会话
pub async fn mcp_delete_handler(
    State(app_state): State<GatewayAppState>,
    headers: HeaderMap,
) -> Response {
    // 1. Origin 验证 (MUST)
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok());
    if !validate_origin(origin, None) {
        return forbidden_origin_response();
    }

    // 2. 提取 MCP-Session-Id
    let session_id = match headers
        .get(MCP_SESSION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
    {
        Some(sid) => sid.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(JsonRpcResponse::error(
                    None,
                    -32003,
                    "Missing MCP-Session-Id header".to_string(),
                )),
            )
                .into_response();
        }
    };

    // 3. 终止会话
    let mut store = app_state.mcp_sessions.write().await;
    match store.remove_session(&session_id) {
        Some(_) => {
            // 会话已终止
            StatusCode::OK.into_response()
        }
        None => {
            // 会话不存在
            session_not_found_response(&session_id)
        }
    }
}

/// 生成 403 Forbidden Origin 响应
fn forbidden_origin_response() -> Response {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": {
            "code": -32001,
            "message": "Forbidden: Invalid origin"
        }
    });
    (StatusCode::FORBIDDEN, Json(response)).into_response()
}

/// 生成 404 Session Not Found 响应
fn session_not_found_response(session_id: &str) -> Response {
    let response = JsonRpcResponse::error(
        None,
        -32002,
        format!(
            "Session not found or expired: {}. Please reinitialize.",
            session_id
        ),
    );
    (StatusCode::NOT_FOUND, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(Some(serde_json::json!(1)), serde_json::json!({"result": "ok"}));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.jsonrpc, "2.0");
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(Some(serde_json::json!(1)), -32600, "Test error".to_string());
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Test error");
    }

    #[test]
    fn test_json_rpc_method_not_found() {
        let response = JsonRpcResponse::method_not_found(Some(serde_json::json!(1)));
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
    }

    #[test]
    fn test_json_rpc_parse_error() {
        let response = JsonRpcResponse::parse_error();
        assert!(response.id.is_none());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32700);
    }

    // ===== Story 11.5: 上下文路由测试 =====

    #[test]
    fn test_parse_work_dir_from_root_uri() {
        let params = serde_json::json!({
            "rootUri": "file:///home/user/projects/mantra"
        });

        let result = parse_work_dir_from_params(&params);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
    }

    #[test]
    fn test_parse_work_dir_from_workspace_folders() {
        let params = serde_json::json!({
            "workspaceFolders": [
                {
                    "uri": "file:///home/user/projects/mantra",
                    "name": "mantra"
                }
            ]
        });

        let result = parse_work_dir_from_params(&params);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
    }

    #[test]
    fn test_parse_work_dir_workspace_folders_priority() {
        // workspaceFolders 应该优先于 rootUri
        let params = serde_json::json!({
            "rootUri": "file:///other/path",
            "workspaceFolders": [
                {
                    "uri": "file:///home/user/projects/mantra",
                    "name": "mantra"
                }
            ]
        });

        let result = parse_work_dir_from_params(&params);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
    }

    #[test]
    fn test_parse_work_dir_from_root_path() {
        let params = serde_json::json!({
            "rootPath": "/home/user/projects/mantra"
        });

        let result = parse_work_dir_from_params(&params);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
    }

    #[test]
    fn test_parse_work_dir_no_params() {
        let params = serde_json::json!({});

        let result = parse_work_dir_from_params(&params);
        assert!(result.is_none());
    }

    #[test]
    fn test_uri_to_path_unix() {
        let result = uri_to_path("file:///home/user/projects");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects"));
    }

    #[test]
    fn test_uri_to_path_with_spaces() {
        let result = uri_to_path("file:///home/user/my%20projects");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/my projects"));
    }

    #[test]
    fn test_uri_to_path_invalid() {
        let result = uri_to_path("http://example.com");
        assert!(result.is_none());
    }

    // ===== Story 11.5: tools/call 参数验证测试 =====

    /// 创建测试用的 GatewayAppState
    fn create_test_app_state() -> GatewayAppState {
        let state = Arc::new(RwLock::new(GatewayState::with_defaults()));
        let stats = Arc::new(GatewayStats::new());
        GatewayAppState::new(state, stats)
    }

    /// 创建带有已注册会话的测试 GatewayAppState
    fn create_test_app_state_with_session() -> (GatewayAppState, String) {
        let mut gateway_state = GatewayState::with_defaults();
        let session = gateway_state.register_session();
        let session_id = session.session_id.clone();

        let state = Arc::new(RwLock::new(gateway_state));
        let stats = Arc::new(GatewayStats::new());
        let app_state = GatewayAppState::new(state, stats);
        (app_state, session_id)
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_params() {
        let app_state = create_test_app_state();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/call".to_string(),
            params: None, // 缺少 params
        };

        let response = handle_tools_call(&app_state, "test-session", &request).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602); // Invalid params
        assert!(error.message.contains("Missing params"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_tool_name() {
        let app_state = create_test_app_state();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "arguments": {}
            })), // 缺少 name
        };

        let response = handle_tools_call(&app_state, "test-session", &request).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing tool name"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_invalid_tool_name_format() {
        let app_state = create_test_app_state();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "invalid_tool_name_without_slash",
                "arguments": {}
            })),
        };

        let response = handle_tools_call(&app_state, "test-session", &request).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Invalid tool name format"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_valid_format_not_implemented() {
        let app_state = create_test_app_state();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "service_name/tool_name",
                "arguments": {"key": "value"}
            })),
        };

        let response = handle_tools_call(&app_state, "test-session", &request).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        // 当前返回 -32603 (Internal error) 因为转发未实现
        assert_eq!(error.code, -32603);
        assert!(error.message.contains("not yet implemented"));
    }

    // ===== Story 11.5: tools/list 测试 =====

    #[tokio::test]
    async fn test_handle_tools_list_returns_empty_list() {
        let app_state = create_test_app_state();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(&app_state, "test-session", &request).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert!(tools.is_empty());
    }

    // ===== Story 11.5: initialize 测试 =====

    #[tokio::test]
    async fn test_handle_initialize_stores_work_dir() {
        let (app_state, session_id) = create_test_app_state_with_session();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "rootUri": "file:///home/user/projects/test"
            })),
        };

        let response = handle_initialize(&app_state, &session_id, &request).await;
        assert!(response.error.is_none());

        // 验证 work_dir 已存储
        let state_guard = app_state.state.read().await;
        let session = state_guard.get_session(&session_id).unwrap();
        assert!(session.work_dir.is_some());
        assert_eq!(
            session.work_dir.as_ref().unwrap(),
            &PathBuf::from("/home/user/projects/test")
        );
    }

    #[tokio::test]
    async fn test_handle_initialize_no_work_dir() {
        let (app_state, session_id) = create_test_app_state_with_session();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "capabilities": {}
            })),
        };

        let response = handle_initialize(&app_state, &session_id, &request).await;
        assert!(response.error.is_none());

        // 验证 work_dir 为 None
        let state_guard = app_state.state.read().await;
        let session = state_guard.get_session(&session_id).unwrap();
        assert!(session.work_dir.is_none());
    }

    #[tokio::test]
    async fn test_handle_initialize_with_workspace_folders() {
        let (app_state, session_id) = create_test_app_state_with_session();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "workspaceFolders": [
                    {
                        "uri": "file:///home/user/workspace/project1",
                        "name": "project1"
                    },
                    {
                        "uri": "file:///home/user/workspace/project2",
                        "name": "project2"
                    }
                ]
            })),
        };

        let response = handle_initialize(&app_state, &session_id, &request).await;
        assert!(response.error.is_none());

        // 验证 work_dir 使用第一个 workspace folder
        let state_guard = app_state.state.read().await;
        let session = state_guard.get_session(&session_id).unwrap();
        assert!(session.work_dir.is_some());
        assert_eq!(
            session.work_dir.as_ref().unwrap(),
            &PathBuf::from("/home/user/workspace/project1")
        );
    }

    // ===== Story 11.10: Tool Policy 拦截测试 =====

    #[test]
    fn test_is_tool_blocked_allow_all() {
        use crate::models::mcp::{ToolPolicy, ToolPolicyMode};

        let policy = ToolPolicy {
            mode: ToolPolicyMode::AllowAll,
            allowed_tools: vec![],
            denied_tools: vec![],
        };

        // AllowAll 模式下，所有工具都被允许
        assert!(!is_tool_blocked("read_file", &policy));
        assert!(!is_tool_blocked("write_file", &policy));
    }

    #[test]
    fn test_is_tool_blocked_deny_all() {
        use crate::models::mcp::{ToolPolicy, ToolPolicyMode};

        let policy = ToolPolicy {
            mode: ToolPolicyMode::DenyAll,
            allowed_tools: vec![],
            denied_tools: vec![],
        };

        // DenyAll 模式下，所有工具都被阻止
        assert!(is_tool_blocked("read_file", &policy));
        assert!(is_tool_blocked("write_file", &policy));
    }

    #[test]
    fn test_is_tool_blocked_custom() {
        use crate::models::mcp::{ToolPolicy, ToolPolicyMode};

        let policy = ToolPolicy {
            mode: ToolPolicyMode::Custom,
            allowed_tools: vec!["read_file".to_string()],
            denied_tools: vec![],
        };

        // Custom 模式下，只有 allowed_tools 中的工具被允许
        assert!(!is_tool_blocked("read_file", &policy));
        assert!(is_tool_blocked("write_file", &policy));
    }

    #[test]
    fn test_is_tool_blocked_denied_overrides() {
        use crate::models::mcp::{ToolPolicy, ToolPolicyMode};

        let policy = ToolPolicy {
            mode: ToolPolicyMode::AllowAll,
            allowed_tools: vec![],
            denied_tools: vec!["write_file".to_string()],
        };

        // denied_tools 优先级最高
        assert!(!is_tool_blocked("read_file", &policy));
        assert!(is_tool_blocked("write_file", &policy));
    }

    #[test]
    fn test_tool_blocked_error_response() {
        let response = tool_blocked_error(Some(serde_json::json!(1)), "git-mcp/write_file");
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Tool not found"));
        assert!(error.message.contains("git-mcp/write_file"));
    }
}
