//! Gateway Tauri 命令
//!
//! Story 11.1: SSE Server 核心 - Task 7
//! Story 11.5: 上下文路由 - Task 8 (Tauri IPC 命令支持)
//!
//! 提供 Gateway Server 的 Tauri IPC 命令

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::AppState;
use crate::error::AppError;
use crate::gateway::{GatewayServerManager, SessionProjectContext};
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
#[tauri::command]
pub async fn start_gateway(
    gateway_state: State<'_, GatewayServerState>,
    app_state: State<'_, AppState>,
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

    // 启动 Server
    manager
        .start()
        .await
        .map_err(|e| AppError::internal(e))?;

    // 启动后更新数据库中的端口
    {
        let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
        db.set_gateway_port(Some(manager.current_port() as i32))
            .map_err(|e| AppError::internal(e.to_string()))?;
        db.set_gateway_enabled(true)
            .map_err(|e| AppError::internal(e.to_string()))?;
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
#[tauri::command]
pub async fn restart_gateway(
    gateway_state: State<'_, GatewayServerState>,
    app_state: State<'_, AppState>,
    new_port: Option<u16>,
) -> Result<GatewayStatusResponse, AppError> {
    let mut manager = gateway_state.manager.lock().await;

    manager
        .restart(new_port)
        .await
        .map_err(|e| AppError::internal(e))?;

    // 更新数据库中的端口
    {
        let db = app_state.db.lock().map_err(|_| AppError::LockError)?;
        db.set_gateway_port(Some(manager.current_port() as i32))
            .map_err(|e| AppError::internal(e.to_string()))?;
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
