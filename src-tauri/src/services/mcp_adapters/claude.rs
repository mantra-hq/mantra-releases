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
                            local_project_path: None,
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

// ===== Story 11.21: Local Scope 支持 =====

impl ClaudeAdapter {
    /// 扫描 ~/.claude.json 中的 projects.* local scope 配置 (Story 11.21)
    ///
    /// Claude Code 支持在 `~/.claude.json` 中为特定项目配置 MCP 服务器：
    /// ```json
    /// {
    ///   "projects": {
    ///     "/path/to/project-a": {
    ///       "mcpServers": { ... }
    ///     },
    ///     "/path/to/project-b": {
    ///       "mcpServers": { ... }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// 这些配置属于 "local" scope，存储在用户配置文件中但针对特定项目。
    ///
    /// # Arguments
    /// * `user_config_path` - ~/.claude.json 的路径
    /// * `content` - 配置文件内容
    ///
    /// # Returns
    /// 所有 local scope 的 MCP 服务列表
    pub fn parse_local_scopes(
        &self,
        user_config_path: &Path,
        content: &str,
    ) -> Result<Vec<DetectedService>, AdapterError> {
        let stripped = strip_json_comments(content);
        let config: ClaudeConfigFileWithProjects = serde_json::from_str(&stripped)?;

        let mut services = Vec::new();

        if let Some(projects) = config.projects {
            for (project_path, project_config) in projects {
                if let Some(mcp_servers) = project_config.mcp_servers {
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
                                    source_file: user_config_path.to_path_buf(),
                                    adapter_id: self.id().to_string(),
                                    scope: ConfigScope::Local,
                                    local_project_path: Some(project_path.clone()),
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
                                    source_file: user_config_path.to_path_buf(),
                                    adapter_id: self.id().to_string(),
                                    scope: ConfigScope::Local,
                                    local_project_path: Some(project_path.clone()),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(services)
    }

    /// 获取指定项目的 local scope 服务 (Story 11.21)
    ///
    /// # Arguments
    /// * `user_config_path` - ~/.claude.json 的路径
    /// * `content` - 配置文件内容
    /// * `target_project_path` - 目标项目路径
    ///
    /// # Returns
    /// 该项目的 local scope MCP 服务列表
    pub fn parse_local_scope_for_project(
        &self,
        user_config_path: &Path,
        content: &str,
        target_project_path: &str,
    ) -> Result<Vec<DetectedService>, AdapterError> {
        let all_services = self.parse_local_scopes(user_config_path, content)?;

        Ok(all_services
            .into_iter()
            .filter(|s| {
                s.local_project_path
                    .as_ref()
                    .map_or(false, |p| p == target_project_path)
            })
            .collect())
    }

    /// 列出所有有 local scope 配置的项目路径 (Story 11.21)
    ///
    /// # Arguments
    /// * `content` - ~/.claude.json 配置文件内容
    ///
    /// # Returns
    /// 项目路径列表和每个项目的 MCP 服务数量
    pub fn list_local_scope_projects(
        &self,
        content: &str,
    ) -> Result<Vec<LocalScopeProject>, AdapterError> {
        let stripped = strip_json_comments(content);
        let config: ClaudeConfigFileWithProjects = serde_json::from_str(&stripped)?;

        let mut projects = Vec::new();

        if let Some(project_map) = config.projects {
            for (project_path, project_config) in project_map {
                let service_count = project_config
                    .mcp_servers
                    .as_ref()
                    .map_or(0, |servers| servers.len());

                if service_count > 0 {
                    let service_names: Vec<String> = project_config
                        .mcp_servers
                        .as_ref()
                        .map_or(Vec::new(), |servers| servers.keys().cloned().collect());

                    projects.push(LocalScopeProject {
                        project_path,
                        service_count,
                        service_names,
                    });
                }
            }
        }

        // 按项目路径排序
        projects.sort_by(|a, b| a.project_path.cmp(&b.project_path));

        Ok(projects)
    }
}

/// Local Scope 项目信息 (Story 11.21)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalScopeProject {
    /// 项目路径（~/.claude.json 中 projects 下的键）
    pub project_path: String,
    /// MCP 服务数量
    pub service_count: usize,
    /// MCP 服务名称列表
    pub service_names: Vec<String>,
}

// ===== Story 11.21: Local Scope 接管（备份/清空/恢复）=====

impl ClaudeAdapter {
    /// 清空所有 local scope 的 mcpServers (Story 11.21 - AC3)
    ///
    /// 将 `~/.claude.json` 中每个 `projects.{path}.mcpServers` 清空为 `{}`，
    /// 但保留 `projects.{path}` 下的其他配置（如 `allowedTools`、`allowedPaths` 等）。
    ///
    /// # Arguments
    /// * `original_content` - 原始 ~/.claude.json 内容
    ///
    /// # Returns
    /// 更新后的 JSON 字符串
    pub fn clear_local_scope_mcp_servers(
        &self,
        original_content: &str,
    ) -> Result<String, AdapterError> {
        let stripped = strip_json_comments(original_content);
        let mut root: serde_json::Value = if stripped.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&stripped)?
        };

        // 获取 projects 对象并清空每个项目的 mcpServers
        if let Some(projects) = root.get_mut("projects").and_then(|p| p.as_object_mut()) {
            for (_path, project_config) in projects.iter_mut() {
                if let Some(obj) = project_config.as_object_mut() {
                    // 清空 mcpServers 但保留其他字段
                    obj.insert("mcpServers".to_string(), serde_json::json!({}));
                }
            }
        }

        serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
    }

    /// 清空指定项目的 local scope mcpServers (Story 11.21)
    ///
    /// 仅清空指定项目路径的 mcpServers，不影响其他项目。
    ///
    /// # Arguments
    /// * `original_content` - 原始 ~/.claude.json 内容
    /// * `project_path` - 要清空的项目路径
    ///
    /// # Returns
    /// 更新后的 JSON 字符串
    pub fn clear_local_scope_for_project(
        &self,
        original_content: &str,
        project_path: &str,
    ) -> Result<String, AdapterError> {
        let stripped = strip_json_comments(original_content);
        let mut root: serde_json::Value = if stripped.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&stripped)?
        };

        // 获取 projects 对象并清空指定项目的 mcpServers
        if let Some(projects) = root.get_mut("projects").and_then(|p| p.as_object_mut()) {
            if let Some(project_config) = projects.get_mut(project_path) {
                if let Some(obj) = project_config.as_object_mut() {
                    obj.insert("mcpServers".to_string(), serde_json::json!({}));
                }
            }
        }

        serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
    }

    /// 注入 Gateway 配置并清空所有 local scope (Story 11.21)
    ///
    /// 同时执行以下操作：
    /// 1. 将 user scope 的 `mcpServers` 替换为 Gateway 配置
    /// 2. 清空所有 `projects.{path}.mcpServers`
    ///
    /// # Arguments
    /// * `original_content` - 原始 ~/.claude.json 内容
    /// * `config` - Gateway 注入配置
    ///
    /// # Returns
    /// 更新后的 JSON 字符串
    pub fn inject_gateway_with_local_scope_clear(
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

        // 1. 注入 Gateway 到 user scope (mcpServers)
        let gateway_config = serde_json::json!({
            "mantra-gateway": {
                "url": config.url,
                "headers": {
                    "Authorization": config.authorization_header()
                }
            }
        });
        obj.insert("mcpServers".to_string(), gateway_config);

        // 2. 清空所有 local scope 的 mcpServers
        if let Some(projects) = obj.get_mut("projects").and_then(|p| p.as_object_mut()) {
            for (_path, project_config) in projects.iter_mut() {
                if let Some(proj_obj) = project_config.as_object_mut() {
                    proj_obj.insert("mcpServers".to_string(), serde_json::json!({}));
                }
            }
        }

        serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
    }

    /// 恢复指定项目的 local scope mcpServers (Story 11.21 - AC4)
    ///
    /// 将备份的 mcpServers 内容恢复到指定项目。
    /// - 如果项目条目已存在，仅更新 mcpServers
    /// - 如果项目条目不存在，创建新条目
    /// - 不影响其他项目的配置
    /// - 不影响 user scope 的配置
    ///
    /// # Arguments
    /// * `original_content` - 当前 ~/.claude.json 内容
    /// * `project_path` - 要恢复的项目路径
    /// * `backup_mcp_servers` - 备份的 mcpServers JSON 内容
    ///
    /// # Returns
    /// 更新后的 JSON 字符串
    pub fn restore_local_scope_mcp_servers(
        &self,
        original_content: &str,
        project_path: &str,
        backup_mcp_servers: &serde_json::Value,
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

        // 确保 projects 字段存在
        if !obj.contains_key("projects") {
            obj.insert("projects".to_string(), serde_json::json!({}));
        }

        let projects = obj
            .get_mut("projects")
            .and_then(|p| p.as_object_mut())
            .ok_or_else(|| {
                AdapterError::InvalidFormat("projects must be a JSON object".to_string())
            })?;

        // 检查项目是否存在
        if let Some(project_config) = projects.get_mut(project_path) {
            // 项目存在，更新 mcpServers
            if let Some(proj_obj) = project_config.as_object_mut() {
                proj_obj.insert("mcpServers".to_string(), backup_mcp_servers.clone());
            }
        } else {
            // 项目不存在，创建新条目
            projects.insert(
                project_path.to_string(),
                serde_json::json!({
                    "mcpServers": backup_mcp_servers
                }),
            );
        }

        serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
    }

    /// 提取指定项目的 local scope mcpServers 用于备份 (Story 11.21 - AC2)
    ///
    /// # Arguments
    /// * `content` - ~/.claude.json 内容
    /// * `project_path` - 项目路径
    ///
    /// # Returns
    /// 该项目的 mcpServers JSON 对象（用于备份存储）
    pub fn extract_local_scope_backup(
        &self,
        content: &str,
        project_path: &str,
    ) -> Result<serde_json::Value, AdapterError> {
        let stripped = strip_json_comments(content);
        let root: serde_json::Value = serde_json::from_str(&stripped)?;

        // 获取指定项目的 mcpServers
        let mcp_servers = root
            .get("projects")
            .and_then(|p| p.get(project_path))
            .and_then(|proj| proj.get("mcpServers"))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        Ok(mcp_servers)
    }
}

// ===== 配置文件结构定义 =====

/// Claude 配置文件结构（仅顶层 mcpServers）
#[derive(Debug, serde::Deserialize)]
struct ClaudeConfigFile {
    #[serde(alias = "mcpServers")]
    mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

/// Claude 配置文件结构（包含 projects 字段）(Story 11.21)
///
/// 用于解析 local scope 配置
#[derive(Debug, serde::Deserialize)]
struct ClaudeConfigFileWithProjects {
    /// 顶层 mcpServers（user scope）
    #[serde(alias = "mcpServers")]
    #[allow(dead_code)]
    mcp_servers: Option<HashMap<String, McpServerConfig>>,
    /// 项目特定配置（local scope）
    projects: Option<HashMap<String, ProjectConfig>>,
}

/// 项目特定配置 (Story 11.21)
#[derive(Debug, serde::Deserialize)]
struct ProjectConfig {
    /// 项目的 MCP 服务器配置
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

    // ===== Story 11.21: Local Scope 测试 =====

    #[test]
    fn test_parse_local_scopes_basic() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {
                "user-service": {"command": "npx", "args": ["-y", "user-mcp"]}
            },
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {
                        "project-a-mcp": {"command": "npx", "args": ["-y", "project-a-mcp"]}
                    }
                },
                "/home/user/project-b": {
                    "mcpServers": {
                        "project-b-mcp": {"command": "uvx", "args": ["project-b-mcp"]},
                        "project-b-http": {"url": "http://localhost:8080/mcp"}
                    }
                }
            }
        }"#;

        let path = Path::new("/home/user/.claude.json");
        let services = adapter.parse_local_scopes(path, content).unwrap();

        // 应该有 3 个 local scope 服务（不包括顶层 user scope 的服务）
        assert_eq!(services.len(), 3);

        // 验证所有服务都是 local scope
        for service in &services {
            assert_eq!(service.scope, ConfigScope::Local);
            assert!(service.local_project_path.is_some());
            assert_eq!(service.adapter_id, "claude");
        }

        // 验证 project-a 的服务
        let project_a_service = services.iter().find(|s| s.name == "project-a-mcp").unwrap();
        assert_eq!(project_a_service.local_project_path, Some("/home/user/project-a".to_string()));
        assert_eq!(project_a_service.command, "npx");

        // 验证 project-b 的服务
        let project_b_stdio = services.iter().find(|s| s.name == "project-b-mcp").unwrap();
        assert_eq!(project_b_stdio.local_project_path, Some("/home/user/project-b".to_string()));

        let project_b_http = services.iter().find(|s| s.name == "project-b-http").unwrap();
        assert_eq!(project_b_http.transport_type, crate::models::mcp::McpTransportType::Http);
        assert_eq!(project_b_http.url, Some("http://localhost:8080/mcp".to_string()));
    }

    #[test]
    fn test_parse_local_scopes_empty_projects() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {"user-service": {"command": "npx"}}
        }"#;

        let path = Path::new("/home/user/.claude.json");
        let services = adapter.parse_local_scopes(path, content).unwrap();

        assert!(services.is_empty());
    }

    #[test]
    fn test_parse_local_scopes_project_without_mcp_servers() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "projects": {
                "/home/user/empty-project": {
                    "allowedPaths": ["/tmp"]
                }
            }
        }"#;

        let path = Path::new("/home/user/.claude.json");
        let services = adapter.parse_local_scopes(path, content).unwrap();

        assert!(services.is_empty());
    }

    #[test]
    fn test_parse_local_scope_for_project() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {
                        "service-a1": {"command": "a1"},
                        "service-a2": {"command": "a2"}
                    }
                },
                "/home/user/project-b": {
                    "mcpServers": {
                        "service-b1": {"command": "b1"}
                    }
                }
            }
        }"#;

        let path = Path::new("/home/user/.claude.json");

        // 查询 project-a
        let services_a = adapter.parse_local_scope_for_project(path, content, "/home/user/project-a").unwrap();
        assert_eq!(services_a.len(), 2);

        // 查询 project-b
        let services_b = adapter.parse_local_scope_for_project(path, content, "/home/user/project-b").unwrap();
        assert_eq!(services_b.len(), 1);

        // 查询不存在的项目
        let services_none = adapter.parse_local_scope_for_project(path, content, "/home/user/nonexistent").unwrap();
        assert!(services_none.is_empty());
    }

    #[test]
    fn test_list_local_scope_projects() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {
                        "service-a1": {"command": "a1"},
                        "service-a2": {"command": "a2"}
                    }
                },
                "/home/user/project-b": {
                    "mcpServers": {
                        "service-b1": {"command": "b1"}
                    }
                },
                "/home/user/empty-project": {
                    "allowedPaths": []
                }
            }
        }"#;

        let projects = adapter.list_local_scope_projects(content).unwrap();

        // 应该有 2 个项目（空项目被排除）
        assert_eq!(projects.len(), 2);

        // 验证按路径排序
        assert_eq!(projects[0].project_path, "/home/user/project-a");
        assert_eq!(projects[1].project_path, "/home/user/project-b");

        // 验证服务数量
        assert_eq!(projects[0].service_count, 2);
        assert_eq!(projects[1].service_count, 1);

        // 验证服务名称
        assert!(projects[0].service_names.contains(&"service-a1".to_string()));
        assert!(projects[0].service_names.contains(&"service-a2".to_string()));
        assert!(projects[1].service_names.contains(&"service-b1".to_string()));
    }

    #[test]
    fn test_parse_local_scopes_with_comments() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            // User-level MCP servers
            "mcpServers": {},
            /* Project-specific configurations */
            "projects": {
                "/home/user/my-project": {
                    "mcpServers": {
                        // Git MCP for this project
                        "git-mcp": {"command": "npx", "args": ["-y", "@anthropic/git-mcp"]}
                    }
                }
            }
        }"#;

        let path = Path::new("/home/user/.claude.json");
        let services = adapter.parse_local_scopes(path, content).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "git-mcp");
        assert_eq!(services[0].local_project_path, Some("/home/user/my-project".to_string()));
    }

    // ===== Story 11.21: Local Scope 接管（清空/恢复）测试 =====

    #[test]
    fn test_clear_local_scope_mcp_servers() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {
                "user-service": {"command": "npx"}
            },
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {
                        "service-a": {"command": "a"}
                    },
                    "allowedTools": ["*"]
                },
                "/home/user/project-b": {
                    "mcpServers": {
                        "service-b1": {"command": "b1"},
                        "service-b2": {"command": "b2"}
                    }
                }
            },
            "autoApprove": ["read"]
        }"#;

        let result = adapter.clear_local_scope_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证 user scope 的 mcpServers 被保留
        assert!(parsed["mcpServers"]["user-service"].is_object());

        // 验证 project-a 的 mcpServers 被清空
        assert_eq!(parsed["projects"]["/home/user/project-a"]["mcpServers"], serde_json::json!({}));
        // 验证 project-a 的其他字段被保留
        assert_eq!(parsed["projects"]["/home/user/project-a"]["allowedTools"], serde_json::json!(["*"]));

        // 验证 project-b 的 mcpServers 被清空
        assert_eq!(parsed["projects"]["/home/user/project-b"]["mcpServers"], serde_json::json!({}));

        // 验证顶层其他字段被保留
        assert_eq!(parsed["autoApprove"], serde_json::json!(["read"]));
    }

    #[test]
    fn test_clear_local_scope_for_project() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {"service-a": {"command": "a"}}
                },
                "/home/user/project-b": {
                    "mcpServers": {"service-b": {"command": "b"}}
                }
            }
        }"#;

        let result = adapter.clear_local_scope_for_project(content, "/home/user/project-a").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // project-a 被清空
        assert_eq!(parsed["projects"]["/home/user/project-a"]["mcpServers"], serde_json::json!({}));
        // project-b 保持不变
        assert!(parsed["projects"]["/home/user/project-b"]["mcpServers"]["service-b"].is_object());
    }

    #[test]
    fn test_clear_local_scope_nonexistent_project() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {"service-a": {"command": "a"}}
                }
            }
        }"#;

        // 清空不存在的项目不会报错
        let result = adapter.clear_local_scope_for_project(content, "/home/user/nonexistent").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // project-a 保持不变
        assert!(parsed["projects"]["/home/user/project-a"]["mcpServers"]["service-a"].is_object());
    }

    #[test]
    fn test_inject_gateway_with_local_scope_clear() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {
                "old-user-service": {"command": "old"}
            },
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {"local-service-a": {"command": "a"}},
                    "allowedTools": ["read", "write"]
                }
            },
            "permissions": {"allowedPaths": ["/tmp"]}
        }"#;

        let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "test-token");
        let result = adapter.inject_gateway_with_local_scope_clear(content, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证 user scope 被 gateway 替换
        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
        assert_eq!(parsed["mcpServers"]["mantra-gateway"]["url"], "http://127.0.0.1:8080/mcp");
        assert!(parsed["mcpServers"]["old-user-service"].is_null());

        // 验证 local scope 被清空
        assert_eq!(parsed["projects"]["/home/user/project-a"]["mcpServers"], serde_json::json!({}));
        // 验证 local scope 的其他字段被保留
        assert_eq!(parsed["projects"]["/home/user/project-a"]["allowedTools"], serde_json::json!(["read", "write"]));

        // 验证顶层其他字段被保留
        assert!(parsed["permissions"]["allowedPaths"].is_array());
    }

    #[test]
    fn test_restore_local_scope_mcp_servers_existing_project() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {"gateway": {"url": "http://..."}},
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {},
                    "allowedTools": ["*"]
                }
            }
        }"#;

        let backup = serde_json::json!({
            "restored-service": {"command": "npx", "args": ["-y", "restored-mcp"]}
        });

        let result = adapter.restore_local_scope_mcp_servers(content, "/home/user/project-a", &backup).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证 mcpServers 被恢复
        assert!(parsed["projects"]["/home/user/project-a"]["mcpServers"]["restored-service"].is_object());
        assert_eq!(
            parsed["projects"]["/home/user/project-a"]["mcpServers"]["restored-service"]["command"],
            "npx"
        );

        // 验证其他字段被保留
        assert_eq!(parsed["projects"]["/home/user/project-a"]["allowedTools"], serde_json::json!(["*"]));

        // 验证 user scope 不受影响
        assert!(parsed["mcpServers"]["gateway"].is_object());
    }

    #[test]
    fn test_restore_local_scope_mcp_servers_new_project() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {"gateway": {"url": "http://..."}}
        }"#;

        let backup = serde_json::json!({
            "new-project-service": {"command": "new"}
        });

        let result = adapter.restore_local_scope_mcp_servers(content, "/home/user/new-project", &backup).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证新项目被创建
        assert!(parsed["projects"]["/home/user/new-project"]["mcpServers"]["new-project-service"].is_object());
    }

    #[test]
    fn test_restore_local_scope_does_not_affect_other_projects() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {}
                },
                "/home/user/project-b": {
                    "mcpServers": {"existing": {"command": "existing"}}
                }
            }
        }"#;

        let backup = serde_json::json!({
            "restored-a": {"command": "a"}
        });

        let result = adapter.restore_local_scope_mcp_servers(content, "/home/user/project-a", &backup).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // project-a 被恢复
        assert!(parsed["projects"]["/home/user/project-a"]["mcpServers"]["restored-a"].is_object());

        // project-b 不受影响
        assert!(parsed["projects"]["/home/user/project-b"]["mcpServers"]["existing"].is_object());
    }

    #[test]
    fn test_extract_local_scope_backup() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "projects": {
                "/home/user/project-a": {
                    "mcpServers": {
                        "service-1": {"command": "cmd1", "args": ["arg1"]},
                        "service-2": {"url": "http://localhost:8080"}
                    },
                    "allowedTools": ["*"]
                }
            }
        }"#;

        let backup = adapter.extract_local_scope_backup(content, "/home/user/project-a").unwrap();

        // 验证备份内容
        assert!(backup["service-1"].is_object());
        assert_eq!(backup["service-1"]["command"], "cmd1");
        assert!(backup["service-2"].is_object());
        assert_eq!(backup["service-2"]["url"], "http://localhost:8080");
    }

    #[test]
    fn test_extract_local_scope_backup_nonexistent() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "projects": {}
        }"#;

        let backup = adapter.extract_local_scope_backup(content, "/home/user/nonexistent").unwrap();

        // 不存在的项目返回空对象
        assert_eq!(backup, serde_json::json!({}));
    }

    #[test]
    fn test_clear_local_scope_empty_projects() {
        let adapter = ClaudeAdapter;
        let content = r#"{
            "mcpServers": {"user": {"command": "user"}}
        }"#;

        // 没有 projects 字段也能正常处理
        let result = adapter.clear_local_scope_mcp_servers(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // user scope 保持不变
        assert!(parsed["mcpServers"]["user"].is_object());
    }
}
