//! Gateway Tauri 命令
//!
//! Story 11.1: SSE Server 核心 - Task 7
//!
//! 提供 Gateway Server 的 Tauri IPC 命令

use serde::Serialize;
use tauri::State;

use crate::commands::AppState;
use crate::error::AppError;
use crate::gateway::GatewayServerManager;
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

    let (active_connections, total_connections, total_requests) = if let Some(state) = manager.state() {
        let state_guard = state.read().await;
        (
            state_guard.active_connections(),
            0u64, // Stats would need to be accessed from GatewayAppState
            0u64,
        )
    } else {
        (0, 0, 0)
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

/// 内部函数：获取状态
fn get_gateway_status_internal(
    manager: &GatewayServerManager,
) -> Result<GatewayStatusResponse, AppError> {
    Ok(GatewayStatusResponse {
        running: manager.is_running(),
        port: if manager.is_running() {
            Some(manager.current_port())
        } else {
            None
        },
        auth_token: manager.auth_token().to_string(),
        active_connections: 0, // Simplified for now
        total_connections: 0,
        total_requests: 0,
    })
}
