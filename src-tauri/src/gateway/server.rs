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
    http::{header, Method},
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{oneshot, watch, RwLock};
use tower_http::cors::{AllowOrigin, CorsLayer};

use super::aggregator::SharedMcpAggregator;
use super::auth::auth_middleware;
use super::handlers::{
    health_handler, mcp_delete_handler, mcp_get_handler, mcp_post_handler, message_handler,
    sse_handler, GatewayAppState,
};
use super::lpm_query::SharedLpmQueryClient;
use super::policy::SharedPolicyResolver;
use super::project_services_query::SharedProjectServicesQueryClient;
use super::session::MCP_SESSION_ID_HEADER;
use super::state::{GatewayConfig, GatewayState, GatewayStats};

/// 默认端口
const DEFAULT_PORT_RANGE_START: u16 = 39600;

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
    /// MCP 协议聚合器 (Story 11.17)
    aggregator: Option<SharedMcpAggregator>,
    /// Tool Policy 解析器 (Story 11.9 Phase 2)
    policy_resolver: Option<SharedPolicyResolver>,
    /// LPM 查询客户端 (Story 11.27)
    lpm_client: Option<SharedLpmQueryClient>,
    /// 项目服务查询客户端 (Story 11.28)
    project_services_client: Option<SharedProjectServicesQueryClient>,
}

impl GatewayServer {
    /// 创建新的 Server 实例
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            config,
            aggregator: None,
            policy_resolver: None,
            lpm_client: None,
            project_services_client: None,
        }
    }

    /// 创建带默认配置的 Server 实例
    pub fn with_defaults() -> Self {
        Self::new(GatewayConfig::default())
    }

    /// 设置 MCP 聚合器
    ///
    /// Story 11.17: MCP 协议聚合器
    pub fn set_aggregator(&mut self, aggregator: SharedMcpAggregator) {
        self.aggregator = Some(aggregator);
    }

    /// 设置 Tool Policy 解析器
    ///
    /// Story 11.9 Phase 2: 工具策略完整实现
    pub fn set_policy_resolver(&mut self, policy_resolver: SharedPolicyResolver) {
        self.policy_resolver = Some(policy_resolver);
    }

    /// 设置 LPM 查询客户端
    ///
    /// Story 11.27: MCP Roots LPM 集成
    pub fn set_lpm_client(&mut self, lpm_client: SharedLpmQueryClient) {
        self.lpm_client = Some(lpm_client);
    }

    /// 设置项目服务查询客户端
    ///
    /// Story 11.28: MCP 严格模式服务过滤
    pub fn set_project_services_client(&mut self, client: SharedProjectServicesQueryClient) {
        self.project_services_client = Some(client);
    }

    /// 创建带聚合器的 Server 实例
    pub fn with_aggregator(config: GatewayConfig, aggregator: SharedMcpAggregator) -> Self {
        Self {
            config,
            aggregator: Some(aggregator),
            policy_resolver: None,
            lpm_client: None,
            project_services_client: None,
        }
    }

    /// 创建带聚合器和 PolicyResolver 的 Server 实例
    ///
    /// Story 11.9 Phase 2: 工具策略完整实现
    pub fn with_aggregator_and_policy(
        config: GatewayConfig,
        aggregator: SharedMcpAggregator,
        policy_resolver: SharedPolicyResolver,
    ) -> Self {
        Self {
            config,
            aggregator: Some(aggregator),
            policy_resolver: Some(policy_resolver),
            lpm_client: None,
            project_services_client: None,
        }
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
        // 确定目标端口：优先使用显式传入的端口，其次使用配置端口，最后使用默认端口
        let explicit_port = port.is_some() || self.config.port > 0;
        let target_port = port
            .or(if self.config.port > 0 { Some(self.config.port) } else { None })
            .unwrap_or(DEFAULT_PORT_RANGE_START);

        let addr = SocketAddr::from(([127, 0, 0, 1], target_port));
        let (listener, actual_port) = match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                // 获取实际绑定的端口（当 target_port=0 时由 OS 分配）
                let actual = listener.local_addr()
                    .map(|addr| addr.port())
                    .unwrap_or(target_port);
                (listener, actual)
            }
            Err(e) => {
                // 如果是默认端口（非用户显式配置）且被占用，自动尝试 OS 分配
                if !explicit_port {
                    let fallback_addr = SocketAddr::from(([127, 0, 0, 1], 0u16));
                    match tokio::net::TcpListener::bind(fallback_addr).await {
                        Ok(listener) => {
                            let actual = listener.local_addr()
                                .map(|addr| addr.port())
                                .unwrap_or(0);
                            eprintln!("[Gateway] 默认端口 {} 被占用，自动分配端口 {}", target_port, actual);
                            (listener, actual)
                        }
                        Err(e2) => {
                            return Err(format!(
                                "无法绑定端口 {}：{}。自动分配也失败：{}。",
                                target_port, e, e2
                            ));
                        }
                    }
                } else {
                    return Err(format!(
                        "无法绑定端口 {}：{}。请检查该端口是否被其他程序占用，或在设置中修改 Gateway 端口。",
                        target_port, e
                    ));
                }
            }
        };

        // 创建共享状态
        let mut config = self.config.clone();
        config.port = actual_port;
        let state = Arc::new(RwLock::new(GatewayState::new(config)));
        let stats = Arc::new(GatewayStats::new());

        // 创建应用状态
        // Story 11.27: 使用 with_all 统一创建，支持所有可选组件
        // Story 11.28: 添加 project_services_client
        let app_state = GatewayAppState::with_all(
            state.clone(),
            stats.clone(),
            self.aggregator.clone(),
            self.policy_resolver.clone(),
            self.lpm_client.clone(),
            self.project_services_client.clone(),
        );

        // 创建受保护路由（需要认证）
        // 旧版端点（向后兼容 Story 11.1 的 SSE Transport）
        let legacy_routes = Router::new()
            .route("/sse", get(sse_handler))
            .route("/message", post(message_handler))
            .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

        // Story 11.14: MCP Streamable HTTP 端点
        // /mcp 端点支持 POST、GET、DELETE
        let mcp_routes = Router::new()
            .route("/mcp", post(mcp_post_handler))
            .route("/mcp", get(mcp_get_handler))
            .route("/mcp", delete(mcp_delete_handler))
            .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

        // 创建 CORS 层
        // 允许 tauri://localhost 和开发模式下的 localhost/127.0.0.1 (任意端口)
        // Story 11.14: 扩展允许的 HTTP 方法和 Headers
        let cors = CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|origin, _| {
                if let Ok(origin_str) = origin.to_str() {
                    // 允许 Tauri 应用
                    if origin_str == "tauri://localhost" {
                        return true;
                    }
                    // 允许 localhost 和 127.0.0.1 的任意端口 (开发模式)
                    if origin_str.starts_with("http://localhost")
                        || origin_str.starts_with("http://127.0.0.1")
                    {
                        return true;
                    }
                }
                false
            }))
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
                // Story 11.14: MCP Streamable HTTP 规范 Headers
                axum::http::HeaderName::from_static(MCP_SESSION_ID_HEADER),
                axum::http::HeaderName::from_static("mcp-protocol-version"),
            ])
            .expose_headers([
                // 允许客户端读取 MCP-Session-Id 响应 Header
                axum::http::HeaderName::from_static(MCP_SESSION_ID_HEADER),
            ])
            .allow_credentials(false);

        // 创建完整路由
        let app = Router::new()
            // 公开端点（不需要认证）
            .route("/health", get(health_handler))
            // Story 11.14: MCP Streamable HTTP 端点
            .merge(mcp_routes)
            // 旧版端点（向后兼容）
            .merge(legacy_routes)
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
            port: actual_port,
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
    /// MCP 协议聚合器 (Story 11.17)
    aggregator: Option<SharedMcpAggregator>,
    /// Tool Policy 解析器 (Story 11.9 Phase 2)
    policy_resolver: Option<SharedPolicyResolver>,
    /// LPM 查询客户端 (Story 11.27)
    lpm_client: Option<SharedLpmQueryClient>,
    /// 项目服务查询客户端 (Story 11.28)
    project_services_client: Option<SharedProjectServicesQueryClient>,
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
            aggregator: None,
            policy_resolver: None,
            lpm_client: None,
            project_services_client: None,
        }
    }

    /// 创建带默认配置的 ServerManager
    pub fn with_defaults() -> Self {
        Self::new(GatewayConfig::default())
    }

    /// 设置 MCP 聚合器
    ///
    /// Story 11.17: MCP 协议聚合器
    pub fn set_aggregator(&mut self, aggregator: SharedMcpAggregator) {
        self.aggregator = Some(aggregator);
    }

    /// 获取 MCP 聚合器引用
    ///
    /// Story 11.17: MCP 协议聚合器
    pub fn aggregator(&self) -> Option<&SharedMcpAggregator> {
        self.aggregator.as_ref()
    }

    /// 设置 Tool Policy 解析器
    ///
    /// Story 11.9 Phase 2: 工具策略完整实现
    pub fn set_policy_resolver(&mut self, policy_resolver: SharedPolicyResolver) {
        self.policy_resolver = Some(policy_resolver);
    }

    /// 获取 Tool Policy 解析器引用
    ///
    /// Story 11.9 Phase 2: 工具策略完整实现
    pub fn policy_resolver(&self) -> Option<&SharedPolicyResolver> {
        self.policy_resolver.as_ref()
    }

    /// 设置 LPM 查询客户端
    ///
    /// Story 11.27: MCP Roots LPM 集成
    pub fn set_lpm_client(&mut self, lpm_client: SharedLpmQueryClient) {
        self.lpm_client = Some(lpm_client);
    }

    /// 获取 LPM 查询客户端引用
    ///
    /// Story 11.27: MCP Roots LPM 集成
    pub fn lpm_client(&self) -> Option<&SharedLpmQueryClient> {
        self.lpm_client.as_ref()
    }

    /// 设置项目服务查询客户端
    ///
    /// Story 11.28: MCP 严格模式服务过滤
    pub fn set_project_services_client(&mut self, client: SharedProjectServicesQueryClient) {
        self.project_services_client = Some(client);
    }

    /// 获取项目服务查询客户端引用
    ///
    /// Story 11.28: MCP 严格模式服务过滤
    pub fn project_services_client(&self) -> Option<&SharedProjectServicesQueryClient> {
        self.project_services_client.as_ref()
    }

    /// 启动 Server
    pub async fn start(&mut self) -> Result<(), String> {
        if self.handle.is_some() {
            return Ok(()); // 已经在运行
        }

        // Story 11.17: 使用聚合器创建 Server
        // Story 11.9 Phase 2: 同时注入 PolicyResolver
        // Story 11.27: 同时注入 LpmClient
        let mut server = match &self.aggregator {
            Some(aggregator) => GatewayServer::with_aggregator(self.config.clone(), aggregator.clone()),
            None => GatewayServer::new(self.config.clone()),
        };

        // 注入 PolicyResolver
        if let Some(policy_resolver) = &self.policy_resolver {
            server.set_policy_resolver(policy_resolver.clone());
        }

        // Story 11.27: 注入 LpmClient
        if let Some(lpm_client) = &self.lpm_client {
            server.set_lpm_client(lpm_client.clone());
        }

        // Story 11.28: 注入 ProjectServicesClient
        if let Some(client) = &self.project_services_client {
            server.set_project_services_client(client.clone());
        }

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

        // Story 11.17: 重新启动时使用聚合器
        // Story 11.9 Phase 2: 同时注入 PolicyResolver
        // Story 11.27: 同时注入 LpmClient
        let mut server = match &self.aggregator {
            Some(aggregator) => GatewayServer::with_aggregator(self.config.clone(), aggregator.clone()),
            None => GatewayServer::new(self.config.clone()),
        };

        // 注入 PolicyResolver
        if let Some(policy_resolver) = &self.policy_resolver {
            server.set_policy_resolver(policy_resolver.clone());
        }

        // Story 11.27: 注入 LpmClient
        if let Some(lpm_client) = &self.lpm_client {
            server.set_lpm_client(lpm_client.clone());
        }

        // Story 11.28: 注入 ProjectServicesClient
        if let Some(client) = &self.project_services_client {
            server.set_project_services_client(client.clone());
        }

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
            port: 0, // 让操作系统分配端口（测试用）
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);

        let handle = server.start(Some(0)).await.expect("Server should start with OS-assigned port");
        let port = handle.port();
        assert!(port > 0);

        // 验证端口被占用
        assert!(!GatewayServer::check_port_available(port).await);

        // 关闭服务器
        handle.shutdown();

        // 等待一小段时间让端口释放
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_server_manager_lifecycle() {
        // 先找一个空闲端口用于测试
        let test_listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .expect("Should bind to OS-assigned port");
        let test_port = test_listener.local_addr().unwrap().port();
        drop(test_listener); // 释放端口

        let config = GatewayConfig {
            port: test_port,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let mut manager = GatewayServerManager::new(config);

        // 启动
        manager.start().await.expect("Manager should start");
        assert!(manager.is_running());

        let port = manager.current_port();
        assert!(port > 0);

        // 停止
        manager.stop();
        assert!(!manager.is_running());
    }

    #[tokio::test]
    async fn test_server_strict_port_binding() {
        // 测试严格端口绑定：指定端口被占用时应报错
        let config = GatewayConfig {
            port: DEFAULT_PORT_RANGE_START,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config.clone());

        // 第一个服务器应该成功启动
        let handle1 = server.start(Some(DEFAULT_PORT_RANGE_START)).await;
        if handle1.is_err() {
            // 端口可能已被其他进程占用，跳过测试
            return;
        }
        let handle1 = handle1.unwrap();
        assert_eq!(handle1.port(), DEFAULT_PORT_RANGE_START);

        // 第二个服务器尝试绑定同一端口应该失败
        let server2 = GatewayServer::new(config);
        let result = server2.start(Some(DEFAULT_PORT_RANGE_START)).await;
        assert!(result.is_err());

        // 清理
        handle1.shutdown();
    }
}
