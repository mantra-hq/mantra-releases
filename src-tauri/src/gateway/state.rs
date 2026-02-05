//! Gateway 共享状态定义
//!
//! Story 11.1: SSE Server 核心
//! Story 11.5: 上下文路由 - Task 3 (会话状态扩展)
//! Story 11.26: MCP Roots 机制 - Task 1 (roots capability 支持)
//!
//! 使用 Arc<RwLock<>> 进行共享状态管理

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// 会话项目上下文信息
///
/// Story 11.5: 上下文路由 - Task 3.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProjectContext {
    /// 项目 ID
    pub project_id: String,
    /// 项目名称
    pub project_name: String,
    /// 匹配的路径
    pub matched_path: PathBuf,
    /// 是否为手动覆盖
    pub is_manual_override: bool,
}

/// SSE 客户端会话信息
///
/// Story 11.26: 添加 roots capability 支持
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
    /// 解析的工作目录 (Story 11.5)
    pub work_dir: Option<PathBuf>,
    /// 项目上下文（自动路由或手动覆盖）(Story 11.5)
    pub project_context: Option<SessionProjectContext>,
    /// Client 是否支持 roots capability (Story 11.26)
    pub supports_roots: bool,
    /// Client 是否支持 roots.listChanged 通知 (Story 11.26)
    pub roots_list_changed: bool,
    /// 待响应的 roots/list 请求 ID (Story 11.26)
    pub pending_roots_request_id: Option<String>,
    /// 已解析的 roots 路径列表 (Story 11.26)
    pub roots_paths: Vec<PathBuf>,
    /// roots 请求是否超时 (Story 11.26)
    pub roots_request_timed_out: bool,
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
            work_dir: None,
            project_context: None,
            supports_roots: false,
            roots_list_changed: false,
            pending_roots_request_id: None,
            roots_paths: Vec::new(),
            roots_request_timed_out: false,
        }
    }

    /// 更新最后活跃时间
    pub fn touch(&mut self) {
        self.last_active = chrono::Utc::now();
    }

    /// 设置工作目录
    ///
    /// Story 11.5: 上下文路由 - Task 3.1
    pub fn set_work_dir(&mut self, work_dir: PathBuf) {
        self.work_dir = Some(work_dir);
    }

    /// 设置 roots capability 信息
    ///
    /// Story 11.26: MCP Roots 机制 - Task 1.2
    pub fn set_roots_capability(&mut self, supports_roots: bool, list_changed: bool) {
        self.supports_roots = supports_roots;
        self.roots_list_changed = list_changed;
    }

    /// 设置已解析的 roots 路径列表
    ///
    /// Story 11.26: MCP Roots 机制 - Task 1.3
    pub fn set_roots_paths(&mut self, paths: Vec<PathBuf>) {
        self.roots_paths = paths;
    }

    /// 设置项目上下文（自动路由）
    ///
    /// Story 11.5: 上下文路由 - Task 3.3
    pub fn set_auto_context(
        &mut self,
        project_id: String,
        project_name: String,
        matched_path: PathBuf,
    ) {
        self.project_context = Some(SessionProjectContext {
            project_id,
            project_name,
            matched_path,
            is_manual_override: false,
        });
    }

    /// 设置手动覆盖上下文
    ///
    /// Story 11.5: 上下文路由 - Task 3.2
    pub fn set_manual_override(&mut self, project_id: String, project_name: String) {
        self.project_context = Some(SessionProjectContext {
            project_id,
            project_name,
            matched_path: PathBuf::new(),
            is_manual_override: true,
        });
    }

    /// 清除手动覆盖
    ///
    /// 清除后会回退到自动路由的上下文
    pub fn clear_manual_override(&mut self) {
        if let Some(ctx) = &self.project_context {
            if ctx.is_manual_override {
                self.project_context = None;
            }
        }
    }

    /// 获取有效的项目上下文
    ///
    /// Story 11.5: 上下文路由 - Task 3.4
    /// 返回当前生效的项目上下文（手动覆盖优先）
    pub fn get_effective_project(&self) -> Option<&SessionProjectContext> {
        self.project_context.as_ref()
    }

    /// 检查是否有手动覆盖
    pub fn has_manual_override(&self) -> bool {
        self.project_context
            .as_ref()
            .map(|ctx| ctx.is_manual_override)
            .unwrap_or(false)
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

    // ===== Story 11.5: 上下文路由测试 =====

    #[test]
    fn test_client_session_new_fields() {
        let session = ClientSession::new();
        assert!(session.work_dir.is_none());
        assert!(session.project_context.is_none());
    }

    #[test]
    fn test_set_work_dir() {
        let mut session = ClientSession::new();
        session.set_work_dir(PathBuf::from("/home/user/projects"));

        assert!(session.work_dir.is_some());
        assert_eq!(
            session.work_dir.unwrap(),
            PathBuf::from("/home/user/projects")
        );
    }

    #[test]
    fn test_set_auto_context() {
        let mut session = ClientSession::new();
        session.set_auto_context(
            "proj-123".to_string(),
            "My Project".to_string(),
            PathBuf::from("/home/user/projects/myproject"),
        );

        let ctx = session.get_effective_project().unwrap();
        assert_eq!(ctx.project_id, "proj-123");
        assert_eq!(ctx.project_name, "My Project");
        assert_eq!(
            ctx.matched_path,
            PathBuf::from("/home/user/projects/myproject")
        );
        assert!(!ctx.is_manual_override);
        assert!(!session.has_manual_override());
    }

    #[test]
    fn test_set_manual_override() {
        let mut session = ClientSession::new();
        session.set_manual_override("proj-456".to_string(), "Override Project".to_string());

        let ctx = session.get_effective_project().unwrap();
        assert_eq!(ctx.project_id, "proj-456");
        assert_eq!(ctx.project_name, "Override Project");
        assert!(ctx.is_manual_override);
        assert!(session.has_manual_override());
    }

    #[test]
    fn test_manual_override_replaces_auto_context() {
        let mut session = ClientSession::new();

        // 先设置自动上下文
        session.set_auto_context(
            "auto-proj".to_string(),
            "Auto Project".to_string(),
            PathBuf::from("/auto/path"),
        );
        assert_eq!(
            session.get_effective_project().unwrap().project_id,
            "auto-proj"
        );

        // 设置手动覆盖
        session.set_manual_override("manual-proj".to_string(), "Manual Project".to_string());

        // 应该返回手动覆盖的上下文
        let ctx = session.get_effective_project().unwrap();
        assert_eq!(ctx.project_id, "manual-proj");
        assert!(ctx.is_manual_override);
    }

    #[test]
    fn test_clear_manual_override() {
        let mut session = ClientSession::new();

        // 设置手动覆盖
        session.set_manual_override("manual-proj".to_string(), "Manual Project".to_string());
        assert!(session.has_manual_override());

        // 清除手动覆盖
        session.clear_manual_override();
        assert!(!session.has_manual_override());
        assert!(session.project_context.is_none());
    }

    #[test]
    fn test_clear_manual_override_preserves_auto_context() {
        let mut session = ClientSession::new();

        // 设置自动上下文
        session.set_auto_context(
            "auto-proj".to_string(),
            "Auto Project".to_string(),
            PathBuf::from("/auto/path"),
        );

        // 清除手动覆盖不应该影响自动上下文
        session.clear_manual_override();

        // 自动上下文应该保持不变
        let ctx = session.get_effective_project().unwrap();
        assert_eq!(ctx.project_id, "auto-proj");
        assert!(!ctx.is_manual_override);
    }

    #[test]
    fn test_get_effective_project_none() {
        let session = ClientSession::new();
        assert!(session.get_effective_project().is_none());
    }

    #[test]
    fn test_session_project_context_serialization() {
        let ctx = SessionProjectContext {
            project_id: "proj-123".to_string(),
            project_name: "Test Project".to_string(),
            matched_path: PathBuf::from("/test/path"),
            is_manual_override: false,
        };

        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("proj-123"));
        assert!(json.contains("Test Project"));

        let deserialized: SessionProjectContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.project_id, ctx.project_id);
        assert_eq!(deserialized.project_name, ctx.project_name);
    }
}
