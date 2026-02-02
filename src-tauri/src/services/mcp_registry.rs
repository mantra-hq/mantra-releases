//! MCP 服务注册表
//!
//! Story 11.5: 上下文路由 - Task 1
//!
//! 负责从数据库加载服务配置，并提供按项目或全局查询能力

use std::collections::HashMap;
use std::sync::Arc;

use crate::models::mcp::{McpService, McpServiceWithOverride};
use crate::storage::{Database, StorageError};

use super::EnvManager;

/// MCP 服务注册表
///
/// 负责从数据库加载服务配置，并提供按项目或全局查询能力
pub struct McpRegistry {
    db: Arc<Database>,
    env_manager: Arc<EnvManager>,
}

impl McpRegistry {
    /// 创建新的服务注册表
    ///
    /// # Arguments
    /// * `db` - 数据库连接
    /// * `env_manager` - 环境变量管理器
    pub fn new(db: Arc<Database>, env_manager: Arc<EnvManager>) -> Self {
        Self { db, env_manager }
    }

    /// 获取全局启用的 MCP 服务（未关联项目的服务）
    ///
    /// 用于无法匹配项目时的回退策略
    ///
    /// # Returns
    /// 启用的全局 MCP 服务列表
    pub fn get_global_services(&self) -> Result<Vec<McpService>, StorageError> {
        self.db
            .list_mcp_services()
            .map(|services| services.into_iter().filter(|s| s.enabled).collect())
    }

    /// 获取项目关联的 MCP 服务
    ///
    /// 包含项目级配置覆盖
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    ///
    /// # Returns
    /// 项目关联的启用 MCP 服务列表，包含项目级配置覆盖
    pub fn get_project_services(
        &self,
        project_id: &str,
    ) -> Result<Vec<McpServiceWithOverride>, StorageError> {
        self.db
            .get_project_services(project_id)
            .map(|services| services.into_iter().filter(|s| s.service.enabled).collect())
    }

    /// 合并服务配置与项目覆盖
    ///
    /// 项目覆盖可以修改 args 和 env
    ///
    /// # Arguments
    /// * `service` - 原始服务配置
    /// * `override_config` - 项目级配置覆盖
    ///
    /// # Returns
    /// 合并后的服务配置
    pub fn merge_service_config(
        &self,
        service: &McpService,
        override_config: Option<&serde_json::Value>,
    ) -> McpService {
        let mut merged = service.clone();

        if let Some(override_obj) = override_config.and_then(|v| v.as_object()) {
            // 覆盖 args
            if let Some(args) = override_obj.get("args") {
                if let Ok(new_args) = serde_json::from_value(args.clone()) {
                    merged.args = Some(new_args);
                }
            }

            // 合并 env（不是覆盖，而是合并）
            if let Some(env_override) = override_obj.get("env") {
                if let Some(base_env) = &merged.env {
                    let mut merged_env = base_env.clone();
                    if let (Some(base_obj), Some(override_obj)) =
                        (merged_env.as_object_mut(), env_override.as_object())
                    {
                        for (k, v) in override_obj {
                            base_obj.insert(k.clone(), v.clone());
                        }
                    }
                    merged.env = Some(merged_env);
                } else {
                    merged.env = Some(env_override.clone());
                }
            }
        }

        merged
    }

    /// 解析环境变量引用，构建子进程环境
    ///
    /// # Arguments
    /// * `service` - MCP 服务配置
    ///
    /// # Returns
    /// 环境变量映射（key -> value），变量引用已解析为实际值
    pub async fn build_process_env(
        &self,
        service: &McpService,
    ) -> Result<HashMap<String, String>, StorageError> {
        super::env_manager::build_mcp_env(service, &self.db, &self.env_manager)
    }

    /// 获取数据库连接引用
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// 获取环境变量管理器引用
    pub fn env_manager(&self) -> &EnvManager {
        &self.env_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource};

    fn create_test_db() -> Arc<Database> {
        Arc::new(Database::new_in_memory().unwrap())
    }

    fn create_test_env_manager() -> Arc<EnvManager> {
        let key_bytes: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        Arc::new(EnvManager::new(&key_bytes))
    }

    fn create_test_project(db: &Database, id: &str, name: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        let cwd = format!("/path/to/{}", id);
        db.connection()
            .execute(
                "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?4)",
                [id, name, &cwd, &now],
            )
            .unwrap();
    }

    #[test]
    fn test_get_global_services() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db.clone(), env_manager);

        // 创建两个服务，一个启用一个禁用
        let enabled_req = CreateMcpServiceRequest {
            name: "enabled-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let _enabled = db.create_mcp_service(&enabled_req).unwrap();

        let disabled_req = CreateMcpServiceRequest {
            name: "disabled-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let disabled = db.create_mcp_service(&disabled_req).unwrap();
        db.toggle_mcp_service(&disabled.id, false).unwrap();

        // 获取全局服务应该只返回启用的
        let services = registry.get_global_services().unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "enabled-service");
    }

    #[test]
    fn test_get_project_services() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db.clone(), env_manager);

        // 创建项目
        create_test_project(&db, "proj1", "Project 1");

        // 创建服务并关联到项目
        let req = CreateMcpServiceRequest {
            name: "project-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let service = db.create_mcp_service(&req).unwrap();
        db.link_service_to_project("proj1", &service.id, None)
            .unwrap();

        // 获取项目服务
        let services = registry.get_project_services("proj1").unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].service.name, "project-service");
    }

    #[test]
    fn test_get_project_services_filters_disabled() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db.clone(), env_manager);

        create_test_project(&db, "proj1", "Project 1");

        // 创建两个服务
        let enabled_req = CreateMcpServiceRequest {
            name: "enabled-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let enabled = db.create_mcp_service(&enabled_req).unwrap();

        let disabled_req = CreateMcpServiceRequest {
            name: "disabled-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        let disabled = db.create_mcp_service(&disabled_req).unwrap();
        db.toggle_mcp_service(&disabled.id, false).unwrap();

        // 关联两个服务到项目
        db.link_service_to_project("proj1", &enabled.id, None)
            .unwrap();
        db.link_service_to_project("proj1", &disabled.id, None)
            .unwrap();

        // 获取项目服务应该只返回启用的
        let services = registry.get_project_services("proj1").unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].service.name, "enabled-service");
    }

    #[test]
    fn test_merge_service_config_override_args() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db, env_manager);

        let service = McpService {
            id: "test-id".to_string(),
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: Some(vec!["--original".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let override_config = serde_json::json!({
            "args": ["--override", "--new"]
        });

        let merged = registry.merge_service_config(&service, Some(&override_config));
        assert_eq!(
            merged.args,
            Some(vec!["--override".to_string(), "--new".to_string()])
        );
    }

    #[test]
    fn test_merge_service_config_merge_env() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db, env_manager);

        let service = McpService {
            id: "test-id".to_string(),
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "BASE_VAR": "base_value",
                "SHARED_VAR": "original"
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let override_config = serde_json::json!({
            "env": {
                "NEW_VAR": "new_value",
                "SHARED_VAR": "overridden"
            }
        });

        let merged = registry.merge_service_config(&service, Some(&override_config));
        let env = merged.env.unwrap();
        let env_obj = env.as_object().unwrap();

        assert_eq!(env_obj.get("BASE_VAR").unwrap(), "base_value");
        assert_eq!(env_obj.get("NEW_VAR").unwrap(), "new_value");
        assert_eq!(env_obj.get("SHARED_VAR").unwrap(), "overridden");
    }

    #[test]
    fn test_merge_service_config_add_env_to_empty() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db, env_manager);

        let service = McpService {
            id: "test-id".to_string(),
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None, // No env initially
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let override_config = serde_json::json!({
            "env": {
                "NEW_VAR": "new_value"
            }
        });

        let merged = registry.merge_service_config(&service, Some(&override_config));
        let env = merged.env.unwrap();
        let env_obj = env.as_object().unwrap();

        assert_eq!(env_obj.get("NEW_VAR").unwrap(), "new_value");
    }

    #[test]
    fn test_merge_service_config_no_override() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db, env_manager);

        let service = McpService {
            id: "test-id".to_string(),
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: Some(vec!["--original".to_string()]),
            env: Some(serde_json::json!({"VAR": "value"})),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let merged = registry.merge_service_config(&service, None);
        assert_eq!(merged.args, service.args);
        assert_eq!(merged.env, service.env);
    }

    #[tokio::test]
    async fn test_build_process_env() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db.clone(), env_manager.clone());

        // 设置环境变量
        db.set_env_variable(&env_manager, "API_KEY", "sk-secret123", None)
            .unwrap();

        let service = McpService {
            id: "test-id".to_string(),
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "API_KEY": "$API_KEY",
                "DEBUG": "true"
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let env = registry.build_process_env(&service).await.unwrap();
        assert_eq!(env.get("API_KEY"), Some(&"sk-secret123".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_build_process_env_missing_var() {
        let db = create_test_db();
        let env_manager = create_test_env_manager();
        let registry = McpRegistry::new(db, env_manager);

        let service = McpService {
            id: "test-id".to_string(),
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "API_KEY": "$NONEXISTENT_VAR"
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        // 变量不存在时保留原始引用
        let env = registry.build_process_env(&service).await.unwrap();
        assert_eq!(env.get("API_KEY"), Some(&"$NONEXISTENT_VAR".to_string()));
    }
}
