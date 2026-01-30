//! HTTP/SSE 请求处理器
//!
//! Story 11.1: SSE Server 核心 - Task 2 & Task 4
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

/// SSE 端点查询参数
#[derive(Debug, Deserialize)]
pub struct SseQuery {
    pub token: Option<String>,
}

/// Message 端点查询参数
#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub session_id: String,
    pub token: Option<String>,
}

/// JSON-RPC 请求
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
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

    /// 解析错误
    pub fn parse_error() -> Self {
        Self::error(None, -32700, "Parse error".to_string())
    }

    /// 无效请求
    pub fn invalid_request(id: Option<serde_json::Value>) -> Self {
        Self::error(id, -32600, "Invalid Request".to_string())
    }
}

/// Gateway 共享应用状态
#[derive(Clone)]
pub struct GatewayAppState {
    pub state: Arc<RwLock<GatewayState>>,
    pub stats: Arc<GatewayStats>,
    pub port: u16,
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
        state.register_session(app_state.port)
    };

    let session_id = session.session_id.clone();
    let message_endpoint = session.message_endpoint.clone();
    let state_clone = app_state.state.clone();

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
    );

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
    // 目前仅返回方法未找到，实际路由逻辑在后续故事实现
    let response = match request.method.as_str() {
        "initialize" => {
            // MCP 初始化方法 - 返回基本服务器信息
            JsonRpcResponse::success(
                request.id,
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
        "ping" => {
            // 简单的 ping 方法
            JsonRpcResponse::success(request.id, serde_json::json!({}))
        }
        _ => {
            // 其他方法暂不支持
            JsonRpcResponse::method_not_found(request.id)
        }
    };

    (StatusCode::OK, Json(response)).into_response()
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
}
