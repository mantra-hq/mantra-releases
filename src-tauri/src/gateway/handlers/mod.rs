//! HTTP/SSE 请求处理器
//!
//! Story 11.1: SSE Server 核心 - Task 2 & Task 4
//! Story 11.5: 上下文路由 - Task 4 & Task 5
//! Story 11.14: MCP Streamable HTTP 规范合规 - Task 3
//!
//! 实现 `/sse` SSE 端点、`/message` JSON-RPC 端点和 `/mcp` Streamable HTTP 端点

mod health;
mod legacy;
mod mcp_streamable;
mod methods;

pub use health::*;
pub use legacy::*;
pub use mcp_streamable::*;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::session::{McpSessionStore, SharedMcpSessionStore, SharedServerToClientManager, ServerToClientManager};
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
/// Story 11.17: 添加 MCP Aggregator
/// Story 11.9 Phase 2: 添加 PolicyResolver
/// Story 11.26: 添加 Server-to-Client Manager
#[derive(Clone)]
pub struct GatewayAppState {
    pub state: Arc<RwLock<GatewayState>>,
    pub stats: Arc<GatewayStats>,
    /// MCP Streamable HTTP 会话存储 (Story 11.14)
    pub mcp_sessions: SharedMcpSessionStore,
    /// MCP 协议聚合器 (Story 11.17)
    pub aggregator: Option<super::aggregator::SharedMcpAggregator>,
    /// Tool Policy 解析器 (Story 11.9 Phase 2)
    pub policy_resolver: Option<super::policy::SharedPolicyResolver>,
    /// Server-to-Client 通信管理器 (Story 11.26)
    pub s2c_manager: SharedServerToClientManager,
}

impl GatewayAppState {
    /// 创建应用状态
    pub fn new(state: Arc<RwLock<GatewayState>>, stats: Arc<GatewayStats>) -> Self {
        Self {
            state,
            stats,
            mcp_sessions: Arc::new(RwLock::new(McpSessionStore::new())),
            aggregator: None,
            policy_resolver: None,
            s2c_manager: Arc::new(ServerToClientManager::new()),
        }
    }

    /// 创建带 Aggregator 的应用状态
    pub fn with_aggregator(
        state: Arc<RwLock<GatewayState>>,
        stats: Arc<GatewayStats>,
        aggregator: super::aggregator::SharedMcpAggregator,
    ) -> Self {
        Self {
            state,
            stats,
            mcp_sessions: Arc::new(RwLock::new(McpSessionStore::new())),
            aggregator: Some(aggregator),
            policy_resolver: None,
            s2c_manager: Arc::new(ServerToClientManager::new()),
        }
    }

    /// 创建带 Aggregator 和 PolicyResolver 的应用状态 (Story 11.9 Phase 2)
    pub fn with_aggregator_and_policy(
        state: Arc<RwLock<GatewayState>>,
        stats: Arc<GatewayStats>,
        aggregator: super::aggregator::SharedMcpAggregator,
        policy_resolver: super::policy::SharedPolicyResolver,
    ) -> Self {
        Self {
            state,
            stats,
            mcp_sessions: Arc::new(RwLock::new(McpSessionStore::new())),
            aggregator: Some(aggregator),
            policy_resolver: Some(policy_resolver),
            s2c_manager: Arc::new(ServerToClientManager::new()),
        }
    }
}

/// 会话清理守卫
///
/// 当此结构体被 drop 时，自动从状态中移除对应的会话
pub(super) struct SessionCleanupGuard {
    pub session_id: String,
    pub state: Arc<RwLock<GatewayState>>,
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

/// MCP Session 清理守卫
///
/// 当此结构体被 drop 时，自动从 MCP Session Store 中移除对应的会话
pub(super) struct McpSessionCleanupGuard {
    pub session_id: Option<String>,
    pub session_store: SharedMcpSessionStore,
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

/// 生成 403 Forbidden Origin 响应
pub(super) fn forbidden_origin_response() -> Response {
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
pub(super) fn session_not_found_response(session_id: &str) -> Response {
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
mod tests;
