//! Cursor Adapter
//!
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 支持 Cursor (.cursor/mcp.json, ~/.cursor/mcp.json) 配置文件解析和 Gateway 注入。

use std::collections::HashMap;
use std::path::Path;

use super::{
    common::{merge_json_config, strip_json_comments},
    AdapterError, ConfigScope, DetectedService, GatewayInjectionConfig, McpToolAdapter,
};

/// Cursor 适配器
pub struct CursorAdapter;

impl McpToolAdapter for CursorAdapter {
    fn id(&self) -> &'static str {
        "cursor"
    }

    fn name(&self) -> &'static str {
        "Cursor"
    }

    fn scan_patterns(&self) -> Vec<(ConfigScope, String)> {
        vec![
            (ConfigScope::Project, ".cursor/mcp.json".to_string()),
            (ConfigScope::User, "~/.cursor/mcp.json".to_string()),
        ]
    }

    fn parse(
        &self,
        path: &Path,
        content: &str,
        scope: ConfigScope,
    ) -> Result<Vec<DetectedService>, AdapterError> {
        let stripped = strip_json_comments(content);
        let config: CursorConfigFile = serde_json::from_str(&stripped)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                if let McpServerConfig::Stdio { command, args, env } = server {
                    services.push(DetectedService {
                        name,
                        transport_type: crate::models::mcp::McpTransportType::Stdio,
                        command,
                        args,
                        env,
                        url: None,
                        headers: None,
                        source_file: path.to_path_buf(),
                        adapter_id: self.id().to_string(),
                        scope,
                        local_project_path: None,
                    });
                }
                // 跳过 URL 模式的服务
            }
        }

        Ok(services)
    }

    fn inject_gateway(
        &self,
        original_content: &str,
        config: &GatewayInjectionConfig,
    ) -> Result<String, AdapterError> {
        // 注意: Cursor 可能也要求 HTTP 类型显式指定 "type": "http"
        let gateway_config = serde_json::json!({
            "mantra-gateway": {
                "type": "http",
                "url": config.url,
                "headers": {
                    "Authorization": config.authorization_header()
                }
            }
        });

        merge_json_config(original_content, "mcpServers", gateway_config)
    }

    /// Story 11.25: 清空项目级配置中的 mcpServers
    fn clear_mcp_servers(&self, original_content: &str) -> Result<String, AdapterError> {
        let stripped = strip_json_comments(original_content);
        let mut root: serde_json::Value = if stripped.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&stripped)?
        };

        if let Some(obj) = root.as_object_mut() {
            obj.insert("mcpServers".to_string(), serde_json::json!({}));
        }

        serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
    }
}

// ===== 配置文件结构定义 =====

/// Cursor 配置文件结构
#[derive(Debug, serde::Deserialize)]
struct CursorConfigFile {
    #[serde(alias = "mcpServers")]
    mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

/// MCP 服务器配置
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum McpServerConfig {
    /// stdio 传输模式（命令行启动）
    Stdio {
        command: String,
        #[serde(default)]
        args: Option<Vec<String>>,
        #[serde(default)]
        env: Option<HashMap<String, String>>,
    },
    /// URL 传输模式
    #[allow(dead_code)]
    Url {
        url: String,
        #[serde(default)]
        headers: Option<HashMap<String, String>>,
    },
}

// ===== 单元测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_adapter_id_and_name() {
        let adapter = CursorAdapter;
        assert_eq!(adapter.id(), "cursor");
        assert_eq!(adapter.name(), "Cursor");
    }

    #[test]
    fn test_cursor_scan_patterns() {
        let adapter = CursorAdapter;
        let patterns = adapter.scan_patterns();

        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&(ConfigScope::Project, ".cursor/mcp.json".to_string())));
        assert!(patterns.contains(&(ConfigScope::User, "~/.cursor/mcp.json".to_string())));
    }

    #[test]
    fn test_cursor_parse_basic() {
        let adapter = CursorAdapter;
        let content = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/dir"]
                }
            }
        }"#;

        let path = Path::new("/project/.cursor/mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "filesystem");
        assert_eq!(services[0].command, "npx");
        assert_eq!(services[0].adapter_id, "cursor");
        assert_eq!(services[0].scope, ConfigScope::Project);
    }

    #[test]
    fn test_cursor_parse_with_env() {
        let adapter = CursorAdapter;
        let content = r#"{
            "mcpServers": {
                "database": {
                    "command": "uvx",
                    "args": ["mcp-server-postgres"],
                    "env": {
                        "DATABASE_URL": "$DATABASE_URL",
                        "DEBUG": "true"
                    }
                }
            }
        }"#;

        let path = Path::new("/project/.cursor/mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        let service = &services[0];
        assert!(service.env.is_some());
        let env = service.env.as_ref().unwrap();
        assert_eq!(env.get("DATABASE_URL"), Some(&"$DATABASE_URL".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_cursor_parse_user_scope() {
        let adapter = CursorAdapter;
        let content = r#"{
            "mcpServers": {
                "global-mcp": {
                    "command": "mcp-server"
                }
            }
        }"#;

        let path = Path::new("/home/user/.cursor/mcp.json");
        let services = adapter.parse(path, content, ConfigScope::User).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].scope, ConfigScope::User);
    }

    #[test]
    fn test_cursor_parse_skip_url_servers() {
        let adapter = CursorAdapter;
        let content = r#"{
            "mcpServers": {
                "local-server": {
                    "command": "local-mcp"
                },
                "remote-server": {
                    "url": "http://remote.example.com/mcp"
                }
            }
        }"#;

        let path = Path::new("/project/.cursor/mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "local-server");
    }

    #[test]
    fn test_cursor_inject_gateway() {
        let adapter = CursorAdapter;
        let original = r#"{
            "mcpServers": {
                "existing": {"command": "existing-mcp"}
            }
        }"#;

        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "cursor-token",
        );

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证 gateway 注入
        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["url"],
            "http://127.0.0.1:8080/mcp"
        );
        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["headers"]["Authorization"],
            "Bearer cursor-token"
        );
    }

    #[test]
    fn test_cursor_inject_gateway_empty_file() {
        let adapter = CursorAdapter;
        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "token",
        );

        let result = adapter.inject_gateway("", &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
    }

    #[test]
    fn test_cursor_parse_with_comments() {
        let adapter = CursorAdapter;
        let content = r#"{
            // Cursor MCP config
            "mcpServers": {
                /* My server */
                "my-server": {
                    "command": "my-mcp"
                }
            }
        }"#;

        let path = Path::new("/project/.cursor/mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "my-server");
    }

    // ===== Story 11.25: clear_mcp_servers 测试 =====

    #[test]
    fn test_cursor_clear_mcp_servers_basic() {
        let adapter = CursorAdapter;
        let content = r#"{
            "mcpServers": {
                "filesystem": {"command": "npx", "args": ["-y", "@mcp/filesystem"]},
                "database": {"command": "uvx", "args": ["mcp-postgres"]}
            }
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
    }

    #[test]
    fn test_cursor_clear_mcp_servers_preserves_other_fields() {
        let adapter = CursorAdapter;
        let content = r#"{
            "mcpServers": {"old": {"command": "old"}},
            "otherSetting": "value",
            "nested": {"key": 123}
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
        assert_eq!(parsed["otherSetting"], "value");
        assert_eq!(parsed["nested"]["key"], 123);
    }

    #[test]
    fn test_cursor_clear_mcp_servers_empty_content() {
        let adapter = CursorAdapter;

        let result = adapter.clear_mcp_servers("").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
    }

    #[test]
    fn test_cursor_clear_mcp_servers_with_comments() {
        let adapter = CursorAdapter;
        let content = r#"{
            // Cursor project config
            "mcpServers": {
                /* MCP server */
                "test": {"command": "test-mcp"}
            }
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
    }
}
