//! 项目服务查询模块
//!
//! Story 11.28: MCP 严格模式服务过滤 - Task 1
//!
//! 由于 rusqlite 不支持跨线程，Gateway 运行在独立的 Tokio 任务中，
//! 无法直接访问 Tauri State 中的数据库连接。
//!
//! 本模块通过 channel 机制实现 Gateway 与主线程之间的项目服务查询通信：
//! 1. Gateway 端发送查询请求 (ProjectServicesQueryRequest)
//! 2. 主线程执行 Database::get_project_service_links()
//! 3. 主线程返回查询结果 (ProjectServicesQueryResponse)

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// 项目服务查询请求
///
/// Story 11.28: Task 1.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServicesQueryRequest {
    /// 请求 ID (用于匹配响应)
    pub request_id: String,
    /// 项目 ID
    pub project_id: String,
}

/// 项目服务查询响应
///
/// Story 11.28: Task 1.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServicesQueryResponse {
    /// 请求 ID (匹配请求)
    pub request_id: String,
    /// 项目关联的服务 ID 列表
    pub service_ids: Vec<String>,
}

/// 内部查询请求 (包含响应 channel)
pub struct PendingProjectServicesQuery {
    pub request: ProjectServicesQueryRequest,
    pub response_tx: oneshot::Sender<ProjectServicesQueryResponse>,
}

/// 项目服务查询客户端 (Gateway 端)
///
/// Story 11.28: Task 1.4
///
/// Gateway 端使用此客户端发送项目服务查询请求并等待响应
pub struct ProjectServicesQueryClient {
    /// 查询请求发送器
    query_tx: mpsc::Sender<PendingProjectServicesQuery>,
}

impl ProjectServicesQueryClient {
    /// 默认超时时间 (秒)
    pub const DEFAULT_TIMEOUT_SECS: u64 = 5;

    /// 创建新的项目服务查询客户端
    ///
    /// 返回 (客户端, 接收器)，接收器由主线程使用
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<PendingProjectServicesQuery>) {
        let (query_tx, query_rx) = mpsc::channel(buffer_size);
        let client = Self { query_tx };
        (client, query_rx)
    }

    /// 查询项目关联的服务 ID 列表
    ///
    /// Story 11.28: Task 1.4 - 实现 query_project_services
    ///
    /// # Arguments
    /// * `project_id` - 项目 ID
    ///
    /// # Returns
    /// 关联的服务 ID 集合，如果查询失败或超时则返回空集合
    pub async fn query_project_services(&self, project_id: &str) -> HashSet<String> {
        self.query_project_services_with_timeout(project_id, Self::DEFAULT_TIMEOUT_SECS)
            .await
    }

    /// 查询项目服务 (带自定义超时)
    pub async fn query_project_services_with_timeout(
        &self,
        project_id: &str,
        timeout_secs: u64,
    ) -> HashSet<String> {
        // 1. 生成唯一的请求 ID
        let request_id = Uuid::new_v4().to_string();

        // 2. 创建 oneshot channel 用于接收响应
        let (response_tx, response_rx) = oneshot::channel();

        // 3. 构造请求
        let request = ProjectServicesQueryRequest {
            request_id: request_id.clone(),
            project_id: project_id.to_string(),
        };

        let pending_query = PendingProjectServicesQuery {
            request,
            response_tx,
        };

        // 4. 发送查询请求
        if self.query_tx.send(pending_query).await.is_err() {
            eprintln!("[ProjectServicesQueryClient] Failed to send query request: channel closed");
            return HashSet::new();
        }

        // 5. 等待响应 (带超时)
        let result = tokio::time::timeout(Duration::from_secs(timeout_secs), response_rx).await;

        match result {
            Ok(Ok(response)) => {
                // 成功收到响应
                eprintln!(
                    "[ProjectServicesQueryClient] Project {} has {} services",
                    project_id,
                    response.service_ids.len()
                );
                response.service_ids.into_iter().collect()
            }
            Ok(Err(_)) => {
                // Response channel 关闭
                eprintln!(
                    "[ProjectServicesQueryClient] Response channel closed for project: {}",
                    project_id
                );
                HashSet::new()
            }
            Err(_) => {
                // 超时
                eprintln!(
                    "[ProjectServicesQueryClient] Query timeout for project: {} ({}s)",
                    project_id, timeout_secs
                );
                HashSet::new()
            }
        }
    }
}

impl Clone for ProjectServicesQueryClient {
    fn clone(&self) -> Self {
        Self {
            query_tx: self.query_tx.clone(),
        }
    }
}

/// 项目服务查询服务 (主线程端)
///
/// Story 11.28: Task 1.3
///
/// 主线程使用此服务处理来自 Gateway 的项目服务查询请求
pub struct ProjectServicesQueryService {
    /// 查询请求接收器
    query_rx: mpsc::Receiver<PendingProjectServicesQuery>,
}

impl ProjectServicesQueryService {
    /// 创建项目服务查询服务
    pub fn new(query_rx: mpsc::Receiver<PendingProjectServicesQuery>) -> Self {
        Self { query_rx }
    }

    /// 启动服务 (在后台运行)
    ///
    /// 使用 spawn_blocking 在阻塞线程中执行同步的数据库查询
    ///
    /// # Arguments
    /// * `db_path` - 数据库文件路径
    pub async fn run_with_db_path(mut self, db_path: PathBuf) {
        use crate::storage::Database;

        eprintln!(
            "[ProjectServicesQueryService] Started with db_path: {:?}",
            db_path
        );

        while let Some(pending) = self.query_rx.recv().await {
            let request = pending.request;
            let response_tx = pending.response_tx;
            let db_path_clone = db_path.clone();

            eprintln!(
                "[ProjectServicesQueryService] Received query for project: {}",
                request.project_id
            );

            // 使用 spawn_blocking 在同步线程中执行数据库查询
            let project_id = request.project_id.clone();
            let result = tokio::task::spawn_blocking(move || {
                // 在阻塞线程中创建数据库连接和执行查询
                let db = match Database::new(&db_path_clone) {
                    Ok(db) => db,
                    Err(e) => {
                        eprintln!(
                            "[ProjectServicesQueryService] Failed to create database connection: {}",
                            e
                        );
                        return Vec::new();
                    }
                };

                // 查询项目关联的服务
                match db.get_project_service_links(&project_id) {
                    Ok(links) => links.into_iter().map(|l| l.service_id).collect(),
                    Err(e) => {
                        eprintln!(
                            "[ProjectServicesQueryService] Failed to query project services: {}",
                            e
                        );
                        Vec::new()
                    }
                }
            })
            .await;

            // 构造响应
            let service_ids = result.unwrap_or_default();
            eprintln!(
                "[ProjectServicesQueryService] Found {} services for project: {}",
                service_ids.len(),
                request.project_id
            );

            let response = ProjectServicesQueryResponse {
                request_id: request.request_id,
                service_ids,
            };

            // 发送响应
            let _ = response_tx.send(response);
        }

        eprintln!("[ProjectServicesQueryService] Stopped");
    }
}

/// 线程安全的项目服务查询客户端包装
pub type SharedProjectServicesQueryClient = Arc<ProjectServicesQueryClient>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_project_services_query_request_serialization() {
        let request = ProjectServicesQueryRequest {
            request_id: "req-123".to_string(),
            project_id: "proj-456".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("req-123"));
        assert!(json.contains("proj-456"));

        let deserialized: ProjectServicesQueryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.request_id, "req-123");
        assert_eq!(deserialized.project_id, "proj-456");
    }

    #[tokio::test]
    async fn test_project_services_query_response_serialization() {
        let response = ProjectServicesQueryResponse {
            request_id: "req-123".to_string(),
            service_ids: vec!["svc-1".to_string(), "svc-2".to_string()],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("svc-1"));
        assert!(json.contains("svc-2"));

        let deserialized: ProjectServicesQueryResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.service_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_project_services_query_timeout() {
        let (client, _query_rx) = ProjectServicesQueryClient::new(16);
        // 不启动服务，模拟超时

        // 测试超时（使用 1 秒超时加速测试）
        let result = client
            .query_project_services_with_timeout("proj-123", 1)
            .await;
        assert!(result.is_empty());
    }

    /// Story 11.28 Task 5.1: 测试项目服务查询成功
    #[tokio::test]
    async fn test_project_services_query_success() {
        let (client, mut query_rx) = ProjectServicesQueryClient::new(16);

        // 启动模拟服务端
        tokio::spawn(async move {
            if let Some(pending) = query_rx.recv().await {
                // 模拟成功查询
                let response = ProjectServicesQueryResponse {
                    request_id: pending.request.request_id,
                    service_ids: vec!["svc-1".to_string(), "svc-2".to_string(), "svc-3".to_string()],
                };
                let _ = pending.response_tx.send(response);
            }
        });

        // 执行查询
        let result = client.query_project_services("proj-123").await;

        // 验证结果
        assert_eq!(result.len(), 3);
        assert!(result.contains("svc-1"));
        assert!(result.contains("svc-2"));
        assert!(result.contains("svc-3"));
    }

    /// Story 11.28 Task 5.2: 测试项目服务查询无结果
    #[tokio::test]
    async fn test_project_services_query_empty() {
        let (client, mut query_rx) = ProjectServicesQueryClient::new(16);

        // 启动模拟服务端 - 返回空列表
        tokio::spawn(async move {
            if let Some(pending) = query_rx.recv().await {
                let response = ProjectServicesQueryResponse {
                    request_id: pending.request.request_id,
                    service_ids: vec![],
                };
                let _ = pending.response_tx.send(response);
            }
        });

        // 执行查询
        let result = client.query_project_services("proj-unknown").await;

        // 验证结果 - 应该返回空集合
        assert!(result.is_empty());
    }

    /// 测试默认超时值
    #[test]
    fn test_project_services_query_default_timeout() {
        assert_eq!(ProjectServicesQueryClient::DEFAULT_TIMEOUT_SECS, 5);
    }
}
