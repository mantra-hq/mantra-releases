//! MCP 服务存储操作
//!
//! Story 11.2: MCP 服务数据模型 - Task 3
//!
//! 提供 mcp_services 表的 CRUD 操作

use regex;
use rusqlite::{params, OptionalExtension, Row};
use std::collections::HashMap;
use std::path::PathBuf;

use super::database::Database;
use super::error::StorageError;
use crate::models::mcp::{
    CreateMcpServiceRequest, McpService, McpServiceSource, McpServiceTool, McpServiceWithOverride,
    McpTransportType, ProjectMcpService, TakeoverBackup, TakeoverScope, TakeoverStatus, ToolPolicy,
    ToolType, UpdateMcpServiceRequest,
};

/// 从数据库行解析 McpService
fn parse_mcp_service_row(row: &Row) -> rusqlite::Result<McpService> {
    let source_str: String = row.get(5)?;
    let source = McpServiceSource::from_str(&source_str).unwrap_or(McpServiceSource::Manual);
    let enabled: i32 = row.get(7)?;

    // 解析 args JSON
    let args_json: Option<String> = row.get(3)?;
    let args = args_json.and_then(|s| serde_json::from_str(&s).ok());

    // 解析 env JSON
    let env_json: Option<String> = row.get(4)?;
    let env = env_json.and_then(|s| serde_json::from_str(&s).ok());

    // 解析 transport_type（默认 stdio）
    let transport_type_str: String = row.get(10).unwrap_or_else(|_| "stdio".to_string());
    let transport_type =
        McpTransportType::from_str(&transport_type_str).unwrap_or(McpTransportType::Stdio);

    // 解析 url
    let url: Option<String> = row.get(11).ok().flatten();

    // 解析 headers JSON
    let headers_json: Option<String> = row.get(12).ok().flatten();
    let headers: Option<HashMap<String, String>> =
        headers_json.and_then(|s| serde_json::from_str(&s).ok());

    // 解析 default_tool_policy JSON (Story 11.9 Phase 2)
    let default_tool_policy_json: Option<String> = row.get(13).ok().flatten();
    let default_tool_policy: Option<ToolPolicy> =
        default_tool_policy_json.and_then(|s| serde_json::from_str(&s).ok());

    // 解析 source_adapter_id 和 source_scope (Story 11.19)
    let source_adapter_id: Option<String> = row.get(14).ok().flatten();
    let source_scope: Option<String> = row.get(15).ok().flatten();

    Ok(McpService {
        id: row.get(0)?,
        name: row.get(1)?,
        transport_type,
        command: row.get(2)?,
        args,
        env,
        url,
        headers,
        source,
        source_file: row.get(6)?,
        source_adapter_id,
        source_scope,
        enabled: enabled != 0,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        default_tool_policy,
    })
}

/// 从数据库行解析 TakeoverBackup (Story 11.16)
///
/// 期望的列顺序: id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
fn parse_takeover_backup_row(row: &Row) -> rusqlite::Result<TakeoverBackup> {
    let tool_type_str: String = row.get(1)?;
    let status_str: String = row.get(6)?;
    let scope_str: String = row.get(7)?;
    let project_path_str: Option<String> = row.get(8)?;

    Ok(TakeoverBackup {
        id: row.get(0)?,
        tool_type: ToolType::from_str(&tool_type_str).unwrap_or(ToolType::ClaudeCode),
        scope: TakeoverScope::from_str(&scope_str).unwrap_or(TakeoverScope::User),
        project_path: project_path_str.map(PathBuf::from),
        original_path: PathBuf::from(row.get::<_, String>(2)?),
        backup_path: PathBuf::from(row.get::<_, String>(3)?),
        taken_over_at: row.get(4)?,
        restored_at: row.get(5)?,
        status: TakeoverStatus::from_str(&status_str).unwrap_or(TakeoverStatus::Active),
    })
}

impl Database {
    /// 创建 MCP 服务
    ///
    /// # Arguments
    /// * `request` - 创建服务的请求参数
    ///
    /// # Returns
    /// 创建的 MCP 服务
    pub fn create_mcp_service(
        &self,
        request: &CreateMcpServiceRequest,
    ) -> Result<McpService, StorageError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // 序列化 args、env 和 headers 为 JSON
        let args_json = request.args.as_ref().and_then(|a| serde_json::to_string(a).ok());
        let env_json = request.env.as_ref().and_then(|e| serde_json::to_string(e).ok());
        let headers_json = request
            .headers
            .as_ref()
            .and_then(|h| serde_json::to_string(h).ok());

        self.connection().execute(
            r#"INSERT INTO mcp_services (id, name, command, args, env, source, source_file, enabled, created_at, updated_at, transport_type, url, headers)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8, ?9, ?10, ?11)"#,
            params![
                &id,
                &request.name,
                &request.command,
                &args_json,
                &env_json,
                request.source.as_str(),
                &request.source_file,
                &now,
                request.transport_type.as_str(),
                &request.url,
                &headers_json,
            ],
        )?;

        self.get_mcp_service(&id)
    }

    /// 创建 MCP 服务（带来源追踪字段）(Story 11.19)
    ///
    /// # Arguments
    /// * `request` - 创建服务的请求参数
    /// * `source_adapter_id` - 首次导入时的工具来源
    /// * `source_scope` - 首次导入时的 scope ('project' | 'user')
    ///
    /// # Returns
    /// 创建的 MCP 服务
    pub fn create_mcp_service_with_source(
        &self,
        request: &CreateMcpServiceRequest,
        source_adapter_id: Option<&str>,
        source_scope: Option<&str>,
    ) -> Result<McpService, StorageError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // 序列化 args、env 和 headers 为 JSON
        let args_json = request.args.as_ref().and_then(|a| serde_json::to_string(a).ok());
        let env_json = request.env.as_ref().and_then(|e| serde_json::to_string(e).ok());
        let headers_json = request
            .headers
            .as_ref()
            .and_then(|h| serde_json::to_string(h).ok());

        self.connection().execute(
            r#"INSERT INTO mcp_services (id, name, command, args, env, source, source_file, enabled, created_at, updated_at, transport_type, url, headers, source_adapter_id, source_scope)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8, ?9, ?10, ?11, ?12, ?13)"#,
            params![
                &id,
                &request.name,
                &request.command,
                &args_json,
                &env_json,
                request.source.as_str(),
                &request.source_file,
                &now,
                request.transport_type.as_str(),
                &request.url,
                &headers_json,
                source_adapter_id,
                source_scope,
            ],
        )?;

        self.get_mcp_service(&id)
    }

    /// 按 ID 获取 MCP 服务
    ///
    /// # Arguments
    /// * `id` - 服务 ID
    ///
    /// # Returns
    /// MCP 服务，如果不存在则返回 None
    pub fn get_mcp_service(&self, id: &str) -> Result<McpService, StorageError> {
        self.connection()
            .query_row(
                r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at, transport_type, url, headers, default_tool_policy, source_adapter_id, source_scope
                   FROM mcp_services WHERE id = ?1"#,
                [id],
                parse_mcp_service_row,
            )
            .map_err(StorageError::Database)
    }

    /// 按名称获取 MCP 服务
    ///
    /// 注意：名称不唯一，可能返回多个结果，此方法返回第一个匹配项
    ///
    /// # Arguments
    /// * `name` - 服务名称
    ///
    /// # Returns
    /// MCP 服务，如果不存在则返回 None
    pub fn get_mcp_service_by_name(&self, name: &str) -> Result<Option<McpService>, StorageError> {
        let result = self.connection().query_row(
            r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at, transport_type, url, headers, default_tool_policy, source_adapter_id, source_scope
               FROM mcp_services WHERE name = ?1 LIMIT 1"#,
            [name],
            parse_mcp_service_row,
        );

        match result {
            Ok(service) => Ok(Some(service)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Database(e)),
        }
    }

    /// 列出所有 MCP 服务
    ///
    /// # Returns
    /// 所有 MCP 服务列表
    pub fn list_mcp_services(&self) -> Result<Vec<McpService>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at, transport_type, url, headers, default_tool_policy, source_adapter_id, source_scope
               FROM mcp_services ORDER BY name ASC"#,
        )?;

        let services = stmt
            .query_map([], parse_mcp_service_row)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(services)
    }

    /// 按来源列出 MCP 服务
    ///
    /// # Arguments
    /// * `source` - 服务来源
    ///
    /// # Returns
    /// 指定来源的 MCP 服务列表
    pub fn list_mcp_services_by_source(
        &self,
        source: &McpServiceSource,
    ) -> Result<Vec<McpService>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at, transport_type, url, headers, default_tool_policy, source_adapter_id, source_scope
               FROM mcp_services WHERE source = ?1 ORDER BY name ASC"#,
        )?;

        let services = stmt
            .query_map([source.as_str()], parse_mcp_service_row)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(services)
    }

    /// 更新 MCP 服务
    ///
    /// # Arguments
    /// * `id` - 服务 ID
    /// * `update` - 更新参数
    ///
    /// # Returns
    /// 更新后的 MCP 服务
    pub fn update_mcp_service(
        &self,
        id: &str,
        update: &UpdateMcpServiceRequest,
    ) -> Result<McpService, StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        // 获取当前服务
        let current = self.get_mcp_service(id)?;

        // 合并更新
        let name = update.name.as_ref().unwrap_or(&current.name);
        let transport_type = update
            .transport_type
            .as_ref()
            .unwrap_or(&current.transport_type);
        let command = update.command.as_ref().unwrap_or(&current.command);
        let args = update.args.as_ref().or(current.args.as_ref());
        let env = update.env.as_ref().or(current.env.as_ref());
        let url = update.url.as_ref().or(current.url.as_ref());
        let headers = update.headers.as_ref().or(current.headers.as_ref());
        let enabled = update.enabled.unwrap_or(current.enabled);

        // 序列化 args、env 和 headers 为 JSON
        let args_json = args.and_then(|a| serde_json::to_string(a).ok());
        let env_json = env.and_then(|e| serde_json::to_string(e).ok());
        let headers_json = headers.and_then(|h| serde_json::to_string(h).ok());

        self.connection().execute(
            r#"UPDATE mcp_services SET name = ?1, command = ?2, args = ?3, env = ?4, enabled = ?5, updated_at = ?6, transport_type = ?7, url = ?8, headers = ?9
               WHERE id = ?10"#,
            params![
                name,
                command,
                &args_json,
                &env_json,
                enabled as i32,
                &now,
                transport_type.as_str(),
                url,
                &headers_json,
                id,
            ],
        )?;

        self.get_mcp_service(id)
    }

    /// 删除 MCP 服务
    ///
    /// # Arguments
    /// * `id` - 服务 ID
    pub fn delete_mcp_service(&self, id: &str) -> Result<(), StorageError> {
        let affected = self.connection().execute(
            "DELETE FROM mcp_services WHERE id = ?1",
            [id],
        )?;

        if affected == 0 {
            return Err(StorageError::NotFound(format!("MCP service not found: {}", id)));
        }

        Ok(())
    }

    /// 切换 MCP 服务启用状态
    ///
    /// # Arguments
    /// * `id` - 服务 ID
    /// * `enabled` - 是否启用
    ///
    /// # Returns
    /// 更新后的 MCP 服务
    pub fn toggle_mcp_service(&self, id: &str, enabled: bool) -> Result<McpService, StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        let affected = self.connection().execute(
            "UPDATE mcp_services SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled as i32, &now, id],
        )?;

        if affected == 0 {
            return Err(StorageError::NotFound(format!("MCP service not found: {}", id)));
        }

        self.get_mcp_service(id)
    }

    // ===== 服务级默认 Tool Policy (Story 11.9 Phase 2) =====

    /// 获取服务的默认 Tool Policy
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    ///
    /// # Returns
    /// 服务的默认 Tool Policy，如果未配置则返回默认策略 (AllowAll)
    pub fn get_service_default_policy(&self, service_id: &str) -> Result<ToolPolicy, StorageError> {
        let service = self.get_mcp_service(service_id)?;
        Ok(service.get_default_tool_policy())
    }

    /// 更新服务的默认 Tool Policy
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    /// * `policy` - Tool Policy，传 None 清除默认策略
    ///
    /// # Returns
    /// 更新后的 MCP 服务
    pub fn update_service_default_policy(
        &self,
        service_id: &str,
        policy: Option<&ToolPolicy>,
    ) -> Result<McpService, StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        // 序列化 policy 为 JSON
        let policy_json = policy.and_then(|p| serde_json::to_string(p).ok());

        let affected = self.connection().execute(
            "UPDATE mcp_services SET default_tool_policy = ?1, updated_at = ?2 WHERE id = ?3",
            params![&policy_json, &now, service_id],
        )?;

        if affected == 0 {
            return Err(StorageError::NotFound(format!(
                "MCP service not found: {}",
                service_id
            )));
        }

        self.get_mcp_service(service_id)
    }

    // ===== 项目关联存储 (Task 4) =====

    /// 关联 MCP 服务到项目
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    /// * `service_id` - 服务 ID
    /// * `config_override` - 项目级配置覆盖（可选）
    pub fn link_service_to_project(
        &self,
        project_id: &str,
        service_id: &str,
        config_override: Option<&serde_json::Value>,
    ) -> Result<ProjectMcpService, StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        // 序列化 config_override 为 JSON
        let config_json = config_override.and_then(|c| serde_json::to_string(c).ok());

        self.connection().execute(
            r#"INSERT INTO project_mcp_services (project_id, service_id, config_override, created_at)
               VALUES (?1, ?2, ?3, ?4)"#,
            params![project_id, service_id, &config_json, &now],
        )?;

        Ok(ProjectMcpService {
            project_id: project_id.to_string(),
            service_id: service_id.to_string(),
            config_override: config_override.cloned(),
            detected_adapter_id: None,
            detected_config_path: None,
            created_at: now,
        })
    }

    /// 关联 MCP 服务到项目（带检测信息）(Story 11.19)
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    /// * `service_id` - 服务 ID
    /// * `config_override` - 项目级配置覆盖（可选）
    /// * `detected_adapter_id` - 检测到此服务的工具 ID
    /// * `detected_config_path` - 检测到此服务的配置文件路径
    pub fn link_service_to_project_with_detection(
        &self,
        project_id: &str,
        service_id: &str,
        config_override: Option<&serde_json::Value>,
        detected_adapter_id: Option<&str>,
        detected_config_path: Option<&str>,
    ) -> Result<ProjectMcpService, StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        // 序列化 config_override 为 JSON
        let config_json = config_override.and_then(|c| serde_json::to_string(c).ok());

        self.connection().execute(
            r#"INSERT INTO project_mcp_services (project_id, service_id, config_override, created_at, detected_adapter_id, detected_config_path)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![project_id, service_id, &config_json, &now, detected_adapter_id, detected_config_path],
        )?;

        Ok(ProjectMcpService {
            project_id: project_id.to_string(),
            service_id: service_id.to_string(),
            config_override: config_override.cloned(),
            detected_adapter_id: detected_adapter_id.map(|s| s.to_string()),
            detected_config_path: detected_config_path.map(|s| s.to_string()),
            created_at: now,
        })
    }

    /// 解除项目与 MCP 服务的关联
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    /// * `service_id` - 服务 ID
    pub fn unlink_service_from_project(
        &self,
        project_id: &str,
        service_id: &str,
    ) -> Result<(), StorageError> {
        let affected = self.connection().execute(
            "DELETE FROM project_mcp_services WHERE project_id = ?1 AND service_id = ?2",
            params![project_id, service_id],
        )?;

        if affected == 0 {
            return Err(StorageError::NotFound(format!(
                "Project-service link not found: {} - {}",
                project_id, service_id
            )));
        }

        Ok(())
    }

    /// 获取项目的 MCP 服务列表（包含配置覆盖）
    ///
    /// Story 11.19: 扩展返回 detected_adapter_id 和 detected_config_path
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    ///
    /// # Returns
    /// 项目关联的 MCP 服务列表，包含项目级配置覆盖和检测信息
    pub fn get_project_services(
        &self,
        project_id: &str,
    ) -> Result<Vec<McpServiceWithOverride>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT s.id, s.name, s.command, s.args, s.env, s.source, s.source_file, s.enabled, s.created_at, s.updated_at,
                      s.transport_type, s.url, s.headers, s.default_tool_policy, s.source_adapter_id, s.source_scope,
                      ps.config_override, ps.detected_adapter_id, ps.detected_config_path
               FROM mcp_services s
               INNER JOIN project_mcp_services ps ON s.id = ps.service_id
               WHERE ps.project_id = ?1
               ORDER BY s.name ASC"#,
        )?;

        let services = stmt
            .query_map([project_id], |row| {
                let service = parse_mcp_service_row(row)?;
                // config_override 在索引 16, detected_adapter_id 在索引 17, detected_config_path 在索引 18
                let config_json: Option<String> = row.get(16)?;
                let config_override = config_json.and_then(|s| serde_json::from_str(&s).ok());
                let detected_adapter_id: Option<String> = row.get(17)?;
                let detected_config_path: Option<String> = row.get(18)?;

                Ok(McpServiceWithOverride {
                    service,
                    config_override,
                    detected_adapter_id,
                    detected_config_path,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(services)
    }

    /// 获取 MCP 服务关联的项目 ID 列表
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    ///
    /// # Returns
    /// 关联的项目 ID 列表
    pub fn get_service_projects(&self, service_id: &str) -> Result<Vec<String>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT project_id FROM project_mcp_services WHERE service_id = ?1",
        )?;

        let project_ids = stmt
            .query_map([service_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(project_ids)
    }

    /// 更新项目级 MCP 服务配置覆盖
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    /// * `service_id` - 服务 ID
    /// * `config_override` - 新的配置覆盖
    pub fn update_project_service_override(
        &self,
        project_id: &str,
        service_id: &str,
        config_override: Option<&serde_json::Value>,
    ) -> Result<(), StorageError> {
        let config_json = config_override.and_then(|c| serde_json::to_string(c).ok());

        let affected = self.connection().execute(
            "UPDATE project_mcp_services SET config_override = ?1 WHERE project_id = ?2 AND service_id = ?3",
            params![&config_json, project_id, service_id],
        )?;

        if affected == 0 {
            return Err(StorageError::NotFound(format!(
                "Project-service link not found: {} - {}",
                project_id, service_id
            )));
        }

        Ok(())
    }

    // ===== Story 11.4: 受影响服务查询 (Task 3) =====

    /// 查找引用指定环境变量的 MCP 服务
    ///
    /// 通过搜索 env JSON 字段中的变量引用来查找受影响的服务
    /// 支持 `$VAR_NAME` 和 `${VAR_NAME}` 两种格式
    ///
    /// # Arguments
    /// * `var_name` - 环境变量名称
    ///
    /// # Returns
    /// 引用该变量的 MCP 服务列表
    pub fn find_services_using_env_var(&self, var_name: &str) -> Result<Vec<McpService>, StorageError> {
        // 构建搜索模式：匹配 $VAR_NAME 或 ${VAR_NAME}
        // 使用粗略的 SQL LIKE 预过滤，然后在应用层精确匹配
        let pattern_simple = format!("%${}%", var_name);
        let pattern_braced = format!("%${{{}}}%", var_name);

        let mut stmt = self.connection().prepare(
            r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at, transport_type, url, headers, default_tool_policy, source_adapter_id, source_scope
               FROM mcp_services
               WHERE env LIKE ?1 OR env LIKE ?2
               ORDER BY name ASC"#,
        )?;

        // 用于提取环境变量引用的正则表达式
        let extract_re = regex::Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}|\$([A-Z_][A-Z0-9_]*)").unwrap();

        let services: Vec<McpService> = stmt
            .query_map(params![&pattern_simple, &pattern_braced], parse_mcp_service_row)?
            .filter_map(|r| r.ok())
            .filter(|service| {
                // 在应用层进行精确匹配：提取所有变量引用，检查是否包含目标变量
                if let Some(env) = &service.env {
                    let env_str = env.to_string();
                    for cap in extract_re.captures_iter(&env_str) {
                        // 获取变量名（可能在第一个或第二个捕获组）
                        if let Some(captured_name) = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str()) {
                            if captured_name == var_name {
                                return true;
                            }
                        }
                    }
                }
                false
            })
            .collect();

        Ok(services)
    }

    /// 获取项目与服务的关联记录
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    /// * `service_id` - 服务 ID
    ///
    /// # Returns
    /// 关联记录，如果不存在则返回 None
    pub fn get_project_service_link(
        &self,
        project_id: &str,
        service_id: &str,
    ) -> Result<Option<ProjectMcpService>, StorageError> {
        let result = self.connection().query_row(
            r#"SELECT project_id, service_id, config_override, created_at, detected_adapter_id, detected_config_path
               FROM project_mcp_services WHERE project_id = ?1 AND service_id = ?2"#,
            params![project_id, service_id],
            |row| {
                let config_json: Option<String> = row.get(2)?;
                let config_override = config_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ProjectMcpService {
                    project_id: row.get(0)?,
                    service_id: row.get(1)?,
                    config_override,
                    detected_adapter_id: row.get(4)?,
                    detected_config_path: row.get(5)?,
                    created_at: row.get(3)?,
                })
            },
        );

        match result {
            Ok(link) => Ok(Some(link)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Database(e)),
        }
    }

    /// 获取项目的所有服务关联记录 (Story 11.19)
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    ///
    /// # Returns
    /// 项目的所有关联记录列表
    pub fn get_project_service_links(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMcpService>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT project_id, service_id, config_override, created_at, detected_adapter_id, detected_config_path
               FROM project_mcp_services WHERE project_id = ?1"#,
        )?;

        let links = stmt
            .query_map(params![project_id], |row| {
                let config_json: Option<String> = row.get(2)?;
                let config_override = config_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ProjectMcpService {
                    project_id: row.get(0)?,
                    service_id: row.get(1)?,
                    config_override,
                    detected_adapter_id: row.get(4)?,
                    detected_config_path: row.get(5)?,
                    created_at: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(links)
    }

    // ===== Story 11.10: MCP 服务工具缓存 =====

    /// 获取服务的缓存工具列表
    ///
    /// Story 11.10: Project-Level Tool Management - Task 2.4
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    ///
    /// # Returns
    /// 缓存的工具列表
    pub fn get_cached_service_tools(
        &self,
        service_id: &str,
    ) -> Result<Vec<McpServiceTool>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, service_id, tool_name, description, input_schema, cached_at
               FROM mcp_service_tools WHERE service_id = ?1
               ORDER BY tool_name ASC"#,
        )?;

        let tools = stmt
            .query_map([service_id], |row| {
                let input_schema_json: Option<String> = row.get(4)?;
                let input_schema = input_schema_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(McpServiceTool {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    input_schema,
                    cached_at: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tools)
    }

    /// 缓存服务的工具列表
    ///
    /// Story 11.10: Project-Level Tool Management - Task 2.4
    ///
    /// 替换该服务的所有缓存工具（先删后插）
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    /// * `tools` - 工具列表（名称、描述、输入 Schema）
    pub fn cache_service_tools(
        &self,
        service_id: &str,
        tools: &[(String, Option<String>, Option<serde_json::Value>)],
    ) -> Result<(), StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        // 删除旧缓存
        self.connection().execute(
            "DELETE FROM mcp_service_tools WHERE service_id = ?1",
            [service_id],
        )?;

        // 插入新工具
        let mut stmt = self.connection().prepare(
            r#"INSERT INTO mcp_service_tools (id, service_id, tool_name, description, input_schema, cached_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
        )?;

        for (name, description, input_schema) in tools {
            let id = uuid::Uuid::new_v4().to_string();
            let input_schema_json = input_schema
                .as_ref()
                .and_then(|s| serde_json::to_string(s).ok());

            stmt.execute(params![
                &id,
                service_id,
                name,
                description,
                &input_schema_json,
                &now,
            ])?;
        }

        Ok(())
    }

    /// 清除服务的工具缓存
    ///
    /// Story 11.10: Project-Level Tool Management
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    pub fn clear_service_tools_cache(&self, service_id: &str) -> Result<(), StorageError> {
        self.connection().execute(
            "DELETE FROM mcp_service_tools WHERE service_id = ?1",
            [service_id],
        )?;
        Ok(())
    }

    /// 获取所有服务的工具缓存时间
    ///
    /// Story 11.10: Project-Level Tool Management
    ///
    /// 用于检查哪些缓存已过期
    ///
    /// # Returns
    /// 服务 ID -> 最新缓存时间 的映射
    pub fn get_service_tools_cache_times(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT service_id, MAX(cached_at) as latest_cached_at
               FROM mcp_service_tools
               GROUP BY service_id"#,
        )?;

        let times = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(times)
    }

    // ===== Story 11.15: MCP 接管备份存储 =====

    /// 创建接管备份记录
    ///
    /// Story 11.15: MCP 接管流程重构 - Task 2.3
    ///
    /// # Arguments
    /// * `backup` - 备份记录
    pub fn create_takeover_backup(&self, backup: &TakeoverBackup) -> Result<(), StorageError> {
        let project_path_str = backup.project_path.as_ref().map(|p| p.to_string_lossy().to_string());
        self.connection().execute(
            r#"INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                &backup.id,
                backup.tool_type.as_str(),
                backup.original_path.to_string_lossy().to_string(),
                backup.backup_path.to_string_lossy().to_string(),
                &backup.taken_over_at,
                &backup.restored_at,
                backup.status.as_str(),
                backup.scope.as_str(),
                &project_path_str,
            ],
        )?;
        Ok(())
    }

    /// 获取接管备份记录列表
    ///
    /// Story 11.15: MCP 接管流程重构 - Task 2.4
    ///
    /// # Arguments
    /// * `status` - 可选的状态筛选（active/restored）
    ///
    /// # Returns
    /// 备份记录列表，按接管时间倒序排列
    pub fn get_takeover_backups(
        &self,
        status: Option<TakeoverStatus>,
    ) -> Result<Vec<TakeoverBackup>, StorageError> {
        let status_str = status.as_ref().map(|s| s.as_str().to_string());

        let sql = match &status_str {
            Some(_) => {
                r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
                   FROM mcp_takeover_backups
                   WHERE status = ?1
                   ORDER BY taken_over_at DESC"#
            }
            None => {
                r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
                   FROM mcp_takeover_backups
                   ORDER BY taken_over_at DESC"#
            }
        };

        let mut stmt = self.connection().prepare(sql)?;

        let backups = if let Some(ref s) = status_str {
            stmt.query_map([s.as_str()], parse_takeover_backup_row)?
                .filter_map(|r| r.ok())
                .collect()
        } else {
            stmt.query_map([], parse_takeover_backup_row)?
                .filter_map(|r| r.ok())
                .collect()
        };

        Ok(backups)
    }

    /// 按 ID 获取接管备份记录
    ///
    /// Story 11.15: MCP 接管流程重构
    ///
    /// # Arguments
    /// * `id` - 备份 ID
    pub fn get_takeover_backup_by_id(&self, id: &str) -> Result<Option<TakeoverBackup>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
               FROM mcp_takeover_backups
               WHERE id = ?1"#,
        )?;

        let backup = stmt
            .query_row([id], parse_takeover_backup_row)
            .optional()?;

        Ok(backup)
    }

    /// 按工具类型获取活跃的接管备份
    ///
    /// Story 11.15: MCP 接管流程重构
    ///
    /// # Arguments
    /// * `tool_type` - 工具类型
    ///
    /// # Returns
    /// 该工具类型的活跃备份（如果存在）
    pub fn get_active_takeover_by_tool(
        &self,
        tool_type: &ToolType,
    ) -> Result<Option<TakeoverBackup>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
               FROM mcp_takeover_backups
               WHERE tool_type = ?1 AND status = 'active'
               ORDER BY taken_over_at DESC
               LIMIT 1"#,
        )?;

        let backup = stmt
            .query_row([tool_type.as_str()], parse_takeover_backup_row)
            .optional()?;

        Ok(backup)
    }

    /// 按工具类型和作用域获取活跃的接管备份 (Story 11.16)
    ///
    /// # Arguments
    /// * `tool_type` - 工具类型
    /// * `scope` - 接管作用域
    /// * `project_path` - 项目路径（仅 project 作用域需要）
    pub fn get_active_takeover_by_tool_and_scope(
        &self,
        tool_type: &ToolType,
        scope: &TakeoverScope,
        project_path: Option<&str>,
    ) -> Result<Option<TakeoverBackup>, StorageError> {
        let backup = match scope {
            TakeoverScope::User => {
                let mut stmt = self.connection().prepare(
                    r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
                       FROM mcp_takeover_backups
                       WHERE tool_type = ?1 AND status = 'active' AND scope = 'user'
                       ORDER BY taken_over_at DESC
                       LIMIT 1"#,
                )?;
                stmt.query_row([tool_type.as_str()], parse_takeover_backup_row)
                    .optional()?
            }
            TakeoverScope::Project => {
                let mut stmt = self.connection().prepare(
                    r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
                       FROM mcp_takeover_backups
                       WHERE tool_type = ?1 AND status = 'active' AND scope = 'project' AND project_path = ?2
                       ORDER BY taken_over_at DESC
                       LIMIT 1"#,
                )?;
                stmt.query_row(
                    params![tool_type.as_str(), project_path.unwrap_or("")],
                    parse_takeover_backup_row,
                )
                .optional()?
            }
        };

        Ok(backup)
    }

    /// 获取项目的所有活跃接管备份 (Story 11.16)
    ///
    /// # Arguments
    /// * `project_path` - 项目路径
    pub fn get_active_takeovers_by_project(
        &self,
        project_path: &str,
    ) -> Result<Vec<TakeoverBackup>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
               FROM mcp_takeover_backups
               WHERE scope = 'project' AND project_path = ?1 AND status = 'active'
               ORDER BY taken_over_at DESC"#,
        )?;

        let backups = stmt
            .query_map([project_path], parse_takeover_backup_row)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(backups)
    }

    /// 按原始配置文件路径获取活跃接管备份
    ///
    /// 用于判断某个配置文件是否已被接管，避免重复创建备份
    ///
    /// # Arguments
    /// * `original_path` - 原始配置文件路径
    pub fn get_active_takeover_by_original_path(
        &self,
        original_path: &str,
    ) -> Result<Option<TakeoverBackup>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
               FROM mcp_takeover_backups
               WHERE original_path = ?1 AND status = 'active'
               ORDER BY taken_over_at DESC
               LIMIT 1"#,
        )?;

        let backup = stmt
            .query_row([original_path], parse_takeover_backup_row)
            .optional()?;

        Ok(backup)
    }

    /// 更新备份记录状态为已恢复
    ///
    /// Story 11.15: MCP 接管流程重构 - Task 2.5
    ///
    /// # Arguments
    /// * `id` - 备份 ID
    pub fn update_backup_status_restored(&self, id: &str) -> Result<(), StorageError> {
        let now = chrono::Utc::now().to_rfc3339();
        let affected = self.connection().execute(
            r#"UPDATE mcp_takeover_backups
               SET status = 'restored', restored_at = ?1
               WHERE id = ?2"#,
            params![&now, id],
        )?;

        if affected == 0 {
            return Err(StorageError::NotFound(format!(
                "Takeover backup not found: {}",
                id
            )));
        }

        Ok(())
    }

    /// 删除备份记录
    ///
    /// Story 11.15: MCP 接管流程重构
    ///
    /// # Arguments
    /// * `id` - 备份 ID
    pub fn delete_takeover_backup(&self, id: &str) -> Result<(), StorageError> {
        self.connection().execute(
            "DELETE FROM mcp_takeover_backups WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request() -> CreateMcpServiceRequest {
        CreateMcpServiceRequest {
            name: "git-mcp".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            env: Some(serde_json::json!({"DEBUG": "true"})),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        }
    }

    #[test]
    fn test_create_mcp_service() {
        let db = Database::new_in_memory().unwrap();
        let request = create_test_request();

        let service = db.create_mcp_service(&request).unwrap();

        assert!(!service.id.is_empty());
        assert_eq!(service.name, "git-mcp");
        assert_eq!(service.command, "npx");
        assert_eq!(service.args, Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]));
        assert_eq!(service.source, McpServiceSource::Manual);
        assert!(service.enabled);
    }

    #[test]
    fn test_get_mcp_service() {
        let db = Database::new_in_memory().unwrap();
        let request = create_test_request();

        let created = db.create_mcp_service(&request).unwrap();
        let fetched = db.get_mcp_service(&created.id).unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, created.name);
    }

    #[test]
    fn test_get_mcp_service_not_found() {
        let db = Database::new_in_memory().unwrap();

        let result = db.get_mcp_service("non-existent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_mcp_service_by_name() {
        let db = Database::new_in_memory().unwrap();
        let request = create_test_request();

        db.create_mcp_service(&request).unwrap();
        let fetched = db.get_mcp_service_by_name("git-mcp").unwrap();

        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "git-mcp");
    }

    #[test]
    fn test_get_mcp_service_by_name_not_found() {
        let db = Database::new_in_memory().unwrap();

        let fetched = db.get_mcp_service_by_name("non-existent").unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_list_mcp_services() {
        let db = Database::new_in_memory().unwrap();

        // Create multiple services
        let request1 = CreateMcpServiceRequest {
            name: "alpha-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let request2 = CreateMcpServiceRequest {
            name: "beta-service".to_string(),
            transport_type: Default::default(),
            command: "uvx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some("/home/user/.mcp.json".to_string()),
        };

        db.create_mcp_service(&request1).unwrap();
        db.create_mcp_service(&request2).unwrap();

        let services = db.list_mcp_services().unwrap();

        assert_eq!(services.len(), 2);
        // Should be sorted by name
        assert_eq!(services[0].name, "alpha-service");
        assert_eq!(services[1].name, "beta-service");
    }

    #[test]
    fn test_list_mcp_services_by_source() {
        let db = Database::new_in_memory().unwrap();

        let request1 = CreateMcpServiceRequest {
            name: "manual-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let request2 = CreateMcpServiceRequest {
            name: "imported-service".to_string(),
            transport_type: Default::default(),
            command: "uvx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some("/home/user/.mcp.json".to_string()),
        };

        db.create_mcp_service(&request1).unwrap();
        db.create_mcp_service(&request2).unwrap();

        let manual_services = db.list_mcp_services_by_source(&McpServiceSource::Manual).unwrap();
        let imported_services = db.list_mcp_services_by_source(&McpServiceSource::Imported).unwrap();

        assert_eq!(manual_services.len(), 1);
        assert_eq!(manual_services[0].name, "manual-service");

        assert_eq!(imported_services.len(), 1);
        assert_eq!(imported_services[0].name, "imported-service");
    }

    #[test]
    fn test_update_mcp_service() {
        let db = Database::new_in_memory().unwrap();
        let request = create_test_request();

        let created = db.create_mcp_service(&request).unwrap();

        let update = UpdateMcpServiceRequest {
            name: Some("updated-name".to_string()),
            command: Some("uvx".to_string()),
            ..Default::default()
        };

        let updated = db.update_mcp_service(&created.id, &update).unwrap();

        assert_eq!(updated.name, "updated-name");
        assert_eq!(updated.command, "uvx");
        // Args should be preserved
        assert_eq!(updated.args, created.args);
    }

    #[test]
    fn test_update_mcp_service_partial() {
        let db = Database::new_in_memory().unwrap();
        let request = create_test_request();

        let created = db.create_mcp_service(&request).unwrap();

        // Only update name
        let update = UpdateMcpServiceRequest {
            name: Some("new-name".to_string()),
            ..Default::default()
        };

        let updated = db.update_mcp_service(&created.id, &update).unwrap();

        assert_eq!(updated.name, "new-name");
        assert_eq!(updated.command, created.command);
        assert_eq!(updated.args, created.args);
        assert_eq!(updated.env, created.env);
    }

    #[test]
    fn test_delete_mcp_service() {
        let db = Database::new_in_memory().unwrap();
        let request = create_test_request();

        let created = db.create_mcp_service(&request).unwrap();

        db.delete_mcp_service(&created.id).unwrap();

        let result = db.get_mcp_service(&created.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_mcp_service_not_found() {
        let db = Database::new_in_memory().unwrap();

        let result = db.delete_mcp_service("non-existent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_toggle_mcp_service() {
        let db = Database::new_in_memory().unwrap();
        let request = create_test_request();

        let created = db.create_mcp_service(&request).unwrap();
        assert!(created.enabled);

        // Disable
        let disabled = db.toggle_mcp_service(&created.id, false).unwrap();
        assert!(!disabled.enabled);

        // Re-enable
        let enabled = db.toggle_mcp_service(&created.id, true).unwrap();
        assert!(enabled.enabled);
    }

    #[test]
    fn test_toggle_mcp_service_not_found() {
        let db = Database::new_in_memory().unwrap();

        let result = db.toggle_mcp_service("non-existent-id", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_service_with_env_variables() {
        let db = Database::new_in_memory().unwrap();

        let request = CreateMcpServiceRequest {
            name: "openai-mcp".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/openai-mcp".to_string()]),
            env: Some(serde_json::json!({
                "OPENAI_API_KEY": "$OPENAI_API_KEY",
                "DEBUG": "true"
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some("/home/user/.claude/mcp.json".to_string()),
        };

        let service = db.create_mcp_service(&request).unwrap();

        assert_eq!(service.env, Some(serde_json::json!({
            "OPENAI_API_KEY": "$OPENAI_API_KEY",
            "DEBUG": "true"
        })));
    }

    #[test]
    fn test_mcp_service_timestamps() {
        let db = Database::new_in_memory().unwrap();
        let request = create_test_request();

        let created = db.create_mcp_service(&request).unwrap();

        // created_at and updated_at should be set
        assert!(!created.created_at.is_empty());
        assert!(!created.updated_at.is_empty());
        assert_eq!(created.created_at, created.updated_at);

        // After update, updated_at should change
        std::thread::sleep(std::time::Duration::from_millis(10));
        let update = UpdateMcpServiceRequest {
            name: Some("new-name".to_string()),
            ..Default::default()
        };
        let updated = db.update_mcp_service(&created.id, &update).unwrap();

        assert_eq!(updated.created_at, created.created_at);
        assert_ne!(updated.updated_at, created.updated_at);
    }

    // ===== Task 4: 项目关联存储测试 =====

    fn create_test_project(db: &Database, id: &str, name: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        // 使用唯一的 cwd 路径避免 UNIQUE 约束冲突
        let cwd = format!("/path/to/{}", id);
        db.connection()
            .execute(
                "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?4)",
                [id, name, &cwd, &now],
            )
            .unwrap();
    }

    #[test]
    fn test_link_service_to_project() {
        let db = Database::new_in_memory().unwrap();

        // Create project and service
        create_test_project(&db, "proj1", "Project 1");
        let service = db.create_mcp_service(&create_test_request()).unwrap();

        // Link them
        let link = db.link_service_to_project("proj1", &service.id, None).unwrap();

        assert_eq!(link.project_id, "proj1");
        assert_eq!(link.service_id, service.id);
        assert!(link.config_override.is_none());
    }

    #[test]
    fn test_link_service_to_project_with_override() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");
        let service = db.create_mcp_service(&create_test_request()).unwrap();

        let override_config = serde_json::json!({"args": ["--custom-arg"]});
        let link = db
            .link_service_to_project("proj1", &service.id, Some(&override_config))
            .unwrap();

        assert_eq!(link.config_override, Some(override_config));
    }

    #[test]
    fn test_unlink_service_from_project() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");
        let service = db.create_mcp_service(&create_test_request()).unwrap();

        db.link_service_to_project("proj1", &service.id, None).unwrap();
        db.unlink_service_from_project("proj1", &service.id).unwrap();

        // Verify link is removed
        let link = db.get_project_service_link("proj1", &service.id).unwrap();
        assert!(link.is_none());
    }

    #[test]
    fn test_unlink_service_from_project_not_found() {
        let db = Database::new_in_memory().unwrap();

        let result = db.unlink_service_from_project("proj1", "svc1");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_project_services() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");

        let service1 = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "alpha-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let service2 = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "beta-service".to_string(),
                transport_type: Default::default(),
                command: "uvx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // Link both services
        db.link_service_to_project("proj1", &service1.id, None).unwrap();
        let override_config = serde_json::json!({"args": ["--verbose"]});
        db.link_service_to_project("proj1", &service2.id, Some(&override_config))
            .unwrap();

        // Get project services
        let services = db.get_project_services("proj1").unwrap();

        assert_eq!(services.len(), 2);
        // Should be sorted by name
        assert_eq!(services[0].service.name, "alpha-service");
        assert!(services[0].config_override.is_none());
        assert_eq!(services[1].service.name, "beta-service");
        assert_eq!(services[1].config_override, Some(override_config));
    }

    #[test]
    fn test_get_project_services_empty() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");

        let services = db.get_project_services("proj1").unwrap();
        assert!(services.is_empty());
    }

    #[test]
    fn test_get_service_projects() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");
        create_test_project(&db, "proj2", "Project 2");

        let service = db.create_mcp_service(&create_test_request()).unwrap();

        // Link service to both projects
        db.link_service_to_project("proj1", &service.id, None).unwrap();
        db.link_service_to_project("proj2", &service.id, None).unwrap();

        // Get service projects
        let project_ids = db.get_service_projects(&service.id).unwrap();

        assert_eq!(project_ids.len(), 2);
        assert!(project_ids.contains(&"proj1".to_string()));
        assert!(project_ids.contains(&"proj2".to_string()));
    }

    #[test]
    fn test_get_service_projects_empty() {
        let db = Database::new_in_memory().unwrap();

        let service = db.create_mcp_service(&create_test_request()).unwrap();

        let project_ids = db.get_service_projects(&service.id).unwrap();
        assert!(project_ids.is_empty());
    }

    #[test]
    fn test_update_project_service_override() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");
        let service = db.create_mcp_service(&create_test_request()).unwrap();

        // Link without override
        db.link_service_to_project("proj1", &service.id, None).unwrap();

        // Update with override
        let override_config = serde_json::json!({"args": ["--new-arg"]});
        db.update_project_service_override("proj1", &service.id, Some(&override_config))
            .unwrap();

        // Verify
        let link = db.get_project_service_link("proj1", &service.id).unwrap().unwrap();
        assert_eq!(link.config_override, Some(override_config));
    }

    #[test]
    fn test_update_project_service_override_to_none() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");
        let service = db.create_mcp_service(&create_test_request()).unwrap();

        let override_config = serde_json::json!({"args": ["--arg"]});
        db.link_service_to_project("proj1", &service.id, Some(&override_config))
            .unwrap();

        // Clear override
        db.update_project_service_override("proj1", &service.id, None)
            .unwrap();

        // Verify
        let link = db.get_project_service_link("proj1", &service.id).unwrap().unwrap();
        assert!(link.config_override.is_none());
    }

    #[test]
    fn test_update_project_service_override_not_found() {
        let db = Database::new_in_memory().unwrap();

        let result = db.update_project_service_override("proj1", "svc1", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_project_service_link() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");
        let service = db.create_mcp_service(&create_test_request()).unwrap();

        db.link_service_to_project("proj1", &service.id, None).unwrap();

        let link = db.get_project_service_link("proj1", &service.id).unwrap();
        assert!(link.is_some());
        assert_eq!(link.unwrap().project_id, "proj1");
    }

    #[test]
    fn test_get_project_service_link_not_found() {
        let db = Database::new_in_memory().unwrap();

        let link = db.get_project_service_link("proj1", "svc1").unwrap();
        assert!(link.is_none());
    }

    #[test]
    fn test_cascade_delete_removes_links() {
        let db = Database::new_in_memory().unwrap();

        create_test_project(&db, "proj1", "Project 1");
        let service = db.create_mcp_service(&create_test_request()).unwrap();

        db.link_service_to_project("proj1", &service.id, None).unwrap();

        // Delete service
        db.delete_mcp_service(&service.id).unwrap();

        // Link should be removed due to CASCADE
        let link = db.get_project_service_link("proj1", &service.id).unwrap();
        assert!(link.is_none());
    }

    // ===== Story 11.4: 受影响服务查询测试 =====

    #[test]
    fn test_find_services_using_env_var_simple_format() {
        let db = Database::new_in_memory().unwrap();

        // 创建使用 $OPENAI_API_KEY 的服务
        let request = CreateMcpServiceRequest {
            name: "openai-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "OPENAI_API_KEY": "$OPENAI_API_KEY",
                "DEBUG": "true"
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();

        // 创建不使用该变量的服务
        let request2 = CreateMcpServiceRequest {
            name: "other-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "DEBUG": "true"
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request2).unwrap();

        let affected = db.find_services_using_env_var("OPENAI_API_KEY").unwrap();
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].name, "openai-service");
    }

    #[test]
    fn test_find_services_using_env_var_braced_format() {
        let db = Database::new_in_memory().unwrap();

        // 创建使用 ${ANTHROPIC_API_KEY} 的服务
        let request = CreateMcpServiceRequest {
            name: "anthropic-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "API_KEY": "${ANTHROPIC_API_KEY}",
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();

        let affected = db.find_services_using_env_var("ANTHROPIC_API_KEY").unwrap();
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].name, "anthropic-service");
    }

    #[test]
    fn test_find_services_using_env_var_multiple_services() {
        let db = Database::new_in_memory().unwrap();

        // 创建多个使用同一变量的服务
        for name in ["service-a", "service-b", "service-c"] {
            let request = CreateMcpServiceRequest {
                name: name.to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: Some(serde_json::json!({
                    "API_KEY": "$SHARED_KEY",
                })),
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            };
            db.create_mcp_service(&request).unwrap();
        }

        let affected = db.find_services_using_env_var("SHARED_KEY").unwrap();
        assert_eq!(affected.len(), 3);
        // 应该按名称排序
        assert_eq!(affected[0].name, "service-a");
        assert_eq!(affected[1].name, "service-b");
        assert_eq!(affected[2].name, "service-c");
    }

    #[test]
    fn test_find_services_using_env_var_no_match() {
        let db = Database::new_in_memory().unwrap();

        let request = CreateMcpServiceRequest {
            name: "some-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "OTHER_VAR": "$OTHER_VAR",
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();

        let affected = db.find_services_using_env_var("NONEXISTENT_VAR").unwrap();
        assert!(affected.is_empty());
    }

    #[test]
    fn test_find_services_using_env_var_no_env() {
        let db = Database::new_in_memory().unwrap();

        // 创建没有 env 字段的服务
        let request = CreateMcpServiceRequest {
            name: "no-env-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();

        let affected = db.find_services_using_env_var("ANY_VAR").unwrap();
        assert!(affected.is_empty());
    }

    #[test]
    fn test_find_services_using_env_var_no_substring_false_positive() {
        let db = Database::new_in_memory().unwrap();

        // 创建使用 $OPENAI_API_KEY 的服务（包含 API_KEY 子串）
        let request1 = CreateMcpServiceRequest {
            name: "openai-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "KEY": "$OPENAI_API_KEY",
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request1).unwrap();

        // 创建使用 $API_KEY 的服务
        let request2 = CreateMcpServiceRequest {
            name: "api-key-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "KEY": "$API_KEY",
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request2).unwrap();

        // 搜索 API_KEY 应该只返回 api-key-service，而不是 openai-service
        let affected = db.find_services_using_env_var("API_KEY").unwrap();
        assert_eq!(affected.len(), 1, "Should only match exact variable name, not substring");
        assert_eq!(affected[0].name, "api-key-service");

        // 搜索 OPENAI_API_KEY 应该只返回 openai-service
        let affected = db.find_services_using_env_var("OPENAI_API_KEY").unwrap();
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].name, "openai-service");
    }

    // ===== Story 11.15: 接管备份存储测试 =====

    fn create_test_backup(tool_type: ToolType) -> TakeoverBackup {
        TakeoverBackup::new(
            tool_type,
            PathBuf::from("/home/user/.claude.json"),
            PathBuf::from("/home/user/.claude.json.mantra-backup.20260201"),
        )
    }

    #[test]
    fn test_create_takeover_backup() {
        let db = Database::new_in_memory().unwrap();
        let backup = create_test_backup(ToolType::ClaudeCode);

        db.create_takeover_backup(&backup).unwrap();

        // Verify it was created
        let fetched = db.get_takeover_backup_by_id(&backup.id).unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, backup.id);
        assert_eq!(fetched.tool_type, ToolType::ClaudeCode);
        assert_eq!(fetched.status, TakeoverStatus::Active);
    }

    #[test]
    fn test_get_takeover_backups_all() {
        let db = Database::new_in_memory().unwrap();

        // Create multiple backups
        let backup1 = create_test_backup(ToolType::ClaudeCode);
        let backup2 = TakeoverBackup::new(
            ToolType::Cursor,
            PathBuf::from("/home/user/.cursor/mcp.json"),
            PathBuf::from("/home/user/.cursor/mcp.json.backup"),
        );

        db.create_takeover_backup(&backup1).unwrap();
        db.create_takeover_backup(&backup2).unwrap();

        let backups = db.get_takeover_backups(None).unwrap();
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn test_get_takeover_backups_by_status() {
        let db = Database::new_in_memory().unwrap();

        // Create active backup
        let backup1 = create_test_backup(ToolType::ClaudeCode);
        db.create_takeover_backup(&backup1).unwrap();

        // Create and restore another backup
        let backup2 = TakeoverBackup::new(
            ToolType::Cursor,
            PathBuf::from("/home/user/.cursor/mcp.json"),
            PathBuf::from("/home/user/.cursor/mcp.json.backup"),
        );
        db.create_takeover_backup(&backup2).unwrap();
        db.update_backup_status_restored(&backup2.id).unwrap();

        // Filter by active
        let active = db.get_takeover_backups(Some(TakeoverStatus::Active)).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].tool_type, ToolType::ClaudeCode);

        // Filter by restored
        let restored = db.get_takeover_backups(Some(TakeoverStatus::Restored)).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].tool_type, ToolType::Cursor);
    }

    #[test]
    fn test_get_active_takeover_by_tool() {
        let db = Database::new_in_memory().unwrap();

        // No backup exists
        let result = db.get_active_takeover_by_tool(&ToolType::ClaudeCode).unwrap();
        assert!(result.is_none());

        // Create backup
        let backup = create_test_backup(ToolType::ClaudeCode);
        db.create_takeover_backup(&backup).unwrap();

        let result = db.get_active_takeover_by_tool(&ToolType::ClaudeCode).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, backup.id);

        // Different tool type should return None
        let result = db.get_active_takeover_by_tool(&ToolType::Cursor).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_active_takeover_excludes_restored() {
        let db = Database::new_in_memory().unwrap();

        let backup = create_test_backup(ToolType::ClaudeCode);
        db.create_takeover_backup(&backup).unwrap();
        db.update_backup_status_restored(&backup.id).unwrap();

        // Should not return restored backups
        let result = db.get_active_takeover_by_tool(&ToolType::ClaudeCode).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_backup_status_restored() {
        let db = Database::new_in_memory().unwrap();

        let backup = create_test_backup(ToolType::ClaudeCode);
        db.create_takeover_backup(&backup).unwrap();

        // Update status
        db.update_backup_status_restored(&backup.id).unwrap();

        // Verify
        let fetched = db.get_takeover_backup_by_id(&backup.id).unwrap().unwrap();
        assert_eq!(fetched.status, TakeoverStatus::Restored);
        assert!(fetched.restored_at.is_some());
    }

    #[test]
    fn test_update_backup_status_not_found() {
        let db = Database::new_in_memory().unwrap();

        let result = db.update_backup_status_restored("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_takeover_backup() {
        let db = Database::new_in_memory().unwrap();

        let backup = create_test_backup(ToolType::ClaudeCode);
        db.create_takeover_backup(&backup).unwrap();

        db.delete_takeover_backup(&backup.id).unwrap();

        let fetched = db.get_takeover_backup_by_id(&backup.id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_takeover_backup_preserves_paths() {
        let db = Database::new_in_memory().unwrap();

        let original = PathBuf::from("/home/user/.claude.json");
        let backup_path = PathBuf::from("/home/user/.claude.json.mantra-backup.20260201");

        let backup = TakeoverBackup::new(
            ToolType::ClaudeCode,
            original.clone(),
            backup_path.clone(),
        );
        db.create_takeover_backup(&backup).unwrap();

        let fetched = db.get_takeover_backup_by_id(&backup.id).unwrap().unwrap();
        assert_eq!(fetched.original_path, original);
        assert_eq!(fetched.backup_path, backup_path);
    }

    // ===== Story 11.16: 接管作用域存储测试 =====

    fn create_test_project_backup(tool_type: ToolType, project_path: &str) -> TakeoverBackup {
        TakeoverBackup::new_with_scope(
            tool_type,
            PathBuf::from(format!("{}/.mcp.json", project_path)),
            PathBuf::from(format!("{}/.mcp.json.backup", project_path)),
            TakeoverScope::Project,
            Some(PathBuf::from(project_path)),
        )
    }

    #[test]
    fn test_create_takeover_backup_with_scope() {
        let db = Database::new_in_memory().unwrap();

        // 用户级备份
        let user_backup = create_test_backup(ToolType::ClaudeCode);
        db.create_takeover_backup(&user_backup).unwrap();

        let fetched = db.get_takeover_backup_by_id(&user_backup.id).unwrap().unwrap();
        assert_eq!(fetched.scope, TakeoverScope::User);
        assert!(fetched.project_path.is_none());

        // 项目级备份
        let project_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project");
        db.create_takeover_backup(&project_backup).unwrap();

        let fetched = db.get_takeover_backup_by_id(&project_backup.id).unwrap().unwrap();
        assert_eq!(fetched.scope, TakeoverScope::Project);
        assert_eq!(fetched.project_path, Some(PathBuf::from("/home/user/project")));
    }

    #[test]
    fn test_get_active_takeover_by_tool_and_scope_user() {
        let db = Database::new_in_memory().unwrap();

        // 创建用户级和项目级备份
        let user_backup = create_test_backup(ToolType::ClaudeCode);
        let project_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project");

        db.create_takeover_backup(&user_backup).unwrap();
        db.create_takeover_backup(&project_backup).unwrap();

        // 按用户级作用域查询
        let result = db
            .get_active_takeover_by_tool_and_scope(&ToolType::ClaudeCode, &TakeoverScope::User, None)
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().scope, TakeoverScope::User);
    }

    #[test]
    fn test_get_active_takeover_by_tool_and_scope_project() {
        let db = Database::new_in_memory().unwrap();

        // 创建多个项目的备份
        let project1_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project1");
        let project2_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project2");

        db.create_takeover_backup(&project1_backup).unwrap();
        db.create_takeover_backup(&project2_backup).unwrap();

        // 按项目级作用域查询
        let result = db
            .get_active_takeover_by_tool_and_scope(
                &ToolType::ClaudeCode,
                &TakeoverScope::Project,
                Some("/home/user/project1"),
            )
            .unwrap();

        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.scope, TakeoverScope::Project);
        assert_eq!(found.project_path, Some(PathBuf::from("/home/user/project1")));
    }

    #[test]
    fn test_get_active_takeover_by_tool_and_scope_not_found() {
        let db = Database::new_in_memory().unwrap();

        // 创建用户级备份
        let user_backup = create_test_backup(ToolType::ClaudeCode);
        db.create_takeover_backup(&user_backup).unwrap();

        // 查询不存在的项目级备份
        let result = db
            .get_active_takeover_by_tool_and_scope(
                &ToolType::ClaudeCode,
                &TakeoverScope::Project,
                Some("/nonexistent"),
            )
            .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_get_active_takeover_by_original_path() {
        let db = Database::new_in_memory().unwrap();

        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            PathBuf::from("/project/.mcp.json"),
            PathBuf::from("/project/.mcp.json.mantra-backup.20260203"),
            TakeoverScope::Project,
            Some(PathBuf::from("/project")),
        );
        db.create_takeover_backup(&backup).unwrap();

        // 查询存在的路径
        let result = db
            .get_active_takeover_by_original_path("/project/.mcp.json")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, backup.id);

        // 查询不存在的路径
        let result = db
            .get_active_takeover_by_original_path("/other/.mcp.json")
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_active_takeover_by_original_path_excludes_restored() {
        let db = Database::new_in_memory().unwrap();

        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            PathBuf::from("/project/.mcp.json"),
            PathBuf::from("/project/.mcp.json.mantra-backup.20260203"),
            TakeoverScope::Project,
            Some(PathBuf::from("/project")),
        );
        db.create_takeover_backup(&backup).unwrap();

        // 标记为已恢复
        db.update_backup_status_restored(&backup.id).unwrap();

        // 查询应返回 None（已恢复的不算活跃）
        let result = db
            .get_active_takeover_by_original_path("/project/.mcp.json")
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_active_takeovers_by_project() {
        let db = Database::new_in_memory().unwrap();

        // 创建同一个项目的多个工具备份
        let claude_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project");
        let cursor_backup = TakeoverBackup::new_with_scope(
            ToolType::Cursor,
            PathBuf::from("/home/user/project/.mcp.json"),
            PathBuf::from("/home/user/project/.mcp.json.cursor-backup"),
            TakeoverScope::Project,
            Some(PathBuf::from("/home/user/project")),
        );

        // 不同项目的备份
        let other_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/other");

        db.create_takeover_backup(&claude_backup).unwrap();
        db.create_takeover_backup(&cursor_backup).unwrap();
        db.create_takeover_backup(&other_backup).unwrap();

        // 查询特定项目的备份
        let backups = db.get_active_takeovers_by_project("/home/user/project").unwrap();

        assert_eq!(backups.len(), 2);
        assert!(backups.iter().all(|b| b.project_path == Some(PathBuf::from("/home/user/project"))));
    }

    #[test]
    fn test_get_active_takeovers_by_project_excludes_restored() {
        let db = Database::new_in_memory().unwrap();

        // 创建两个备份，其中一个已恢复
        let backup1 = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project");
        let backup2 = create_test_project_backup(ToolType::Cursor, "/home/user/project");

        db.create_takeover_backup(&backup1).unwrap();
        db.create_takeover_backup(&backup2).unwrap();
        db.update_backup_status_restored(&backup2.id).unwrap();

        // 只应该返回活跃的备份
        let backups = db.get_active_takeovers_by_project("/home/user/project").unwrap();

        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].id, backup1.id);
    }

    #[test]
    fn test_takeover_backup_scope_in_get_backups() {
        let db = Database::new_in_memory().unwrap();

        let user_backup = create_test_backup(ToolType::ClaudeCode);
        let project_backup = create_test_project_backup(ToolType::Cursor, "/home/user/project");

        db.create_takeover_backup(&user_backup).unwrap();
        db.create_takeover_backup(&project_backup).unwrap();

        // get_takeover_backups 应该返回正确的 scope
        let backups = db.get_takeover_backups(None).unwrap();

        assert_eq!(backups.len(), 2);
        let user_found = backups.iter().find(|b| b.id == user_backup.id).unwrap();
        let project_found = backups.iter().find(|b| b.id == project_backup.id).unwrap();

        assert_eq!(user_found.scope, TakeoverScope::User);
        assert_eq!(project_found.scope, TakeoverScope::Project);
        assert_eq!(project_found.project_path, Some(PathBuf::from("/home/user/project")));
    }

    // ===== Story 11.9 Phase 2: Default Tool Policy Tests =====

    #[test]
    fn test_get_service_default_policy_none() {
        let db = Database::new_in_memory().unwrap();

        // 创建服务（无默认策略）
        let request = CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let service = db.create_mcp_service(&request).unwrap();

        // 获取默认策略应返回 AllowAll
        let policy = db.get_service_default_policy(&service.id).unwrap();
        assert!(policy.is_allow_all());
    }

    #[test]
    fn test_update_service_default_policy() {
        let db = Database::new_in_memory().unwrap();

        // 创建服务
        let request = CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let service = db.create_mcp_service(&request).unwrap();

        // 更新默认策略为 Custom（仅允许特定工具）
        let policy = ToolPolicy::custom(vec!["read_file".to_string()]);
        let updated = db.update_service_default_policy(&service.id, Some(&policy)).unwrap();
        assert!(updated.default_tool_policy.is_some());

        let retrieved_policy = db.get_service_default_policy(&service.id).unwrap();
        assert!(retrieved_policy.is_custom());
    }

    #[test]
    fn test_update_service_default_policy_custom() {
        let db = Database::new_in_memory().unwrap();

        // 创建服务
        let request = CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let service = db.create_mcp_service(&request).unwrap();

        // 更新为 Custom 策略
        let policy = ToolPolicy::custom(vec!["read_file".to_string(), "list_commits".to_string()]);
        db.update_service_default_policy(&service.id, Some(&policy)).unwrap();

        let retrieved = db.get_service_default_policy(&service.id).unwrap();
        assert!(retrieved.is_custom());
        let allowed = retrieved.allowed_tools.unwrap();
        assert!(allowed.contains(&"read_file".to_string()));
        assert!(allowed.contains(&"list_commits".to_string()));
    }

    #[test]
    fn test_update_service_default_policy_clear() {
        let db = Database::new_in_memory().unwrap();

        // 创建服务
        let request = CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let service = db.create_mcp_service(&request).unwrap();

        // 先设置 Custom 策略
        let policy = ToolPolicy::custom(vec!["some_tool".to_string()]);
        db.update_service_default_policy(&service.id, Some(&policy)).unwrap();

        // 然后清除策略
        let updated = db.update_service_default_policy(&service.id, None).unwrap();
        assert!(updated.default_tool_policy.is_none());

        // 获取策略应返回默认值（AllowAll）
        let retrieved = db.get_service_default_policy(&service.id).unwrap();
        assert!(retrieved.is_allow_all());
    }

    #[test]
    fn test_update_service_default_policy_not_found() {
        let db = Database::new_in_memory().unwrap();

        let policy = ToolPolicy::default();
        let result = db.update_service_default_policy("nonexistent-id", Some(&policy));
        assert!(result.is_err());
    }
}
