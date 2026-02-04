//! 健康检查端点处理器

use axum::{extract::State, response::IntoResponse, Json};

use super::GatewayAppState;

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
