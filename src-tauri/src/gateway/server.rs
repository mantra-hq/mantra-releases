//! Gateway Server 实现
//!
//! Story 11.1: SSE Server 核心 - Task 5
//! Story 11.8: MCP Gateway Architecture Refactor - Task 8
//!
//! 使用 Axum 创建 HTTP Server，支持:
//! - HTTP Transport (POST /message)
//! - SSE Transport (GET /sse) - 向后兼容
//! - Authorization Header 认证
//! - 严格的 CORS 策略

use axum::{
    http::{header, HeaderValue, Method},
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{oneshot, watch, RwLock};
use tower_http::cors::CorsLayer;

use super::auth::auth_middleware;
use super::handlers::{health_handler, message_handler, sse_handler, GatewayAppState};
use super::state::{GatewayConfig, GatewayState, GatewayStats};

/// 默认端口范围起始
const DEFAULT_PORT_RANGE_START: u16 = 39600;
/// 默认端口范围结束
const DEFAULT_PORT_RANGE_END: u16 = 39699;

/// Server 控制句柄
///
/// 用于控制 Server 的生命周期
pub struct GatewayServerHandle {
    /// 关闭信号发送器
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// 当前运行的端口
    port: u16,
    /// 共享状态引用
    state: Arc<RwLock<GatewayState>>,
    /// 统计信息引用
    stats: Arc<GatewayStats>,
}

impl GatewayServerHandle {
    /// 获取当前端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 获取共享状态引用
    pub fn state(&self) -> Arc<RwLock<GatewayState>> {
        self.state.clone()
    }

    /// 获取统计信息引用
    pub fn stats(&self) -> Arc<GatewayStats> {
        self.stats.clone()
    }

    /// 关闭 Server
    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // 发送关闭信号到状态
        let state = self.state.clone();
        tokio::spawn(async move {
            let state_guard = state.read().await;
            state_guard.send_shutdown();
        });
    }
}

impl Drop for GatewayServerHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Gateway Server
pub struct GatewayServer {
    config: GatewayConfig,
}

impl GatewayServer {
    /// 创建新的 Server 实例
    pub fn new(config: GatewayConfig) -> Self {
        Self { config }
    }

    /// 创建带默认配置的 Server 实例
    pub fn with_defaults() -> Self {
        Self::new(GatewayConfig::default())
    }

    /// 启动 Server
    ///
    /// # Arguments
    /// * `port` - 可选端口号，如果不提供则使用配置中的端口或自动分配
    ///
    /// # Returns
    /// GatewayServerHandle 用于控制 Server 生命周期
    pub async fn start(&self, port: Option<u16>) -> Result<GatewayServerHandle, String> {
        // 确定要使用的端口并绑定 (原子操作，避免 TOCTOU 竞争)
        let (listener, port) = match port.or(if self.config.port > 0 {
            Some(self.config.port)
        } else {
            None
        }) {
            Some(p) => {
                // 尝试绑定指定端口
                let addr = SocketAddr::from(([127, 0, 0, 1], p));
                match tokio::net::TcpListener::bind(addr).await {
                    Ok(listener) => (listener, p),
                    Err(_) => {
                        // 指定端口被占用，尝试自动分配
                        self.find_and_bind_port().await?
                    }
                }
            }
            None => {
                // 自动分配端口
                self.find_and_bind_port().await?
            }
        };

        // 创建共享状态
        let mut config = self.config.clone();
        config.port = port;
        let state = Arc::new(RwLock::new(GatewayState::new(config)));
        let stats = Arc::new(GatewayStats::new());

        // 创建应用状态
        let app_state = GatewayAppState::new(state.clone(), stats.clone());

        // 创建受保护路由（需要认证）
        let protected_routes = Router::new()
            .route("/sse", get(sse_handler))
            .route("/message", post(message_handler))
            .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

        // 创建 CORS 层 - 严格策略
        // 仅允许 tauri://localhost 和开发模式下的 http://localhost
        let cors = CorsLayer::new()
            .allow_origin([
                "tauri://localhost".parse::<HeaderValue>().unwrap(),
                "http://localhost".parse::<HeaderValue>().unwrap(),
                "http://127.0.0.1".parse::<HeaderValue>().unwrap(),
            ])
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .allow_credentials(false);

        // 创建完整路由
        let app = Router::new()
            // 公开端点（不需要认证）
            .route("/health", get(health_handler))
            // 合并受保护路由
            .merge(protected_routes)
            .layer(cors)
            .with_state(app_state);

        // 创建关闭信号
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        // 在后台运行 Server (使用已绑定的 listener)
        tokio::spawn(async move {
            let graceful = axum::serve(listener, app).with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            });

            if let Err(e) = graceful.await {
                eprintln!("[Mantra Gateway] Server error: {}", e);
            }
        });

        Ok(GatewayServerHandle {
            shutdown_tx: Some(shutdown_tx),
            port,
            state,
            stats,
        })
    }

    /// 检查端口是否可用
    pub async fn check_port_available(port: u16) -> bool {
        tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
            .await
            .is_ok()
    }

    /// 在指定范围内查找可用端口并绑定
    /// 
    /// 返回已绑定的 TcpListener 以避免 TOCTOU 竞争条件
    async fn find_and_bind_port(&self) -> Result<(tokio::net::TcpListener, u16), String> {
        // 首先尝试在首选端口范围内绑定
        for port in DEFAULT_PORT_RANGE_START..=DEFAULT_PORT_RANGE_END {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
                return Ok((listener, port));
            }
        }
        
        // 如果首选范围都被占用，让操作系统分配端口
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to any port: {}", e))?;
        let port = listener.local_addr()
            .map_err(|e| format!("Failed to get local address: {}", e))?
            .port();
        Ok((listener, port))
    }

    /// 在指定范围内查找可用端口 (向后兼容)
    #[allow(dead_code)]
    async fn find_available_port(&self) -> Result<u16, String> {
        let (listener, port) = self.find_and_bind_port().await?;
        drop(listener); // 释放端口供后续使用
        Ok(port)
    }
}

/// Server 管理器
///
/// 管理 Server 的生命周期，支持热重启
pub struct GatewayServerManager {
    config: GatewayConfig,
    handle: Option<GatewayServerHandle>,
    /// 端口变更通知
    port_tx: watch::Sender<u16>,
    port_rx: watch::Receiver<u16>,
}

impl GatewayServerManager {
    /// 创建新的 ServerManager
    pub fn new(config: GatewayConfig) -> Self {
        let initial_port = if config.port > 0 {
            config.port
        } else {
            DEFAULT_PORT_RANGE_START
        };
        let (port_tx, port_rx) = watch::channel(initial_port);

        Self {
            config,
            handle: None,
            port_tx,
            port_rx,
        }
    }

    /// 创建带默认配置的 ServerManager
    pub fn with_defaults() -> Self {
        Self::new(GatewayConfig::default())
    }

    /// 启动 Server
    pub async fn start(&mut self) -> Result<(), String> {
        if self.handle.is_some() {
            return Ok(()); // 已经在运行
        }

        let server = GatewayServer::new(self.config.clone());
        let handle = server.start(None).await?;
        let port = handle.port();
        let _ = self.port_tx.send(port);
        self.handle = Some(handle);
        Ok(())
    }

    /// 停止 Server
    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.shutdown();
        }
    }

    /// 重启 Server（使用新端口）
    pub async fn restart(&mut self, new_port: Option<u16>) -> Result<(), String> {
        // 先停止
        self.stop();

        // 如果指定了新端口，更新配置
        if let Some(port) = new_port {
            self.config.port = port;
        }

        // 重新启动
        let server = GatewayServer::new(self.config.clone());
        let handle = server.start(new_port).await?;
        let port = handle.port();
        let _ = self.port_tx.send(port);
        self.handle = Some(handle);
        Ok(())
    }

    /// 获取当前端口
    pub fn current_port(&self) -> u16 {
        *self.port_rx.borrow()
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    /// 获取端口变更订阅
    pub fn port_receiver(&self) -> watch::Receiver<u16> {
        self.port_rx.clone()
    }

    /// 获取认证 Token
    pub fn auth_token(&self) -> &str {
        &self.config.auth_token
    }

    /// 获取共享状态引用
    pub fn state(&self) -> Option<Arc<RwLock<GatewayState>>> {
        self.handle.as_ref().map(|h| h.state())
    }

    /// 获取统计信息引用
    pub fn stats(&self) -> Option<Arc<GatewayStats>> {
        self.handle.as_ref().map(|h| h.stats())
    }
}

impl Drop for GatewayServerManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_start_stop() {
        let config = GatewayConfig {
            port: 0, // 自动分配端口
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);

        let handle = server.start(None).await.expect("Server should start");
        let port = handle.port();
        assert!(port >= DEFAULT_PORT_RANGE_START && port <= DEFAULT_PORT_RANGE_END);

        // 验证端口被占用
        assert!(!GatewayServer::check_port_available(port).await);

        // 关闭服务器
        handle.shutdown();

        // 等待一小段时间让端口释放
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_server_manager_lifecycle() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let mut manager = GatewayServerManager::new(config);

        // 启动
        manager.start().await.expect("Manager should start");
        assert!(manager.is_running());

        let port = manager.current_port();
        assert!(port >= DEFAULT_PORT_RANGE_START);

        // 停止
        manager.stop();
        assert!(!manager.is_running());
    }

    #[tokio::test]
    async fn test_find_available_port() {
        let server = GatewayServer::with_defaults();
        let port = server.find_available_port().await;
        assert!(port.is_ok());
        let port = port.unwrap();
        assert!(port >= DEFAULT_PORT_RANGE_START && port <= DEFAULT_PORT_RANGE_END);
    }
}
