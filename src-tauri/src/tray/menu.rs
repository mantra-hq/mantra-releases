// Story 11.7: 托盘菜单构建

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Runtime};

use super::icons::{get_state_emoji, TrayIconState};
use super::TrayError;

/// 菜单项 ID 常量
pub struct MenuIds;

impl MenuIds {
    pub const OPEN: &'static str = "tray_open";
    pub const HUB_STATUS: &'static str = "tray_hub_status";
    pub const HUB_CONNECTIONS: &'static str = "tray_hub_connections";
    pub const TOGGLE_HUB: &'static str = "tray_toggle_hub";
    pub const QUIT: &'static str = "tray_quit";
}

/// 构建托盘菜单
pub fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    hub_running: bool,
    connection_count: u32,
) -> Result<Menu<R>, TrayError> {
    // 状态指示器
    let status_emoji = if hub_running {
        get_state_emoji(TrayIconState::Active)
    } else {
        get_state_emoji(TrayIconState::Normal)
    };

    let status_text = if hub_running {
        format!("{} 运行中", status_emoji)
    } else {
        format!("{} 已停止", status_emoji)
    };

    let connections_text = format!("活跃连接: {}", connection_count);

    // 构建 MCP Hub 状态子菜单
    let hub_status_item = MenuItem::with_id(
        app,
        MenuIds::HUB_STATUS,
        &status_text,
        false,
        None::<&str>,
    )
    .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let hub_connections_item = MenuItem::with_id(
        app,
        MenuIds::HUB_CONNECTIONS,
        &connections_text,
        false,
        None::<&str>,
    )
    .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let hub_submenu = Submenu::with_items(
        app,
        "MCP Hub 状态",
        true,
        &[&hub_status_item, &hub_connections_item],
    )
    .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    // 切换 MCP Hub 按钮
    let toggle_text = if hub_running {
        "停止 MCP Hub"
    } else {
        "启动 MCP Hub"
    };

    // 构建菜单项
    let open_item = MenuItem::with_id(app, MenuIds::OPEN, "打开 Mantra", true, None::<&str>)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let separator1 = PredefinedMenuItem::separator(app)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let toggle_hub_item =
        MenuItem::with_id(app, MenuIds::TOGGLE_HUB, toggle_text, true, None::<&str>)
            .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let separator2 = PredefinedMenuItem::separator(app)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let quit_item = MenuItem::with_id(app, MenuIds::QUIT, "退出", true, None::<&str>)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    // 构建完整菜单
    Menu::with_items(
        app,
        &[
            &open_item,
            &separator1,
            &hub_submenu,
            &separator2,
            &toggle_hub_item,
            &quit_item,
        ],
    )
    .map_err(|e| TrayError::MenuBuildError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_ids() {
        assert_eq!(MenuIds::OPEN, "tray_open");
        assert_eq!(MenuIds::HUB_STATUS, "tray_hub_status");
        assert_eq!(MenuIds::HUB_CONNECTIONS, "tray_hub_connections");
        assert_eq!(MenuIds::TOGGLE_HUB, "tray_toggle_hub");
        assert_eq!(MenuIds::QUIT, "tray_quit");
    }
}
