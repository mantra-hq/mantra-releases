//! 本地 HTTP Server 模块
//!
//! 提供本地 API 服务，用于与外部工具（如 Claude Code Hook）通信。
//! 监听 127.0.0.1:{port}，仅接受本地请求。

mod config;
mod handlers;
mod server;

pub use config::{LocalServerConfig, DEFAULT_PORT};
pub use server::{LocalServer, ServerHandle, ServerManager};

#[cfg(test)]
mod tests;
