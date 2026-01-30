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
    pub const NO_PROJECTS: &'static str = "tray_no_projects";
    pub const QUIT: &'static str = "tray_quit";
}

/// 项目信息（用于构建切换项目子菜单）
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// 项目 ID
    pub id: String,
    /// 项目显示名称
    pub name: String,
    /// 是否为当前选中项目
    pub is_current: bool,
}

/// 构建托盘菜单
pub fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    gateway_running: bool,
    connection_count: u32,
    current_project: Option<String>,
) -> Result<Menu<R>, TrayError> {
    // 使用空项目列表构建菜单（向后兼容）
    build_tray_menu_with_projects(app, gateway_running, connection_count, current_project, vec![])
}

/// 构建托盘菜单（带项目列表）
///
/// AC3: 菜单包含 "切换项目" 子菜单列出最近项目
pub fn build_tray_menu_with_projects<R: Runtime>(
    app: &AppHandle<R>,
    gateway_running: bool,
    connection_count: u32,
    current_project: Option<String>,
    recent_projects: Vec<ProjectInfo>,
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

    // AC3: 构建"切换项目"子菜单
    let projects_submenu = build_projects_submenu(app, &current_project, &recent_projects)?;

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

    // 构建完整菜单
    Menu::with_items(
        app,
        &[
            &open_item,
            &separator1,
            &gateway_submenu,
            &projects_submenu,
            &separator2,
            &toggle_gateway_item,
            &quit_item,
        ],
    )
    .map_err(|e| TrayError::MenuBuildError(e.to_string()))
}

/// 构建"切换项目"子菜单
fn build_projects_submenu<R: Runtime>(
    app: &AppHandle<R>,
    current_project: &Option<String>,
    recent_projects: &[ProjectInfo],
) -> Result<Submenu<R>, TrayError> {
    // 如果没有项目，显示"无项目"
    if recent_projects.is_empty() {
        let no_projects_item = MenuItem::with_id(
            app,
            MenuIds::NO_PROJECTS,
            "无项目",
            false, // 不可点击
            None::<&str>,
        )
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;

        return Submenu::with_items(app, "切换项目", true, &[&no_projects_item])
            .map_err(|e| TrayError::MenuBuildError(e.to_string()));
    }

    // 使用 Submenu::new 和 append_items 来动态添加项目
    let submenu = Submenu::new(app, "切换项目", true)
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;
    
    for project in recent_projects {
        // 如果是当前项目，添加勾选标记
        let label = if Some(&project.name) == current_project.as_ref() || project.is_current {
            format!("✓ {}", project.name)
        } else {
            project.name.clone()
        };
        
        let item = MenuItem::with_id(
            app,
            &format!("{}{}", MenuIds::PROJECT_PREFIX, project.id),
            &label,
            true, // 可点击
            None::<&str>,
        )
        .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;
        
        submenu.append(&item)
            .map_err(|e| TrayError::MenuBuildError(e.to_string()))?;
    }
    
    Ok(submenu)
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
        assert_eq!(MenuIds::NO_PROJECTS, "tray_no_projects");
        assert_eq!(MenuIds::QUIT, "tray_quit");
    }

    #[test]
    fn test_project_info() {
        let project = ProjectInfo {
            id: "proj-123".to_string(),
            name: "My Project".to_string(),
            is_current: true,
        };
        
        assert_eq!(project.id, "proj-123");
        assert_eq!(project.name, "My Project");
        assert!(project.is_current);
    }

    #[test]
    fn test_project_info_clone() {
        let project = ProjectInfo {
            id: "proj-456".to_string(),
            name: "Another Project".to_string(),
            is_current: false,
        };
        
        let cloned = project.clone();
        assert_eq!(cloned.id, project.id);
        assert_eq!(cloned.name, project.name);
        assert_eq!(cloned.is_current, project.is_current);
    }
}
