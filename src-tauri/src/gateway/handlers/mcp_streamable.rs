//! MCP Streamable HTTP 端点处理器
//!
//! Story 11.14: MCP Streamable HTTP 规范合规 - Task 3

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use async_stream::stream;
use futures::stream::{self};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;

use crate::gateway::origin::validate_origin;
use crate::gateway::session::{create_session_id_header, MCP_SESSION_ID_HEADER};
use crate::gateway::uri_to_local_path;

use super::{
    forbidden_origin_response, session_not_found_response,
    GatewayAppState, JsonRpcRequest, JsonRpcResponse, McpSessionCleanupGuard,
};
use super::methods::{
    handle_initialize, handle_prompts_get, handle_prompts_list, handle_resources_list,
    handle_resources_read, handle_tools_call, handle_tools_list,
};

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
        // JSON-RPC Response - Client 对 Server 请求的响应 (Story 11.26)
        handle_mcp_client_response(&app_state, &headers, &body).await
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
        // Story 11.28: 传递 session_id 用于严格模式过滤
        "resources/list" => handle_resources_list(app_state, &session_id, &request).await,
        "resources/read" => handle_resources_read(app_state, &request).await,
        "prompts/list" => handle_prompts_list(app_state, &session_id, &request).await,
        "prompts/get" => handle_prompts_get(app_state, &request).await,
        _ => JsonRpcResponse::method_not_found(id),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// 处理 MCP initialize 请求
///
/// 创建新的 MCP 会话，返回 MCP-Session-Id Header
/// Story 11.26: 解析 roots capability
/// Story 11.26 Fix: 若支持 roots，返回 SSE 流以发送 roots/list 请求
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

    // 2. 解析 roots capability (Story 11.26 AC1)
    let (supports_roots, roots_list_changed) = if let Some(ref p) = params {
        // DEBUG: 打印 initialize 参数，用于分析 Claude Code 发送的内容
        eprintln!("[Gateway] DEBUG initialize params for session {}: {}", session_id, serde_json::to_string_pretty(p).unwrap_or_default());

        let (supports, list_changed) = parse_roots_capability_from_params(p);
        let mut store = app_state.mcp_sessions.write().await;
        if let Some(session) = store.get_session_mut(&session_id) {
            session.set_roots_capability(supports, list_changed);
        }

        // 记录日志 (Story 11.26 AC5)
        if supports {
            eprintln!(
                "[Gateway] MCP Session {} supports roots capability (listChanged: {})",
                session_id, list_changed
            );
        } else {
            eprintln!(
                "[Gateway] MCP Session {} does not support roots capability, using global services",
                session_id
            );
        }
        (supports, list_changed)
    } else {
        (false, false)
    };

    // 3. 同时注册到旧的 session 系统（向后兼容）
    {
        let mut state = app_state.state.write().await;
        let _old_session = state.register_session();
    }

    // 4. 根据 roots capability 决定响应类型
    // Story 11.26 Fix: 如果支持 roots，使用 SSE 流响应以便发送 roots/list 请求
    if supports_roots {
        sse_initialize_response(app_state, &session_id, id, roots_list_changed).await
    } else {
        json_initialize_response(&session_id, id).await
    }
}

/// 返回普通 JSON 的 initialize 响应（不支持 roots 时使用）
///
/// Story 11.26 Fix: 抽取为独立函数，向后兼容不支持 roots 的 Client
async fn json_initialize_response(
    session_id: &str,
    id: Option<serde_json::Value>,
) -> Response {
    // 构建初始化响应
    // Story 11.17: 声明完整的 tools/resources/prompts capabilities
    let result = serde_json::json!({
        "protocolVersion": "2025-03-26",
        "capabilities": {
            "tools": { "listChanged": true },
            "resources": { "subscribe": true, "listChanged": true },
            "prompts": { "listChanged": true }
        },
        "serverInfo": {
            "name": "mantra-gateway",
            "version": env!("CARGO_PKG_VERSION")
        }
    });

    let json_response = JsonRpcResponse::success(id, result);

    // 返回带 MCP-Session-Id Header 的响应
    let (header_name, header_value) = create_session_id_header(session_id);
    let mut response = (StatusCode::OK, Json(json_response)).into_response();
    response.headers_mut().insert(header_name, header_value);
    response
}

/// 返回 SSE 流的 initialize 响应（支持 roots 时使用）
///
/// Story 11.26 Fix: 在 SSE 流中发送 roots/list 请求
///
/// 流程:
/// 1. 发送 priming event
/// 2. 发送 roots/list 请求
/// 3. 等待 Client POST 响应（带超时）
/// 4. 处理 roots 响应
/// 5. 发送 InitializeResult
async fn sse_initialize_response(
    app_state: &GatewayAppState,
    session_id: &str,
    request_id: Option<serde_json::Value>,
    _roots_list_changed: bool,
) -> Response {
    let session_id = session_id.to_string();
    let session_id_for_header = session_id.clone();
    let app_state = app_state.clone();

    // 生成 roots/list 请求 ID
    let roots_request_id = format!("gateway-roots-{}", uuid::Uuid::new_v4());

    // 预先注册 pending request
    let roots_rx = app_state
        .s2c_manager
        .register_pending_request(&session_id, &roots_request_id)
        .await;

    // 保存 pending 请求 ID 到 session
    {
        let mut store = app_state.mcp_sessions.write().await;
        if let Some(session) = store.get_session_mut(&session_id) {
            session.pending_roots_request_id = Some(roots_request_id.clone());
        }
    }

    // 构建 SSE 流
    let stream = stream! {
        // 1. Priming event
        let priming_id = uuid::Uuid::new_v4().to_string();
        eprintln!("[Gateway] SSE initialize: sending priming event for session {}", session_id);
        yield Ok::<_, Infallible>(Event::default().id(priming_id).data(""));

        // 2. 发送 roots/list 请求
        let roots_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": roots_request_id,
            "method": "roots/list"
        }).to_string();

        eprintln!("[Gateway] SSE initialize: sending roots/list request {} for session {}", roots_request_id, session_id);
        yield Ok(Event::default().data(roots_request));

        // 3. 等待 Client POST 响应（10秒超时）
        const ROOTS_REQUEST_TIMEOUT_SECS: u64 = 10;
        let roots_result = tokio::time::timeout(
            Duration::from_secs(ROOTS_REQUEST_TIMEOUT_SECS),
            roots_rx,
        ).await;

        // 4. 清理 pending 请求 ID
        {
            let mut store = app_state.mcp_sessions.write().await;
            if let Some(session) = store.get_session_mut(&session_id) {
                session.pending_roots_request_id = None;
            }
        }

        // 5. 处理 roots 响应
        match roots_result {
            Ok(Ok(response)) => {
                eprintln!("[Gateway] SSE initialize: received roots/list response for session {}", session_id);
                handle_roots_list_response(&app_state, &session_id, &response).await;
            }
            Ok(Err(_)) => {
                eprintln!("[Gateway] SSE initialize: roots/list response channel closed for session {}", session_id);
                // 清理 pending request
                app_state.s2c_manager.cancel_pending_request(&roots_request_id).await;
            }
            Err(_) => {
                eprintln!("[Gateway] SSE initialize: roots/list request timed out for session {}", session_id);
                // Story 11.26 AC7: 超时处理
                {
                    let mut store = app_state.mcp_sessions.write().await;
                    if let Some(session) = store.get_session_mut(&session_id) {
                        session.roots_request_timed_out = true;
                        eprintln!("[Gateway] Session {} marked as roots request timed out, using global services", session_id);
                    }
                }
                // 清理 pending request
                app_state.s2c_manager.cancel_pending_request(&roots_request_id).await;
            }
        }

        // 6. 发送 InitializeResult
        let init_result = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "protocolVersion": "2025-03-26",
                "capabilities": {
                    "tools": { "listChanged": true },
                    "resources": { "subscribe": true, "listChanged": true },
                    "prompts": { "listChanged": true }
                },
                "serverInfo": {
                    "name": "mantra-gateway",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        }).to_string();

        eprintln!("[Gateway] SSE initialize: sending InitializeResult for session {}", session_id);
        yield Ok(Event::default().data(init_result));

        eprintln!("[Gateway] SSE initialize: stream complete for session {}", session_id);
    };

    // 构建 SSE 响应
    let sse = Sse::new(stream);
    let mut response = sse.into_response();

    // 添加 MCP-Session-Id Header
    let (header_name, header_value) = create_session_id_header(&session_id_for_header);
    response.headers_mut().insert(header_name, header_value);

    response
}

/// 从 initialize 参数解析 roots capability
///
/// Story 11.26: AC1
///
/// 返回 (supports_roots, list_changed)
pub(super) fn parse_roots_capability_from_params(params: &serde_json::Value) -> (bool, bool) {
    if let Some(caps) = params.get("capabilities") {
        if let Some(roots) = caps.get("roots") {
            let list_changed = roots
                .get("listChanged")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            return (true, list_changed);
        }
    }
    (false, false)
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
        // Story 11.28: 传递 session_id 用于严格模式过滤
        "resources/list" => handle_resources_list(app_state, &session_id, &request).await,
        "resources/read" => handle_resources_read(app_state, &request).await,
        "prompts/list" => handle_prompts_list(app_state, &session_id, &request).await,
        "prompts/get" => handle_prompts_get(app_state, &request).await,
        _ => JsonRpcResponse::method_not_found(id),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// 处理 MCP JSON-RPC Notification (无 id)
///
/// 返回 HTTP 202 Accepted (无响应体)
/// Story 11.26: 处理 initialized 通知，触发 roots/list 请求
async fn handle_mcp_notification(
    app_state: &GatewayAppState,
    headers: &HeaderMap,
    body: &serde_json::Value,
) -> Response {
    let method = body
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // 提取 session ID
    let session_id = headers
        .get(MCP_SESSION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // initialized notification 是特殊的 - 标记会话初始化完成
    if method == "notifications/initialized" || method == "initialized" {
        if let Some(ref sid) = session_id {
            // 标记会话为已初始化
            {
                let mut store = app_state.mcp_sessions.write().await;
                if let Some(session) = store.get_session_mut(sid) {
                    session.mark_initialized();
                }
            }

            // Story 11.26 AC2: 如果支持 roots capability，发送 roots/list 请求
            let supports_roots = {
                let store = app_state.mcp_sessions.read().await;
                store.get_session(sid)
                    .map(|s| s.supports_roots)
                    .unwrap_or(false)
            };

            if supports_roots {
                // 在后台触发 roots/list 请求
                let app_state_clone = app_state.clone();
                let sid_clone = sid.clone();
                tokio::spawn(async move {
                    handle_roots_list_request(&app_state_clone, &sid_clone).await;
                });
            }
        }
    }

    // Story 11.26 AC4: roots/list_changed 通知处理
    // 当 Client 通知 roots 列表变更时，重新请求 roots/list
    if method == "notifications/roots/list_changed" {
        if let Some(ref sid) = session_id {
            // 检查 session 是否支持 roots_list_changed
            let (supports_roots, roots_list_changed) = {
                let store = app_state.mcp_sessions.read().await;
                store.get_session(sid)
                    .map(|s| (s.supports_roots, s.roots_list_changed))
                    .unwrap_or((false, false))
            };

            if supports_roots && roots_list_changed {
                eprintln!("[Gateway] Received roots/list_changed notification for session {}", sid);
                // 在后台重新请求 roots/list
                let app_state_clone = app_state.clone();
                let sid_clone = sid.clone();
                tokio::spawn(async move {
                    handle_roots_list_request(&app_state_clone, &sid_clone).await;
                });
            } else if !roots_list_changed {
                eprintln!("[Gateway] Received roots/list_changed notification but session {} does not support listChanged", sid);
            }
        }
    }

    // 返回 202 Accepted
    (StatusCode::ACCEPTED, "").into_response()
}

/// 发送 roots/list 请求并处理响应
///
/// Story 11.26: Task 4.3, Task 5
///
/// 通过 SSE 流发送 roots/list 请求，等待 Client 响应，
/// 然后解析 roots 路径并设置项目上下文。
async fn handle_roots_list_request(
    app_state: &GatewayAppState,
    session_id: &str,
) {
    // 1. 生成唯一的请求 ID
    let request_id = format!("gateway-roots-{}", uuid::Uuid::new_v4());

    // 2. 保存 pending 请求 ID 到 session
    {
        let mut store = app_state.mcp_sessions.write().await;
        if let Some(session) = store.get_session_mut(session_id) {
            session.pending_roots_request_id = Some(request_id.clone());
        }
    }

    // 3. 构造 roots/list 请求
    let request_json = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "roots/list"
    }).to_string();

    eprintln!("[Gateway] Sending roots/list request to session {}: {}", session_id, request_id);

    // 4. 发送请求并等待响应 (10秒超时)
    const ROOTS_REQUEST_TIMEOUT_SECS: u64 = 10;
    let result = app_state.s2c_manager.send_request_and_wait(
        session_id,
        &request_id,
        request_json,
        ROOTS_REQUEST_TIMEOUT_SECS,
    ).await;

    // 5. 清理 pending 请求 ID
    {
        let mut store = app_state.mcp_sessions.write().await;
        if let Some(session) = store.get_session_mut(session_id) {
            session.pending_roots_request_id = None;
        }
    }

    // 6. 处理响应
    match result {
        Ok(response) => {
            eprintln!("[Gateway] Received roots/list response for session {}", session_id);
            handle_roots_list_response(app_state, session_id, &response).await;
        }
        Err(e) => {
            eprintln!("[Gateway] roots/list request failed for session {}: {}", session_id, e);
            // Story 11.26 AC7: 超时处理
            {
                let mut store = app_state.mcp_sessions.write().await;
                if let Some(session) = store.get_session_mut(session_id) {
                    session.roots_request_timed_out = true;
                    eprintln!("[Gateway] Session {} marked as roots request timed out, using global services", session_id);
                }
            }
        }
    }
}

/// 处理 roots/list 响应
///
/// Story 11.26: Task 5
/// Story 11.27: Task 3 - LPM 集成
///
/// 1. 解析 roots 数组中的 URI 和 name
/// 2. 将 file:// URI 转换为本地路径
/// 3. 保存 roots_paths 到 session
/// 4. (Story 11.27) 调用 LPM 查询匹配项目
async fn handle_roots_list_response(
    app_state: &GatewayAppState,
    session_id: &str,
    response: &serde_json::Value,
) {
    use crate::gateway::state::SessionProjectContext;

    // 检查是否有错误
    if let Some(error) = response.get("error") {
        eprintln!("[Gateway] roots/list returned error for session {}: {:?}", session_id, error);
        return;
    }

    // 解析 result.roots 数组
    let roots = match response
        .get("result")
        .and_then(|r| r.get("roots"))
        .and_then(|r| r.as_array())
    {
        Some(r) => r,
        None => {
            eprintln!("[Gateway] roots/list response missing roots array for session {}", session_id);
            return;
        }
    };

    // 解析每个 root 的 URI
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    for root in roots {
        if let Some(uri) = root.get("uri").and_then(|u| u.as_str()) {
            if let Some(path) = uri_to_local_path(uri) {
                let name = root.get("name").and_then(|n| n.as_str()).unwrap_or("<unnamed>");
                eprintln!("[Gateway] Parsed root: name={}, path={:?}", name, path);
                paths.push(path);
            } else {
                eprintln!("[Gateway] Failed to parse root URI: {}", uri);
            }
        }
    }

    if paths.is_empty() {
        eprintln!("[Gateway] No valid roots found for session {}", session_id);
        return;
    }

    // 保存 roots_paths 到 session
    {
        let mut store = app_state.mcp_sessions.write().await;
        if let Some(session) = store.get_session_mut(session_id) {
            session.set_roots_paths(paths.clone());
            // 使用第一个 root 作为工作目录
            if let Some(first_path) = paths.first() {
                session.set_work_dir(first_path.clone());
                eprintln!("[Gateway] Session {} work_dir set to: {:?}", session_id, first_path);
            }
        }
    }

    // Story 11.27: 调用 LPM 查询匹配项目
    // AC4: 按顺序遍历每个 root 路径，使用第一个成功匹配的项目
    if let Some(ref lpm_client) = app_state.lpm_client {
        let mut matched = false;
        for path in &paths {
            let path_str = path.to_string_lossy().to_string();
            eprintln!("[Gateway] Story 11.27 AC4: Querying LPM for root path: {}", path_str);

            // 执行 LPM 查询 (5 秒超时)
            if let Some(project_ctx) = lpm_client.query_project_by_path(&path_str).await {
                // Story 11.27 AC2: 匹配成功，设置项目上下文
                eprintln!(
                    "[Gateway] Story 11.27 AC2: Matched project '{}' from root: {} (session: {})",
                    project_ctx.project_name, path_str, session_id
                );

                // 设置 session 的 project_context
                let ctx = SessionProjectContext {
                    project_id: project_ctx.project_id,
                    project_name: project_ctx.project_name,
                    matched_path: project_ctx.matched_path,
                    is_manual_override: false, // roots 自动匹配
                };

                let mut store = app_state.mcp_sessions.write().await;
                if let Some(session) = store.get_session_mut(session_id) {
                    session.set_project_context(ctx);
                    eprintln!("[Gateway] Session {} project_context set", session_id);
                }
                matched = true;
                break; // AC4: 使用第一个成功匹配的项目
            } else {
                // 当前 root 未匹配，继续遍历下一个
                eprintln!(
                    "[Gateway] Story 11.27: No match for root path: {}, trying next...",
                    path_str
                );
            }
        }

        if !matched {
            // Story 11.27 AC3 + AC5: 无匹配项目，清除旧的项目上下文
            eprintln!(
                "[Gateway] Story 11.27 AC3: No Mantra project found for any roots path (session: {})",
                session_id
            );
            let mut store = app_state.mcp_sessions.write().await;
            if let Some(session) = store.get_session_mut(session_id) {
                // AC5: roots 变更时允许从有项目变为无项目
                session.project_context = None;
            }
        }
    } else {
        // LPM 客户端未配置，跳过项目匹配
        eprintln!("[Gateway] LPM client not configured, skipping project matching for session {}", session_id);
    }

    eprintln!("[Gateway] roots/list completed for session {}, {} paths parsed", session_id, paths.len());
}

/// 处理来自 Client 的 JSON-RPC Response
///
/// Story 11.26: Task 3.5 - 处理 Client 对 Server 发起请求的响应
///
/// Client 通过 POST /mcp 发送匹配 Server 请求 ID 的响应
async fn handle_mcp_client_response(
    app_state: &GatewayAppState,
    _headers: &HeaderMap,
    body: &serde_json::Value,
) -> Response {
    // 提取响应的 ID
    let response_id = match body.get("id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            // 尝试从数字 ID 获取
            match body.get("id").and_then(|v| v.as_i64()) {
                Some(id) => id.to_string(),
                None => {
                    eprintln!("[Gateway] Received client response without valid id");
                    return (StatusCode::ACCEPTED, "").into_response();
                }
            }
        }
    };

    // 尝试将响应路由到 pending request
    let matched = app_state.s2c_manager.handle_client_response(&response_id, body.clone()).await;

    if matched {
        eprintln!("[Gateway] Matched client response for request_id: {}", response_id);
    } else {
        eprintln!("[Gateway] No pending request found for response_id: {}", response_id);
    }

    // 返回 202 Accepted
    (StatusCode::ACCEPTED, "").into_response()
}

/// GET /mcp - MCP Streamable HTTP SSE 端点
///
/// 建立 SSE 流用于服务端推送消息。
/// 服务端应立即发送一个包含 event ID 和空 data 的 SSE 事件（priming event）。
///
/// Story 11.26: SSE 流增加 Server-to-Client 消息注入通道
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
    let session_id_for_s2c = mcp_session_id.clone();
    let sessions_for_heartbeat = app_state.mcp_sessions.clone();
    let sessions_for_cleanup = app_state.mcp_sessions.clone();
    let s2c_manager = app_state.s2c_manager.clone();
    let s2c_manager_cleanup = app_state.s2c_manager.clone();

    // 创建 MCP Session 清理守卫 - 当 SSE 流被 drop 时自动清理会话 (M3 修复)
    let cleanup_guard = McpSessionCleanupGuard {
        session_id: session_id_for_cleanup,
        session_store: sessions_for_cleanup,
    };

    // Story 11.26: 注册 Server-to-Client 消息通道
    let s2c_receiver = if let Some(ref sid) = session_id_for_s2c {
        Some(s2c_manager.register_channel(sid, 16).await)
    } else {
        None
    };

    // 创建基础流：priming event + heartbeat
    let base_stream = stream::once(async move {
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
    );

    // Story 11.26: 合并 Server-to-Client 消息流
    let stream: std::pin::Pin<Box<dyn futures::Stream<Item = Result<Event, Infallible>> + Send>> =
        if let Some(rx) = s2c_receiver {
            let session_id_for_unregister = session_id_for_s2c.clone();

            // 创建 Server-to-Client 消息流
            use tokio_stream::StreamExt as TokioStreamExt;
            let s2c_stream = TokioStreamExt::map(
                tokio_stream::wrappers::ReceiverStream::new(rx),
                |msg| {
                    // 将 JSON-RPC 消息包装成 SSE 事件
                    // Story 11.26 AC6: 通过 SSE 流发送 JSON-RPC 请求
                    Ok::<_, Infallible>(Event::default().data(msg))
                }
            );

            // 合并两个流
            use futures::stream::StreamExt as FuturesStreamExt;
            let merged = futures::stream::select(base_stream, s2c_stream);

            // 包装以在流结束时清理
            Box::pin(FuturesStreamExt::chain(
                FuturesStreamExt::map(merged, move |event| {
                    let _guard = &cleanup_guard;
                    event
                }),
                stream::once(async move {
                    // 流结束时注销通道
                    if let Some(ref sid) = session_id_for_unregister {
                        s2c_manager_cleanup.unregister_channel(sid).await;
                    }
                    // 返回一个空事件（不会被发送）
                    Ok::<_, Infallible>(Event::default().comment("cleanup"))
                })
            ))
        } else {
            // 没有 session ID，只使用基础流
            use futures::stream::StreamExt as FuturesStreamExt;
            Box::pin(FuturesStreamExt::map(base_stream, move |event| {
                let _guard = &cleanup_guard;
                event
            }))
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Story 11.26: uri_to_local_path 测试 =====

    #[test]
    fn test_uri_to_local_path_unix() {
        let result = uri_to_local_path("file:///home/user/projects");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/home/user/projects"));
    }

    #[test]
    fn test_uri_to_local_path_with_spaces() {
        let result = uri_to_local_path("file:///home/user/my%20projects");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/home/user/my projects"));
    }

    #[test]
    fn test_uri_to_local_path_invalid_scheme() {
        let result = uri_to_local_path("http://example.com");
        assert!(result.is_none());
    }

    #[test]
    fn test_uri_to_local_path_empty() {
        let result = uri_to_local_path("");
        assert!(result.is_none());
    }

    #[test]
    fn test_uri_to_local_path_with_unicode() {
        let result = uri_to_local_path("file:///home/user/%E9%A1%B9%E7%9B%AE");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/home/user/项目"));
    }

    // ===== Story 11.26: parse_roots_capability_from_params 测试 =====

    #[test]
    fn test_parse_roots_capability_with_list_changed() {
        let params = serde_json::json!({
            "capabilities": {
                "roots": {
                    "listChanged": true
                }
            }
        });
        let (supports, list_changed) = parse_roots_capability_from_params(&params);
        assert!(supports);
        assert!(list_changed);
    }

    #[test]
    fn test_parse_roots_capability_without_list_changed() {
        let params = serde_json::json!({
            "capabilities": {
                "roots": {}
            }
        });
        let (supports, list_changed) = parse_roots_capability_from_params(&params);
        assert!(supports);
        assert!(!list_changed);
    }

    #[test]
    fn test_parse_roots_capability_no_roots() {
        let params = serde_json::json!({
            "capabilities": {
                "tools": {}
            }
        });
        let (supports, list_changed) = parse_roots_capability_from_params(&params);
        assert!(!supports);
        assert!(!list_changed);
    }

    #[test]
    fn test_parse_roots_capability_no_capabilities() {
        let params = serde_json::json!({
            "protocolVersion": "2025-03-26"
        });
        let (supports, list_changed) = parse_roots_capability_from_params(&params);
        assert!(!supports);
        assert!(!list_changed);
    }

    #[test]
    fn test_parse_roots_capability_empty_params() {
        let params = serde_json::json!({});
        let (supports, list_changed) = parse_roots_capability_from_params(&params);
        assert!(!supports);
        assert!(!list_changed);
    }

    #[test]
    fn test_parse_roots_capability_list_changed_false() {
        let params = serde_json::json!({
            "capabilities": {
                "roots": {
                    "listChanged": false
                }
            }
        });
        let (supports, list_changed) = parse_roots_capability_from_params(&params);
        assert!(supports);
        assert!(!list_changed);
    }

    // ===== Story 11.26: handle_roots_list_response 测试 =====

    use crate::gateway::state::{GatewayState, GatewayStats};
    use crate::gateway::session::McpSessionStore;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// 创建测试用的 GatewayAppState (带 MCP session)
    fn create_test_app_state_with_mcp_session() -> (GatewayAppState, String) {
        let state = Arc::new(RwLock::new(GatewayState::with_defaults()));
        let stats = Arc::new(GatewayStats::new());
        let mut mcp_store = McpSessionStore::new();
        let session = mcp_store.create_session();
        let session_id = session.session_id.clone();

        // 设置 supports_roots
        if let Some(s) = mcp_store.get_session_mut(&session_id) {
            s.set_roots_capability(true, true);
        }

        let mut app_state = GatewayAppState::new(state, stats);
        app_state.mcp_sessions = Arc::new(RwLock::new(mcp_store));
        (app_state, session_id)
    }

    #[tokio::test]
    async fn test_handle_roots_list_response_valid() {
        let (app_state, session_id) = create_test_app_state_with_mcp_session();

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "result": {
                "roots": [
                    {"uri": "file:///home/user/project1", "name": "project1"},
                    {"uri": "file:///home/user/project2", "name": "project2"}
                ]
            }
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证 roots_paths 已设置
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert_eq!(session.roots_paths.len(), 2);
        assert_eq!(session.roots_paths[0], std::path::PathBuf::from("/home/user/project1"));
        assert_eq!(session.roots_paths[1], std::path::PathBuf::from("/home/user/project2"));
        // 验证 work_dir 设置为第一个 root
        assert_eq!(session.work_dir, Some(std::path::PathBuf::from("/home/user/project1")));
    }

    #[tokio::test]
    async fn test_handle_roots_list_response_empty_roots() {
        let (app_state, session_id) = create_test_app_state_with_mcp_session();

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "result": {
                "roots": []
            }
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证 roots_paths 未设置
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert!(session.roots_paths.is_empty());
        assert!(session.work_dir.is_none());
    }

    #[tokio::test]
    async fn test_handle_roots_list_response_with_error() {
        let (app_state, session_id) = create_test_app_state_with_mcp_session();

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "error": {
                "code": -32601,
                "message": "Method not supported"
            }
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证 roots_paths 未设置（因为有错误）
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert!(session.roots_paths.is_empty());
    }

    #[tokio::test]
    async fn test_handle_roots_list_response_missing_roots_array() {
        let (app_state, session_id) = create_test_app_state_with_mcp_session();

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "result": {}
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证 roots_paths 未设置（因为没有 roots 数组）
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert!(session.roots_paths.is_empty());
    }

    #[tokio::test]
    async fn test_handle_roots_list_response_invalid_uri() {
        let (app_state, session_id) = create_test_app_state_with_mcp_session();

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "result": {
                "roots": [
                    {"uri": "http://invalid.com/path", "name": "invalid"},
                    {"uri": "file:///home/user/valid", "name": "valid"}
                ]
            }
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证只有有效的 file:// URI 被解析
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert_eq!(session.roots_paths.len(), 1);
        assert_eq!(session.roots_paths[0], std::path::PathBuf::from("/home/user/valid"));
    }

    // ===== Story 11.27: LPM 集成测试 =====

    use crate::gateway::lpm_query::{LpmQueryClient, LpmQueryResponse, SharedLpmQueryClient};

    /// 创建带 LPM 客户端的测试用 GatewayAppState
    fn create_test_app_state_with_lpm(
        lpm_client: SharedLpmQueryClient,
    ) -> (GatewayAppState, String) {
        let state = Arc::new(RwLock::new(GatewayState::with_defaults()));
        let stats = Arc::new(GatewayStats::new());
        let mut mcp_store = McpSessionStore::new();
        let session = mcp_store.create_session();
        let session_id = session.session_id.clone();

        if let Some(s) = mcp_store.get_session_mut(&session_id) {
            s.set_roots_capability(true, true);
        }

        let mut app_state = GatewayAppState::new(state, stats);
        app_state.mcp_sessions = Arc::new(RwLock::new(mcp_store));
        app_state.lpm_client = Some(lpm_client);
        (app_state, session_id)
    }

    /// Story 11.27 Task 5.3: 测试多 roots 遍历 - 第一个无匹配，第二个匹配
    #[tokio::test]
    async fn test_handle_roots_list_response_multi_roots_traversal() {
        use std::path::PathBuf;

        let (lpm_client, mut lpm_rx) = LpmQueryClient::new(16);
        let lpm_client = Arc::new(lpm_client);
        let (app_state, session_id) = create_test_app_state_with_lpm(lpm_client);

        // 启动模拟 LPM 服务 - 第一个路径无匹配，第二个路径匹配
        tokio::spawn(async move {
            let mut count = 0;
            while let Some(pending) = lpm_rx.recv().await {
                count += 1;
                let response = if count == 1 {
                    // 第一个 root 无匹配
                    LpmQueryResponse {
                        request_id: pending.request.request_id,
                        project_id: None,
                        project_name: None,
                        matched_path: None,
                    }
                } else {
                    // 第二个 root 匹配
                    LpmQueryResponse {
                        request_id: pending.request.request_id,
                        project_id: Some("proj-multi-root".to_string()),
                        project_name: Some("Multi Root Project".to_string()),
                        matched_path: Some(PathBuf::from("/home/user/project2")),
                    }
                };
                let _ = pending.response_tx.send(response);
            }
        });

        // 发送包含两个 root 的响应
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "result": {
                "roots": [
                    {"uri": "file:///home/user/project1", "name": "project1"},
                    {"uri": "file:///home/user/project2", "name": "project2"}
                ]
            }
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证：应该使用第二个 root 的匹配结果
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert!(session.project_context.is_some(), "应该设置 project_context");
        let ctx = session.project_context.as_ref().unwrap();
        assert_eq!(ctx.project_id, "proj-multi-root");
        assert_eq!(ctx.project_name, "Multi Root Project");
        assert!(!ctx.is_manual_override, "应该是自动匹配");
    }

    /// Story 11.27 Task 5.3: 测试单个 root 成功匹配
    #[tokio::test]
    async fn test_handle_roots_list_response_single_root_match() {
        use std::path::PathBuf;

        let (lpm_client, mut lpm_rx) = LpmQueryClient::new(16);
        let lpm_client = Arc::new(lpm_client);
        let (app_state, session_id) = create_test_app_state_with_lpm(lpm_client);

        // 启动模拟 LPM 服务 - 返回成功匹配
        tokio::spawn(async move {
            if let Some(pending) = lpm_rx.recv().await {
                let response = LpmQueryResponse {
                    request_id: pending.request.request_id,
                    project_id: Some("proj-single".to_string()),
                    project_name: Some("Single Root Project".to_string()),
                    matched_path: Some(PathBuf::from("/home/user/myproject")),
                };
                let _ = pending.response_tx.send(response);
            }
        });

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "result": {
                "roots": [
                    {"uri": "file:///home/user/myproject", "name": "myproject"}
                ]
            }
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证 project_context 已设置
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert!(session.project_context.is_some());
        let ctx = session.project_context.as_ref().unwrap();
        assert_eq!(ctx.project_id, "proj-single");
        assert_eq!(ctx.project_name, "Single Root Project");
    }

    /// Story 11.27 Task 5.5: 测试 roots 变更后无匹配清除旧上下文
    #[tokio::test]
    async fn test_handle_roots_list_response_clears_context_on_no_match() {
        use crate::gateway::state::SessionProjectContext;
        use std::path::PathBuf;

        let (lpm_client, mut lpm_rx) = LpmQueryClient::new(16);
        let lpm_client = Arc::new(lpm_client);
        let (app_state, session_id) = create_test_app_state_with_lpm(lpm_client);

        // 先设置一个旧的 project_context
        {
            let mut store = app_state.mcp_sessions.write().await;
            if let Some(session) = store.get_session_mut(&session_id) {
                session.set_project_context(SessionProjectContext {
                    project_id: "old-project".to_string(),
                    project_name: "Old Project".to_string(),
                    matched_path: PathBuf::from("/old/path"),
                    is_manual_override: false,
                });
            }
        }

        // 启动模拟 LPM 服务 - 返回无匹配
        tokio::spawn(async move {
            if let Some(pending) = lpm_rx.recv().await {
                let response = LpmQueryResponse {
                    request_id: pending.request.request_id,
                    project_id: None,
                    project_name: None,
                    matched_path: None,
                };
                let _ = pending.response_tx.send(response);
            }
        });

        // 发送新的 roots 响应 - 不匹配任何项目
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "result": {
                "roots": [
                    {"uri": "file:///some/unknown/path", "name": "unknown"}
                ]
            }
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证：旧的 project_context 应被清除
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert!(
            session.project_context.is_none(),
            "AC5: roots 变更无匹配时应清除旧的 project_context"
        );
    }

    /// Story 11.27: 测试无 LPM 客户端时跳过项目匹配
    #[tokio::test]
    async fn test_handle_roots_list_response_no_lpm_client() {
        let (app_state, session_id) = create_test_app_state_with_mcp_session();
        // 默认创建的 app_state 没有 lpm_client

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "result": {
                "roots": [
                    {"uri": "file:///home/user/project", "name": "project"}
                ]
            }
        });

        super::handle_roots_list_response(&app_state, &session_id, &response).await;

        // 验证：roots_paths 设置了，但 project_context 未设置（没有 LPM 客户端）
        let store = app_state.mcp_sessions.read().await;
        let session = store.get_session(&session_id).unwrap();
        assert_eq!(session.roots_paths.len(), 1);
        assert!(session.project_context.is_none(), "没有 LPM 客户端时不应设置 project_context");
    }
}
