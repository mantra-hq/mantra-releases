//! Tool Policy 解析器
//!
//! Story 11.9 Phase 2 → Story 11.18: 工具策略简化重构
//!
//! 提供 PolicyResolver trait 用于解耦 Gateway 和 Storage，
//! 支持从不同来源获取 Tool Policy（数据库、缓存等）。
//!
//! Story 11.18 简化:
//! - 移除 ToolPolicyMode 概念
//! - allowed_tools: None = 继承, Some([]) = 全选, Some([...]) = 部分选
//! - 优先级: 项目级 > 服务默认 > 系统默认(全选)

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::mcp::ToolPolicy;
use crate::storage::Database;

/// Tool Policy 解析器 trait
///
/// Story 11.9 Phase 2 → Story 11.18
///
/// 用于解耦 Gateway 和 Storage，支持多种策略来源。
/// 优先级: 项目级 > 服务默认 > 系统默认(全选)
#[async_trait]
pub trait PolicyResolver: Send + Sync {
    /// 获取指定项目和服务的 Tool Policy
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID，如果为 None 则使用服务默认
    /// * `service_id` - 服务 ID
    ///
    /// # Returns
    /// 解析后的 Tool Policy（已处理继承链）
    async fn get_policy(&self, project_id: Option<&str>, service_id: &str) -> ToolPolicy;

    /// 批量获取多个服务的 Policy
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID，如果为 None 则使用服务默认
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
/// Story 11.18: 简化的策略解析逻辑
///
/// 从数据库获取 Tool Policy:
/// 1. 如果有项目级覆盖且 allowed_tools 非 None → 使用项目级
/// 2. 如果项目级为 None (继承) 或无项目 → 使用服务默认
/// 3. 如果服务无默认策略 → 返回全选 (AllowAll)
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
                // Story 11.18: 如果项目级 allowed_tools 非 None，直接使用
                if !project_policy.is_inherit() {
                    return project_policy;
                }
                // 如果是继承模式 (allowed_tools = None)，继续查找服务默认
            }
        }

        // 2. 获取服务级默认 Policy
        if let Ok(service) = db.get_mcp_service(service_id) {
            if let Some(default_policy) = service.default_tool_policy {
                return default_policy;
            }
        }

        // 3. 返回系统默认 (全选)
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
    use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource, ToolPolicy};
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

        let service = {
            let db_guard = db.lock().unwrap();
            db_guard
                .get_mcp_service_by_name("test-service")
                .unwrap()
                .unwrap()
        };
        let policy = resolver.get_policy(None, &service.id).await;
        // 无默认策略时返回全选
        assert!(policy.is_allow_all());
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_service_default() {
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

            // 设置服务默认策略为部分选
            let policy = ToolPolicy::custom(vec!["read_file".to_string()]);
            db_guard
                .update_service_default_policy(&service.id, Some(&policy))
                .unwrap();

            service.id
        };

        let resolver = StoragePolicyResolver::new(db);

        let policy = resolver.get_policy(None, &service_id).await;
        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file"));
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_project_override() {
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

            // 设置服务默认策略为全选
            let global_policy = ToolPolicy::allow_all();
            db_guard
                .update_service_default_policy(&service.id, Some(&global_policy))
                .unwrap();

            // 创建项目
            let (project, _) = db_guard
                .get_or_create_project("/home/user/test-project")
                .unwrap();

            // 关联服务到项目，设置项目级策略为部分选
            let project_policy = ToolPolicy::custom(vec!["read_file".to_string()]);
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
        let policy = resolver
            .get_policy(Some(&project_id), &service_id)
            .await;
        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file"));

        // 无项目 ID 时应返回服务默认（全选）
        let policy_no_project = resolver.get_policy(None, &service_id).await;
        assert!(policy_no_project.is_allow_all());
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_project_inherit() {
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

            // 设置服务默认策略为部分选
            let service_policy = ToolPolicy::custom(vec!["read_file".to_string()]);
            db_guard
                .update_service_default_policy(&service.id, Some(&service_policy))
                .unwrap();

            // 创建项目
            let (project, _) = db_guard
                .get_or_create_project("/home/user/test-project-inherit")
                .unwrap();

            // 关联服务到项目，设置继承（allowed_tools = null）
            let inherit_policy = ToolPolicy::inherit();
            let config_override = serde_json::json!({
                "toolPolicy": inherit_policy
            });
            db_guard
                .link_service_to_project(&project.id, &service.id, Some(&config_override))
                .unwrap();

            (service.id, project.id)
        };

        let resolver = StoragePolicyResolver::new(db);

        // 项目级继承时应回退到服务默认
        let policy = resolver
            .get_policy(Some(&project_id), &service_id)
            .await;
        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file"));
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_get_policies() {
        let db = create_test_db();

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

            // service1 设置部分选
            let policy1 = ToolPolicy::custom(vec!["read_file".to_string()]);
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
            // service2 保持默认（全选）

            (service1.id, service2.id)
        };

        let resolver = StoragePolicyResolver::new(db);

        let policies = resolver
            .get_policies(None, &[service1_id.clone(), service2_id.clone()])
            .await;

        assert_eq!(policies.len(), 2);
        assert!(policies.get(&service1_id).unwrap().is_custom());
        assert!(policies.get(&service2_id).unwrap().is_allow_all());
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_nonexistent_service() {
        let db = create_test_db();
        let resolver = StoragePolicyResolver::new(db);

        let policy = resolver.get_policy(None, "nonexistent-service-id").await;
        assert!(policy.is_allow_all());
    }

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

            let policy = ToolPolicy::custom(vec!["read_file".to_string()]);
            db_guard
                .update_service_default_policy(&service.id, Some(&policy))
                .unwrap();

            service.id
        };

        let resolver = StoragePolicyResolver::new(db);

        // 使用不存在的 project_id 应回退到服务默认
        let policy = resolver
            .get_policy(Some("nonexistent-project-id"), &service_id)
            .await;
        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_empty_service_ids() {
        let db = create_test_db();
        let resolver = StoragePolicyResolver::new(db);

        let policies = resolver.get_policies(None, &[]).await;
        assert!(policies.is_empty());
    }

    #[tokio::test]
    async fn test_storage_policy_resolver_project_no_override_falls_through() {
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

            // 设置服务默认策略为部分选
            let service_policy = ToolPolicy::custom(vec!["read_file".to_string()]);
            db_guard
                .update_service_default_policy(&service.id, Some(&service_policy))
                .unwrap();

            // 创建项目并关联服务（无 config_override）
            let (project, _) = db_guard
                .get_or_create_project("/home/user/test-project-2")
                .unwrap();

            db_guard
                .link_service_to_project(&project.id, &service.id, None)
                .unwrap();

            (service.id, project.id)
        };

        let resolver = StoragePolicyResolver::new(db);

        // 项目关联了服务但没配置 toolPolicy，应回退到服务默认
        let policy = resolver
            .get_policy(Some(&project_id), &service_id)
            .await;
        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
    }
}
