//! MCP 工具发现与缓存服务
//!
//! Story 11.10: Project-Level Tool Management - Task 2
//!
//! 负责发现 MCP 服务提供的工具列表，并进行缓存管理

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::models::mcp::McpServiceTool;
use crate::storage::{Database, StorageError};

/// 内存缓存条目
struct CacheEntry {
    tools: Vec<McpServiceTool>,
    cached_at: Instant,
}

/// MCP 工具发现服务
///
/// Story 11.10: Project-Level Tool Management - Task 2
///
/// 提供双层缓存机制：
/// - 内存缓存 (TTL: 5 分钟)
/// - SQLite 持久化缓存
pub struct McpToolDiscovery {
    db: Arc<Database>,
    /// 内存缓存: service_id -> CacheEntry
    memory_cache: RwLock<HashMap<String, CacheEntry>>,
    /// 缓存 TTL (秒)
    cache_ttl: Duration,
}

/// 从 tools/list 响应解析的工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: Option<String>,
    /// 输入参数 JSON Schema
    pub input_schema: Option<serde_json::Value>,
}

/// 工具发现结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDiscoveryResult {
    /// 服务 ID
    pub service_id: String,
    /// 工具列表
    pub tools: Vec<ToolDefinition>,
    /// 是否来自缓存
    pub from_cache: bool,
    /// 缓存时间 (ISO 8601)
    pub cached_at: Option<String>,
}

impl McpToolDiscovery {
    /// 创建新的工具发现服务
    ///
    /// # Arguments
    /// * `db` - 数据库连接
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            memory_cache: RwLock::new(HashMap::new()),
            cache_ttl: Duration::from_secs(300), // 5 分钟
        }
    }

    /// 创建带自定义 TTL 的工具发现服务
    ///
    /// # Arguments
    /// * `db` - 数据库连接
    /// * `cache_ttl_secs` - 缓存 TTL (秒)
    #[allow(dead_code)]
    pub fn with_ttl(db: Arc<Database>, cache_ttl_secs: u64) -> Self {
        Self {
            db,
            memory_cache: RwLock::new(HashMap::new()),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
        }
    }

    /// 获取服务的工具列表（优先使用缓存）
    ///
    /// Story 11.10: Task 2.2
    ///
    /// 查找顺序：
    /// 1. 内存缓存（如果未过期）
    /// 2. SQLite 持久化缓存（如果未过期）
    /// 3. 返回 None，表示需要从服务获取
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    ///
    /// # Returns
    /// 工具列表和缓存信息，如果没有有效缓存则返回 None
    pub async fn get_cached_tools(&self, service_id: &str) -> Option<ToolDiscoveryResult> {
        // 1. 尝试内存缓存
        {
            let cache = self.memory_cache.read().await;
            if let Some(entry) = cache.get(service_id) {
                if entry.cached_at.elapsed() < self.cache_ttl {
                    let tools: Vec<ToolDefinition> = entry
                        .tools
                        .iter()
                        .map(|t| ToolDefinition {
                            name: t.name.clone(),
                            description: t.description.clone(),
                            input_schema: t.input_schema.clone(),
                        })
                        .collect();

                    return Some(ToolDiscoveryResult {
                        service_id: service_id.to_string(),
                        tools,
                        from_cache: true,
                        cached_at: Some(entry.tools.first().map(|t| t.cached_at.clone()).unwrap_or_default()),
                    });
                }
            }
        }

        // 2. 尝试 SQLite 持久化缓存
        if let Ok(cached_tools) = self.db.get_cached_service_tools(service_id) {
            if !cached_tools.is_empty() {
                // 检查缓存是否过期
                let ttl_secs = self.cache_ttl.as_secs() as i64;
                if !cached_tools.first().map(|t| t.is_expired(ttl_secs)).unwrap_or(true) {
                    // 更新内存缓存
                    {
                        let mut cache = self.memory_cache.write().await;
                        cache.insert(
                            service_id.to_string(),
                            CacheEntry {
                                tools: cached_tools.clone(),
                                cached_at: Instant::now(),
                            },
                        );
                    }

                    let tools: Vec<ToolDefinition> = cached_tools
                        .iter()
                        .map(|t| ToolDefinition {
                            name: t.name.clone(),
                            description: t.description.clone(),
                            input_schema: t.input_schema.clone(),
                        })
                        .collect();

                    return Some(ToolDiscoveryResult {
                        service_id: service_id.to_string(),
                        tools,
                        from_cache: true,
                        cached_at: cached_tools.first().map(|t| t.cached_at.clone()),
                    });
                }
            }
        }

        None
    }

    /// 缓存服务的工具列表
    ///
    /// Story 11.10: Task 2.3, 2.4
    ///
    /// 同时更新内存缓存和 SQLite 持久化缓存
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    /// * `tools` - 工具列表
    pub async fn cache_tools(
        &self,
        service_id: &str,
        tools: Vec<ToolDefinition>,
    ) -> Result<(), StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        // 1. 更新 SQLite 持久化缓存
        let tool_data: Vec<(String, Option<String>, Option<serde_json::Value>)> = tools
            .iter()
            .map(|t| (t.name.clone(), t.description.clone(), t.input_schema.clone()))
            .collect();

        self.db.cache_service_tools(service_id, &tool_data)?;

        // 2. 更新内存缓存
        let mcp_tools: Vec<McpServiceTool> = tools
            .iter()
            .map(|t| McpServiceTool {
                id: uuid::Uuid::new_v4().to_string(),
                service_id: service_id.to_string(),
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: t.input_schema.clone(),
                cached_at: now.clone(),
            })
            .collect();

        {
            let mut cache = self.memory_cache.write().await;
            cache.insert(
                service_id.to_string(),
                CacheEntry {
                    tools: mcp_tools,
                    cached_at: Instant::now(),
                },
            );
        }

        Ok(())
    }

    /// 强制刷新服务的工具缓存
    ///
    /// Story 11.10: Task 2
    ///
    /// 清除内存缓存和 SQLite 持久化缓存
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    pub async fn invalidate_cache(&self, service_id: &str) -> Result<(), StorageError> {
        // 1. 清除 SQLite 缓存
        self.db.clear_service_tools_cache(service_id)?;

        // 2. 清除内存缓存
        {
            let mut cache = self.memory_cache.write().await;
            cache.remove(service_id);
        }

        Ok(())
    }

    /// 获取所有缓存的服务工具
    ///
    /// 用于批量加载缓存状态
    pub async fn get_all_cached_services(&self) -> Result<Vec<String>, StorageError> {
        let cache_times = self.db.get_service_tools_cache_times()?;
        Ok(cache_times.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Arc<Database> {
        Arc::new(Database::new_in_memory().unwrap())
    }

    #[tokio::test]
    async fn test_cache_and_retrieve_tools() {
        let db = create_test_db();
        let discovery = McpToolDiscovery::new(db.clone());

        // 首先创建一个 MCP 服务
        let service = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // 缓存工具
        let tools = vec![
            ToolDefinition {
                name: "read_file".to_string(),
                description: Some("Read a file".to_string()),
                input_schema: Some(serde_json::json!({"type": "object"})),
            },
            ToolDefinition {
                name: "write_file".to_string(),
                description: Some("Write a file".to_string()),
                input_schema: None,
            },
        ];

        discovery.cache_tools(&service.id, tools.clone()).await.unwrap();

        // 获取缓存
        let result = discovery.get_cached_tools(&service.id).await;
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.service_id, service.id);
        assert!(result.from_cache);
        assert_eq!(result.tools.len(), 2);
        assert_eq!(result.tools[0].name, "read_file");
        assert_eq!(result.tools[1].name, "write_file");
    }

    #[tokio::test]
    async fn test_invalidate_cache() {
        let db = create_test_db();
        let discovery = McpToolDiscovery::new(db.clone());

        let service = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // 缓存工具
        let tools = vec![ToolDefinition {
            name: "test_tool".to_string(),
            description: None,
            input_schema: None,
        }];
        discovery.cache_tools(&service.id, tools).await.unwrap();

        // 验证缓存存在
        assert!(discovery.get_cached_tools(&service.id).await.is_some());

        // 使缓存失效
        discovery.invalidate_cache(&service.id).await.unwrap();

        // 验证缓存已清除
        assert!(discovery.get_cached_tools(&service.id).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        let db = create_test_db();
        // 使用 1 秒 TTL 便于测试
        let discovery = McpToolDiscovery::with_ttl(db.clone(), 1);

        let service = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // 缓存工具
        let tools = vec![ToolDefinition {
            name: "test_tool".to_string(),
            description: None,
            input_schema: None,
        }];
        discovery.cache_tools(&service.id, tools).await.unwrap();

        // 立即验证缓存存在
        assert!(discovery.get_cached_tools(&service.id).await.is_some());

        // 等待 TTL 过期
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 验证缓存已过期
        assert!(discovery.get_cached_tools(&service.id).await.is_none());
    }

    #[tokio::test]
    async fn test_no_cache_returns_none() {
        let db = create_test_db();
        let discovery = McpToolDiscovery::new(db);

        let result = discovery.get_cached_tools("non-existent-service").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_sqlite_cache_persistence() {
        let db = create_test_db();

        let service = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // 使用第一个 discovery 实例缓存工具
        {
            let discovery = McpToolDiscovery::new(db.clone());
            let tools = vec![ToolDefinition {
                name: "persistent_tool".to_string(),
                description: Some("Test persistence".to_string()),
                input_schema: None,
            }];
            discovery.cache_tools(&service.id, tools).await.unwrap();
        }

        // 创建新的 discovery 实例，验证 SQLite 缓存仍然有效
        {
            let discovery = McpToolDiscovery::new(db.clone());
            let result = discovery.get_cached_tools(&service.id).await;
            assert!(result.is_some());

            let result = result.unwrap();
            assert_eq!(result.tools.len(), 1);
            assert_eq!(result.tools[0].name, "persistent_tool");
        }
    }
}
