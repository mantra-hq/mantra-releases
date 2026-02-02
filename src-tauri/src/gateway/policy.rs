//! Tool Policy 解析器
//!
//! Story 11.9 Phase 2: 工具策略完整实现
//!
//! 提供 PolicyResolver trait 用于解耦 Gateway 和 Storage，
//! 支持从不同来源获取 Tool Policy（数据库、缓存等）。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::mcp::ToolPolicy;
use crate::storage::Database;

/// Tool Policy 解析器 trait
///
/// Story 11.9 Phase 2: 工具策略完整实现 - AC #9
///
/// 用于解耦 Gateway 和 Storage，支持多种策略来源。
/// 优先级: 项目级 > 全局默认 > 系统默认(AllowAll)
#[async_trait]
pub trait PolicyResolver: Send + Sync {
    /// 获取指定项目和服务的 Tool Policy
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID，如果为 None 则使用全局默认
    /// * `service_id` - 服务 ID
    ///
    /// # Returns
    /// 合并后的 Tool Policy
    async fn get_policy(&self, project_id: Option<&str>, service_id: &str) -> ToolPolicy;

    /// 批量获取多个服务的 Policy
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID，如果为 None 则使用全局默认
    /// * `service_ids` - 服务 ID 列表
    ///
    /// # Returns
    /// service_id -> ToolPolicy 的映射
    async fn get_policies(
        &self,
        project_id: Option<&str>,
        service_ids: &[String],
    ) -> HashMap<String, ToolPolicy>;
}

/// 共享的 PolicyResolver 类型别名
pub type SharedPolicyResolver = Arc<dyn PolicyResolver>;

/// 基于 Storage 的 PolicyResolver 实现
///
/// 从数据库获取 Tool Policy:
/// 1. 首先检查 project_mcp_services.config_override.toolPolicy（项目级）
/// 2. 如果没有，使用 mcp_services.default_tool_policy（全局默认）
/// 3. 如果都没有，返回默认策略 (AllowAll)
pub struct StoragePolicyResolver {
    db: Arc<Mutex<Database>>,
}

impl StoragePolicyResolver {
    /// 创建新的 StoragePolicyResolver
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PolicyResolver for StoragePolicyResolver {
    async fn get_policy(&self, project_id: Option<&str>, service_id: &str) -> ToolPolicy {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => return ToolPolicy::default(),
        };

        // 1. 如果有 project_id，尝试获取项目级 Policy
        if let Some(pid) = project_id {
            if let Ok(Some(link)) = db.get_project_service_link(pid, service_id) {
                let project_policy = link.get_tool_policy();
                // 如果项目级有配置（非默认），直接返回
                if project_policy.mode != crate::models::mcp::ToolPolicyMode::AllowAll
                    || !project_policy.allowed_tools.is_empty()
                    || !project_policy.denied_tools.is_empty()
                {
                    return project_policy;
                }
            }
        }

        // 2. 获取服务级全局默认 Policy
        if let Ok(service) = db.get_mcp_service(service_id) {
            if let Some(default_policy) = service.default_tool_policy {
                return default_policy;
            }
        }

        // 3. 返回系统默认 (AllowAll)
        ToolPolicy::default()
    }

    async fn get_policies(
        &self,
        project_id: Option<&str>,
        service_ids: &[String],
    ) -> HashMap<String, ToolPolicy> {
        let mut policies = HashMap::new();

        for service_id in service_ids {
            let policy = self.get_policy(project_id, service_id).await;
            policies.insert(service_id.clone(), policy);
        }

        policies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::mcp::{
        CreateMcpServiceRequest, McpServiceSource, ToolPolicy, ToolPolicyMode,
    };
    use crate::storage::Database;

    fn create_test_db() -> Arc<Mutex<Database>> {
        Arc::new(Mutex::new(Database::new_in_memory().unwrap()))
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_default() {
        let db = create_test_db();

        // 创建服务（无默认策略）
        {
            let db_guard = db.lock().unwrap();
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
            db_guard.create_mcp_service(&request).unwrap();
        }

        let resolver = StoragePolicyResolver::new(db.clone());

        // 获取策略应返回默认值
        let service = {
            let db_guard = db.lock().unwrap();
            db_guard.get_mcp_service_by_name("test-service").unwrap().unwrap()
        };
        let policy = resolver.get_policy(None, &service.id).await;
        assert_eq!(policy.mode, ToolPolicyMode::AllowAll);
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_global_default() {
        let db = create_test_db();

        // 创建服务并设置全局默认策略
        let service_id = {
            let db_guard = db.lock().unwrap();
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
            let service = db_guard.create_mcp_service(&request).unwrap();

            // 设置全局默认策略
            let policy = ToolPolicy {
                mode: ToolPolicyMode::DenyAll,
                allowed_tools: vec![],
                denied_tools: vec![],
            };
            db_guard
                .update_service_default_policy(&service.id, Some(&policy))
                .unwrap();

            service.id
        };

        let resolver = StoragePolicyResolver::new(db);

        // 获取策略应返回全局默认
        let policy = resolver.get_policy(None, &service_id).await;
        assert_eq!(policy.mode, ToolPolicyMode::DenyAll);
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_project_override() {
        let db = create_test_db();

        // 创建服务和项目，设置项目级策略
        let (service_id, project_id) = {
            let db_guard = db.lock().unwrap();

            // 创建服务并设置全局默认策略
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
            let service = db_guard.create_mcp_service(&request).unwrap();

            let global_policy = ToolPolicy {
                mode: ToolPolicyMode::AllowAll,
                allowed_tools: vec![],
                denied_tools: vec![],
            };
            db_guard
                .update_service_default_policy(&service.id, Some(&global_policy))
                .unwrap();

            // 创建项目
            let (project, _) = db_guard
                .get_or_create_project("/home/user/test-project")
                .unwrap();

            // 关联服务到项目，设置项目级策略
            let project_policy = ToolPolicy {
                mode: ToolPolicyMode::Custom,
                allowed_tools: vec!["read_file".to_string()],
                denied_tools: vec![],
            };
            let config_override = serde_json::json!({
                "toolPolicy": project_policy
            });
            db_guard
                .link_service_to_project(&project.id, &service.id, Some(&config_override))
                .unwrap();

            (service.id, project.id)
        };

        let resolver = StoragePolicyResolver::new(db);

        // 有项目 ID 时应返回项目级策略
        let policy = resolver.get_policy(Some(&project_id), &service_id).await;
        assert_eq!(policy.mode, ToolPolicyMode::Custom);
        assert_eq!(policy.allowed_tools, vec!["read_file"]);

        // 无项目 ID 时应返回全局默认
        let policy_no_project = resolver.get_policy(None, &service_id).await;
        assert_eq!(policy_no_project.mode, ToolPolicyMode::AllowAll);
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_get_policies() {
        let db = create_test_db();

        // 创建多个服务
        let (service1_id, service2_id) = {
            let db_guard = db.lock().unwrap();

            let request1 = CreateMcpServiceRequest {
                name: "service-1".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            };
            let service1 = db_guard.create_mcp_service(&request1).unwrap();

            // 为 service1 设置 DenyAll 策略
            let policy1 = ToolPolicy {
                mode: ToolPolicyMode::DenyAll,
                allowed_tools: vec![],
                denied_tools: vec![],
            };
            db_guard
                .update_service_default_policy(&service1.id, Some(&policy1))
                .unwrap();

            let request2 = CreateMcpServiceRequest {
                name: "service-2".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            };
            let service2 = db_guard.create_mcp_service(&request2).unwrap();
            // service2 保持默认 AllowAll

            (service1.id, service2.id)
        };

        let resolver = StoragePolicyResolver::new(db);

        let policies = resolver
            .get_policies(None, &[service1_id.clone(), service2_id.clone()])
            .await;

        assert_eq!(policies.len(), 2);
        assert_eq!(policies.get(&service1_id).unwrap().mode, ToolPolicyMode::DenyAll);
        assert_eq!(policies.get(&service2_id).unwrap().mode, ToolPolicyMode::AllowAll);
    }

    /// 不存在的 service_id 应返回 AllowAll 默认策略
    #[tokio::test]
    async fn test_storage_policy_resolver_nonexistent_service() {
        let db = create_test_db();
        let resolver = StoragePolicyResolver::new(db);

        let policy = resolver.get_policy(None, "nonexistent-service-id").await;
        assert_eq!(policy.mode, ToolPolicyMode::AllowAll);
        assert!(policy.allowed_tools.is_empty());
        assert!(policy.denied_tools.is_empty());
    }

    /// 不存在的 project_id 应回退到全局默认策略
    #[tokio::test]
    async fn test_storage_policy_resolver_nonexistent_project() {
        let db = create_test_db();

        let service_id = {
            let db_guard = db.lock().unwrap();
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
            let service = db_guard.create_mcp_service(&request).unwrap();

            let policy = ToolPolicy {
                mode: ToolPolicyMode::DenyAll,
                allowed_tools: vec![],
                denied_tools: vec![],
            };
            db_guard
                .update_service_default_policy(&service.id, Some(&policy))
                .unwrap();

            service.id
        };

        let resolver = StoragePolicyResolver::new(db);

        // 使用不存在的 project_id 应回退到全局默认
        let policy = resolver
            .get_policy(Some("nonexistent-project-id"), &service_id)
            .await;
        assert_eq!(policy.mode, ToolPolicyMode::DenyAll);
    }

    /// 空 service_ids 列表应返回空 HashMap
    #[tokio::test]
    async fn test_storage_policy_resolver_empty_service_ids() {
        let db = create_test_db();
        let resolver = StoragePolicyResolver::new(db);

        let policies = resolver.get_policies(None, &[]).await;
        assert!(policies.is_empty());
    }

    /// 项目关联服务但使用默认 AllowAll 策略时，应回退到全局默认
    #[tokio::test]
    async fn test_storage_policy_resolver_project_default_falls_through() {
        let db = create_test_db();

        let (service_id, project_id) = {
            let db_guard = db.lock().unwrap();

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
            let service = db_guard.create_mcp_service(&request).unwrap();

            // 设置全局默认为 DenyAll
            let global_policy = ToolPolicy {
                mode: ToolPolicyMode::DenyAll,
                allowed_tools: vec![],
                denied_tools: vec![],
            };
            db_guard
                .update_service_default_policy(&service.id, Some(&global_policy))
                .unwrap();

            // 创建项目并关联服务（项目级使用默认 AllowAll）
            let (project, _) = db_guard
                .get_or_create_project("/home/user/test-project-2")
                .unwrap();

            // 关联服务但不配置项目级 toolPolicy (config_override 为空)
            db_guard
                .link_service_to_project(&project.id, &service.id, None)
                .unwrap();

            (service.id, project.id)
        };

        let resolver = StoragePolicyResolver::new(db);

        // 项目关联了服务但没配置 toolPolicy，应回退到全局默认 DenyAll
        let policy = resolver.get_policy(Some(&project_id), &service_id).await;
        assert_eq!(policy.mode, ToolPolicyMode::DenyAll);
    }

    /// 批量获取策略包含项目覆盖
    #[tokio::test]
    async fn test_storage_policy_resolver_get_policies_with_project() {
        let db = create_test_db();

        let (service1_id, service2_id, project_id) = {
            let db_guard = db.lock().unwrap();

            // 创建两个服务
            let request1 = CreateMcpServiceRequest {
                name: "service-a".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            };
            let service1 = db_guard.create_mcp_service(&request1).unwrap();

            let request2 = CreateMcpServiceRequest {
                name: "service-b".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            };
            let service2 = db_guard.create_mcp_service(&request2).unwrap();

            // service1 全局默认 DenyAll
            let policy1 = ToolPolicy {
                mode: ToolPolicyMode::DenyAll,
                allowed_tools: vec![],
                denied_tools: vec![],
            };
            db_guard
                .update_service_default_policy(&service1.id, Some(&policy1))
                .unwrap();

            // 创建项目
            let (project, _) = db_guard
                .get_or_create_project("/home/user/project-policies")
                .unwrap();

            // service1 在该项目中覆盖为 Custom
            let project_policy = ToolPolicy {
                mode: ToolPolicyMode::Custom,
                allowed_tools: vec!["read_file".to_string(), "write_file".to_string()],
                denied_tools: vec![],
            };
            let config_override = serde_json::json!({
                "toolPolicy": project_policy
            });
            db_guard
                .link_service_to_project(&project.id, &service1.id, Some(&config_override))
                .unwrap();

            (service1.id, service2.id, project.id)
        };

        let resolver = StoragePolicyResolver::new(db);

        let policies = resolver
            .get_policies(
                Some(&project_id),
                &[service1_id.clone(), service2_id.clone()],
            )
            .await;

        assert_eq!(policies.len(), 2);
        // service1 应使用项目级 Custom 覆盖
        assert_eq!(
            policies.get(&service1_id).unwrap().mode,
            ToolPolicyMode::Custom
        );
        assert_eq!(
            policies.get(&service1_id).unwrap().allowed_tools,
            vec!["read_file", "write_file"]
        );
        // service2 应使用全局默认 AllowAll
        assert_eq!(
            policies.get(&service2_id).unwrap().mode,
            ToolPolicyMode::AllowAll
        );
    }
}
