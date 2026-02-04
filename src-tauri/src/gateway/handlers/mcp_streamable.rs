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
use futures::stream::{self};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;

use crate::gateway::origin::validate_origin;
use crate::gateway::session::{create_session_id_header, MCP_SESSION_ID_HEADER};

use super::{
    forbidden_origin_response, parse_work_dir_from_params, session_not_found_response,
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
        "resources/list" => handle_resources_list(app_state, &request).await,
        "resources/read" => handle_resources_read(app_state, &request).await,
        "prompts/list" => handle_prompts_list(app_state, &request).await,
        "prompts/get" => handle_prompts_get(app_state, &request).await,
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
        "resources/list" => handle_resources_list(app_state, &request).await,
        "resources/read" => handle_resources_read(app_state, &request).await,
        "prompts/list" => handle_prompts_list(app_state, &request).await,
        "prompts/get" => handle_prompts_get(app_state, &request).await,
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
