//! Services module for Mantra
//!
//! Provides business logic services for the application.

pub mod env_manager;
pub mod mcp_adapters;
pub mod mcp_config;
pub mod mcp_registry;
pub mod mcp_tool_discovery;
pub mod oauth;
pub mod takeover_transaction;

pub use env_manager::EnvManager;
pub use mcp_adapters::{
    AdapterError, ConfigScope, DetectedConfig, DetectedService, GatewayInjectionConfig,
    McpToolAdapter, ToolAdapterRegistry,
};
pub use mcp_registry::McpRegistry;
pub use mcp_tool_discovery::{McpToolDiscovery, ToolDefinition, ToolDiscoveryResult};
pub use oauth::{
    OAuthConfig, OAuthError, OAuthManager, OAuthServiceStatus, OAuthStatus, OAuthToken,
};
pub use takeover_transaction::{RollbackResult, TakeoverOperation, TakeoverTransaction};

