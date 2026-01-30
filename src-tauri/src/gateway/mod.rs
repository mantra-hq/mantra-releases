//! MCP Gateway 模块
//!
//! Story 11.1: SSE Server 核心
//!
//! 提供本地 SSE Server 用于 MCP 协议通信。
//! 监听 127.0.0.1:{port}，仅接受本地请求。

mod auth;
mod error;
mod handlers;
mod server;
mod state;

pub use auth::AuthLayer;
pub use error::GatewayError;
pub use server::{GatewayServer, GatewayServerHandle, GatewayServerManager};
pub use state::{
    ClientSession, GatewayConfig, GatewayState, GatewayStats, SharedGatewayState,
    SharedGatewayStats,
};

#[cfg(test)]
mod tests;
