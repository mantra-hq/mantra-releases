// Story 11.7: 托盘事件处理

use tauri::tray::{MouseButton, TrayIcon, TrayIconEvent};
use tauri::menu::MenuEvent;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

use super::menu::MenuIds;
use super::icons::load_icon;
use crate::GatewayServerState;

/// 处理托盘图标事件（左键点击、双击等）
///
/// AC6: 左键点击显示主窗口
/// AC7: 双击行为 (Windows) - 窗口置顶并获取焦点
pub fn handle_tray_icon_event<R: Runtime>(tray: &TrayIcon<R>, event: TrayIconEvent) {
    match event {
        // AC6: 左键点击行为
        TrayIconEvent::Click {
            button: MouseButton::Left,
            ..
        } => {
            // 左键点击：显示主窗口
            if let Some(window) = tray.app_handle().get_webview_window("main") {
                show_window(&window);
            }
        }
        // AC7: 双击行为 (Windows) - 仅在 Windows 平台处理双击
        #[cfg(target_os = "windows")]
        TrayIconEvent::DoubleClick { .. } => {
            // Windows: 双击显示并聚焦主窗口
            if let Some(window) = tray.app_handle().get_webview_window("main") {
                show_window(&window);
            }
        }
        _ => {}
    }
}

/// 显示并聚焦窗口
fn show_window<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

/// 处理菜单事件
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let id = event.id().as_ref();

    match id {
        MenuIds::OPEN => {
            // 打开主窗口
            if let Some(window) = app.get_webview_window("main") {
                show_window(&window);
            }
        }
        MenuIds::TOGGLE_GATEWAY => {
            // 切换 Gateway 状态
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                toggle_gateway(&app_handle).await;
            });
        }
        MenuIds::QUIT => {
            // 退出应用
            app.exit(0);
        }
        _ => {
            // AC4: 处理项目切换（如果以 PROJECT_PREFIX 开头）
            if id.starts_with(MenuIds::PROJECT_PREFIX) {
                let project_id = &id[MenuIds::PROJECT_PREFIX.len()..];
                // 跳过 "current" 标记（仅显示用）
                if project_id != "current" {
                    println!("[Tray] Project selected: {}", project_id);
                    let app_handle = app.clone();
                    let project_id_owned = project_id.to_string();
                    tauri::async_runtime::spawn(async move {
                        switch_project(&app_handle, &project_id_owned).await;
                    });
                }
            }
        }
    }
}

/// AC4: 切换项目上下文
///
/// 更新托盘显示的当前项目，后续 MCP 会话将基于此项目上下文
async fn switch_project<R: Runtime>(app: &AppHandle<R>, project_id: &str) {
    // 获取项目名称（从 project_id 解析）
    // project_id 格式通常是项目路径或 UUID
    let project_name = project_id.split('/').last().unwrap_or(project_id).to_string();
    
    println!("[Tray] Switching to project: {} ({})", project_name, project_id);
    
    // 更新托盘状态
    let tray_state: tauri::State<'_, crate::tray::TrayState> = app.state();
    {
        let mut tray_manager = tray_state.manager.write().await;
        tray_manager.set_current_project(Some(project_name.clone()));
    }
    
    // AC4: 托盘 tooltip 更新显示当前项目名称
    // 刷新托盘显示
    if let Err(e) = refresh_tray(app).await {
        eprintln!("[Tray] Failed to refresh tray after project switch: {}", e);
    }
    
    // 发送事件到前端，通知项目切换
    // 前端可以调用 gateway_set_project_context 来设置活跃会话的上下文
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("tray://project-switched", serde_json::json!({
            "project_id": project_id,
            "project_name": project_name,
        }));
    }
}

/// 切换 Gateway 服务状态
async fn toggle_gateway<R: Runtime>(app: &AppHandle<R>) {
    let gateway_state: tauri::State<'_, GatewayServerState> = app.state();
    let tray_state: tauri::State<'_, crate::tray::TrayState> = app.state();
    let mut manager = gateway_state.manager.lock().await;

    if manager.is_running() {
        // 停止 Gateway (同步方法)
        manager.stop();
        println!("[Tray] Gateway stopped");

        // 更新托盘状态
        {
            let mut tray_manager = tray_state.manager.write().await;
            tray_manager.set_gateway_running(false);
        }
    } else {
        // 启动 Gateway
        match manager.start().await {
            Ok(_) => {
                println!("[Tray] Gateway started on port {}", manager.current_port());

                // 更新托盘状态
                {
                    let mut tray_manager = tray_state.manager.write().await;
                    tray_manager.set_gateway_running(true);
                }
            }
            Err(e) => {
                eprintln!("[Tray] Failed to start Gateway: {}", e);

                // 设置错误状态
                {
                    let mut tray_manager = tray_state.manager.write().await;
                    tray_manager.set_error();
                }

                // AC5: 显示错误通知
                show_notification(app, "Gateway 启动失败", &e);
            }
        }
    }
    
    // 释放 manager 锁后刷新托盘
    drop(manager);
    
    // 刷新托盘菜单以反映新状态
    if let Err(e) = refresh_tray(app).await {
        eprintln!("[Tray] Failed to refresh tray: {}", e);
    }
}

/// 刷新托盘状态和菜单
async fn refresh_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), super::TrayError> {
    let tray_state: tauri::State<'_, crate::tray::TrayState> = app.state();
    let manager = tray_state.manager.read().await;
    
    // 获取托盘图标实例
    if let Some(tray) = app.tray_by_id("main") {
        // 更新 tooltip
        let tooltip = manager.get_tooltip();
        tray.set_tooltip(Some(&tooltip))?;
        
        // 更新图标
        let icon = load_icon(manager.icon_state)?;
        tray.set_icon(Some(icon))?;
        
        // 更新菜单
        let menu = super::menu::build_tray_menu(
            app,
            manager.gateway_running,
            manager.connection_count,
            manager.current_project.clone(),
        )?;
        tray.set_menu(Some(menu))?;
    }
    
    Ok(())
}

/// 显示系统通知
pub fn show_notification<R: Runtime>(_app: &AppHandle<R>, title: &str, message: &str) {
    // 使用 Tauri 的 notification API（如果可用）
    // 目前先用日志输出
    println!("[Notification] {}: {}", title, message);

    // TODO: 当添加 tauri-plugin-notification 后，使用原生通知
    // use tauri_plugin_notification::NotificationExt;
    // let _ = app.notification()
    //     .builder()
    //     .title(title)
    //     .body(message)
    //     .show();
}

/// 显示主窗口
pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// 隐藏主窗口
pub fn hide_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_event_id_parsing() {
        // 测试项目 ID 解析
        let id = "tray_project_my-project-123";
        if id.starts_with(MenuIds::PROJECT_PREFIX) {
            let project_id = &id[MenuIds::PROJECT_PREFIX.len()..];
            assert_eq!(project_id, "my-project-123");
        } else {
            panic!("Should start with project prefix");
        }
    }

    #[test]
    fn test_menu_ids_matching() {
        assert_eq!(MenuIds::OPEN, "tray_open");
        assert_eq!(MenuIds::TOGGLE_GATEWAY, "tray_toggle_gateway");
        assert_eq!(MenuIds::QUIT, "tray_quit");
    }
}
