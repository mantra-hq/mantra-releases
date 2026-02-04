//! 旧版 SSE/Message 端点处理器 (DEPRECATED)
//!
//! 请迁移至 MCP Streamable HTTP 端点 `/mcp`

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
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;

use super::{
    GatewayAppState, JsonRpcRequest, JsonRpcResponse, MessageQuery, SessionCleanupGuard,
};
use super::methods::{
    handle_initialize, handle_prompts_get, handle_prompts_list, handle_resources_list,
    handle_resources_read, handle_tools_call, handle_tools_list,
};

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
            handle_resources_list(&app_state, &request).await
        }
        "resources/read" => {
            handle_resources_read(&app_state, &request).await
        }
        "prompts/list" => {
            handle_prompts_list(&app_state, &request).await
        }
        "prompts/get" => {
            handle_prompts_get(&app_state, &request).await
        }
        _ => {
            // 其他方法暂不支持
            JsonRpcResponse::method_not_found(request.id)
        }
    };

    (StatusCode::OK, Json(response)).into_response()
}
