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
            if let Some(window) = tray.app_handle().get_webview_window("main") {
                show_window(&window);
            }
        }
        #[cfg(target_os = "windows")]
        TrayIconEvent::DoubleClick { .. } => {
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
            if let Some(window) = app.get_webview_window("main") {
                show_window(&window);
            }
        }
        MenuIds::TOGGLE_HUB => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                toggle_hub(&app_handle).await;
            });
        }
        MenuIds::QUIT => {
            app.exit(0);
        }
        _ => {}
    }
}

/// 切换 MCP Hub 服务状态
async fn toggle_hub<R: Runtime>(app: &AppHandle<R>) {
    let gateway_state: tauri::State<'_, GatewayServerState> = app.state();
    let tray_state: tauri::State<'_, crate::tray::TrayState> = app.state();
    let mut manager = gateway_state.manager.lock().await;

    if manager.is_running() {
        manager.stop();
        println!("[Tray] MCP Hub stopped");

        {
            let mut tray_manager = tray_state.manager.write().await;
            tray_manager.set_hub_running(false);
        }
    } else {
        match manager.start().await {
            Ok(_) => {
                println!("[Tray] MCP Hub started on port {}", manager.current_port());

                {
                    let mut tray_manager = tray_state.manager.write().await;
                    tray_manager.set_hub_running(true);
                }
            }
            Err(e) => {
                eprintln!("[Tray] Failed to start MCP Hub: {}", e);

                {
                    let mut tray_manager = tray_state.manager.write().await;
                    tray_manager.set_error();
                }

                show_notification(app, "MCP Hub 启动失败", &e);
            }
        }
    }

    // 释放 manager 锁后刷新托盘
    drop(manager);

    if let Err(e) = super::refresh_tray(app).await {
        eprintln!("[Tray] Failed to refresh tray: {}", e);
    }
}

/// 显示系统通知
pub fn show_notification<R: Runtime>(_app: &AppHandle<R>, title: &str, message: &str) {
    println!("[Notification] {}: {}", title, message);
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
    fn test_menu_ids_matching() {
        assert_eq!(MenuIds::OPEN, "tray_open");
        assert_eq!(MenuIds::TOGGLE_HUB, "tray_toggle_hub");
        assert_eq!(MenuIds::QUIT, "tray_quit");
    }
}
