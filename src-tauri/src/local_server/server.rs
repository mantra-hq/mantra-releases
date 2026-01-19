//! HTTP Server 实现
//!
//! 使用 axum 创建本地 HTTP Server，支持启动、停止和重启。

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::{oneshot, watch};
use tower_http::cors::{Any, CorsLayer};

use super::config::LocalServerConfig;
use super::handlers::{self, AppState};
use crate::sanitizer::PrivacyScanner;
use crate::storage::Database;

/// Server 控制句柄
///
/// 用于控制 Server 的生命周期
pub struct ServerHandle {
    /// 关闭信号发送器
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// 当前运行的端口
    port: u16,
}

impl ServerHandle {
    /// 获取当前端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 关闭 Server
    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// 本地 HTTP Server
pub struct LocalServer {
    config_dir: PathBuf,
    db: Option<Arc<Mutex<Database>>>,
}

impl LocalServer {
    /// 创建新的 Server 实例
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir, db: None }
    }

    /// 创建带数据库的 Server 实例
    pub fn with_database(config_dir: PathBuf, db: Arc<Mutex<Database>>) -> Self {
        Self { config_dir, db: Some(db) }
    }

    /// 启动 Server
    ///
    /// # Arguments
    /// * `port` - 可选端口号，如果不提供则从配置读取
    ///
    /// # Returns
    /// ServerHandle 用于控制 Server 生命周期
    pub async fn start(&self, port: Option<u16>) -> Result<ServerHandle, String> {
        // 确定端口
        let port = port.unwrap_or_else(|| {
            LocalServerConfig::load(&self.config_dir).local_api_port
        });

        // 验证端口
        LocalServerConfig::validate_port(port)?;

        // 创建 PrivacyScanner
        let scanner = PrivacyScanner::with_config(&self.config_dir)
            .map_err(|e| format!("Failed to create privacy scanner: {}", e))?;

        // 创建共享状态
        let state = Arc::new(AppState {
            scanner,
            config_dir: self.config_dir.clone(),
            db: self.db.clone(),
        });

        // 创建路由
        let app = Router::new()
            .route("/api/privacy/check", post(handlers::privacy_check))
            .route("/api/privacy/check-files", post(handlers::check_files))  // Story 3.11
            .route("/api/health", get(handlers::health_check))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(state);

        // 绑定地址 (仅本地)
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        // 创建 TCP listener
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

        // 创建关闭信号
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        // 在后台运行 Server
        tokio::spawn(async move {
            let graceful = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                });

            if let Err(e) = graceful.await {
                eprintln!("Server error: {}", e);
            }
        });

        Ok(ServerHandle {
            shutdown_tx: Some(shutdown_tx),
            port,
        })
    }

    /// 检查端口是否可用
    pub async fn check_port_available(port: u16) -> bool {
        tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
            .await
            .is_ok()
    }
}

/// Server 管理器
///
/// 管理 Server 的生命周期，支持热重启
pub struct ServerManager {
    config_dir: PathBuf,
    handle: Option<ServerHandle>,
    db: Option<Arc<Mutex<Database>>>,
    /// 端口变更通知
    port_tx: watch::Sender<u16>,
    port_rx: watch::Receiver<u16>,
}

impl ServerManager {
    /// 创建新的 ServerManager
    pub fn new(config_dir: PathBuf) -> Self {
        let config = LocalServerConfig::load(&config_dir);
        let (port_tx, port_rx) = watch::channel(config.local_api_port);

        Self {
            config_dir,
            handle: None,
            db: None,
            port_tx,
            port_rx,
        }
    }

    /// 创建带数据库的 ServerManager
    pub fn with_database(config_dir: PathBuf, db: Arc<Mutex<Database>>) -> Self {
        let config = LocalServerConfig::load(&config_dir);
        let (port_tx, port_rx) = watch::channel(config.local_api_port);

        Self {
            config_dir,
            handle: None,
            db: Some(db),
            port_tx,
            port_rx,
        }
    }

    /// 启动 Server
    pub async fn start(&mut self) -> Result<(), String> {
        if self.handle.is_some() {
            return Ok(()); // 已经在运行
        }

        let server = if let Some(db) = &self.db {
            LocalServer::with_database(self.config_dir.clone(), db.clone())
        } else {
            LocalServer::new(self.config_dir.clone())
        };
        let handle = server.start(None).await?;
        let _ = self.port_tx.send(handle.port());
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
            let mut config = LocalServerConfig::load(&self.config_dir);
            config.local_api_port = port;
            config.save(&self.config_dir)?;
        }

        // 重新启动
        let server = if let Some(db) = &self.db {
            LocalServer::with_database(self.config_dir.clone(), db.clone())
        } else {
            LocalServer::new(self.config_dir.clone())
        };
        let handle = server.start(new_port).await?;
        let _ = self.port_tx.send(handle.port());
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
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        self.stop();
    }
}
