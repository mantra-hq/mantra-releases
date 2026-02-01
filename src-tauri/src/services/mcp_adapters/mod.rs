//! MCP Tool Adapters Module
//!
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 提供模块化的 MCP 工具适配器架构，支持 Claude, Cursor, Codex, Gemini CLI 等工具。
//!
//! ## 架构设计
//!
//! - `McpToolAdapter` trait: 定义适配器的标准接口
//! - `ToolAdapterRegistry`: 统一管理所有适配器
//! - 各工具适配器: claude, cursor, codex, gemini
//!
//! ## 使用示例
//!
//! ```ignore
//! use crate::services::mcp_adapters::{ToolAdapterRegistry, GatewayInjectionConfig};
//!
//! let registry = ToolAdapterRegistry::new();
//! let adapters = registry.all();
//!
//! for adapter in adapters {
//!     let patterns = adapter.scan_patterns();
//!     // ... scan and parse configs
//! }
//! ```

pub mod common;
pub mod claude;
pub mod cursor;
pub mod codex;
pub mod gemini;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::models::mcp::McpTransportType;

// Re-exports
pub use claude::ClaudeAdapter;
pub use cursor::CursorAdapter;
pub use codex::CodexAdapter;
pub use gemini::GeminiAdapter;
pub use common::{merge_json_config, merge_toml_config};

// ===== 核心类型定义 =====

/// 配置作用域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigScope {
    /// 项目级配置 (如 .mcp.json)
    Project,
    /// 用户级配置 (如 ~/.claude.json)
    User,
}

impl ConfigScope {
    /// 获取作用域的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            ConfigScope::Project => "Project",
            ConfigScope::User => "User",
        }
    }
}

/// Gateway 注入配置
///
/// Story 11.14: 更新为 MCP Streamable HTTP 端点 `/mcp`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayInjectionConfig {
    /// HTTP URL: http://127.0.0.1:{port}/mcp (MCP Streamable HTTP 端点)
    pub url: String,
    /// Bearer Token
    pub token: String,
}

impl GatewayInjectionConfig {
    /// 创建新的注入配置
    pub fn new(url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            token: token.into(),
        }
    }

    /// 获取完整的 Authorization header 值
    pub fn authorization_header(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

/// 检测到的 MCP 服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedService {
    /// 服务名称
    pub name: String,
    /// 传输类型
    #[serde(default)]
    pub transport_type: McpTransportType,
    /// 启动命令（stdio 模式）
    #[serde(default)]
    pub command: String,
    /// 命令参数（stdio 模式）
    pub args: Option<Vec<String>>,
    /// 环境变量
    pub env: Option<HashMap<String, String>>,
    /// HTTP 端点 URL（http 模式）
    pub url: Option<String>,
    /// HTTP 请求头（http 模式）
    pub headers: Option<HashMap<String, String>>,
    /// 来源配置文件路径
    pub source_file: PathBuf,
    /// 适配器 ID (替代旧的 source_type)
    pub adapter_id: String,
    /// 配置作用域
    pub scope: ConfigScope,
}

/// 检测到的配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedConfig {
    /// 适配器 ID (替代旧的 source_type)
    pub adapter_id: String,
    /// 配置文件路径
    pub path: PathBuf,
    /// 配置作用域
    pub scope: ConfigScope,
    /// 检测到的服务列表
    pub services: Vec<DetectedService>,
    /// 解析错误（如有）
    pub parse_errors: Vec<String>,
}

// ===== 适配器 Trait 定义 =====

/// MCP 工具适配器 Trait
///
/// 所有工具适配器必须实现此 trait，提供统一的配置解析和注入接口。
pub trait McpToolAdapter: Send + Sync {
    /// 获取工具唯一标识
    fn id(&self) -> &'static str;

    /// 获取工具显示名称
    fn name(&self) -> &'static str;

    /// 获取扫描模式
    ///
    /// 返回 (作用域, 路径模式) 列表
    /// - 项目级路径使用相对路径 (如 ".mcp.json")
    /// - 用户级路径使用 ~ 开头 (如 "~/.claude.json")
    fn scan_patterns(&self) -> Vec<(ConfigScope, String)>;

    /// 解析配置文件
    ///
    /// # Arguments
    /// * `path` - 配置文件路径
    /// * `content` - 配置文件内容
    /// * `scope` - 配置作用域
    ///
    /// # Returns
    /// 解析出的服务列表
    fn parse(&self, path: &Path, content: &str, scope: ConfigScope) -> Result<Vec<DetectedService>, AdapterError>;

    /// 非破坏性注入 Gateway 配置
    ///
    /// # Arguments
    /// * `original_content` - 原始配置文件内容
    /// * `config` - Gateway 注入配置
    ///
    /// # Returns
    /// 注入后的配置文件内容（保留原有设置）
    fn inject_gateway(&self, original_content: &str, config: &GatewayInjectionConfig) -> Result<String, AdapterError>;
}

/// 适配器错误类型
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(String),

    #[error("Invalid config format: {0}")]
    InvalidFormat(String),

    #[error("Merge error: {0}")]
    MergeError(String),
}

// ===== 适配器注册表 =====

/// 工具适配器注册表
///
/// 统一管理所有 MCP 工具适配器，支持动态注册和按 ID 查询。
pub struct ToolAdapterRegistry {
    adapters: HashMap<&'static str, Arc<dyn McpToolAdapter>>,
}

impl ToolAdapterRegistry {
    /// 创建新的注册表，自动注册所有内置适配器
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: HashMap::new(),
        };

        // 注册所有内置适配器
        registry.register(Arc::new(ClaudeAdapter));
        registry.register(Arc::new(CursorAdapter));
        registry.register(Arc::new(CodexAdapter));
        registry.register(Arc::new(GeminiAdapter));

        registry
    }

    /// 注册适配器
    pub fn register(&mut self, adapter: Arc<dyn McpToolAdapter>) {
        self.adapters.insert(adapter.id(), adapter);
    }

    /// 根据 ID 获取适配器
    pub fn get(&self, id: &str) -> Option<Arc<dyn McpToolAdapter>> {
        self.adapters.get(id).cloned()
    }

    /// 获取所有适配器
    pub fn all(&self) -> Vec<Arc<dyn McpToolAdapter>> {
        self.adapters.values().cloned().collect()
    }

    /// 获取所有适配器 ID
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.keys().copied().collect()
    }

    /// 检查适配器是否存在
    pub fn contains(&self, id: &str) -> bool {
        self.adapters.contains_key(id)
    }

    /// 获取适配器数量
    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

impl Default for ToolAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ===== 单元测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_scope_display_name() {
        assert_eq!(ConfigScope::Project.display_name(), "Project");
        assert_eq!(ConfigScope::User.display_name(), "User");
    }

    #[test]
    fn test_gateway_injection_config() {
        let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "test-token");
        assert_eq!(config.url, "http://127.0.0.1:8080/mcp");
        assert_eq!(config.token, "test-token");
        assert_eq!(config.authorization_header(), "Bearer test-token");
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolAdapterRegistry::new();
        assert_eq!(registry.len(), 4);
        assert!(registry.contains("claude"));
        assert!(registry.contains("cursor"));
        assert!(registry.contains("codex"));
        assert!(registry.contains("gemini"));
    }

    #[test]
    fn test_registry_get_adapter() {
        let registry = ToolAdapterRegistry::new();

        let claude = registry.get("claude").unwrap();
        assert_eq!(claude.id(), "claude");
        assert_eq!(claude.name(), "Claude Code");

        let cursor = registry.get("cursor").unwrap();
        assert_eq!(cursor.id(), "cursor");
        assert_eq!(cursor.name(), "Cursor");

        let codex = registry.get("codex").unwrap();
        assert_eq!(codex.id(), "codex");
        assert_eq!(codex.name(), "Codex");

        let gemini = registry.get("gemini").unwrap();
        assert_eq!(gemini.id(), "gemini");
        assert_eq!(gemini.name(), "Gemini CLI");
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = ToolAdapterRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_all_adapters() {
        let registry = ToolAdapterRegistry::new();
        let all = registry.all();
        assert_eq!(all.len(), 4);

        let ids: Vec<_> = all.iter().map(|a| a.id()).collect();
        assert!(ids.contains(&"claude"));
        assert!(ids.contains(&"cursor"));
        assert!(ids.contains(&"codex"));
        assert!(ids.contains(&"gemini"));
    }

    #[test]
    fn test_registry_ids() {
        let registry = ToolAdapterRegistry::new();
        let ids = registry.ids();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&"claude"));
        assert!(ids.contains(&"cursor"));
        assert!(ids.contains(&"codex"));
        assert!(ids.contains(&"gemini"));
    }

    #[test]
    fn test_config_scope_serialization() {
        let project = ConfigScope::Project;
        let user = ConfigScope::User;

        let project_json = serde_json::to_string(&project).unwrap();
        let user_json = serde_json::to_string(&user).unwrap();

        assert_eq!(project_json, "\"project\"");
        assert_eq!(user_json, "\"user\"");

        let project_parsed: ConfigScope = serde_json::from_str(&project_json).unwrap();
        let user_parsed: ConfigScope = serde_json::from_str(&user_json).unwrap();

        assert_eq!(project_parsed, ConfigScope::Project);
        assert_eq!(user_parsed, ConfigScope::User);
    }
}
