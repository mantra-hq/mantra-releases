//! Claude Code/Desktop Adapter
//!
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 支持 Claude Code (.mcp.json, ~/.claude.json) 配置文件解析和 Gateway 注入。

use std::collections::HashMap;
use std::path::Path;

use super::{
    common::strip_json_comments,
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
        // 注意: Claude Code 要求 HTTP 类型的服务器显式指定 "type": "http"
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

        // 2. 处理所有项目的 MCP 启用/禁用列表，确保 Gateway 不被屏蔽
        if let Some(projects) = obj.get_mut("projects").and_then(|p| p.as_object_mut()) {
            for (_path, project_config) in projects.iter_mut() {
                if let Some(proj_obj) = project_config.as_object_mut() {
                    Self::ensure_gateway_enabled_in_project(proj_obj);
                }
            }
        }

        serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
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
    /// 确保 mantra-gateway 不被 disabledMcpjsonServers/enabledMcpjsonServers 屏蔽
    ///
    /// Claude Code 的 `~/.claude.json` 中每个项目可以有：
    /// - `disabledMcpjsonServers`: 被禁用的服务器名称列表
    /// - `enabledMcpjsonServers`: 被启用的服务器名称列表（如果非空，只有在列表中的才启用）
    ///
    /// 此方法确保 Gateway 不会被这些配置屏蔽：
    /// 1. 从 `disabledMcpjsonServers` 中移除 `mantra-gateway`
    /// 2. 如果 `enabledMcpjsonServers` 非空，将 `mantra-gateway` 添加进去
    fn ensure_gateway_enabled_in_project(project_config: &mut serde_json::Map<String, serde_json::Value>) {
        const GATEWAY_NAME: &str = "mantra-gateway";

        // 1. 从 disabledMcpjsonServers 中移除 mantra-gateway
        if let Some(disabled) = project_config.get_mut("disabledMcpjsonServers") {
            if let Some(arr) = disabled.as_array_mut() {
                arr.retain(|v| v.as_str() != Some(GATEWAY_NAME));
            }
        }

        // 2. 如果 enabledMcpjsonServers 非空，确保 mantra-gateway 在列表中
        if let Some(enabled) = project_config.get_mut("enabledMcpjsonServers") {
            if let Some(arr) = enabled.as_array_mut() {
                if !arr.is_empty() {
                    // 列表非空，检查是否已包含 Gateway
                    let contains_gateway = arr.iter().any(|v| v.as_str() == Some(GATEWAY_NAME));
                    if !contains_gateway {
                        arr.push(serde_json::Value::String(GATEWAY_NAME.to_string()));
                    }
                }
            }
        }
    }

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
    /// 3. 确保 Gateway 不被 disabledMcpjsonServers/enabledMcpjsonServers 屏蔽
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
        // 注意: Claude Code 要求 HTTP 类型的服务器显式指定 "type": "http"
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

        // 2. 清空所有 local scope 的 mcpServers，并确保 Gateway 不被屏蔽
        if let Some(projects) = obj.get_mut("projects").and_then(|p| p.as_object_mut()) {
            for (_path, project_config) in projects.iter_mut() {
                if let Some(proj_obj) = project_config.as_object_mut() {
                    proj_obj.insert("mcpServers".to_string(), serde_json::json!({}));
                    // 确保 Gateway 不被 disabledMcpjsonServers/enabledMcpjsonServers 屏蔽
                    Self::ensure_gateway_enabled_in_project(proj_obj);
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
mod tests;
