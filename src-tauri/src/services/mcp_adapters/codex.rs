//! Codex Adapter
//!
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 支持 Codex CLI (.codex/config.toml, ~/.codex/config.toml) 配置文件解析和 Gateway 注入。
//!
//! ## 注意事项
//!
//! - 使用 `toml_edit` crate 保留注释和格式
//! - 认证使用 `http_headers` 而非 `headers`

use std::collections::HashMap;
use std::path::Path;

use super::{
    common::merge_toml_config,
    AdapterError, ConfigScope, DetectedService, GatewayInjectionConfig, McpToolAdapter,
};

/// Codex 适配器
pub struct CodexAdapter;

impl McpToolAdapter for CodexAdapter {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn name(&self) -> &'static str {
        "Codex"
    }

    fn scan_patterns(&self) -> Vec<(ConfigScope, String)> {
        vec![
            (ConfigScope::Project, ".codex/config.toml".to_string()),
            (ConfigScope::User, "~/.codex/config.toml".to_string()),
        ]
    }

    fn parse(
        &self,
        path: &Path,
        content: &str,
        scope: ConfigScope,
    ) -> Result<Vec<DetectedService>, AdapterError> {
        let config: CodexConfigFile = toml::from_str(content)
            .map_err(|e| AdapterError::Toml(e.to_string()))?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                // Codex 支持 stdio 和 HTTP 两种传输模式
                if let Some(command) = server.command {
                    // stdio 模式
                    services.push(DetectedService {
                        name,
                        transport_type: crate::models::mcp::McpTransportType::Stdio,
                        command,
                        args: server.args,
                        env: server.env,
                        url: None,
                        headers: None,
                        source_file: path.to_path_buf(),
                        adapter_id: self.id().to_string(),
                        scope,
                        local_project_path: None,
                    });
                } else if let Some(url) = server.url {
                    // HTTP 模式
                    services.push(DetectedService {
                        name,
                        transport_type: crate::models::mcp::McpTransportType::Http,
                        command: String::new(),
                        args: None,
                        env: None,
                        url: Some(url),
                        headers: server.http_headers,
                        source_file: path.to_path_buf(),
                        adapter_id: self.id().to_string(),
                        scope,
                        local_project_path: None,
                    });
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
        // Codex 使用 TOML 格式，认证使用 http_headers
        let gateway_toml = format!(
            r#"[mcp_servers.mantra-gateway]
url = "{}"

[mcp_servers.mantra-gateway.http_headers]
Authorization = "{}"
"#,
            config.url,
            config.authorization_header()
        );

        merge_toml_config(original_content, "mcp_servers", &gateway_toml)
    }

    /// Story 11.25: 清空项目级配置中的 mcp_servers
    fn clear_mcp_servers(&self, original_content: &str) -> Result<String, AdapterError> {
        use toml_edit::{DocumentMut, Item, Table};

        // 解析原始文档
        let mut doc: DocumentMut = if original_content.trim().is_empty() {
            DocumentMut::new()
        } else {
            original_content.parse().map_err(|e: toml_edit::TomlError| {
                AdapterError::Toml(e.to_string())
            })?
        };

        // 清空 [mcp_servers] 节
        let mcp_table = doc
            .entry("mcp_servers")
            .or_insert(Item::Table(Table::new()));

        if let Some(table) = mcp_table.as_table_mut() {
            table.clear();
        }

        Ok(doc.to_string())
    }
}

// ===== 配置文件结构定义 =====

/// Codex 配置文件结构
#[derive(Debug, serde::Deserialize)]
struct CodexConfigFile {
    #[serde(default)]
    mcp_servers: Option<HashMap<String, CodexMcpServerConfig>>,
}

/// Codex MCP 服务器配置
#[derive(Debug, serde::Deserialize)]
struct CodexMcpServerConfig {
    /// 命令（stdio 模式）
    #[serde(default)]
    command: Option<String>,
    /// 命令参数
    #[serde(default)]
    args: Option<Vec<String>>,
    /// 环境变量
    #[serde(default)]
    env: Option<HashMap<String, String>>,
    /// URL（HTTP 模式）
    #[serde(default)]
    url: Option<String>,
    /// HTTP 头（Codex 使用 http_headers 而非 headers）
    #[serde(default)]
    http_headers: Option<HashMap<String, String>>,
}

// ===== 单元测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_adapter_id_and_name() {
        let adapter = CodexAdapter;
        assert_eq!(adapter.id(), "codex");
        assert_eq!(adapter.name(), "Codex");
    }

    #[test]
    fn test_codex_scan_patterns() {
        let adapter = CodexAdapter;
        let patterns = adapter.scan_patterns();

        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&(ConfigScope::Project, ".codex/config.toml".to_string())));
        assert!(patterns.contains(&(ConfigScope::User, "~/.codex/config.toml".to_string())));
    }

    #[test]
    fn test_codex_parse_basic() {
        let adapter = CodexAdapter;
        let content = r#"
[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path"]
"#;

        let path = Path::new("/project/.codex/config.toml");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "filesystem");
        assert_eq!(services[0].command, "npx");
        assert_eq!(services[0].adapter_id, "codex");
        assert_eq!(services[0].scope, ConfigScope::Project);
    }

    #[test]
    fn test_codex_parse_with_env() {
        let adapter = CodexAdapter;
        let content = r#"
[mcp_servers.database]
command = "uvx"
args = ["mcp-server-postgres"]

[mcp_servers.database.env]
DATABASE_URL = "$DATABASE_URL"
DEBUG = "true"
"#;

        let path = Path::new("/project/.codex/config.toml");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 1);
        let service = &services[0];
        assert!(service.env.is_some());
        let env = service.env.as_ref().unwrap();
        assert_eq!(env.get("DATABASE_URL"), Some(&"$DATABASE_URL".to_string()));
    }

    #[test]
    fn test_codex_parse_multiple_servers() {
        let adapter = CodexAdapter;
        let content = r#"
[mcp_servers.server1]
command = "server1-cmd"

[mcp_servers.server2]
command = "server2-cmd"
args = ["--flag"]
"#;

        let path = Path::new("/project/.codex/config.toml");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert_eq!(services.len(), 2);
        let names: Vec<_> = services.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"server1"));
        assert!(names.contains(&"server2"));
    }

    #[test]
    fn test_codex_parse_http_servers() {
        let adapter = CodexAdapter;
        let content = r#"
[mcp_servers.local]
command = "local-mcp"

[mcp_servers.remote]
url = "http://remote.example.com/mcp"

[mcp_servers.remote.http_headers]
Authorization = "Bearer xxx"

[mcp_servers.deepwiki]
url = "https://mcp.deepwiki.com/mcp"
"#;

        let path = Path::new("/project/.codex/config.toml");
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
        assert!(remote.headers.is_some());

        let deepwiki = services.iter().find(|s| s.name == "deepwiki").unwrap();
        assert_eq!(deepwiki.transport_type, crate::models::mcp::McpTransportType::Http);
        assert_eq!(deepwiki.url, Some("https://mcp.deepwiki.com/mcp".to_string()));
    }

    #[test]
    fn test_codex_parse_empty_servers() {
        let adapter = CodexAdapter;
        let content = r#"
model = "gpt-4"
"#;

        let path = Path::new("/project/.codex/config.toml");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        assert!(services.is_empty());
    }

    #[test]
    fn test_codex_inject_gateway() {
        let adapter = CodexAdapter;
        let original = r#"
# Codex configuration
model = "gpt-4"
temperature = 0.7

[mcp_servers.existing]
command = "existing-mcp"
"#;

        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "codex-token",
        );

        let result = adapter.inject_gateway(original, &config).unwrap();

        // 验证其他配置保留
        assert!(result.contains("model = \"gpt-4\""));
        assert!(result.contains("temperature = 0.7"));

        // 验证 gateway 注入
        assert!(result.contains("mantra-gateway"));
        assert!(result.contains("http://127.0.0.1:8080/mcp"));
        assert!(result.contains("Bearer codex-token"));

        // 验证旧服务被移除
        assert!(!result.contains("existing-mcp"));
    }

    #[test]
    fn test_codex_inject_gateway_empty_file() {
        let adapter = CodexAdapter;
        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "token",
        );

        let result = adapter.inject_gateway("", &config).unwrap();

        assert!(result.contains("mantra-gateway"));
        assert!(result.contains("http://127.0.0.1:8080/mcp"));
    }

    #[test]
    fn test_codex_inject_gateway_preserve_comments() {
        let adapter = CodexAdapter;
        let original = r#"
# This is my Codex config
# Very important settings

model = "gpt-4"

# MCP servers below
[mcp_servers.old]
command = "old-mcp"
"#;

        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "token",
        );

        let result = adapter.inject_gateway(original, &config).unwrap();

        // 验证注释保留
        assert!(result.contains("# This is my Codex config"));
        assert!(result.contains("# Very important settings"));
    }

    #[test]
    fn test_codex_uses_http_headers() {
        let adapter = CodexAdapter;
        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/mcp",
            "token",
        );

        let result = adapter.inject_gateway("", &config).unwrap();

        // Codex 使用 http_headers 而非 headers
        assert!(result.contains("http_headers"));
        assert!(result.contains("Authorization"));
    }

    // ===== Story 11.25: clear_mcp_servers 测试 =====

    #[test]
    fn test_codex_clear_mcp_servers_basic() {
        let adapter = CodexAdapter;
        let content = r#"
[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@mcp/filesystem"]

[mcp_servers.database]
command = "uvx"
args = ["mcp-postgres"]
"#;

        let result = adapter.clear_mcp_servers(content).unwrap();

        // mcp_servers 应该被清空
        assert!(!result.contains("filesystem"));
        assert!(!result.contains("database"));
        // 清空后的表可能显示为 [mcp_servers] 或根本不显示（toml_edit 行为）
    }

    #[test]
    fn test_codex_clear_mcp_servers_preserves_other_settings() {
        let adapter = CodexAdapter;
        let content = r#"
# Codex config
model = "gpt-4"
temperature = 0.7

[mcp_servers.old]
command = "old-mcp"

[other_section]
key = "value"
"#;

        let result = adapter.clear_mcp_servers(content).unwrap();

        // 其他配置保留
        assert!(result.contains("model = \"gpt-4\""));
        assert!(result.contains("temperature = 0.7"));
        assert!(result.contains("[other_section]"));
        assert!(result.contains("key = \"value\""));

        // 旧服务被清空
        assert!(!result.contains("old-mcp"));
    }

    #[test]
    fn test_codex_clear_mcp_servers_preserves_comments() {
        let adapter = CodexAdapter;
        let content = r#"
# This is my Codex config
model = "gpt-4"

# MCP servers
[mcp_servers.test]
command = "test"
"#;

        let result = adapter.clear_mcp_servers(content).unwrap();

        // 注释保留
        assert!(result.contains("# This is my Codex config"));
    }

    #[test]
    fn test_codex_clear_mcp_servers_empty_content() {
        let adapter = CodexAdapter;

        let result = adapter.clear_mcp_servers("").unwrap();

        // 空内容也能正确处理
        assert!(result.contains("mcp_servers"));
    }

    #[test]
    fn test_codex_clear_mcp_servers_no_existing_section() {
        let adapter = CodexAdapter;
        let content = r#"
model = "gpt-4"
temperature = 0.7
"#;

        let result = adapter.clear_mcp_servers(content).unwrap();

        // 应该添加空的 mcp_servers 节
        assert!(result.contains("model = \"gpt-4\""));
        assert!(result.contains("temperature = 0.7"));
    }
}
