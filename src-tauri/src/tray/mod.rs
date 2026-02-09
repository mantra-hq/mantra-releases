// Story 11.7: 系统托盘集成
// 托盘模块入口

pub mod error;
pub mod handlers;
pub mod icons;
pub mod menu;

use std::sync::Arc;
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::RwLock;

pub use error::TrayError;
pub use icons::TrayIconState;
pub use menu::MenuIds;

/// 托盘管理器
pub struct TrayManager {
    /// 当前图标状态
    pub icon_state: TrayIconState,
    /// 连接数
    pub connection_count: u32,
    /// MCP Hub 是否运行中
    pub hub_running: bool,
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TrayManager {
    /// 创建托盘管理器
    pub fn new() -> Self {
        Self {
            icon_state: TrayIconState::Normal,
            connection_count: 0,
            hub_running: false,
        }
    }

    /// 更新图标状态
    pub fn set_icon_state(&mut self, state: TrayIconState) {
        self.icon_state = state;
    }

    /// 更新连接数
    pub fn set_connection_count(&mut self, count: u32) {
        self.connection_count = count;
        if count > 0 && self.hub_running {
            self.icon_state = TrayIconState::Active;
        } else if self.hub_running {
            self.icon_state = TrayIconState::Active;
        } else {
            self.icon_state = TrayIconState::Normal;
        }
    }

    /// 设置 MCP Hub 运行状态
    pub fn set_hub_running(&mut self, running: bool) {
        self.hub_running = running;
        if running {
            self.icon_state = TrayIconState::Active;
        } else {
            self.icon_state = TrayIconState::Normal;
        }
    }

    /// 设置错误状态
    pub fn set_error(&mut self) {
        self.icon_state = TrayIconState::Error;
    }

    /// 获取 tooltip 文本
    pub fn get_tooltip(&self) -> String {
        match self.icon_state {
            TrayIconState::Active => {
                format!("Mantra MCP Hub - {} 连接", self.connection_count)
            }
            TrayIconState::Error => "Mantra MCP Hub - 错误".to_string(),
            _ => "Mantra MCP Hub".to_string(),
        }
    }
}

/// 托盘状态（用于 Tauri State 管理）
pub struct TrayState {
    pub manager: Arc<RwLock<TrayManager>>,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            manager: Arc::new(RwLock::new(TrayManager::new())),
        }
    }
}

/// 初始化系统托盘
pub fn init_tray<R: Runtime>(app: &AppHandle<R>) -> Result<TrayIcon<R>, TrayError> {
    let menu = menu::build_tray_menu(app, false, 0)?;
    let icon = icons::load_icon(TrayIconState::Normal)?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Mantra MCP Hub")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            handlers::handle_tray_icon_event(tray, event);
        })
        .on_menu_event(|app, event| {
            handlers::handle_menu_event(app, event);
        })
        .build(app)?;

    Ok(tray)
}

/// 刷新托盘状态和菜单
pub async fn refresh_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), TrayError> {
    let tray_state: tauri::State<'_, TrayState> = app.state();
    let manager = tray_state.manager.read().await;

    if let Some(tray) = app.tray_by_id("main") {
        // 更新 tooltip
        let tooltip = manager.get_tooltip();
        tray.set_tooltip(Some(&tooltip))?;

        // 更新图标
        let icon = icons::load_icon(manager.icon_state)?;
        tray.set_icon(Some(icon))?;

        // 更新菜单
        let menu = menu::build_tray_menu(
            app,
            manager.hub_running,
            manager.connection_count,
        )?;
        tray.set_menu(Some(menu))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_manager_default() {
        let manager = TrayManager::new();
        assert_eq!(manager.icon_state, TrayIconState::Normal);
        assert_eq!(manager.connection_count, 0);
        assert!(!manager.hub_running);
    }

    #[test]
    fn test_tray_manager_set_hub_running() {
        let mut manager = TrayManager::new();

        manager.set_hub_running(true);
        assert!(manager.hub_running);
        assert_eq!(manager.icon_state, TrayIconState::Active);

        manager.set_hub_running(false);
        assert!(!manager.hub_running);
        assert_eq!(manager.icon_state, TrayIconState::Normal);
    }

    #[test]
    fn test_tray_manager_set_connection_count() {
        let mut manager = TrayManager::new();
        manager.set_hub_running(true);

        manager.set_connection_count(5);
        assert_eq!(manager.connection_count, 5);
        assert_eq!(manager.icon_state, TrayIconState::Active);
    }

    #[test]
    fn test_tray_manager_set_error() {
        let mut manager = TrayManager::new();

        manager.set_error();
        assert_eq!(manager.icon_state, TrayIconState::Error);
    }

    #[test]
    fn test_tray_manager_tooltip() {
        let mut manager = TrayManager::new();

        // 默认状态
        assert_eq!(manager.get_tooltip(), "Mantra MCP Hub");

        // Hub 运行中
        manager.set_hub_running(true);
        manager.set_connection_count(2);
        assert_eq!(manager.get_tooltip(), "Mantra MCP Hub - 2 连接");

        // 错误状态
        manager.set_error();
        assert_eq!(manager.get_tooltip(), "Mantra MCP Hub - 错误");
    }

    #[test]
    fn test_tray_state_default() {
        let state = TrayState::default();
        assert!(Arc::strong_count(&state.manager) == 1);
    }
}
