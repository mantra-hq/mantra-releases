//! MCP Gateway 模块
//!
//! Story 11.1: SSE Server 核心
//! Story 11.5: 上下文路由
//! Story 11.10: Project-Level Tool Management
//! Story 11.11: Integrated MCP Inspector - HTTP Transport
//! Story 11.12: Remote MCP OAuth Support
//! Story 11.14: MCP Streamable HTTP 规范合规
//! Story 11.17: MCP 协议聚合器
//!
//! 提供本地 SSE Server 用于 MCP 协议通信。
//! 监听 127.0.0.1:{port}，仅接受本地请求。

pub mod aggregator;
mod auth;
mod error;
mod handlers;
pub mod http_forwarder;
pub mod http_transport;
mod origin;
pub mod process_manager;
pub mod router;
mod server;
mod session;
mod state;

pub use auth::AuthLayer;
pub use error::GatewayError;
// Story 11.10: Tool Policy 拦截辅助函数
pub use handlers::{is_tool_blocked, log_tool_blocked, tool_blocked_error};
// Story 11.12: HTTP 转发器
pub use http_forwarder::{AuthType, ForwarderError, HttpForwarder, RemoteServiceConfig, RetryingForwarder};
// Story 11.11: HTTP 传输
pub use http_transport::{HttpTransportError, McpHttpClient};
// Story 11.14: Origin 验证
pub use origin::{origin_middleware, validate_origin, OriginValidatorConfig};
pub use process_manager::{McpProcessManager, ProcessError, RunningProcess, SharedProcessManager};
pub use router::{ContextRouter, ProjectContext};
pub use server::{GatewayServer, GatewayServerHandle, GatewayServerManager};
// Story 11.14: MCP Session 管理
pub use session::{
    create_session_id_header, extract_session_id, McpSession, McpSessionStore,
    SharedMcpSessionStore, MCP_SESSION_ID_HEADER,
};
pub use state::{
    ClientSession, GatewayConfig, GatewayState, GatewayStats, SessionProjectContext,
    SharedGatewayState, SharedGatewayStats,
};
// Story 11.17: MCP 协议聚合器
pub use aggregator::{
    AggregatorError, McpAggregator, McpPrompt, McpResource, McpTool, ServiceCapabilities,
    SharedMcpAggregator, WarmupResult,
};

#[cfg(test)]
mod tests;
