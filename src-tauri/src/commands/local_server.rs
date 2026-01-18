//! Local API Server commands (Story 3.11)
//!
//! Tauri IPC commands for managing the local HTTP API server.

use tauri::{AppHandle, Manager};

use crate::error::AppError;
use crate::local_server::{LocalServerConfig, DEFAULT_PORT};
use crate::LocalServerState;

/// 获取本地 API Server 状态
///
/// # Returns
/// JSON 对象包含:
/// - `running`: 是否正在运行
/// - `port`: 当前端口
#[tauri::command]
pub async fn get_local_server_status(
    state: tauri::State<'_, LocalServerState>,
) -> Result<LocalServerStatus, AppError> {
    let manager = state.manager.lock().await;
    Ok(LocalServerStatus {
        running: manager.is_running(),
        port: manager.current_port(),
    })
}

/// 获取本地 API Server 配置
///
/// # Returns
/// JSON 对象包含:
/// - `local_api_port`: 配置的端口
#[tauri::command]
pub async fn get_local_server_config(
    app_handle: AppHandle,
) -> Result<LocalServerConfigResponse, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("Failed to get app data dir: {}", e)))?;

    let config = LocalServerConfig::load(&app_data_dir);
    Ok(LocalServerConfigResponse {
        local_api_port: config.local_api_port,
        default_port: DEFAULT_PORT,
    })
}

/// 更新本地 API Server 端口
///
/// # Arguments
/// * `port` - 新端口号 (1024-65535)
///
/// # Returns
/// 更新后的状态
#[tauri::command]
pub async fn update_local_server_port(
    port: u16,
    app_handle: AppHandle,
    state: tauri::State<'_, LocalServerState>,
) -> Result<LocalServerStatus, AppError> {
    // 验证端口
    LocalServerConfig::validate_port(port)
        .map_err(|e| AppError::Validation(e))?;

    let _app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("Failed to get app data dir: {}", e)))?;

    // 检查端口是否可用（排除当前占用的端口）
    let mut manager = state.manager.lock().await;
    let current_port = manager.current_port();
    
    if port != current_port {
        // 检查新端口是否可用
        let available = crate::local_server::LocalServer::check_port_available(port).await;
        if !available {
            return Err(AppError::Validation(format!(
                "Port {} is already in use",
                port
            )));
        }
    }

    // 重启 Server 使用新端口
    manager.restart(Some(port)).await
        .map_err(|e| AppError::internal(format!("Failed to restart server: {}", e)))?;

    Ok(LocalServerStatus {
        running: manager.is_running(),
        port: manager.current_port(),
    })
}

/// 启动本地 API Server
#[tauri::command]
pub async fn start_local_server(
    state: tauri::State<'_, LocalServerState>,
) -> Result<LocalServerStatus, AppError> {
    let mut manager = state.manager.lock().await;
    manager.start().await
        .map_err(|e| AppError::internal(format!("Failed to start server: {}", e)))?;

    Ok(LocalServerStatus {
        running: manager.is_running(),
        port: manager.current_port(),
    })
}

/// 停止本地 API Server
#[tauri::command]
pub async fn stop_local_server(
    state: tauri::State<'_, LocalServerState>,
) -> Result<LocalServerStatus, AppError> {
    let mut manager = state.manager.lock().await;
    manager.stop();

    Ok(LocalServerStatus {
        running: manager.is_running(),
        port: manager.current_port(),
    })
}

/// Server 状态响应
#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalServerStatus {
    /// 是否正在运行
    pub running: bool,
    /// 当前端口
    pub port: u16,
}

/// Server 配置响应
#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalServerConfigResponse {
    /// 配置的端口
    pub local_api_port: u16,
    /// 默认端口
    pub default_port: u16,
}
