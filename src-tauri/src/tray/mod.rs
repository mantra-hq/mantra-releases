// Story 11.7: 系统托盘集成
// 托盘模块入口

pub mod error;
pub mod handlers;
pub mod icons;
pub mod menu;

use std::sync::Arc;
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Runtime};
use tokio::sync::RwLock;

pub use error::TrayError;
pub use icons::TrayIconState;
pub use menu::MenuIds;

/// 托盘管理器
pub struct TrayManager {
    /// 当前图标状态
    pub icon_state: TrayIconState,
    /// 当前项目上下文
    pub current_project: Option<String>,
    /// 连接数
    pub connection_count: u32,
    /// Gateway 是否运行中
    pub gateway_running: bool,
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
            current_project: None,
            connection_count: 0,
            gateway_running: false,
        }
    }

    /// 更新图标状态
    pub fn set_icon_state(&mut self, state: TrayIconState) {
        self.icon_state = state;
    }

    /// 更新连接数
    pub fn set_connection_count(&mut self, count: u32) {
        self.connection_count = count;
        // 根据连接数更新图标状态
        if count > 0 && self.gateway_running {
            self.icon_state = TrayIconState::Active;
        } else if self.gateway_running {
            self.icon_state = TrayIconState::Active;
        } else {
            self.icon_state = TrayIconState::Normal;
        }
    }

    /// 设置当前项目
    pub fn set_current_project(&mut self, project: Option<String>) {
        self.current_project = project;
    }

    /// 设置 Gateway 运行状态
    pub fn set_gateway_running(&mut self, running: bool) {
        self.gateway_running = running;
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
        match (&self.current_project, self.icon_state) {
            (Some(project), TrayIconState::Active) => {
                format!("Mantra - {} ({} 连接)", project, self.connection_count)
            }
            (None, TrayIconState::Active) => {
                format!("Mantra Gateway - {} 连接", self.connection_count)
            }
            (Some(project), TrayIconState::Error) => {
                format!("Mantra - {} (错误)", project)
            }
            (None, TrayIconState::Error) => "Mantra Gateway - 错误".to_string(),
            (Some(project), _) => {
                format!("Mantra - {}", project)
            }
            (None, _) => "Mantra Gateway".to_string(),
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
    let menu = menu::build_tray_menu(app, false, 0, None)?;
    let icon = icons::load_icon(TrayIconState::Normal)?;

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Mantra Gateway")
        .menu(&menu)
        .show_menu_on_left_click(false) // 左键不显示菜单，改为显示窗口
        .on_tray_icon_event(|tray, event| {
            handlers::handle_tray_icon_event(tray, event);
        })
        .on_menu_event(|app, event| {
            handlers::handle_menu_event(app, event);
        })
        .build(app)?;

    Ok(tray)
}

/// 更新托盘状态
pub async fn update_tray_state<R: Runtime>(
    app: &AppHandle<R>,
    tray: &TrayIcon<R>,
    state: &TrayState,
) -> Result<(), TrayError> {
    let manager = state.manager.read().await;

    // 更新 tooltip
    let tooltip = manager.get_tooltip();
    tray.set_tooltip(Some(&tooltip))?;

    // 更新图标
    let icon = icons::load_icon(manager.icon_state)?;
    tray.set_icon(Some(icon))?;

    // 更新菜单
    let menu = menu::build_tray_menu(
        app,
        manager.gateway_running,
        manager.connection_count,
        manager.current_project.clone(),
    )?;
    tray.set_menu(Some(menu))?;

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
        assert!(!manager.gateway_running);
        assert!(manager.current_project.is_none());
    }

    #[test]
    fn test_tray_manager_set_gateway_running() {
        let mut manager = TrayManager::new();

        manager.set_gateway_running(true);
        assert!(manager.gateway_running);
        assert_eq!(manager.icon_state, TrayIconState::Active);

        manager.set_gateway_running(false);
        assert!(!manager.gateway_running);
        assert_eq!(manager.icon_state, TrayIconState::Normal);
    }

    #[test]
    fn test_tray_manager_set_connection_count() {
        let mut manager = TrayManager::new();
        manager.set_gateway_running(true);

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
        assert_eq!(manager.get_tooltip(), "Mantra Gateway");

        // Gateway 运行中
        manager.set_gateway_running(true);
        manager.set_connection_count(2);
        assert_eq!(manager.get_tooltip(), "Mantra Gateway - 2 连接");

        // 带项目上下文
        manager.set_current_project(Some("my-project".to_string()));
        assert_eq!(manager.get_tooltip(), "Mantra - my-project (2 连接)");

        // 错误状态
        manager.set_error();
        assert_eq!(manager.get_tooltip(), "Mantra - my-project (错误)");
    }

    #[test]
    fn test_tray_state_default() {
        let state = TrayState::default();
        // TrayState 应该能正常创建
        assert!(Arc::strong_count(&state.manager) == 1);
    }
}
