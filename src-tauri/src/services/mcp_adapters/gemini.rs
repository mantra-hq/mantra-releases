//! Gemini CLI Adapter
//!
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 支持 Gemini CLI (.gemini/settings.json, ~/.gemini/settings.json) 配置文件解析和 Gateway 注入。

use std::collections::HashMap;
use std::path::Path;

use super::{
    common::{merge_json_config, strip_json_comments},
    AdapterError, ConfigScope, DetectedService, GatewayInjectionConfig, McpToolAdapter,
};

/// Gemini CLI 适配器
pub struct GeminiAdapter;

impl McpToolAdapter for GeminiAdapter {
    fn id(&self) -> &'static str {
        "gemini"
    }

    fn name(&self) -> &'static str {
        "Gemini CLI"
    }

    fn scan_patterns(&self) -> Vec<(ConfigScope, String)> {
        vec![
            (ConfigScope::Project, ".gemini/settings.json".to_string()),
            (ConfigScope::User, "~/.gemini/settings.json".to_string()),
        ]
    }

    fn parse(
        &self,
        path: &Path,
        content: &str,
        scope: ConfigScope,
    ) -> Result<Vec<DetectedService>, AdapterError> {
        let stripped = strip_json_comments(content);
        let config: GeminiConfigFile = serde_json::from_str(&stripped)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                if let McpServerConfig::Stdio { command, args, env } = server {
                    services.push(DetectedService {
                        name,
                        command,
                        args,
                        env,
                        source_file: path.to_path_buf(),
                        adapter_id: self.id().to_string(),
                        scope,
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

/// Gemini 配置文件结构
#[derive(Debug, serde::Deserialize)]
struct GeminiConfigFile {
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
    fn test_gemini_adapter_id_and_name() {
        let adapter = GeminiAdapter;
        assert_eq!(adapter.id(), "gemini");
        assert_eq!(adapter.name(), "Gemini CLI");
    }

    #[test]
    fn test_gemini_scan_patterns() {
        let adapter = GeminiAdapter;
        let patterns = adapter.scan_patterns();

        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&(ConfigScope::Project, ".gemini/settings.json".to_string())));
        assert!(patterns.contains(&(ConfigScope::User, "~/.gemini/settings.json".to_string())));
    }

    #[test]
    fn test_gemini_parse_basic() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/filesystem-mcp"]
                }
            }
        }"#;

        let path = Path::new("/project/.gemini/settings.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "filesystem");
        assert_eq!(services[0].command, "npx");
        assert_eq!(services[0].adapter_id, "gemini");
        assert_eq!(services[0].scope, ConfigScope::Project);
    }

    #[test]
    fn test_gemini_parse_with_env() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {
                "database": {
                    "command": "uvx",
                    "args": ["mcp-server-postgres"],
                    "env": {
                        "DATABASE_URL": "$DATABASE_URL"
                    }
                }
            }
        }"#;

        let path = Path::new("/project/.gemini/settings.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert!(services[0].env.is_some());
    }

    #[test]
    fn test_gemini_parse_skip_url_servers() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {
                "local": {
                    "command": "local-mcp"
                },
                "remote": {
                    "url": "http://remote.example.com/mcp"
                }
            }
        }"#;

        let path = Path::new("/project/.gemini/settings.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "local");
    }

    #[test]
    fn test_gemini_parse_preserves_other_settings() {
        let adapter = GeminiAdapter;
        // Gemini 配置文件可能包含其他设置，解析时应该只关注 mcpServers
        let content = r#"{
            "model": "gemini-pro",
            "temperature": 0.7,
            "mcpServers": {
                "test": {
                    "command": "test-mcp"
                }
            }
        }"#;

        let path = Path::new("/project/.gemini/settings.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test");
    }

    #[test]
    fn test_gemini_inject_gateway() {
        let adapter = GeminiAdapter;
        let original = r#"{
            "model": "gemini-pro",
            "temperature": 0.7,
            "mcpServers": {
                "old": {"command": "old-mcp"}
            }
        }"#;

        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/message",
            "gemini-token",
        );

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证其他设置保留
        assert_eq!(parsed["model"], "gemini-pro");
        assert_eq!(parsed["temperature"], 0.7);

        // 验证 gateway 注入
        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["url"],
            "http://127.0.0.1:8080/message"
        );
        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["headers"]["Authorization"],
            "Bearer gemini-token"
        );
    }

    #[test]
    fn test_gemini_inject_gateway_empty_file() {
        let adapter = GeminiAdapter;
        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/message",
            "token",
        );

        let result = adapter.inject_gateway("", &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
    }

    #[test]
    fn test_gemini_inject_gateway_preserve_all_settings() {
        let adapter = GeminiAdapter;
        let original = r#"{
            "model": "gemini-pro",
            "temperature": 0.7,
            "maxOutputTokens": 8192,
            "topK": 40,
            "topP": 0.95,
            "safetySettings": [
                {"category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_MEDIUM_AND_ABOVE"}
            ],
            "mcpServers": {}
        }"#;

        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/message",
            "token",
        );

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证所有设置保留
        assert_eq!(parsed["model"], "gemini-pro");
        assert_eq!(parsed["temperature"], 0.7);
        assert_eq!(parsed["maxOutputTokens"], 8192);
        assert_eq!(parsed["topK"], 40);
        assert_eq!(parsed["topP"], 0.95);
        assert!(parsed["safetySettings"].is_array());
    }

    #[test]
    fn test_gemini_parse_with_comments() {
        let adapter = GeminiAdapter;
        let content = r#"{
            // Gemini settings
            "model": "gemini-pro",
            /* MCP servers */
            "mcpServers": {
                "test": {
                    "command": "test-mcp"
                }
            }
        }"#;

        let path = Path::new("/project/.gemini/settings.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test");
    }
}
