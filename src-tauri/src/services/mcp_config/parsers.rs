//! 配置解析器实现（旧版，向后兼容）
//!
//! Story 11.8: 这些解析器已弃用，推荐使用 `McpToolAdapter` trait

use std::fs;
use std::path::Path;

use super::types::*;

/// 移除 JSON 注释（支持 JSONC）
///
/// 支持移除 // 单行注释和 /* */ 块注释
///
/// Note: 此函数为向后兼容保留，新代码应使用 `mcp_adapters::common::strip_json_comments`
pub fn strip_json_comments(input: &str) -> String {
    crate::services::mcp_adapters::common::strip_json_comments(input)
}

/// MCP 配置解析器 trait (已弃用)
///
/// Story 11.8: 使用 `McpToolAdapter` trait 替代
#[deprecated(since = "0.7.0", note = "Use McpToolAdapter trait from mcp_adapters module")]
#[allow(deprecated)]
pub trait McpConfigParser {
    /// 解析配置文件
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError>;

    /// 获取配置来源类型
    fn source_type(&self) -> ConfigSource;

    /// 生成影子模式配置
    fn generate_shadow_config(&self, gateway_url: &str) -> String;
}

/// Claude Code 配置解析器 (已弃用)
///
/// Story 11.8: 使用 `ClaudeAdapter` 替代
#[deprecated(since = "0.7.0", note = "Use ClaudeAdapter from mcp_adapters module")]
pub struct ClaudeCodeConfigParser;

#[allow(deprecated)]
impl McpConfigParser for ClaudeCodeConfigParser {
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError> {
        let content = fs::read_to_string(path)?;
        let content = strip_json_comments(&content);
        let config: McpConfigFile = serde_json::from_str(&content)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                if let McpServerConfig::Stdio { command, args, env } = server {
                    services.push(DetectedService {
                        name,
                        transport_type: Default::default(),
                        command,
                        args,
                        env,
                        url: None,
                        headers: None,
                        source_file: path.to_path_buf(),
                        adapter_id: "claude".to_string(),
                        scope: None,
                    });
                }
            }
        }

        Ok(services)
    }

    fn source_type(&self) -> ConfigSource {
        ConfigSource::ClaudeCode
    }

    fn generate_shadow_config(&self, gateway_url: &str) -> String {
        serde_json::json!({
            "mcpServers": {
                "mantra-gateway": {
                    "url": gateway_url
                }
            }
        })
        .to_string()
    }
}

/// Cursor 配置解析器 (已弃用)
///
/// Story 11.8: 使用 `CursorAdapter` 替代
#[deprecated(since = "0.7.0", note = "Use CursorAdapter from mcp_adapters module")]
pub struct CursorConfigParser;

#[allow(deprecated)]
impl McpConfigParser for CursorConfigParser {
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError> {
        let content = fs::read_to_string(path)?;
        let content = strip_json_comments(&content);
        let config: McpConfigFile = serde_json::from_str(&content)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                if let McpServerConfig::Stdio { command, args, env } = server {
                    services.push(DetectedService {
                        name,
                        transport_type: Default::default(),
                        command,
                        args,
                        env,
                        url: None,
                        headers: None,
                        source_file: path.to_path_buf(),
                        adapter_id: "cursor".to_string(),
                        scope: None,
                    });
                }
            }
        }

        Ok(services)
    }

    fn source_type(&self) -> ConfigSource {
        ConfigSource::Cursor
    }

    fn generate_shadow_config(&self, gateway_url: &str) -> String {
        serde_json::json!({
            "mcpServers": {
                "mantra-gateway": {
                    "url": gateway_url
                }
            }
        })
        .to_string()
    }
}

/// Claude Desktop 配置解析器 (已弃用)
///
/// Story 11.8: 使用 `ClaudeAdapter` 替代
#[deprecated(since = "0.7.0", note = "Use ClaudeAdapter from mcp_adapters module")]
pub struct ClaudeDesktopConfigParser;

#[allow(deprecated)]
impl McpConfigParser for ClaudeDesktopConfigParser {
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError> {
        let content = fs::read_to_string(path)?;
        let content = strip_json_comments(&content);
        let config: McpConfigFile = serde_json::from_str(&content)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                if let McpServerConfig::Stdio { command, args, env } = server {
                    services.push(DetectedService {
                        name,
                        transport_type: Default::default(),
                        command,
                        args,
                        env,
                        url: None,
                        headers: None,
                        source_file: path.to_path_buf(),
                        adapter_id: "claude".to_string(),
                        scope: None,
                    });
                }
            }
        }

        Ok(services)
    }

    fn source_type(&self) -> ConfigSource {
        ConfigSource::ClaudeDesktop
    }

    fn generate_shadow_config(&self, gateway_url: &str) -> String {
        serde_json::json!({
            "mcpServers": {
                "mantra-gateway": {
                    "url": gateway_url
                }
            }
        })
        .to_string()
    }
}

/// 解析单个配置文件 (旧版，向后兼容)
#[allow(deprecated)]
pub(super) fn parse_config_file_legacy(path: &Path, source: ConfigSource) -> DetectedConfig {
    let parser: Box<dyn McpConfigParser> = match source {
        ConfigSource::ClaudeCode => Box::new(ClaudeCodeConfigParser),
        ConfigSource::Cursor => Box::new(CursorConfigParser),
        ConfigSource::ClaudeDesktop => Box::new(ClaudeDesktopConfigParser),
        ConfigSource::Codex | ConfigSource::Gemini => {
            // 新工具使用新的适配器架构
            return DetectedConfig {
                adapter_id: source.to_adapter_id().to_string(),
                path: path.to_path_buf(),
                scope: None,
                services: Vec::new(),
                parse_errors: vec!["Use new adapter architecture".to_string()],
            };
        }
    };

    match parser.parse(path) {
        Ok(services) => DetectedConfig {
            adapter_id: source.to_adapter_id().to_string(),
            path: path.to_path_buf(),
            scope: None,
            services,
            parse_errors: Vec::new(),
        },
        Err(e) => DetectedConfig {
            adapter_id: source.to_adapter_id().to_string(),
            path: path.to_path_buf(),
            scope: None,
            services: Vec::new(),
            parse_errors: vec![e.to_string()],
        },
    }
}

/// 生成影子模式配置 (旧版，向后兼容)
///
/// Story 11.8: 推荐使用 `generate_shadow_config_v2` 替代
#[allow(deprecated)]
pub fn generate_shadow_config(source: &ConfigSource, gateway_url: &str) -> String {
    let parser: Box<dyn McpConfigParser> = match source {
        ConfigSource::ClaudeCode => Box::new(ClaudeCodeConfigParser),
        ConfigSource::Cursor => Box::new(CursorConfigParser),
        ConfigSource::ClaudeDesktop => Box::new(ClaudeDesktopConfigParser),
        ConfigSource::Codex | ConfigSource::Gemini => {
            // 新工具使用 generate_shadow_config_v2
            return serde_json::json!({
                "mcpServers": {
                    "mantra-gateway": {
                        "url": gateway_url
                    }
                }
            })
            .to_string();
        }
    };
    parser.generate_shadow_config(gateway_url)
}

/// 使用新适配器架构生成影子配置
///
/// Story 11.8: 使用 HTTP Transport + Authorization Header
pub fn generate_shadow_config_v2(
    adapter_id: &str,
    gateway_url: &str,
    token: &str,
) -> Result<String, String> {
    use crate::services::mcp_adapters::{GatewayInjectionConfig, ToolAdapterRegistry};

    let registry = ToolAdapterRegistry::new();
    let adapter = registry
        .get(adapter_id)
        .ok_or_else(|| format!("Unknown adapter: {}", adapter_id))?;

    let config = GatewayInjectionConfig::new(gateway_url, token);
    adapter
        .inject_gateway("", &config)
        .map_err(|e| e.to_string())
}
