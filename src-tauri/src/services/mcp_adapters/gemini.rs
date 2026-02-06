//! Gemini CLI Adapter
//!
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 支持 Gemini CLI (.gemini/settings.json, ~/.gemini/settings.json) 配置文件解析和 Gateway 注入。

use std::collections::HashMap;
use std::path::Path;

use super::{
    common::strip_json_comments, AdapterError, ConfigScope, DetectedService,
    GatewayInjectionConfig, McpToolAdapter,
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
                            local_project_path: None,
                        });
                    }
                    McpServerConfig::Http { url, headers, .. } => {
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
                            local_project_path: None,
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
        let stripped = strip_json_comments(original_content);
        let mut root: serde_json::Value = if stripped.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&stripped)?
        };

        let obj = root.as_object_mut().ok_or_else(|| {
            AdapterError::InvalidFormat("Root must be a JSON object".to_string())
        })?;

        // 1. 注入 Gateway 到 mcpServers
        // 注意: 显式指定 "type": "http" 以兼容最新的 MCP 配置格式要求
        let gateway_config = serde_json::json!({
            "mantra-gateway": {
                "type": "http",
                "url": config.url,
                "headers": {
                    "Authorization": config.authorization_header()
                }
            }
        });
        obj.insert("mcpServers".to_string(), gateway_config);

        // 2. 确保 Gateway 不被 mcp.allowed/mcp.excluded 屏蔽
        Self::ensure_gateway_enabled_in_mcp_settings(obj);

        serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
    }

    /// Story 11.25: 清空项目级配置中的 mcpServers + 清理 mcp.allowed / mcp.excluded
    fn clear_mcp_servers(&self, original_content: &str) -> Result<String, AdapterError> {
        let stripped = strip_json_comments(original_content);
        let mut root: serde_json::Value = if stripped.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&stripped)?
        };

        if let Some(obj) = root.as_object_mut() {
            // 1. 清空 mcpServers
            obj.insert("mcpServers".to_string(), serde_json::json!({}));

            // 2. 清理 mcp.allowed / mcp.excluded（如存在）
            // 根据 Story 11.25 设计决策，项目级配置清理时需要移除这些过滤字段
            if let Some(mcp) = obj.get_mut("mcp").and_then(|m| m.as_object_mut()) {
                mcp.remove("allowed");
                mcp.remove("excluded");
            }
        }

        serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
    }
}

impl GeminiAdapter {
    /// 确保 mantra-gateway 不被 mcp.allowed/mcp.excluded 屏蔽
    ///
    /// Gemini CLI 的 `settings.json` 中可以有：
    /// - `mcp.allowed`: MCP 服务器允许列表（如果非空，只有在列表中的才启用）
    /// - `mcp.excluded`: MCP 服务器排除列表
    ///
    /// 此方法确保 Gateway 不会被这些配置屏蔽：
    /// 1. 从 `mcp.excluded` 中移除 `mantra-gateway`
    /// 2. 如果 `mcp.allowed` 非空，将 `mantra-gateway` 添加进去
    fn ensure_gateway_enabled_in_mcp_settings(
        root: &mut serde_json::Map<String, serde_json::Value>,
    ) {
        const GATEWAY_NAME: &str = "mantra-gateway";

        // 检查是否存在 mcp 配置对象
        if let Some(mcp) = root.get_mut("mcp").and_then(|m| m.as_object_mut()) {
            // 1. 从 mcp.excluded 中移除 mantra-gateway
            if let Some(excluded) = mcp.get_mut("excluded") {
                if let Some(arr) = excluded.as_array_mut() {
                    arr.retain(|v| v.as_str() != Some(GATEWAY_NAME));
                }
            }

            // 2. 如果 mcp.allowed 非空，确保 mantra-gateway 在列表中
            if let Some(allowed) = mcp.get_mut("allowed") {
                if let Some(arr) = allowed.as_array_mut() {
                    if !arr.is_empty() {
                        // 列表非空，检查是否已包含 Gateway
                        let contains_gateway =
                            arr.iter().any(|v| v.as_str() == Some(GATEWAY_NAME));
                        if !contains_gateway {
                            arr.push(serde_json::Value::String(GATEWAY_NAME.to_string()));
                        }
                    }
                }
            }
        }
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
    /// HTTP 传输模式（URL 连接，支持 Streamable HTTP / SSE）
    Http {
        url: String,
        #[serde(default, rename = "type")]
        transport_type: Option<String>,
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
    fn test_gemini_parse_http_servers() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {
                "local": {
                    "command": "local-mcp"
                },
                "remote": {
                    "url": "http://remote.example.com/mcp"
                },
                "http-typed": {
                    "type": "http",
                    "url": "https://mcp.deepwiki.com/mcp"
                }
            }
        }"#;

        let path = Path::new("/project/.gemini/settings.json");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 3);

        // 验证 stdio 服务
        let local = services.iter().find(|s| s.name == "local").unwrap();
        assert_eq!(local.transport_type, crate::models::mcp::McpTransportType::Stdio);
        assert_eq!(local.command, "local-mcp");

        // 验证 HTTP 服务
        let remote = services.iter().find(|s| s.name == "remote").unwrap();
        assert_eq!(remote.transport_type, crate::models::mcp::McpTransportType::Http);
        assert_eq!(remote.url, Some("http://remote.example.com/mcp".to_string()));

        let http_typed = services.iter().find(|s| s.name == "http-typed").unwrap();
        assert_eq!(http_typed.transport_type, crate::models::mcp::McpTransportType::Http);
        assert_eq!(http_typed.url, Some("https://mcp.deepwiki.com/mcp".to_string()));
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
            "http://127.0.0.1:8080/mcp",
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
            "http://127.0.0.1:8080/mcp"
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
            "http://127.0.0.1:8080/mcp",
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
            "http://127.0.0.1:8080/mcp",
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

    // ===== mcp.allowed / mcp.excluded 处理测试 =====

    #[test]
    fn test_gemini_inject_gateway_removes_from_excluded() {
        let adapter = GeminiAdapter;
        let original = r#"{
            "model": "gemini-pro",
            "mcp": {
                "excluded": ["some-server", "mantra-gateway", "another-server"]
            },
            "mcpServers": {
                "old": {"command": "old-mcp"}
            }
        }"#;

        let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "token");

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证 mcp.excluded 中移除了 mantra-gateway
        let excluded = parsed["mcp"]["excluded"].as_array().unwrap();
        assert_eq!(excluded.len(), 2);
        assert!(excluded.iter().any(|v| v == "some-server"));
        assert!(excluded.iter().any(|v| v == "another-server"));
        assert!(!excluded.iter().any(|v| v == "mantra-gateway"));

        // 验证 gateway 注入
        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
    }

    #[test]
    fn test_gemini_inject_gateway_adds_to_allowed_if_nonempty() {
        let adapter = GeminiAdapter;
        let original = r#"{
            "model": "gemini-pro",
            "mcp": {
                "allowed": ["trusted-server", "another-trusted"]
            },
            "mcpServers": {}
        }"#;

        let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "token");

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证 mcp.allowed 中添加了 mantra-gateway
        let allowed = parsed["mcp"]["allowed"].as_array().unwrap();
        assert_eq!(allowed.len(), 3);
        assert!(allowed.iter().any(|v| v == "trusted-server"));
        assert!(allowed.iter().any(|v| v == "another-trusted"));
        assert!(allowed.iter().any(|v| v == "mantra-gateway"));
    }

    #[test]
    fn test_gemini_inject_gateway_skips_empty_allowed() {
        let adapter = GeminiAdapter;
        let original = r#"{
            "model": "gemini-pro",
            "mcp": {
                "allowed": []
            },
            "mcpServers": {}
        }"#;

        let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "token");

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 空的 allowed 数组应该保持为空（表示允许所有）
        let allowed = parsed["mcp"]["allowed"].as_array().unwrap();
        assert!(allowed.is_empty());
    }

    #[test]
    fn test_gemini_inject_gateway_handles_both_allowed_and_excluded() {
        let adapter = GeminiAdapter;
        let original = r#"{
            "model": "gemini-pro",
            "mcp": {
                "allowed": ["server-a", "server-b"],
                "excluded": ["bad-server", "mantra-gateway"]
            },
            "mcpServers": {}
        }"#;

        let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "token");

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证从 excluded 中移除
        let excluded = parsed["mcp"]["excluded"].as_array().unwrap();
        assert_eq!(excluded.len(), 1);
        assert!(!excluded.iter().any(|v| v == "mantra-gateway"));

        // 验证添加到 allowed
        let allowed = parsed["mcp"]["allowed"].as_array().unwrap();
        assert_eq!(allowed.len(), 3);
        assert!(allowed.iter().any(|v| v == "mantra-gateway"));
    }

    #[test]
    fn test_gemini_inject_gateway_already_in_allowed() {
        let adapter = GeminiAdapter;
        let original = r#"{
            "mcp": {
                "allowed": ["mantra-gateway", "other-server"]
            },
            "mcpServers": {}
        }"#;

        let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "token");

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 不应该重复添加
        let allowed = parsed["mcp"]["allowed"].as_array().unwrap();
        assert_eq!(allowed.len(), 2);
        let gateway_count = allowed.iter().filter(|v| v.as_str() == Some("mantra-gateway")).count();
        assert_eq!(gateway_count, 1);
    }

    #[test]
    fn test_gemini_inject_gateway_no_mcp_settings() {
        let adapter = GeminiAdapter;
        let original = r#"{
            "model": "gemini-pro",
            "mcpServers": {}
        }"#;

        let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "token");

        let result = adapter.inject_gateway(original, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 没有 mcp 配置时不应该出错
        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
        // mcp 字段可能不存在或为 null
        assert!(parsed.get("mcp").is_none() || parsed["mcp"].is_null());
    }

    // ===== Story 11.25: clear_mcp_servers 测试 =====

    #[test]
    fn test_gemini_clear_mcp_servers_basic() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {
                "filesystem": {"command": "npx", "args": ["-y", "@mcp/filesystem"]},
                "database": {"command": "uvx", "args": ["mcp-postgres"]}
            },
            "model": "gemini-pro"
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // mcpServers 被清空
        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
        // 其他设置保留
        assert_eq!(parsed["model"], "gemini-pro");
    }

    #[test]
    fn test_gemini_clear_mcp_servers_removes_mcp_allowed_excluded() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {"test": {"command": "test"}},
            "model": "gemini-pro",
            "mcp": {
                "allowed": ["server-a", "server-b"],
                "excluded": ["bad-server"],
                "otherSetting": "keep-this"
            }
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // mcpServers 被清空
        assert_eq!(parsed["mcpServers"], serde_json::json!({}));

        // mcp.allowed 和 mcp.excluded 被移除
        assert!(parsed["mcp"].get("allowed").is_none());
        assert!(parsed["mcp"].get("excluded").is_none());

        // mcp 中的其他设置保留
        assert_eq!(parsed["mcp"]["otherSetting"], "keep-this");
    }

    #[test]
    fn test_gemini_clear_mcp_servers_preserves_other_fields() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {"old": {"command": "old"}},
            "model": "gemini-pro",
            "temperature": 0.7,
            "maxOutputTokens": 8192,
            "safetySettings": [{"category": "HARM", "threshold": "BLOCK"}]
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // mcpServers 被清空
        assert_eq!(parsed["mcpServers"], serde_json::json!({}));

        // 所有其他设置保留
        assert_eq!(parsed["model"], "gemini-pro");
        assert_eq!(parsed["temperature"], 0.7);
        assert_eq!(parsed["maxOutputTokens"], 8192);
        assert!(parsed["safetySettings"].is_array());
    }

    #[test]
    fn test_gemini_clear_mcp_servers_empty_content() {
        let adapter = GeminiAdapter;

        let result = adapter.clear_mcp_servers("").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
    }

    #[test]
    fn test_gemini_clear_mcp_servers_with_comments() {
        let adapter = GeminiAdapter;
        let content = r#"{
            // Gemini project settings
            "mcpServers": {
                /* MCP server */
                "test": {"command": "test-mcp"}
            },
            "model": "gemini-pro"
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
        assert_eq!(parsed["model"], "gemini-pro");
    }

    #[test]
    fn test_gemini_clear_mcp_servers_no_mcp_object() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {"test": {"command": "test"}},
            "model": "gemini-pro"
        }"#;

        // 没有 mcp 对象时也能正常清理
        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
    }

    #[test]
    fn test_gemini_clear_mcp_servers_only_allowed() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {"test": {"command": "test"}},
            "mcp": {
                "allowed": ["server-a"]
            }
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
        assert!(parsed["mcp"].get("allowed").is_none());
    }

    #[test]
    fn test_gemini_clear_mcp_servers_only_excluded() {
        let adapter = GeminiAdapter;
        let content = r#"{
            "mcpServers": {"test": {"command": "test"}},
            "mcp": {
                "excluded": ["bad-server"]
            }
        }"#;

        let result = adapter.clear_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["mcpServers"], serde_json::json!({}));
        assert!(parsed["mcp"].get("excluded").is_none());
    }
}
