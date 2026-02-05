//! MCP Gateway 模块
//!
//! Story 11.1: SSE Server 核心
//! Story 11.5: 上下文路由
//! Story 11.9 Phase 2: 工具策略完整实现 - PolicyResolver
//! Story 11.10: Project-Level Tool Management
//! Story 11.11: Integrated MCP Inspector - HTTP Transport
//! Story 11.12: Remote MCP OAuth Support
//! Story 11.14: MCP Streamable HTTP 规范合规
//! Story 11.17: MCP 协议聚合器
//! Story 11.26: MCP Roots 机制
//! Story 11.27: MCP Roots LPM 集成
//! Story 11.28: MCP 严格模式服务过滤
//!
//! 提供本地 SSE Server 用于 MCP 协议通信。
//! 监听 127.0.0.1:{port}，仅接受本地请求。

use std::path::PathBuf;

pub mod aggregator;
mod auth;
mod error;
mod handlers;
pub mod http_forwarder;
pub mod http_transport;
pub mod lpm_query;
mod origin;
pub mod policy;
pub mod process_manager;
pub mod project_services_query;
pub mod router;
mod server;
mod session;
mod state;

/// 将 file:// URI 转换为本地路径
///
/// Story 11.26: MCP Roots 机制 - 共享 utility 函数
///
/// 支持:
/// - Unix: `file:///home/user/projects` -> `/home/user/projects`
/// - Windows: `file:///C:/Users/user/projects` -> `C:/Users/user/projects`
/// - URL 编码: `file:///home/user/my%20projects` -> `/home/user/my projects`
pub fn uri_to_local_path(uri: &str) -> Option<PathBuf> {
    if !uri.starts_with("file://") {
        return None;
    }

    let path = &uri[7..];

    // Windows: file:///C:/path -> C:/path
    #[cfg(target_os = "windows")]
    {
        if path.starts_with('/') && path.len() > 2 && path.chars().nth(2) == Some(':') {
            return Some(PathBuf::from(&path[1..]));
        }
    }

    // Unix: file:///path -> /path
    // URL 解码
    if let Ok(decoded) = urlencoding::decode(path) {
        return Some(PathBuf::from(decoded.as_ref()));
    }
    Some(PathBuf::from(path))
}

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
// Story 11.9 Phase 2: PolicyResolver
pub use policy::{PolicyResolver, SharedPolicyResolver, StoragePolicyResolver};
// Story 11.27: LPM 查询
pub use lpm_query::{
    LpmProjectContext, LpmQueryClient, LpmQueryRequest, LpmQueryResponse, LpmQueryService,
    SharedLpmQueryClient,
};
// Story 11.28: 项目服务查询
pub use project_services_query::{
    ProjectServicesQueryClient, ProjectServicesQueryRequest, ProjectServicesQueryResponse,
    ProjectServicesQueryService, SharedProjectServicesQueryClient,
};

#[cfg(test)]
mod tests;
