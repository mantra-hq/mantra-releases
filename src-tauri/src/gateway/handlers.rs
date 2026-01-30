//! HTTP/SSE 请求处理器
//!
//! Story 11.1: SSE Server 核心 - Task 2 & Task 4
//! Story 11.5: 上下文路由 - Task 4 & Task 5
//!
//! 实现 `/sse` SSE 端点和 `/message` JSON-RPC 端点

use axum::{
    extract::{Query, State},
    http::StatusCode,
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

use super::state::{GatewayState, GatewayStats};

/// Message 端点查询参数
#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub session_id: String,
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
/// 
/// 注意：由于 rusqlite::Connection 不是 Send + Sync，
/// router 和 registry 需要通过 Tauri 状态管理在外部提供，
/// 而不是直接存储在这里。
#[derive(Clone)]
pub struct GatewayAppState {
    pub state: Arc<RwLock<GatewayState>>,
    pub stats: Arc<GatewayStats>,
}

impl GatewayAppState {
    /// 创建应用状态
    pub fn new(state: Arc<RwLock<GatewayState>>, stats: Arc<GatewayStats>) -> Self {
        Self { state, stats }
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
pub async fn sse_handler(
    State(app_state): State<GatewayAppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
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
/// 此故事仅实现框架，实际消息转发逻辑在后续故事实现
pub async fn message_handler(
    State(app_state): State<GatewayAppState>,
    Query(query): Query<MessageQuery>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    // 增加请求计数
    app_state.stats.increment_requests();

    // 验证会话存在
    {
        let state = app_state.state.read().await;
        if state.get_session(&query.session_id).is_none() {
            let response = JsonRpcResponse::error(
                request.id.clone(),
                -32002,
                format!("Session not found: {}", query.session_id),
            );
            return (StatusCode::NOT_FOUND, Json(response)).into_response();
        }
    }

    // 验证 JSON-RPC 版本
    if request.jsonrpc != "2.0" {
        return (StatusCode::BAD_REQUEST, Json(JsonRpcResponse::invalid_request(request.id)))
            .into_response();
    }

    // 更新会话活跃时间
    {
        let mut state = app_state.state.write().await;
        if let Some(session) = state.get_session_mut(&query.session_id) {
            session.touch();
        }
    }

    // 根据方法路由处理
    let response = match request.method.as_str() {
        "initialize" => {
            handle_initialize(&app_state, &query.session_id, &request).await
        }
        "ping" => {
            // 简单的 ping 方法
            JsonRpcResponse::success(request.id, serde_json::json!({}))
        }
        "tools/list" => {
            handle_tools_list(&app_state, &query.session_id, &request).await
        }
        "tools/call" => {
            handle_tools_call(&app_state, &query.session_id, &request).await
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
            "protocolVersion": "2024-11-05",
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
///
/// 返回空的工具列表。实际的工具列表将通过 MCP 子进程管理器
/// 在 Task 6/7 中实现。
///
/// 注意：由于 rusqlite 线程安全限制，服务列表查询将通过
/// Tauri IPC 命令在外部执行。
async fn handle_tools_list(
    _app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // 获取会话的项目上下文（用于日志）
    let _project_context = {
        let state = _app_state.state.read().await;
        state
            .get_session(session_id)
            .and_then(|s| s.get_effective_project().cloned())
    };

    // 当前返回空工具列表
    // 完整实现将在 Task 6/7 中通过 MCP 子进程管理器获取实际工具
    let tools: Vec<serde_json::Value> = Vec::new();

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "tools": tools
        }),
    )
}

/// 处理 tools/call 请求
///
/// Story 11.5: 上下文路由 - Task 7
///
/// 1. 解析工具名称和参数
/// 2. 路由到对应的 MCP 服务
/// 3. 转发请求并返回响应
///
/// 注意：由于 rusqlite 线程安全限制，实际的工具调用转发
/// 需要通过 Tauri IPC 命令在外部执行。当前实现返回占位响应。
async fn handle_tools_call(
    _app_state: &GatewayAppState,
    _session_id: &str,
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

    let _service_name = parts[0];
    let _actual_tool_name = parts[1];

    // 3. 当前返回占位响应
    // 完整实现需要：
    // - 查找服务配置
    // - 启动 MCP 子进程（如果未运行）
    // - 转发请求到子进程
    // - 返回子进程响应
    //
    // 由于 rusqlite 线程安全限制，这些操作需要通过 Tauri IPC 执行
    JsonRpcResponse::error(
        request.id.clone(),
        -32603,
        "Tool call forwarding not yet implemented. Use Tauri IPC commands.".to_string(),
    )
}

/// GET /health - 健康检查端点
pub async fn health_handler(State(app_state): State<GatewayAppState>) -> impl IntoResponse {
    let state = app_state.state.read().await;
    let active_connections = state.active_connections();
    let total_connections = app_state.stats.get_total_connections();
    let total_requests = app_state.stats.get_total_requests();

    Json(serde_json::json!({
        "status": "ok",
        "service": "mantra-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "stats": {
            "activeConnections": active_connections,
            "totalConnections": total_connections,
            "totalRequests": total_requests
        }
    }))
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
}
