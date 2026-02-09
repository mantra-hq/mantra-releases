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
    /// 连接数
    pub connection_count: u32,
    /// MCP Hub 是否运行中
    pub hub_running: bool,
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
        connection_count: manager.connection_count,
        hub_running: manager.hub_running,
        tooltip: manager.get_tooltip(),
    })
}

/// 更新托盘 MCP Hub 状态
#[tauri::command]
pub async fn update_tray_gateway_status(
    tray_state: State<'_, TrayState>,
    running: bool,
    connection_count: u32,
) -> Result<TrayStatusResponse, AppError> {
    let mut manager = tray_state.manager.write().await;

    manager.set_hub_running(running);
    manager.set_connection_count(connection_count);

    Ok(TrayStatusResponse {
        icon_state: manager.icon_state.into(),
        connection_count: manager.connection_count,
        hub_running: manager.hub_running,
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
        connection_count: manager.connection_count,
        hub_running: manager.hub_running,
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
