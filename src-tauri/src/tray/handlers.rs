// Story 11.7: 托盘事件处理

use tauri::tray::{MouseButton, TrayIcon, TrayIconEvent};
use tauri::menu::MenuEvent;
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use super::menu::MenuIds;
use crate::GatewayServerState;

/// 处理托盘图标事件（左键点击、双击等）
pub fn handle_tray_icon_event<R: Runtime>(tray: &TrayIcon<R>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            ..
        } => {
            // 左键点击：显示主窗口
            if let Some(window) = tray.app_handle().get_webview_window("main") {
                show_window(&window);
            }
        }
        TrayIconEvent::DoubleClick { .. } => {
            // 双击：显示并聚焦主窗口（主要用于 Windows）
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
            // 处理项目切换（如果以 PROJECT_PREFIX 开头）
            if id.starts_with(MenuIds::PROJECT_PREFIX) {
                let project_id = &id[MenuIds::PROJECT_PREFIX.len()..];
                println!("[Tray] Project selected: {}", project_id);
                // TODO: 实现项目切换逻辑
            }
        }
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

                // 显示错误通知（如果可用）
                show_notification(app, "Gateway 启动失败", &e);
            }
        }
    }
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
