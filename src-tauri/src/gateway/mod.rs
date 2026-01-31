//! MCP Gateway 模块
//!
//! Story 11.1: SSE Server 核心
//! Story 11.5: 上下文路由
//! Story 11.10: Project-Level Tool Management
//!
//! 提供本地 SSE Server 用于 MCP 协议通信。
//! 监听 127.0.0.1:{port}，仅接受本地请求。

mod auth;
mod error;
mod handlers;
pub mod process_manager;
pub mod router;
mod server;
mod state;

pub use auth::AuthLayer;
pub use error::GatewayError;
// Story 11.10: Tool Policy 拦截辅助函数
pub use handlers::{is_tool_blocked, log_tool_blocked, tool_blocked_error};
pub use process_manager::{McpProcessManager, ProcessError, RunningProcess, SharedProcessManager};
pub use router::{ContextRouter, ProjectContext};
pub use server::{GatewayServer, GatewayServerHandle, GatewayServerManager};
pub use state::{
    ClientSession, GatewayConfig, GatewayState, GatewayStats, SessionProjectContext,
    SharedGatewayState, SharedGatewayStats,
};

#[cfg(test)]
mod tests;
