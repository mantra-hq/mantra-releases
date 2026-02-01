//! Claude Code/Desktop Adapter
//!
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 支持 Claude Code (.mcp.json, ~/.claude.json) 配置文件解析和 Gateway 注入。

use std::collections::HashMap;
use std::path::Path;

use super::{
    common::{merge_json_config, strip_json_comments},
    AdapterError, ConfigScope, DetectedService, GatewayInjectionConfig, McpToolAdapter,
};

/// Claude Code/Desktop 适配器
pub struct ClaudeAdapter;

impl McpToolAdapter for ClaudeAdapter {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn name(&self) -> &'static str {
        "Claude Code"
    }

    fn scan_patterns(&self) -> Vec<(ConfigScope, String)> {
        vec![
            (ConfigScope::Project, ".mcp.json".to_string()),
            (ConfigScope::User, "~/.claude.json".to_string()),
        ]
    }

    fn parse(
        &self,
        path: &Path,
        content: &str,
        scope: ConfigScope,
    ) -> Result<Vec<DetectedService>, AdapterError> {
        let stripped = strip_json_comments(content);
        let config: ClaudeConfigFile = serde_json::from_str(&stripped)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                match server {
                    McpServerConfig::Stdio { command, args, env } => {
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
                        });
                    }
                    McpServerConfig::Http { url, headers } => {
                        services.push(DetectedService {
                            name,
                            transport_type: crate::models::mcp::McpTransportType::Http,
                            command: String::new(),
                            args: None,
                            env: None,
                            url: Some(url),
                            headers,
                            source_file: path.to_path_buf(),
                            adapter_id: self.id().to_string(),
                            scope,
                        });
                    }
                }
            }
        }

        Ok(services)
    }

    fn inject_gateway(
        &self,
        original_content: &str,
        config: &GatewayInjectionConfig,
    ) -> Result<String, AdapterError> {
        let gateway_config = serde_json::json!({
            "mantra-gateway": {
                "url": config.url,
                "headers": {
                    "Authorization": config.authorization_header()
                }
            }
        });

        merge_json_config(original_content, "mcpServers", gateway_config)
    }
}

// ===== 配置文件结构定义 =====

/// Claude 配置文件结构
#[derive(Debug, serde::Deserialize)]
struct ClaudeConfigFile {
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
    /// HTTP 传输模式（URL 连接）
    Http {
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
    fn test_claude_adapter_id_and_name() {
        let adapter = ClaudeAdapter;
        assert_eq!(adapter.id(), "claude");
        assert_eq!(adapter.name(), "Claude Code");
    }

    #[test]
    fn test_claude_scan_patterns() {
        let adapter = ClaudeAdapter;
        let patterns = adapter.scan_patterns();

        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&(ConfigScope::Project, ".mcp.json".to_string())));
        assert!(patterns.contains(&(ConfigScope::User, "~/.claude.json".to_string())));
    }

    #[test]
    fn test_claude_parse_basic() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {
                "git-mcp": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/git-mcp"]
                },
                "postgres-mcp": {
                    "command": "uvx",
                    "args": ["mcp-server-postgres"],
                    "env": {
                        "DATABASE_URL": "$DATABASE_URL"
                    }
                }
            }
        }"#;

        let path = Path::new("/test/.mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 2);

        let git_mcp = services.iter().find(|s| s.name == "git-mcp").unwrap();
        assert_eq!(git_mcp.command, "npx");
        assert_eq!(git_mcp.args, Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]));
        assert_eq!(git_mcp.adapter_id, "claude");
        assert_eq!(git_mcp.scope, ConfigScope::Project);

        let postgres_mcp = services.iter().find(|s| s.name == "postgres-mcp").unwrap();
        assert_eq!(postgres_mcp.command, "uvx");
        assert!(postgres_mcp.env.is_some());
    }

    #[test]
    fn test_claude_parse_with_comments() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            // MCP configuration for Claude
            "mcpServers": {
                /* Git server */
                "git-mcp": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/git-mcp"]
                }
            }
        }"#;

        let path = Path::new("/test/.mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "git-mcp");
    }

    #[test]
    fn test_claude_parse_includes_http_servers() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {
                "local-server": {
                    "command": "npx",
                    "args": ["-y", "local-mcp"]
                },
                "remote-gateway": {
                    "url": "http://remote.example.com/message",
                    "headers": {"Authorization": "Bearer xxx"}
                }
            }
        }"#;

        let path = Path::new("/test/.mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        // Both stdio and HTTP services should be parsed
        assert_eq!(services.len(), 2);

        let local = services.iter().find(|s| s.name == "local-server").unwrap();
        assert_eq!(local.transport_type, crate::models::mcp::McpTransportType::Stdio);
        assert_eq!(local.command, "npx");

        let remote = services.iter().find(|s| s.name == "remote-gateway").unwrap();
        assert_eq!(remote.transport_type, crate::models::mcp::McpTransportType::Http);
        assert_eq!(remote.url, Some("http://remote.example.com/message".to_string()));
        assert!(remote.headers.is_some());
    }

    #[test]
    fn test_claude_parse_empty_servers() {
        let adapter = ClaudeAdapter;
        let content = r#"{"mcpServers": {}}"#;

        let path = Path::new("/test/.mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert!(services.is_empty());
    }

    #[test]
    fn test_claude_parse_no_servers_key() {
        let adapter = ClaudeAdapter;
        let content = r#"{"autoApprove": ["read"]}"#;

        let path = Path::new("/test/.mcp.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert!(services.is_empty());
    }

    #[test]
    fn test_claude_inject_gateway() {
        let adapter = ClaudeAdapter;
        let original = r#"{
            "autoApprove": ["read", "write"],
            "mcpServers": {
                "old-server": {"command": "old"}
            }
        }"#;

        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "test-token-123",
        );

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证 autoApprove 保留
        assert_eq!(parsed["autoApprove"], serde_json::json!(["read", "write"]));

        // 验证 gateway 注入
        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["url"],
            "http://127.0.0.1:8080/mcp"
        );
        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["headers"]["Authorization"],
            "Bearer test-token-123"
        );

        // 验证旧服务被移除
        assert!(parsed["mcpServers"]["old-server"].is_null());
    }

    #[test]
    fn test_claude_inject_gateway_empty_file() {
        let adapter = ClaudeAdapter;
        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "token",
        );

        let result = adapter.inject_gateway("", &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
    }

    #[test]
    fn test_claude_inject_gateway_with_permissions() {
        let adapter = ClaudeAdapter;
        let original = r#"{
            "permissions": {
                "allowedPaths": ["/home/user/projects"],
                "disallowedTools": ["dangerous_tool"]
            },
            "mcpServers": {}
        }"#;

        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "token",
        );

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证 permissions 保留
        assert_eq!(
            parsed["permissions"]["allowedPaths"],
            serde_json::json!(["/home/user/projects"])
        );
        assert_eq!(
            parsed["permissions"]["disallowedTools"],
            serde_json::json!(["dangerous_tool"])
        );
    }
}
