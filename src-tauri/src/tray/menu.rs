// Story 11.7: 托盘菜单构建

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Runtime};

use super::icons::{get_state_emoji, TrayIconState};
use super::TrayError;

/// 菜单项 ID 常量
pub struct MenuIds;

impl MenuIds {
    pub const OPEN: &'static str = "tray_open";
    pub const GATEWAY_STATUS: &'static str = "tray_gateway_status";
    pub const GATEWAY_CONNECTIONS: &'static str = "tray_gateway_connections";
    pub const TOGGLE_GATEWAY: &'static str = "tray_toggle_gateway";
    pub const PROJECT_PREFIX: &'static str = "tray_project_";
    pub const QUIT: &'static str = "tray_quit";
}

/// 构建托盘菜单
pub fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    gateway_running: bool,
    connection_count: u32,
    current_project: Option<String>,
) -> Result<Menu<R>, TrayError> {
    // 状态指示器
    let status_emoji = if gateway_running {
        get_state_emoji(TrayIconState::Active)
    } else {
        get_state_emoji(TrayIconState::Normal)
    };

    let status_text = if gateway_running {
        format!("{} 运行中", status_emoji)
    } else {
        format!("{} 已停止", status_emoji)
    };

    let connections_text = format!("活跃连接: {}", connection_count);

    // 构建 Gateway 状态子菜单
    let gateway_status_item = MenuItem::with_id(
        app,
        MenuIds::GATEWAY_STATUS,
        &status_text,
        false, // 不可点击
        None::<&str>,
    )
    .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let gateway_connections_item = MenuItem::with_id(
        app,
        MenuIds::GATEWAY_CONNECTIONS,
        &connections_text,
        false, // 不可点击
        None::<&str>,
    )
    .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let gateway_submenu = Submenu::with_items(
        app,
        "Gateway 状态",
        true,
        &[&gateway_status_item, &gateway_connections_item],
    )
    .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    // 切换 Gateway 按钮
    let toggle_text = if gateway_running {
        "停止 Gateway"
    } else {
        "启动 Gateway"
    };

    // 构建菜单项
    let open_item = MenuItem::with_id(app, MenuIds::OPEN, "打开 Mantra", true, None::<&str>)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let separator1 = PredefinedMenuItem::separator(app)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let toggle_gateway_item =
        MenuItem::with_id(app, MenuIds::TOGGLE_GATEWAY, toggle_text, true, None::<&str>)
            .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let separator2 = PredefinedMenuItem::separator(app)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    let quit_item = MenuItem::with_id(app, MenuIds::QUIT, "退出", true, None::<&str>)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

    // 根据是否有当前项目构建菜单
    if let Some(project) = current_project {
        let project_info = MenuItem::with_id(
            app,
            &format!("{}{}", MenuIds::PROJECT_PREFIX, "current"),
            &format!("📁 {}", project),
            false, // 不可点击，仅显示
            None::<&str>,
        )
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

        Menu::with_items(
            app,
            &[
                &open_item,
                &separator1,
                &project_info,
                &gateway_submenu,
                &separator2,
                &toggle_gateway_item,
                &quit_item,
            ],
        )
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))
    } else {
        Menu::with_items(
            app,
            &[
                &open_item,
                &separator1,
                &gateway_submenu,
                &separator2,
                &toggle_gateway_item,
                &quit_item,
            ],
        )
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_ids() {
        assert_eq!(MenuIds::OPEN, "tray_open");
        assert_eq!(MenuIds::GATEWAY_STATUS, "tray_gateway_status");
        assert_eq!(MenuIds::GATEWAY_CONNECTIONS, "tray_gateway_connections");
        assert_eq!(MenuIds::TOGGLE_GATEWAY, "tray_toggle_gateway");
        assert_eq!(MenuIds::PROJECT_PREFIX, "tray_project_");
        assert_eq!(MenuIds::QUIT, "tray_quit");
    }
}
