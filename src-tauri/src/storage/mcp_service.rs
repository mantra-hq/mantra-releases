//! MCP 服务存储操作
//!
//! Story 11.2: MCP 服务数据模型 - Task 3
//!
//! 提供 mcp_services 表的 CRUD 操作

use regex;
use rusqlite::{params, Row};

use super::database::Database;
use super::error::StorageError;
use crate::models::mcp::{
    CreateMcpServiceRequest, McpService, McpServiceSource, McpServiceTool, McpServiceWithOverride,
    ProjectMcpService, UpdateMcpServiceRequest,
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

    Ok(McpService {
        id: row.get(0)?,
        name: row.get(1)?,
        command: row.get(2)?,
        args,
        env,
        source,
        source_file: row.get(6)?,
        enabled: enabled != 0,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
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

        // 序列化 args 和 env 为 JSON
        let args_json = request.args.as_ref().and_then(|a| serde_json::to_string(a).ok());
        let env_json = request.env.as_ref().and_then(|e| serde_json::to_string(e).ok());

        self.connection().execute(
            r#"INSERT INTO mcp_services (id, name, command, args, env, source, source_file, enabled, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)"#,
            params![
                &id,
                &request.name,
                &request.command,
                &args_json,
                &env_json,
                request.source.as_str(),
                &request.source_file,
                &now,
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
                r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at
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
            r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at
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
            r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at
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
            r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at
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
        let command = update.command.as_ref().unwrap_or(&current.command);
        let args = update.args.as_ref().or(current.args.as_ref());
        let env = update.env.as_ref().or(current.env.as_ref());
        let enabled = update.enabled.unwrap_or(current.enabled);

        // 序列化 args 和 env 为 JSON
        let args_json = args.and_then(|a| serde_json::to_string(a).ok());
        let env_json = env.and_then(|e| serde_json::to_string(e).ok());

        self.connection().execute(
            r#"UPDATE mcp_services SET name = ?1, command = ?2, args = ?3, env = ?4, enabled = ?5, updated_at = ?6
               WHERE id = ?7"#,
            params![
                name,
                command,
                &args_json,
                &env_json,
                enabled as i32,
                &now,
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
    /// # Arguments
    /// * `project_id` - 项目 ID
    ///
    /// # Returns
    /// 项目关联的 MCP 服务列表，包含项目级配置覆盖
    pub fn get_project_services(
        &self,
        project_id: &str,
    ) -> Result<Vec<McpServiceWithOverride>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT s.id, s.name, s.command, s.args, s.env, s.source, s.source_file, s.enabled, s.created_at, s.updated_at,
                      ps.config_override
               FROM mcp_services s
               INNER JOIN project_mcp_services ps ON s.id = ps.service_id
               WHERE ps.project_id = ?1
               ORDER BY s.name ASC"#,
        )?;

        let services = stmt
            .query_map([project_id], |row| {
                let service = parse_mcp_service_row(row)?;
                let config_json: Option<String> = row.get(10)?;
                let config_override = config_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(McpServiceWithOverride {
                    service,
                    config_override,
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
            r#"SELECT id, name, command, args, env, source, source_file, enabled, created_at, updated_at
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
            r#"SELECT project_id, service_id, config_override, created_at
               FROM project_mcp_services WHERE project_id = ?1 AND service_id = ?2"#,
            params![project_id, service_id],
            |row| {
                let config_json: Option<String> = row.get(2)?;
                let config_override = config_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ProjectMcpService {
                    project_id: row.get(0)?,
                    service_id: row.get(1)?,
                    config_override,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request() -> CreateMcpServiceRequest {
        CreateMcpServiceRequest {
            name: "git-mcp".to_string(),
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            env: Some(serde_json::json!({"DEBUG": "true"})),
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
            command: "npx".to_string(),
            args: None,
            env: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let request2 = CreateMcpServiceRequest {
            name: "beta-service".to_string(),
            command: "uvx".to_string(),
            args: None,
            env: None,
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
            command: "npx".to_string(),
            args: None,
            env: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let request2 = CreateMcpServiceRequest {
            name: "imported-service".to_string(),
            command: "uvx".to_string(),
            args: None,
            env: None,
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
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/openai-mcp".to_string()]),
            env: Some(serde_json::json!({
                "OPENAI_API_KEY": "$OPENAI_API_KEY",
                "DEBUG": "true"
            })),
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
                command: "npx".to_string(),
                args: None,
                env: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let service2 = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "beta-service".to_string(),
                command: "uvx".to_string(),
                args: None,
                env: None,
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
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "OPENAI_API_KEY": "$OPENAI_API_KEY",
                "DEBUG": "true"
            })),
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();

        // 创建不使用该变量的服务
        let request2 = CreateMcpServiceRequest {
            name: "other-service".to_string(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "DEBUG": "true"
            })),
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
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "API_KEY": "${ANTHROPIC_API_KEY}",
            })),
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
                command: "npx".to_string(),
                args: None,
                env: Some(serde_json::json!({
                    "API_KEY": "$SHARED_KEY",
                })),
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
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "OTHER_VAR": "$OTHER_VAR",
            })),
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
            command: "npx".to_string(),
            args: None,
            env: None,
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
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "KEY": "$OPENAI_API_KEY",
            })),
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request1).unwrap();

        // 创建使用 $API_KEY 的服务
        let request2 = CreateMcpServiceRequest {
            name: "api-key-service".to_string(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "KEY": "$API_KEY",
            })),
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
}
