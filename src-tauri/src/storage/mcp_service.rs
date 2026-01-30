//! MCP 服务存储操作
//!
//! Story 11.2: MCP 服务数据模型 - Task 3
//!
//! 提供 mcp_services 表的 CRUD 操作

use rusqlite::{params, Row};

use super::database::Database;
use super::error::StorageError;
use crate::models::mcp::{
    CreateMcpServiceRequest, McpService, McpServiceSource, McpServiceWithOverride,
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
        let args_json = request.args.as_ref().map(|a| serde_json::to_string(a).ok()).flatten();
        let env_json = request.env.as_ref().map(|e| serde_json::to_string(e).ok()).flatten();

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
        let args_json = args.map(|a| serde_json::to_string(a).ok()).flatten();
        let env_json = env.map(|e| serde_json::to_string(e).ok()).flatten();

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
        let config_json = config_override
            .map(|c| serde_json::to_string(c).ok())
            .flatten();

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
        let config_json = config_override
            .map(|c| serde_json::to_string(c).ok())
            .flatten();

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
}
