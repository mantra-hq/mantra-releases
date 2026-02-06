//! LPM 查询模块
//!
//! Story 11.27: MCP Roots LPM 集成 - Task 1, Task 2
//!
//! 由于 rusqlite 不支持跨线程，Gateway 运行在独立的 Tokio 任务中，
//! 无法直接访问 Tauri State 中的数据库连接。
//!
//! 本模块通过 channel 机制实现 Gateway 与主线程之间的 LPM 查询通信：
//! 1. Gateway 端发送查询请求 (LpmQueryRequest)
//! 2. 主线程执行 ContextRouter::find_project_by_path()
//! 3. 主线程返回查询结果 (LpmQueryResponse)

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// LPM 查询请求
///
/// Story 11.27: Task 1.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpmQueryRequest {
    /// 请求 ID (用于匹配响应)
    pub request_id: String,
    /// 要查询的路径
    pub path: String,
}

/// LPM 查询响应
///
/// Story 11.27: Task 1.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpmQueryResponse {
    /// 请求 ID (匹配请求)
    pub request_id: String,
    /// 匹配到的项目 ID
    pub project_id: Option<String>,
    /// 匹配到的项目名称
    pub project_name: Option<String>,
    /// 匹配到的路径
    pub matched_path: Option<PathBuf>,
}

/// 项目上下文 (LPM 查询结果)
///
/// Story 11.27: AC2 - 设置 session 的项目上下文
#[derive(Debug, Clone)]
pub struct LpmProjectContext {
    /// 项目 ID
    pub project_id: String,
    /// 项目名称
    pub project_name: String,
    /// 匹配的路径
    pub matched_path: PathBuf,
    /// 匹配来源
    pub match_source: String,
}

/// 内部查询请求 (包含响应 channel)
pub struct PendingQuery {
    pub request: LpmQueryRequest,
    pub response_tx: oneshot::Sender<LpmQueryResponse>,
}

/// LPM 查询客户端 (Gateway 端)
///
/// Story 11.27: Task 2
///
/// Gateway 端使用此客户端发送 LPM 查询请求并等待响应
pub struct LpmQueryClient {
    /// 查询请求发送器
    query_tx: mpsc::Sender<PendingQuery>,
}

impl LpmQueryClient {
    /// 默认超时时间 (秒)
    ///
    /// Story 11.27: AC6 - 5 秒超时
    pub const DEFAULT_TIMEOUT_SECS: u64 = 5;

    /// 创建新的 LPM 查询客户端
    ///
    /// 返回 (客户端, 接收器)，接收器由主线程使用
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<PendingQuery>) {
        let (query_tx, query_rx) = mpsc::channel(buffer_size);
        let client = Self { query_tx };
        (client, query_rx)
    }

    /// 查询项目 (通过路径)
    ///
    /// Story 11.27: Task 2.2 - 实现 query_project_by_path
    /// Story 11.27: AC6 - 5 秒超时机制
    ///
    /// # Arguments
    /// * `path` - 要查询的路径
    ///
    /// # Returns
    /// 匹配的项目上下文，如果没有匹配或超时则返回 None
    pub async fn query_project_by_path(&self, path: &str) -> Option<LpmProjectContext> {
        self.query_project_by_path_with_timeout(path, Self::DEFAULT_TIMEOUT_SECS)
            .await
    }

    /// 查询项目 (带自定义超时)
    ///
    /// Story 11.27: Task 2.4 - 实现超时机制
    pub async fn query_project_by_path_with_timeout(
        &self,
        path: &str,
        timeout_secs: u64,
    ) -> Option<LpmProjectContext> {
        // 1. 生成唯一的请求 ID
        let request_id = Uuid::new_v4().to_string();

        // 2. 创建 oneshot channel 用于接收响应
        let (response_tx, response_rx) = oneshot::channel();

        // 3. 构造请求
        let request = LpmQueryRequest {
            request_id: request_id.clone(),
            path: path.to_string(),
        };

        let pending_query = PendingQuery {
            request,
            response_tx,
        };

        // 4. 发送查询请求
        if self.query_tx.send(pending_query).await.is_err() {
            eprintln!("[LpmQueryClient] Failed to send query request: channel closed");
            return None;
        }

        // 5. 等待响应 (带超时)
        let result = tokio::time::timeout(Duration::from_secs(timeout_secs), response_rx).await;

        match result {
            Ok(Ok(response)) => {
                // 成功收到响应
                if let (Some(project_id), Some(project_name)) =
                    (response.project_id, response.project_name)
                {
                    Some(LpmProjectContext {
                        project_id,
                        project_name,
                        matched_path: response.matched_path.unwrap_or_else(|| PathBuf::from(path)),
                        match_source: "roots".to_string(),
                    })
                } else {
                    // AC3: 无匹配项目
                    eprintln!(
                        "[LpmQueryClient] No Mantra project found for roots path: {}",
                        path
                    );
                    None
                }
            }
            Ok(Err(_)) => {
                // Response channel 关闭
                eprintln!(
                    "[LpmQueryClient] Response channel closed for path: {}",
                    path
                );
                None
            }
            Err(_) => {
                // AC6: 超时
                eprintln!(
                    "[LpmQueryClient] LPM query timeout for path: {} ({}s)",
                    path, timeout_secs
                );
                None
            }
        }
    }
}

impl Clone for LpmQueryClient {
    fn clone(&self) -> Self {
        Self {
            query_tx: self.query_tx.clone(),
        }
    }
}

/// LPM 查询服务 (主线程端)
///
/// Story 11.27: Task 1.3
///
/// 主线程使用此服务处理来自 Gateway 的 LPM 查询请求
pub struct LpmQueryService {
    /// 查询请求接收器
    query_rx: mpsc::Receiver<PendingQuery>,
}

impl LpmQueryService {
    /// 创建 LPM 查询服务
    pub fn new(query_rx: mpsc::Receiver<PendingQuery>) -> Self {
        Self { query_rx }
    }

    /// 启动服务 (在后台运行)
    ///
    /// 使用 spawn_blocking 在阻塞线程中执行同步的数据库查询，
    /// 解决 rusqlite::Connection 不是 Send/Sync 的问题。
    ///
    /// # Arguments
    /// * `db_path` - 数据库文件路径
    pub async fn run_with_db_path(mut self, db_path: PathBuf) {
        use crate::gateway::ContextRouter;
        use crate::storage::Database;

        eprintln!("[LpmQueryService] Started with db_path: {:?}", db_path);

        while let Some(pending) = self.query_rx.recv().await {
            let request = pending.request;
            let response_tx = pending.response_tx;
            let db_path_clone = db_path.clone();

            eprintln!(
                "[LpmQueryService] Received query for path: {}",
                request.path
            );

            // 使用 spawn_blocking 在同步线程中执行数据库查询
            let path_for_query = request.path.clone();
            let result = tokio::task::spawn_blocking(move || {
                // 在阻塞线程中创建轻量级数据库连接（不运行 schema/migrations）
                let db = match Database::open_for_query(&db_path_clone) {
                    Ok(db) => Arc::new(db),
                    Err(e) => {
                        eprintln!("[LpmQueryService] Failed to create database connection: {}", e);
                        return None;
                    }
                };
                let router = ContextRouter::new(db);
                router.find_project_by_path_sync(&path_for_query)
                    .map(|ctx| (ctx.project_id, ctx.project_name, ctx.matched_path))
            }).await;

            // 构造响应
            let response = match result {
                Ok(Some((project_id, project_name, matched_path))) => {
                    eprintln!(
                        "[LpmQueryService] Matched project '{}' (id={}) for path: {}",
                        project_name, project_id, request.path
                    );
                    LpmQueryResponse {
                        request_id: request.request_id,
                        project_id: Some(project_id),
                        project_name: Some(project_name),
                        matched_path: Some(matched_path),
                    }
                }
                Ok(None) | Err(_) => {
                    eprintln!(
                        "[LpmQueryService] No project found for path: {}",
                        request.path
                    );
                    LpmQueryResponse {
                        request_id: request.request_id,
                        project_id: None,
                        project_name: None,
                        matched_path: None,
                    }
                }
            };

            // 发送响应
            let _ = response_tx.send(response);
        }

        eprintln!("[LpmQueryService] Stopped");
    }
}

/// 线程安全的 LPM 查询客户端包装
pub type SharedLpmQueryClient = Arc<LpmQueryClient>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lpm_query_request_serialization() {
        let request = LpmQueryRequest {
            request_id: "req-123".to_string(),
            path: "/home/user/project".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("req-123"));
        assert!(json.contains("/home/user/project"));

        let deserialized: LpmQueryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.request_id, "req-123");
        assert_eq!(deserialized.path, "/home/user/project");
    }

    #[tokio::test]
    async fn test_lpm_query_response_serialization() {
        let response = LpmQueryResponse {
            request_id: "req-123".to_string(),
            project_id: Some("proj-456".to_string()),
            project_name: Some("Test Project".to_string()),
            matched_path: Some(PathBuf::from("/test/path")),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("proj-456"));
        assert!(json.contains("Test Project"));

        let deserialized: LpmQueryResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.project_id, Some("proj-456".to_string()));
    }

    #[tokio::test]
    async fn test_lpm_query_timeout() {
        let (client, _query_rx) = LpmQueryClient::new(16);
        // 不启动服务，模拟超时

        // 测试超时（使用 1 秒超时加速测试）
        let result = client
            .query_project_by_path_with_timeout("/some/path", 1)
            .await;
        assert!(result.is_none());
    }

    /// Story 11.27 Task 5.1: 测试 LPM 查询成功匹配
    #[tokio::test]
    async fn test_lpm_query_success_match() {
        let (client, mut query_rx) = LpmQueryClient::new(16);

        // 启动模拟服务端
        tokio::spawn(async move {
            if let Some(pending) = query_rx.recv().await {
                // 模拟成功匹配
                let response = LpmQueryResponse {
                    request_id: pending.request.request_id,
                    project_id: Some("proj-123".to_string()),
                    project_name: Some("Test Project".to_string()),
                    matched_path: Some(PathBuf::from("/home/user/test-project")),
                };
                let _ = pending.response_tx.send(response);
            }
        });

        // 执行查询
        let result = client
            .query_project_by_path("/home/user/test-project/src")
            .await;

        // 验证结果
        assert!(result.is_some());
        let ctx = result.unwrap();
        assert_eq!(ctx.project_id, "proj-123");
        assert_eq!(ctx.project_name, "Test Project");
        assert_eq!(ctx.matched_path, PathBuf::from("/home/user/test-project"));
        assert_eq!(ctx.match_source, "roots");
    }

    /// Story 11.27 Task 5.2: 测试 LPM 查询无匹配
    #[tokio::test]
    async fn test_lpm_query_no_match() {
        let (client, mut query_rx) = LpmQueryClient::new(16);

        // 启动模拟服务端 - 返回无匹配
        tokio::spawn(async move {
            if let Some(pending) = query_rx.recv().await {
                // 模拟无匹配
                let response = LpmQueryResponse {
                    request_id: pending.request.request_id,
                    project_id: None,
                    project_name: None,
                    matched_path: None,
                };
                let _ = pending.response_tx.send(response);
            }
        });

        // 执行查询
        let result = client
            .query_project_by_path("/some/unknown/path")
            .await;

        // 验证结果 - 应该返回 None
        assert!(result.is_none());
    }

    /// Story 11.27 Task 5.1: 测试 LPM 查询 - 连续多次查询
    #[tokio::test]
    async fn test_lpm_query_multiple_queries() {
        let (client, mut query_rx) = LpmQueryClient::new(16);

        // 启动模拟服务端 - 处理多个查询
        tokio::spawn(async move {
            let mut count = 0;
            while let Some(pending) = query_rx.recv().await {
                count += 1;
                let response = if count == 1 {
                    // 第一个查询：无匹配
                    LpmQueryResponse {
                        request_id: pending.request.request_id,
                        project_id: None,
                        project_name: None,
                        matched_path: None,
                    }
                } else {
                    // 第二个查询：成功匹配
                    LpmQueryResponse {
                        request_id: pending.request.request_id,
                        project_id: Some("proj-456".to_string()),
                        project_name: Some("Second Project".to_string()),
                        matched_path: Some(PathBuf::from("/home/user/second")),
                    }
                };
                let _ = pending.response_tx.send(response);
            }
        });

        // 第一次查询 - 无匹配
        let result1 = client.query_project_by_path("/unknown/path").await;
        assert!(result1.is_none());

        // 第二次查询 - 成功
        let result2 = client.query_project_by_path("/home/user/second/src").await;
        assert!(result2.is_some());
        let ctx = result2.unwrap();
        assert_eq!(ctx.project_id, "proj-456");
        assert_eq!(ctx.project_name, "Second Project");
    }

    /// Story 11.27 Task 5.4: 测试默认超时值
    #[test]
    fn test_lpm_query_default_timeout() {
        assert_eq!(LpmQueryClient::DEFAULT_TIMEOUT_SECS, 5);
    }

    /// Story 11.27: 测试 LpmProjectContext 结构
    #[test]
    fn test_lpm_project_context_fields() {
        let ctx = LpmProjectContext {
            project_id: "proj-123".to_string(),
            project_name: "My Project".to_string(),
            matched_path: PathBuf::from("/test/path"),
            match_source: "roots".to_string(),
        };

        assert_eq!(ctx.project_id, "proj-123");
        assert_eq!(ctx.project_name, "My Project");
        assert_eq!(ctx.matched_path, PathBuf::from("/test/path"));
        assert_eq!(ctx.match_source, "roots");
    }
}

