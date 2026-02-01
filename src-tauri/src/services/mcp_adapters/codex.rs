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
                // Codex 使用 stdio 传输模式
                if let Some(command) = server.command {
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
                    });
                }
                // 跳过 URL 模式的服务（如已配置的 gateway）
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
    #[allow(dead_code)]
    #[serde(default)]
    url: Option<String>,
    /// HTTP 头（Codex 使用 http_headers 而非 headers）
    #[allow(dead_code)]
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
    fn test_codex_parse_skip_url_servers() {
        let adapter = CodexAdapter;
        let content = r#"
[mcp_servers.local]
command = "local-mcp"

[mcp_servers.remote]
url = "http://remote.example.com/mcp"

[mcp_servers.remote.http_headers]
Authorization = "Bearer xxx"
"#;

        let path = Path::new("/project/.codex/config.toml");
        let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

        // URL 服务应该被跳过（没有 command）
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "local");
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
            "http://127.0.0.1:8080/message",
            "codex-token",
        );

        let result = adapter.inject_gateway(original, &config).unwrap();

        // 验证其他配置保留
        assert!(result.contains("model = \"gpt-4\""));
        assert!(result.contains("temperature = 0.7"));

        // 验证 gateway 注入
        assert!(result.contains("mantra-gateway"));
        assert!(result.contains("http://127.0.0.1:8080/message"));
        assert!(result.contains("Bearer codex-token"));

        // 验证旧服务被移除
        assert!(!result.contains("existing-mcp"));
    }

    #[test]
    fn test_codex_inject_gateway_empty_file() {
        let adapter = CodexAdapter;
        let config = GatewayInjectionConfig::new(
            "http://127.0.0.1:8080/message",
            "token",
        );

        let result = adapter.inject_gateway("", &config).unwrap();

        assert!(result.contains("mantra-gateway"));
        assert!(result.contains("http://127.0.0.1:8080/message"));
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
            "http://127.0.0.1:8080/message",
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
            "http://127.0.0.1:8080/message",
            "token",
        );

        let result = adapter.inject_gateway("", &config).unwrap();

        // Codex 使用 http_headers 而非 headers
        assert!(result.contains("http_headers"));
        assert!(result.contains("Authorization"));
    }
}
