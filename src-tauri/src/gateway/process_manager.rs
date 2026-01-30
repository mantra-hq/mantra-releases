//! MCP 子进程管理器
//!
//! Story 11.5: 上下文路由 - Task 6
//!
//! 负责启动、管理和回收 MCP 服务子进程

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, RwLock};

use crate::models::mcp::McpService;

/// 进程错误
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Failed to spawn process: {0}")]
    SpawnError(String),
    #[error("Process communication error: {0}")]
    CommunicationError(String),
    #[error("Process timeout")]
    Timeout,
    #[error("Process exited unexpectedly")]
    ProcessExited,
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
}

/// 进程请求
struct ProcessRequest {
    /// JSON-RPC 请求
    request: serde_json::Value,
    /// 响应通道
    response_tx: oneshot::Sender<Result<serde_json::Value, ProcessError>>,
}

/// MCP 进程实例
struct McpProcess {
    /// 进程句柄
    _child: Child,
    /// 请求发送通道
    request_tx: mpsc::Sender<ProcessRequest>,
    /// 最后活跃时间
    last_active: chrono::DateTime<chrono::Utc>,
    /// 服务名称
    service_name: String,
}

/// MCP 子进程管理器
///
/// 负责启动、管理和回收 MCP 服务子进程
pub struct McpProcessManager {
    /// 运行中的进程 (service_id -> McpProcess)
    processes: RwLock<HashMap<String, McpProcess>>,
    /// 空闲超时时间（秒）
    idle_timeout_secs: u64,
}

impl McpProcessManager {
    /// 默认空闲超时时间（5 分钟）
    const DEFAULT_IDLE_TIMEOUT: u64 = 300;
    /// 请求超时时间（30 秒）
    pub const REQUEST_TIMEOUT: u64 = 30;

    /// 创建新的进程管理器
    pub fn new() -> Self {
        Self {
            processes: RwLock::new(HashMap::new()),
            idle_timeout_secs: Self::DEFAULT_IDLE_TIMEOUT,
        }
    }

    /// 创建带自定义超时的进程管理器
    pub fn with_idle_timeout(idle_timeout_secs: u64) -> Self {
        Self {
            processes: RwLock::new(HashMap::new()),
            idle_timeout_secs,
        }
    }

    /// 获取或启动 MCP 服务进程
    ///
    /// 如果进程已运行，直接返回；否则启动新进程
    ///
    /// # Arguments
    /// * `service` - MCP 服务配置
    /// * `env` - 环境变量映射
    pub async fn get_or_spawn(
        &self,
        service: &McpService,
        env: HashMap<String, String>,
    ) -> Result<(), ProcessError> {
        // 检查是否已运行
        {
            let processes = self.processes.read().await;
            if processes.contains_key(&service.id) {
                return Ok(());
            }
        }

        // 启动新进程
        self.spawn_process(service, env).await
    }

    /// 启动 MCP 服务进程
    async fn spawn_process(
        &self,
        service: &McpService,
        env: HashMap<String, String>,
    ) -> Result<(), ProcessError> {
        let mut cmd = Command::new(&service.command);

        // 设置参数
        if let Some(args) = &service.args {
            cmd.args(args);
        }

        // 设置环境变量
        for (key, value) in env {
            cmd.env(key, value);
        }

        // 配置 stdio
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| ProcessError::SpawnError(e.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ProcessError::SpawnError("Failed to get stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ProcessError::SpawnError("Failed to get stdout".to_string()))?;

        // 创建通信通道
        let (request_tx, request_rx) = mpsc::channel::<ProcessRequest>(32);

        // 启动 I/O 处理任务
        let service_id = service.id.clone();
        tokio::spawn(Self::process_io_loop(service_id.clone(), stdin, stdout, request_rx));

        // 保存进程
        {
            let mut processes = self.processes.write().await;
            processes.insert(
                service.id.clone(),
                McpProcess {
                    _child: child,
                    request_tx,
                    last_active: chrono::Utc::now(),
                    service_name: service.name.clone(),
                },
            );
        }

        Ok(())
    }

    /// 进程 I/O 循环
    async fn process_io_loop(
        service_id: String,
        mut stdin: ChildStdin,
        stdout: ChildStdout,
        mut request_rx: mpsc::Receiver<ProcessRequest>,
    ) {
        let mut reader = BufReader::new(stdout).lines();
        let mut pending_requests: HashMap<
            serde_json::Value,
            oneshot::Sender<Result<serde_json::Value, ProcessError>>,
        > = HashMap::new();

        loop {
            tokio::select! {
                // 处理传入请求
                Some(req) = request_rx.recv() => {
                    // 发送请求到 stdin
                    let request_json = match serde_json::to_string(&req.request) {
                        Ok(json) => json,
                        Err(e) => {
                            let _ = req.response_tx.send(Err(ProcessError::CommunicationError(e.to_string())));
                            continue;
                        }
                    };

                    if let Err(e) = stdin.write_all(format!("{}\n", request_json).as_bytes()).await {
                        let _ = req.response_tx.send(Err(ProcessError::CommunicationError(e.to_string())));
                        continue;
                    }

                    if let Err(e) = stdin.flush().await {
                        let _ = req.response_tx.send(Err(ProcessError::CommunicationError(e.to_string())));
                        continue;
                    }

                    // 保存待处理请求
                    if let Some(id) = req.request.get("id") {
                        pending_requests.insert(id.clone(), req.response_tx);
                    }
                }

                // 处理 stdout 响应
                result = reader.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&line) {
                                if let Some(id) = response.get("id") {
                                    if let Some(tx) = pending_requests.remove(id) {
                                        let _ = tx.send(Ok(response));
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            // EOF - 进程已退出
                            eprintln!("[process_manager] Process {} stdout closed", service_id);
                            break;
                        }
                        Err(e) => {
                            eprintln!("[process_manager] Error reading stdout for {}: {}", service_id, e);
                            break;
                        }
                    }
                }
            }
        }

        // 清理待处理请求
        for (_, tx) in pending_requests {
            let _ = tx.send(Err(ProcessError::ProcessExited));
        }
    }

    /// 发送请求到 MCP 服务
    ///
    /// # Arguments
    /// * `service_id` - 服务 ID
    /// * `request` - JSON-RPC 请求
    ///
    /// # Returns
    /// JSON-RPC 响应
    pub async fn send_request(
        &self,
        service_id: &str,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, ProcessError> {
        let (response_tx, response_rx) = oneshot::channel();

        // 获取进程并发送请求
        {
            let mut processes = self.processes.write().await;
            if let Some(process) = processes.get_mut(service_id) {
                process.last_active = chrono::Utc::now();
                process
                    .request_tx
                    .send(ProcessRequest {
                        request,
                        response_tx,
                    })
                    .await
                    .map_err(|_| ProcessError::ProcessExited)?;
            } else {
                return Err(ProcessError::ServiceNotFound(service_id.to_string()));
            }
        }

        // 等待响应
        tokio::time::timeout(
            std::time::Duration::from_secs(Self::REQUEST_TIMEOUT),
            response_rx,
        )
        .await
        .map_err(|_| ProcessError::Timeout)?
        .map_err(|_| ProcessError::ProcessExited)?
    }

    /// 停止指定服务进程
    pub async fn stop_process(&self, service_id: &str) {
        let mut processes = self.processes.write().await;
        if let Some(mut process) = processes.remove(service_id) {
            let _ = process._child.kill().await;
        }
    }

    /// 停止所有进程
    pub async fn stop_all(&self) {
        let mut processes = self.processes.write().await;
        for (_, mut process) in processes.drain() {
            let _ = process._child.kill().await;
        }
    }

    /// 获取运行中的进程列表
    pub async fn list_running(&self) -> Vec<RunningProcess> {
        let processes = self.processes.read().await;
        processes
            .iter()
            .map(|(id, p)| RunningProcess {
                service_id: id.clone(),
                service_name: p.service_name.clone(),
                last_active: p.last_active,
            })
            .collect()
    }

    /// 清理空闲进程
    pub async fn cleanup_idle(&self) {
        let now = chrono::Utc::now();
        let timeout = chrono::Duration::seconds(self.idle_timeout_secs as i64);

        let mut to_remove = Vec::new();

        {
            let processes = self.processes.read().await;
            for (id, process) in processes.iter() {
                if now - process.last_active > timeout {
                    to_remove.push(id.clone());
                }
            }
        }

        for id in to_remove {
            self.stop_process(&id).await;
        }
    }

    /// 检查进程是否正在运行
    pub async fn is_running(&self, service_id: &str) -> bool {
        let processes = self.processes.read().await;
        processes.contains_key(service_id)
    }
}

impl Default for McpProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 运行中的进程信息
#[derive(Debug, Clone)]
pub struct RunningProcess {
    /// 服务 ID
    pub service_id: String,
    /// 服务名称
    pub service_name: String,
    /// 最后活跃时间
    pub last_active: chrono::DateTime<chrono::Utc>,
}

/// 线程安全的进程管理器包装
pub type SharedProcessManager = Arc<McpProcessManager>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_manager_new() {
        let manager = McpProcessManager::new();
        let running = manager.list_running().await;
        assert!(running.is_empty());
    }

    #[tokio::test]
    async fn test_process_manager_with_idle_timeout() {
        let manager = McpProcessManager::with_idle_timeout(60);
        assert_eq!(manager.idle_timeout_secs, 60);
    }

    #[tokio::test]
    async fn test_is_running_false() {
        let manager = McpProcessManager::new();
        assert!(!manager.is_running("non-existent").await);
    }

    #[tokio::test]
    async fn test_stop_non_existent_process() {
        let manager = McpProcessManager::new();
        // 不应该 panic
        manager.stop_process("non-existent").await;
    }

    #[tokio::test]
    async fn test_stop_all_empty() {
        let manager = McpProcessManager::new();
        // 不应该 panic
        manager.stop_all().await;
    }

    #[tokio::test]
    async fn test_cleanup_idle_empty() {
        let manager = McpProcessManager::new();
        // 不应该 panic
        manager.cleanup_idle().await;
    }

    #[tokio::test]
    async fn test_send_request_service_not_found() {
        let manager = McpProcessManager::new();
        let result = manager
            .send_request(
                "non-existent",
                serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "test"}),
            )
            .await;

        assert!(result.is_err());
        match result {
            Err(ProcessError::ServiceNotFound(id)) => assert_eq!(id, "non-existent"),
            _ => panic!("Expected ServiceNotFound error"),
        }
    }

    // 注意：实际的进程启动测试需要一个真实的 MCP 服务可执行文件
    // 这些测试在集成测试中进行
}
