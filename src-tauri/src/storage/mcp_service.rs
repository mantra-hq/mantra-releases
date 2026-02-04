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
    let backup_hash: Option<String> = row.get(9)?;

    Ok(TakeoverBackup {
        id: row.get(0)?,
        tool_type: ToolType::from_str(&tool_type_str).unwrap_or(ToolType::ClaudeCode),
        scope: TakeoverScope::from_str(&scope_str).unwrap_or(TakeoverScope::User),
        project_path: project_path_str.map(PathBuf::from),
        original_path: PathBuf::from(row.get::<_, String>(2)?),
        backup_path: PathBuf::from(row.get::<_, String>(3)?),
        backup_hash,
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
            r#"INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
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
                &backup.backup_hash,
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
                r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
                   FROM mcp_takeover_backups
                   WHERE status = ?1
                   ORDER BY taken_over_at DESC"#
            }
            None => {
                r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
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
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
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
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
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

    /// 按工具类型和作用域获取活跃的接管备份 (Story 11.16, 11.21)
    ///
    /// # Arguments
    /// * `tool_type` - 工具类型
    /// * `scope` - 接管作用域
    /// * `project_path` - 项目路径（project/local 作用域需要）
    pub fn get_active_takeover_by_tool_and_scope(
        &self,
        tool_type: &ToolType,
        scope: &TakeoverScope,
        project_path: Option<&str>,
    ) -> Result<Option<TakeoverBackup>, StorageError> {
        let backup = match scope {
            TakeoverScope::User => {
                let mut stmt = self.connection().prepare(
                    r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
                       FROM mcp_takeover_backups
                       WHERE tool_type = ?1 AND status = 'active' AND scope = 'user'
                       ORDER BY taken_over_at DESC
                       LIMIT 1"#,
                )?;
                stmt.query_row([tool_type.as_str()], parse_takeover_backup_row)
                    .optional()?
            }
            TakeoverScope::Project | TakeoverScope::Local => {
                // Project 和 Local scope 都需要按 project_path 查询
                let mut stmt = self.connection().prepare(
                    r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
                       FROM mcp_takeover_backups
                       WHERE tool_type = ?1 AND status = 'active' AND scope = ?2 AND project_path = ?3
                       ORDER BY taken_over_at DESC
                       LIMIT 1"#,
                )?;
                stmt.query_row(
                    params![tool_type.as_str(), scope.as_str(), project_path.unwrap_or("")],
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
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
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

    /// 获取所有 local scope 的活跃接管备份 (Story 11.21)
    ///
    /// 用于列出 Claude Code 所有项目的 local scope 接管状态
    pub fn get_active_local_scope_takeovers(
        &self,
    ) -> Result<Vec<TakeoverBackup>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
               FROM mcp_takeover_backups
               WHERE scope = 'local' AND status = 'active'
               ORDER BY project_path ASC, taken_over_at DESC"#,
        )?;

        let backups = stmt
            .query_map([], parse_takeover_backup_row)?
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
            r#"SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path, backup_hash
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
mod tests;
