//! 托盘 Tauri 命令
//!
//! Story 11.7: 系统托盘集成 - Task 6
//!
//! 提供托盘状态更新的 Tauri IPC 命令

use serde::Serialize;
use tauri::State;

use crate::error::AppError;
use crate::tray::{TrayIconState, TrayState};

/// 托盘状态响应
#[derive(Debug, Clone, Serialize)]
pub struct TrayStatusResponse {
    /// 图标状态
    pub icon_state: String,
    /// 当前项目
    pub current_project: Option<String>,
    /// 连接数
    pub connection_count: u32,
    /// Gateway 是否运行中
    pub gateway_running: bool,
    /// Tooltip 文本
    pub tooltip: String,
}

impl From<TrayIconState> for String {
    fn from(state: TrayIconState) -> Self {
        match state {
            TrayIconState::Normal => "normal".to_string(),
            TrayIconState::Active => "active".to_string(),
            TrayIconState::Error => "error".to_string(),
        }
    }
}

/// 获取托盘状态
#[tauri::command]
pub async fn get_tray_status(tray_state: State<'_, TrayState>) -> Result<TrayStatusResponse, AppError> {
    let manager = tray_state.manager.read().await;

    Ok(TrayStatusResponse {
        icon_state: manager.icon_state.into(),
        current_project: manager.current_project.clone(),
        connection_count: manager.connection_count,
        gateway_running: manager.gateway_running,
        tooltip: manager.get_tooltip(),
    })
}

/// 更新托盘 Gateway 状态
#[tauri::command]
pub async fn update_tray_gateway_status(
    tray_state: State<'_, TrayState>,
    running: bool,
    connection_count: u32,
) -> Result<TrayStatusResponse, AppError> {
    let mut manager = tray_state.manager.write().await;

    manager.set_gateway_running(running);
    manager.set_connection_count(connection_count);

    Ok(TrayStatusResponse {
        icon_state: manager.icon_state.into(),
        current_project: manager.current_project.clone(),
        connection_count: manager.connection_count,
        gateway_running: manager.gateway_running,
        tooltip: manager.get_tooltip(),
    })
}

/// 更新托盘当前项目
#[tauri::command]
pub async fn update_tray_project(
    tray_state: State<'_, TrayState>,
    project_name: Option<String>,
) -> Result<TrayStatusResponse, AppError> {
    let mut manager = tray_state.manager.write().await;

    manager.set_current_project(project_name);

    Ok(TrayStatusResponse {
        icon_state: manager.icon_state.into(),
        current_project: manager.current_project.clone(),
        connection_count: manager.connection_count,
        gateway_running: manager.gateway_running,
        tooltip: manager.get_tooltip(),
    })
}

/// 设置托盘错误状态
#[tauri::command]
pub async fn set_tray_error(tray_state: State<'_, TrayState>) -> Result<TrayStatusResponse, AppError> {
    let mut manager = tray_state.manager.write().await;

    manager.set_error();

    Ok(TrayStatusResponse {
        icon_state: manager.icon_state.into(),
        current_project: manager.current_project.clone(),
        connection_count: manager.connection_count,
        gateway_running: manager.gateway_running,
        tooltip: manager.get_tooltip(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_icon_state_to_string() {
        assert_eq!(String::from(TrayIconState::Normal), "normal");
        assert_eq!(String::from(TrayIconState::Active), "active");
        assert_eq!(String::from(TrayIconState::Error), "error");
    }
}
