//! Gateway Tauri 命令
//!
//! Story 11.1: SSE Server 核心 - Task 7
//! Story 11.5: 上下文路由 - Task 8 (Tauri IPC 命令支持)
//! Story 11.12: Remote MCP OAuth Support - Task 6 (OAuth IPC 命令)
//! Story 11.17: MCP 协议聚合器 - Task 8 (缓存刷新 IPC 命令)
//! Story 11.27: MCP Roots LPM 集成 - Task 1.3 (LPM 查询服务)
//!
//! 提供 Gateway Server 的 Tauri IPC 命令

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::commands::{AppState, McpState};
use crate::error::AppError;
use crate::gateway::{
    GatewayServerManager, LpmQueryClient, LpmQueryService, McpAggregator,
    SessionProjectContext, StoragePolicyResolver, WarmupResult,
};
use crate::services::mcp_config::sync_active_takeovers;
use crate::services::oauth::{
    CallbackResult, InMemoryTokenStore, OAuthConfig, OAuthManager, OAuthServiceStatus,
};
use crate::storage::{GatewayConfigRecord, GatewayConfigUpdate};
use crate::GatewayServerState;

/// Gateway 状态响应
#[derive(Debug, Clone, Serialize)]
pub struct GatewayStatusResponse {
    /// 是否正在运行
    pub running: bool,
    /// 当前端口
    pub port: Option<u16>,
    /// 认证 Token
    pub auth_token: String,
    /// 活跃连接数
    pub active_connections: usize,
    /// 累计连接数
    pub total_connections: u64,
    /// 累计请求数
    pub total_requests: u64,
}

/// 获取 Gateway 状态
#[tauri::command]
pub async fn get_gateway_status(
    gateway_state: State<'_, GatewayServerState>,
) -> Result<GatewayStatusResponse, AppError> {
    let manager = gateway_state.manager.lock().await;

    let (active_connections, total_connections, total_requests) = match (manager.state(), manager.stats()) {
        (Some(state), Some(stats)) => {
            let state_guard = state.read().await;
            (
                state_guard.active_connections(),
                stats.get_total_connections(),
                stats.get_total_requests(),
            )
        }
        _ => (0, 0, 0),
    };

    Ok(GatewayStatusResponse {
        running: manager.is_running(),
        port: if manager.is_running() {
            Some(manager.current_port())
        } else {
            None
        },
        auth_token: manager.auth_token().to_string(),
        active_connections,
        total_connections,
        total_requests,
    })
}

/// 获取 Gateway 配置（从数据库）
#[tauri::command]
pub async fn get_gateway_config(
    app_state: State<'_, AppState>,
) -> Result<GatewayConfigRecord, AppError> {
    let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_gateway_config().map_err(|e| AppError::internal(e.to_string()))
}

/// 更新 Gateway 配置（到数据库）
#[tauri::command]
pub async fn update_gateway_config(
    app_state: State<'_, AppState>,
    update: GatewayConfigUpdate,
) -> Result<GatewayConfigRecord, AppError> {
    let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
    db.update_gateway_config(&update).map_err(|e| AppError::internal(e.to_string()))
}

/// 启动 Gateway Server
///
/// Story 11.17: 启动时创建 McpAggregator 并执行 warmup
/// Story 11.27: 启动时创建 LPM 查询服务
#[tauri::command]
pub async fn start_gateway(
    app_handle: tauri::AppHandle,
    gateway_state: State<'_, GatewayServerState>,
    app_state: State<'_, AppState>,
    mcp_state: State<'_, McpState>,
) -> Result<GatewayStatusResponse, AppError> {
    let mut manager = gateway_state.manager.lock().await;

    if manager.is_running() {
        return get_gateway_status_internal(&manager);
    }

    // 从数据库获取配置（确保存在）
    {
        let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
        let _config = db.get_gateway_config().map_err(|e| AppError::internal(e.to_string()))?;
    };

    // Story 11.17: 创建 McpAggregator 并执行 warmup
    let aggregator = {
        // 1. 获取所有启用的 MCP 服务
        let services = {
            let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
            db.list_mcp_services()
                .map_err(|e| AppError::internal(e.to_string()))?
                .into_iter()
                .filter(|s| s.enabled)
                .collect::<Vec<_>>()
        };

        eprintln!("[Gateway] Creating aggregator with {} enabled services", services.len());

        // 2. 创建 aggregator
        let aggregator = Arc::new(McpAggregator::new(services));

        // 3. 获取环境变量用于 warmup
        let env_vars = {
            let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
            let env_manager = &mcp_state.env_manager;
            db.list_env_variables()
                .map_err(|e| AppError::internal(e.to_string()))?
                .into_iter()
                .filter_map(|var| {
                    db.get_env_variable(env_manager, &var.name)
                        .ok()
                        .flatten()
                        .map(|value| (var.name, value))
                })
                .collect::<std::collections::HashMap<String, String>>()
        };

        // 4. 执行 warmup（在设置到 manager 之前释放锁）
        let env_resolver = move |var_name: &str| -> Option<String> {
            env_vars.get(var_name).cloned()
        };

        let warmup_result = aggregator.warmup(env_resolver).await;
        eprintln!(
            "[Gateway] Warmup completed: {}/{} services succeeded, {} failed",
            warmup_result.succeeded, warmup_result.total, warmup_result.failed
        );
        for (name, err) in &warmup_result.errors {
            eprintln!("[Gateway] Service '{}' warmup error: {}", name, err);
        }

        aggregator
    };

    // 5. 设置 aggregator 到 manager
    manager.set_aggregator(aggregator);

    // Story 11.9 Phase 2: 创建并注入 PolicyResolver
    {
        let policy_resolver = Arc::new(StoragePolicyResolver::new(mcp_state.db.clone()));
        manager.set_policy_resolver(policy_resolver);
    }

    // Story 11.27: 创建并注入 LPM 查询服务
    {
        // 1. 创建 LPM 查询客户端和服务
        let (lpm_client, lpm_query_rx) = LpmQueryClient::new(64);
        let lpm_service = LpmQueryService::new(lpm_query_rx);

        // 2. 注入 LPM 客户端到 manager
        manager.set_lpm_client(Arc::new(lpm_client));

        // 3. 获取数据库路径用于创建新连接
        // 注意：由于 rusqlite 的 Connection 不是 Send/Sync，
        // LPM 服务需要自己的数据库连接，在 spawn_blocking 线程中创建和使用
        let app_data_dir = app_handle.path().app_data_dir()
            .map_err(|e| AppError::internal(format!("Failed to get app data dir: {}", e)))?;
        let db_path = app_data_dir.join(crate::DATABASE_FILENAME);

        // 4. 启动 LPM 查询服务（后台任务）
        eprintln!("[Gateway] Story 11.27: Starting LPM query service");
        tokio::spawn(async move {
            lpm_service.run_with_db_path(db_path).await;
        });
    }

    // 启动 Server
    manager
        .start()
        .await
        .map_err(|e| AppError::internal(e))?;

    // 启动后更新数据库中的端口，并同步接管配置
    {
        let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
        db.set_gateway_port(Some(manager.current_port() as i32))
            .map_err(|e| AppError::internal(e.to_string()))?;
        db.set_gateway_enabled(true)
            .map_err(|e| AppError::internal(e.to_string()))?;

        // 同步所有活跃的接管配置 (Gateway URL 和 Token 可能已变化)
        let gateway_url = format!("http://127.0.0.1:{}/mcp", manager.current_port());
        let gateway_token = manager.auth_token();
        if let Err(e) = sync_active_takeovers(&db, &gateway_url, &gateway_token) {
            // 同步失败不阻断启动，只记录警告
            eprintln!("[Gateway] Failed to sync active takeovers: {:?}", e);
        }
    }

    get_gateway_status_internal(&manager)
}

/// 停止 Gateway Server
#[tauri::command]
pub async fn stop_gateway(
    gateway_state: State<'_, GatewayServerState>,
    app_state: State<'_, AppState>,
) -> Result<GatewayStatusResponse, AppError> {
    let mut manager = gateway_state.manager.lock().await;

    if !manager.is_running() {
        return get_gateway_status_internal(&manager);
    }

    manager.stop();

    // 更新数据库状态
    {
        let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
        db.set_gateway_enabled(false)
            .map_err(|e| AppError::internal(e.to_string()))?;
    }

    get_gateway_status_internal(&manager)
}

/// 重启 Gateway Server
///
/// Story 11.17: 重启时重新初始化 McpAggregator
#[tauri::command]
pub async fn restart_gateway(
    gateway_state: State<'_, GatewayServerState>,
    app_state: State<'_, AppState>,
    mcp_state: State<'_, McpState>,
    new_port: Option<u16>,
) -> Result<GatewayStatusResponse, AppError> {
    let mut manager = gateway_state.manager.lock().await;

    // Story 11.17: 重新创建 McpAggregator 并执行 warmup
    let aggregator = {
        let services = {
            let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
            db.list_mcp_services()
                .map_err(|e| AppError::internal(e.to_string()))?
                .into_iter()
                .filter(|s| s.enabled)
                .collect::<Vec<_>>()
        };

        eprintln!("[Gateway] Restarting with {} enabled services", services.len());

        let aggregator = Arc::new(McpAggregator::new(services));

        let env_vars = {
            let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
            let env_manager = &mcp_state.env_manager;
            db.list_env_variables()
                .map_err(|e| AppError::internal(e.to_string()))?
                .into_iter()
                .filter_map(|var| {
                    db.get_env_variable(env_manager, &var.name)
                        .ok()
                        .flatten()
                        .map(|value| (var.name, value))
                })
                .collect::<std::collections::HashMap<String, String>>()
        };

        let env_resolver = move |var_name: &str| -> Option<String> {
            env_vars.get(var_name).cloned()
        };

        let warmup_result = aggregator.warmup(env_resolver).await;
        eprintln!(
            "[Gateway] Warmup completed: {}/{} services succeeded",
            warmup_result.succeeded, warmup_result.total
        );

        aggregator
    };

    manager.set_aggregator(aggregator);

    // Story 11.9 Phase 2: 重新创建并注入 PolicyResolver
    {
        let policy_resolver = Arc::new(StoragePolicyResolver::new(mcp_state.db.clone()));
        manager.set_policy_resolver(policy_resolver);
    }

    manager
        .restart(new_port)
        .await
        .map_err(|e| AppError::internal(e))?;

    // 更新数据库中的端口，并同步接管配置
    {
        let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
        db.set_gateway_port(Some(manager.current_port() as i32))
            .map_err(|e| AppError::internal(e.to_string()))?;

        // 同步所有活跃的接管配置 (Gateway URL 和 Token 可能已变化)
        let gateway_url = format!("http://127.0.0.1:{}/mcp", manager.current_port());
        let gateway_token = manager.auth_token();
        if let Err(e) = sync_active_takeovers(&db, &gateway_url, &gateway_token) {
            eprintln!("[Gateway] Failed to sync active takeovers: {:?}", e);
        }
    }

    get_gateway_status_internal(&manager)
}

/// 重新生成 Gateway Token
#[tauri::command]
pub async fn regenerate_gateway_token(
    app_state: State<'_, AppState>,
) -> Result<String, AppError> {
    let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
    db.regenerate_gateway_token().map_err(|e| AppError::internal(e.to_string()))
}

/// 内部函数：获取状态（同步版本，用于启动/停止后立即返回）
fn get_gateway_status_internal(
    manager: &GatewayServerManager,
) -> Result<GatewayStatusResponse, AppError> {
    let (active_connections, total_connections, total_requests) = match manager.stats() {
        Some(stats) => (
            0, // 活跃连接数需要异步获取，这里简化处理
            stats.get_total_connections(),
            stats.get_total_requests(),
        ),
        None => (0, 0, 0),
    };

    Ok(GatewayStatusResponse {
        running: manager.is_running(),
        port: if manager.is_running() {
            Some(manager.current_port())
        } else {
            None
        },
        auth_token: manager.auth_token().to_string(),
        active_connections,
        total_connections,
        total_requests,
    })
}

// ===== Story 11.5: 上下文路由 - Tauri IPC 命令 =====

/// 会话上下文响应
#[derive(Debug, Clone, Serialize)]
pub struct SessionContextResponse {
    /// 会话 ID
    pub session_id: String,
    /// 工作目录
    pub work_dir: Option<PathBuf>,
    /// 项目上下文
    pub project_context: Option<SessionProjectContext>,
    /// 是否有手动覆盖
    pub has_manual_override: bool,
}

/// 会话列表项
#[derive(Debug, Clone, Serialize)]
pub struct SessionListItem {
    /// 会话 ID
    pub session_id: String,
    /// 工作目录
    pub work_dir: Option<PathBuf>,
    /// 项目 ID（如果有）
    pub project_id: Option<String>,
    /// 项目名称（如果有）
    pub project_name: Option<String>,
    /// 是否有手动覆盖
    pub has_manual_override: bool,
    /// 连接时间
    pub connected_at: String,
    /// 最后活跃时间
    pub last_active: String,
}

/// 设置项目上下文请求
#[derive(Debug, Clone, Deserialize)]
pub struct SetProjectContextRequest {
    /// 会话 ID
    pub session_id: String,
    /// 项目 ID
    pub project_id: String,
    /// 项目名称
    pub project_name: String,
}

/// 设置会话的项目上下文（手动覆盖）
///
/// Story 11.5: 上下文路由 - Task 8.1 (AC: #2)
///
/// 用于系统托盘手动选择项目上下文
#[tauri::command]
pub async fn gateway_set_project_context(
    gateway_state: State<'_, GatewayServerState>,
    request: SetProjectContextRequest,
) -> Result<SessionContextResponse, AppError> {
    let manager = gateway_state.manager.lock().await;

    let state = manager
        .state()
        .ok_or_else(|| AppError::internal("Gateway not running"))?;

    let mut state_guard = state.write().await;
    let session = state_guard
        .get_session_mut(&request.session_id)
        .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

    // 设置手动覆盖
    session.set_manual_override(request.project_id, request.project_name);

    // 返回更新后的上下文
    Ok(SessionContextResponse {
        session_id: session.session_id.clone(),
        work_dir: session.work_dir.clone(),
        project_context: session.project_context.clone(),
        has_manual_override: session.has_manual_override(),
    })
}

/// 清除会话的手动覆盖
///
/// Story 11.5: 上下文路由 - Task 8.2 (AC: #2)
///
/// 清除后会回退到自动路由的上下文
#[tauri::command]
pub async fn gateway_clear_project_context(
    gateway_state: State<'_, GatewayServerState>,
    session_id: String,
) -> Result<SessionContextResponse, AppError> {
    let manager = gateway_state.manager.lock().await;

    let state = manager
        .state()
        .ok_or_else(|| AppError::internal("Gateway not running"))?;

    let mut state_guard = state.write().await;
    let session = state_guard
        .get_session_mut(&session_id)
        .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

    // 清除手动覆盖
    session.clear_manual_override();

    // 返回更新后的上下文
    Ok(SessionContextResponse {
        session_id: session.session_id.clone(),
        work_dir: session.work_dir.clone(),
        project_context: session.project_context.clone(),
        has_manual_override: session.has_manual_override(),
    })
}

/// 获取会话的上下文信息
///
/// Story 11.5: 上下文路由 - Task 8.3 (AC: #5)
#[tauri::command]
pub async fn gateway_get_session_context(
    gateway_state: State<'_, GatewayServerState>,
    session_id: String,
) -> Result<SessionContextResponse, AppError> {
    let manager = gateway_state.manager.lock().await;

    let state = manager
        .state()
        .ok_or_else(|| AppError::internal("Gateway not running"))?;

    let state_guard = state.read().await;
    let session = state_guard
        .get_session(&session_id)
        .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

    Ok(SessionContextResponse {
        session_id: session.session_id.clone(),
        work_dir: session.work_dir.clone(),
        project_context: session.project_context.clone(),
        has_manual_override: session.has_manual_override(),
    })
}

/// 列出所有活跃会话
///
/// Story 11.5: 上下文路由 - Task 8.4 (AC: #2)
///
/// 用于系统托盘显示所有活跃会话及其上下文
#[tauri::command]
pub async fn gateway_list_sessions(
    gateway_state: State<'_, GatewayServerState>,
) -> Result<Vec<SessionListItem>, AppError> {
    let manager = gateway_state.manager.lock().await;

    let state = match manager.state() {
        Some(s) => s,
        None => return Ok(Vec::new()), // Gateway 未运行，返回空列表
    };

    let state_guard = state.read().await;
    let sessions: Vec<SessionListItem> = state_guard
        .sessions
        .values()
        .map(|session| {
            let (project_id, project_name) = session
                .project_context
                .as_ref()
                .map(|ctx| (Some(ctx.project_id.clone()), Some(ctx.project_name.clone())))
                .unwrap_or((None, None));

            SessionListItem {
                session_id: session.session_id.clone(),
                work_dir: session.work_dir.clone(),
                project_id,
                project_name,
                has_manual_override: session.has_manual_override(),
                connected_at: session.connected_at.to_rfc3339(),
                last_active: session.last_active.to_rfc3339(),
            }
        })
        .collect();

    Ok(sessions)
}

// ===== Story 11.12: OAuth IPC 命令 =====

/// OAuth 状态
pub struct OAuthState {
    pub manager: Arc<OAuthManager>,
}

impl OAuthState {
    /// 创建新的 OAuth 状态
    pub fn new() -> Self {
        // 使用内存存储作为默认实现
        // 生产环境应使用 EncryptedTokenStore 或 KeyringTokenStore
        let token_store = Arc::new(InMemoryTokenStore::new());
        Self {
            manager: Arc::new(OAuthManager::new(token_store)),
        }
    }
}

impl Default for OAuthState {
    fn default() -> Self {
        Self::new()
    }
}

/// OAuth 启动流程请求
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthStartFlowRequest {
    /// 服务 ID
    pub service_id: String,
    /// Client ID
    pub client_id: String,
    /// Client Secret (可选)
    pub client_secret: Option<String>,
    /// Authorization URL
    pub authorization_url: String,
    /// Token URL
    pub token_url: String,
    /// Revoke URL (可选)
    pub revoke_url: Option<String>,
    /// 请求的 scopes
    pub scopes: Vec<String>,
    /// 回调端口 (0 表示动态分配)
    #[serde(default)]
    pub callback_port: u16,
}

/// OAuth 启动流程响应
#[derive(Debug, Clone, Serialize)]
pub struct OAuthStartFlowResponse {
    /// 授权 URL
    pub authorization_url: String,
    /// 回调端口
    pub callback_port: u16,
}

/// OAuth 回调处理请求
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthCallbackRequest {
    /// 服务 ID
    pub service_id: String,
    /// Authorization code
    pub code: String,
    /// State 参数
    pub state: String,
    /// 回调端口
    pub callback_port: u16,
    /// OAuth 配置 (用于 token exchange)
    pub config: OAuthStartFlowRequest,
}

/// 启动 OAuth 授权流程
///
/// Story 11.12: Remote MCP OAuth Support - Task 6.1 (AC: 1)
///
/// 1. 生成 PKCE challenge
/// 2. 启动回调服务器
/// 3. 返回授权 URL
/// 4. 打开系统浏览器
#[tauri::command]
pub async fn oauth_start_flow(
    oauth_state: State<'_, OAuthState>,
    app_handle: tauri::AppHandle,
    request: OAuthStartFlowRequest,
) -> Result<OAuthStartFlowResponse, AppError> {
    let config = OAuthConfig {
        service_id: request.service_id,
        client_id: request.client_id,
        client_secret: request.client_secret,
        authorization_url: request.authorization_url,
        token_url: request.token_url,
        revoke_url: request.revoke_url,
        scopes: request.scopes,
        callback_port: request.callback_port,
    };

    // 启动 OAuth 流程
    let (auth_url, callback_handle) = oauth_state
        .manager
        .start_flow(&config)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    let callback_port = callback_handle.port();

    // 打开系统浏览器
    app_handle
        .opener()
        .open_url(&auth_url, None::<&str>)
        .map_err(|e| AppError::internal(format!("Failed to open browser: {}", e)))?;

    // 在后台等待回调
    let manager = oauth_state.manager.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        if let Some(result) = callback_handle
            .wait_for_callback(std::time::Duration::from_secs(300))
            .await
        {
            match result {
                CallbackResult::Success { code, state } => {
                    // 处理回调
                    let _ = manager
                        .handle_callback(&config_clone, &code, &state, callback_port)
                        .await;
                }
                CallbackResult::Denied { error, description } => {
                    eprintln!("[OAuth] Authorization denied: {} - {}", error, description);
                }
            }
        }
    });

    Ok(OAuthStartFlowResponse {
        authorization_url: auth_url,
        callback_port,
    })
}

/// 获取服务的 OAuth 状态
///
/// Story 11.12: Remote MCP OAuth Support - Task 6.2 (AC: 7)
#[tauri::command]
pub async fn oauth_get_status(
    oauth_state: State<'_, OAuthState>,
    service_id: String,
) -> Result<OAuthServiceStatus, AppError> {
    oauth_state
        .manager
        .get_status(&service_id)
        .await
        .map_err(|e| AppError::internal(e.to_string()))
}

/// 断开服务的 OAuth 连接
///
/// Story 11.12: Remote MCP OAuth Support - Task 6.3 (AC: 7)
#[tauri::command]
pub async fn oauth_disconnect(
    oauth_state: State<'_, OAuthState>,
    service_id: String,
    config: OAuthStartFlowRequest,
) -> Result<(), AppError> {
    let oauth_config = OAuthConfig {
        service_id: config.service_id,
        client_id: config.client_id,
        client_secret: config.client_secret,
        authorization_url: config.authorization_url,
        token_url: config.token_url,
        revoke_url: config.revoke_url,
        scopes: config.scopes,
        callback_port: config.callback_port,
    };

    oauth_state
        .manager
        .disconnect(&oauth_config, &service_id)
        .await
        .map_err(|e| AppError::internal(e.to_string()))
}

/// 手动刷新 OAuth Token
///
/// Story 11.12: Remote MCP OAuth Support - Task 6.4 (AC: 6)
#[tauri::command]
pub async fn oauth_refresh_token(
    oauth_state: State<'_, OAuthState>,
    service_id: String,
    config: OAuthStartFlowRequest,
) -> Result<OAuthServiceStatus, AppError> {
    let oauth_config = OAuthConfig {
        service_id: config.service_id.clone(),
        client_id: config.client_id,
        client_secret: config.client_secret,
        authorization_url: config.authorization_url,
        token_url: config.token_url,
        revoke_url: config.revoke_url,
        scopes: config.scopes,
        callback_port: config.callback_port,
    };

    oauth_state
        .manager
        .refresh_token(&oauth_config, &service_id)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    // 返回更新后的状态
    oauth_state
        .manager
        .get_status(&service_id)
        .await
        .map_err(|e| AppError::internal(e.to_string()))
}

// ===== Story 11.17: MCP 协议聚合器 - 缓存刷新 IPC 命令 =====

/// 刷新响应
#[derive(Debug, Clone, Serialize)]
pub struct RefreshResponse {
    /// 总服务数
    pub total: usize,
    /// 成功数
    pub succeeded: usize,
    /// 失败数
    pub failed: usize,
    /// 错误列表
    pub errors: Vec<RefreshError>,
}

/// 刷新错误
#[derive(Debug, Clone, Serialize)]
pub struct RefreshError {
    /// 服务名称
    pub service_name: String,
    /// 错误信息
    pub error: String,
}

impl From<WarmupResult> for RefreshResponse {
    fn from(result: WarmupResult) -> Self {
        Self {
            total: result.total,
            succeeded: result.succeeded,
            failed: result.failed,
            errors: result
                .errors
                .into_iter()
                .map(|(name, err)| RefreshError {
                    service_name: name,
                    error: err,
                })
                .collect(),
        }
    }
}

/// 刷新单个 MCP 服务的缓存
///
/// Story 11.17: MCP 协议聚合器 - Task 8.1 (AC: #8)
///
/// 当服务配置变更时调用此命令刷新服务的工具/资源/提示列表
#[tauri::command]
pub async fn gateway_refresh_service(
    gateway_state: State<'_, GatewayServerState>,
    mcp_state: State<'_, McpState>,
    service_id: String,
) -> Result<(), AppError> {
    let manager = gateway_state.manager.lock().await;

    let aggregator = manager
        .aggregator()
        .ok_or_else(|| AppError::internal("Gateway aggregator not initialized"))?
        .clone();

    // 在 await 之前释放 manager 锁
    drop(manager);

    // 预先获取可能需要的环境变量值
    // 由于闭包需要跨 await 边界，我们预先收集所有可能的环境变量
    let env_vars = {
        let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
        let env_manager = &mcp_state.env_manager;
        db.list_env_variables()
            .map_err(|e| AppError::internal(e.to_string()))?
            .into_iter()
            .filter_map(|var| {
                db.get_env_variable(env_manager, &var.name)
                    .ok()
                    .flatten()
                    .map(|value| (var.name, value))
            })
            .collect::<std::collections::HashMap<String, String>>()
    };

    // 使用预先收集的环境变量创建解析器
    let env_resolver = move |var_name: &str| -> Option<String> {
        env_vars.get(var_name).cloned()
    };

    aggregator
        .refresh_service(&service_id, env_resolver)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(())
}

/// 刷新所有 MCP 服务的缓存
///
/// Story 11.17: MCP 协议聚合器 - Task 8.3 (AC: #8)
///
/// 重新获取所有启用服务的工具/资源/提示列表
#[tauri::command]
pub async fn gateway_refresh_all(
    gateway_state: State<'_, GatewayServerState>,
    mcp_state: State<'_, McpState>,
) -> Result<RefreshResponse, AppError> {
    let manager = gateway_state.manager.lock().await;

    let aggregator = manager
        .aggregator()
        .ok_or_else(|| AppError::internal("Gateway aggregator not initialized"))?
        .clone();

    // 在 await 之前释放 manager 锁
    drop(manager);

    // 预先获取可能需要的环境变量值
    let env_vars = {
        let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
        let env_manager = &mcp_state.env_manager;
        db.list_env_variables()
            .map_err(|e| AppError::internal(e.to_string()))?
            .into_iter()
            .filter_map(|var| {
                db.get_env_variable(env_manager, &var.name)
                    .ok()
                    .flatten()
                    .map(|value| (var.name, value))
            })
            .collect::<std::collections::HashMap<String, String>>()
    };

    // 使用预先收集的环境变量创建解析器
    let env_resolver = move |var_name: &str| -> Option<String> {
        env_vars.get(var_name).cloned()
    };

    let result = aggregator.refresh_all(env_resolver).await;

    Ok(result.into())
}
