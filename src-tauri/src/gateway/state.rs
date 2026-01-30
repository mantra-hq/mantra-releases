//! Gateway 共享状态定义
//!
//! Story 11.1: SSE Server 核心
//! 使用 Arc<RwLock<>> 进行共享状态管理

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// SSE 客户端会话信息
#[derive(Debug, Clone)]
pub struct ClientSession {
    /// 会话 ID
    pub session_id: String,
    /// 消息端点 URL
    pub message_endpoint: String,
    /// 连接时间
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// 最后活跃时间
    pub last_active: chrono::DateTime<chrono::Utc>,
}

impl ClientSession {
    /// 创建新的客户端会话
    pub fn new() -> Self {
        let session_id = Uuid::new_v4().to_string();
        let message_endpoint = format!("/message?session_id={}", session_id);
        let now = chrono::Utc::now();

        Self {
            session_id,
            message_endpoint,
            connected_at: now,
            last_active: now,
        }
    }

    /// 更新最后活跃时间
    pub fn touch(&mut self) {
        self.last_active = chrono::Utc::now();
    }
}

impl Default for ClientSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Gateway 配置
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// 监听端口
    pub port: u16,
    /// 认证 Token
    pub auth_token: String,
    /// 是否启用
    pub enabled: bool,
    /// 是否自动启动
    pub auto_start: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: 0, // 0 表示随机端口
            auth_token: Uuid::new_v4().to_string(),
            enabled: false,
            auto_start: false,
        }
    }
}

/// Gateway 共享状态
///
/// 使用 Arc<RwLock<>> 进行线程安全的状态共享
pub struct GatewayState {
    /// 当前配置
    pub config: GatewayConfig,
    /// 活跃的客户端会话映射 (session_id -> ClientSession)
    pub sessions: HashMap<String, ClientSession>,
    /// 关闭信号发送器
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl GatewayState {
    /// 创建新的 Gateway 状态
    pub fn new(config: GatewayConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            config,
            sessions: HashMap::new(),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// 创建默认配置的 Gateway 状态
    pub fn with_defaults() -> Self {
        Self::new(GatewayConfig::default())
    }

    /// 注册新的客户端会话
    pub fn register_session(&mut self) -> ClientSession {
        let session = ClientSession::new();
        self.sessions.insert(session.session_id.clone(), session.clone());
        session
    }

    /// 移除客户端会话
    pub fn remove_session(&mut self, session_id: &str) -> Option<ClientSession> {
        self.sessions.remove(session_id)
    }

    /// 获取客户端会话
    pub fn get_session(&self, session_id: &str) -> Option<&ClientSession> {
        self.sessions.get(session_id)
    }

    /// 获取可变的客户端会话
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut ClientSession> {
        self.sessions.get_mut(session_id)
    }

    /// 验证 Token
    pub fn validate_token(&self, token: &str) -> bool {
        self.config.auth_token == token
    }

    /// 获取关闭信号接收器
    pub fn subscribe_shutdown(&self) -> Option<broadcast::Receiver<()>> {
        self.shutdown_tx.as_ref().map(|tx| tx.subscribe())
    }

    /// 发送关闭信号
    pub fn send_shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }

    /// 获取活跃连接数
    pub fn active_connections(&self) -> usize {
        self.sessions.len()
    }
}

/// 线程安全的 Gateway 状态包装
pub type SharedGatewayState = Arc<RwLock<GatewayState>>;

/// Gateway 统计信息
pub struct GatewayStats {
    /// 累计连接数
    pub total_connections: AtomicU64,
    /// 累计请求数
    pub total_requests: AtomicU64,
}

impl GatewayStats {
    /// 创建新的统计实例
    pub fn new() -> Self {
        Self {
            total_connections: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
        }
    }

    /// 增加连接计数
    pub fn increment_connections(&self) {
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// 增加请求计数
    pub fn increment_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取累计连接数
    pub fn get_total_connections(&self) -> u64 {
        self.total_connections.load(Ordering::Relaxed)
    }

    /// 获取累计请求数
    pub fn get_total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }
}

impl Default for GatewayStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 线程安全的统计信息包装
pub type SharedGatewayStats = Arc<GatewayStats>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_session_creation() {
        let session = ClientSession::new();
        assert!(!session.session_id.is_empty());
        assert!(session.message_endpoint.contains(&session.session_id));
    }

    #[test]
    fn test_gateway_config_default() {
        let config = GatewayConfig::default();
        assert_eq!(config.port, 0);
        assert!(!config.auth_token.is_empty());
        assert!(!config.enabled);
        assert!(!config.auto_start);
    }

    #[test]
    fn test_gateway_state_session_management() {
        let config = GatewayConfig {
            port: 8080,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let mut state = GatewayState::new(config);

        // 注册会话
        let session = state.register_session();
        assert_eq!(state.active_connections(), 1);

        // 获取会话
        let retrieved = state.get_session(&session.session_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session.session_id);

        // 移除会话
        let removed = state.remove_session(&session.session_id);
        assert!(removed.is_some());
        assert_eq!(state.active_connections(), 0);
    }

    #[test]
    fn test_token_validation() {
        let config = GatewayConfig {
            port: 8080,
            auth_token: "secret-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let state = GatewayState::new(config);

        assert!(state.validate_token("secret-token"));
        assert!(!state.validate_token("wrong-token"));
    }

    #[test]
    fn test_gateway_stats() {
        let stats = GatewayStats::new();

        stats.increment_connections();
        stats.increment_connections();
        stats.increment_requests();

        assert_eq!(stats.get_total_connections(), 2);
        assert_eq!(stats.get_total_requests(), 1);
    }
}
