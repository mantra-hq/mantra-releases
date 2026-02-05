// Mantra Client Library
// Provides Tauri IPC commands for parsing AI conversation logs

pub mod analytics;
pub mod commands;
pub mod error;
pub mod gateway;
pub mod git;
pub mod local_server;
pub mod models;
pub mod parsers;
pub mod sanitizer;
pub mod scanner;
pub mod services;
pub mod storage;
pub mod tray;

use std::sync::{Arc, Mutex};

use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;

use commands::{
    detect_git_repo, find_commit_at_time, get_commit_info, get_file_at_head, get_file_snapshot,
    get_project, get_project_by_cwd, get_project_by_session, get_project_sessions, get_representative_file,
    get_session, get_snapshot_at_time, get_snapshot_with_fallback, import_parsed_sessions,
    import_sessions, list_projects, parse_claude_log, parse_claude_log_string,
    parse_cursor_log, parse_cursor_all, parse_gemini_log, parse_gemini_log_string,
    parse_gemini_all, parse_gemini_project, parse_codex_log, parse_codex_log_string,
    parse_codex_all, parse_codex_project, parse_log_files,
    sanitize_session, sanitize_text, validate_regex, get_builtin_rules, scan_text_for_privacy,
    // Story 3.10: Privacy rules management
    get_privacy_rules, update_privacy_rules, validate_regex_v2,
    // Story 3.7: Interception records
    save_interception_record, get_interception_records, get_interception_stats, delete_interception_records,
    scan_custom_directory, scan_log_directory, AppState,
    list_tree_at_commit, list_files_at_commit,
    // Story 2.19: Project management commands
    sync_project, remove_project, rename_project,
    // Story 1.9: Project cwd update
    update_project_cwd,
    // Story 1.12: View-based project aggregation
    add_project_path, remove_project_path, get_project_paths,
    bind_session_to_project, unbind_session, get_unassigned_sessions, set_project_primary_path,
    get_logical_project_stats, get_sessions_by_physical_path, get_projects_by_physical_path,
    // Story 1.13: Logical project rename
    rename_logical_project, reset_logical_project_name,
    // Story 2.20: Import wizard enhancement
    get_imported_session_ids,
    // Story 2.23: Import with progress events
    import_sessions_with_progress, cancel_import,
    // Story 2.10: Global search
    search_sessions,
    // Platform-specific default paths
    get_default_paths,
    // Story 2.32: Git commits in time range
    get_commits_in_range,
    // Story 2.34: Analytics commands
    get_project_analytics, get_session_metrics, get_session_stats_view,
    // Story 3.11: Local API Server commands
    get_local_server_status, get_local_server_config, update_local_server_port,
    start_local_server, stop_local_server,
    // Story 11.1: Gateway commands
    get_gateway_status, get_gateway_config, update_gateway_config,
    start_gateway, stop_gateway, restart_gateway, regenerate_gateway_token,
    // Story 11.5: Context Routing commands
    gateway_set_project_context, gateway_clear_project_context,
    gateway_get_session_context, gateway_list_sessions,
    // Story 11.2: MCP Service commands
    McpState, list_mcp_services, list_mcp_services_by_source, get_mcp_service,
    get_mcp_service_by_name, create_mcp_service, update_mcp_service, delete_mcp_service,
    toggle_mcp_service, link_mcp_service_to_project, unlink_mcp_service_from_project,
    get_project_mcp_services, get_mcp_service_projects, update_project_mcp_service_override,
    set_env_variable, list_env_variables, delete_env_variable, env_variable_exists,
    // Story 11.4: Env Variable Management commands
    get_env_variable_decrypted, get_affected_mcp_services, batch_set_env_variables,
    validate_env_variable_name,
    // Story 11.3: MCP Config Import commands
    scan_mcp_configs_cmd, preview_mcp_import, execute_mcp_import, rollback_mcp_import,
    // Story 11.15: MCP Takeover Restore commands
    list_active_takeovers, restore_takeover, restore_takeover_by_tool, get_active_takeover,
    // Story 11.16: Takeover scope commands
    get_active_takeovers_by_project, read_config_file_content,
    // Story 11.9: Project Detail MCP Integration commands
    check_project_mcp_status,
    // Story 11.10: Project-Level Tool Management commands
    get_project_tool_policy, update_project_tool_policy, fetch_service_tools,
    // Story 11.9 Phase 2: Service-Level Default Tool Policy commands
    get_service_default_policy, update_service_default_policy,
    // Story 11.19: MCP Smart Takeover Merge Engine commands
    preview_smart_takeover, execute_smart_takeover_cmd,
    // Story 11.20: Full Tool Takeover commands
    preview_full_tool_takeover, detect_installed_tools, scan_all_tool_configs, execute_full_tool_takeover_cmd,
    // Story 11.21: Local Scope commands
    scan_local_scopes, restore_local_scope_takeover_cmd, restore_all_local_scope_takeovers_cmd, get_active_local_scope_takeovers,
    // Story 11.22: Atomic Backup Integrity commands
    list_active_takeovers_with_integrity, delete_invalid_takeover_backups,
    // Story 11.23: Backup Version Management commands
    cleanup_old_takeover_backups, cleanup_all_old_takeover_backups,
    get_backup_stats, list_takeover_backups_with_version, delete_single_takeover_backup,
    // Story 11.7: Tray commands
    get_tray_status, update_tray_gateway_status, update_tray_project, set_tray_error,
    // Story 11.12: OAuth commands
    OAuthState, oauth_start_flow, oauth_get_status, oauth_disconnect, oauth_refresh_token,
    // Story 11.11: MCP Inspector commands
    McpProcessState, mcp_get_service_capabilities, mcp_call_tool, mcp_read_resource,
    mcp_stop_service, mcp_list_running_services,
    // Story 11.17: MCP Aggregator Refresh commands
    gateway_refresh_service, gateway_refresh_all,
};

use storage::Database;
use local_server::ServerManager;
use gateway::{GatewayConfig, GatewayServerManager};
use services::EnvManager;
use tray::TrayState;

/// Database file name
const DATABASE_FILENAME: &str = "mantra.db";

/// Local API Server state
pub struct LocalServerState {
    pub manager: TokioMutex<ServerManager>,
}

/// Gateway Server state (Story 11.1)
pub struct GatewayServerState {
    pub manager: TokioMutex<GatewayServerManager>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Get app data directory for database storage
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Create directory if it doesn't exist
            std::fs::create_dir_all(&app_data_dir)?;

            // Initialize database
            let db_path = app_data_dir.join(DATABASE_FILENAME);
            let db = Database::new(&db_path).expect("Failed to initialize database");

            // Store database in app state
            let db_arc = Arc::new(Mutex::new(db));
            app.manage(AppState { db: Mutex::new(Database::new(&db_path).expect("Failed to create second db connection")) });

            // Story 3.11: 启动本地 API Server（共享数据库连接）
            let server_manager = ServerManager::with_database(app_data_dir.clone(), db_arc.clone());
            app.manage(LocalServerState {
                manager: TokioMutex::new(server_manager),
            });

            // Story 11.1: 初始化 Gateway Server
            // 从数据库加载配置
            let gateway_config = {
                let db = db_arc.lock().map_err(|e| format!("Failed to lock db: {}", e))?;
                match db.get_gateway_config() {
                    Ok(config) => GatewayConfig {
                        port: config.port.map(|p| p as u16).unwrap_or(0),
                        auth_token: config.auth_token,
                        enabled: config.enabled,
                        auto_start: config.auto_start,
                    },
                    Err(_) => GatewayConfig::default(),
                }
            };

            let gateway_manager = GatewayServerManager::new(gateway_config.clone());
            app.manage(GatewayServerState {
                manager: TokioMutex::new(gateway_manager),
            });

            // Story 11.2: 初始化 MCP 服务状态
            let mcp_db = Database::new(&db_path).expect("Failed to create MCP db connection");
            let env_manager = EnvManager::from_machine_id();
            app.manage(McpState {
                db: Arc::new(Mutex::new(mcp_db)),
                env_manager,
            });

            // Story 11.7: 初始化托盘状态
            app.manage(TrayState::default());

            // Story 11.12: 初始化 OAuth 状态
            app.manage(OAuthState::new());

            // Story 11.11: 初始化 MCP 进程管理器状态
            app.manage(McpProcessState::new());

            // Story 11.7: 初始化系统托盘
            match tray::init_tray(app.handle()) {
                Ok(_tray) => {
                    println!("[Mantra] System tray initialized");
                }
                Err(e) => {
                    eprintln!("[Mantra] Failed to initialize system tray: {}", e);
                }
            }

            // Story 11.7: 设置窗口关闭拦截（关闭时隐藏到托盘而非退出）
            if let Some(main_window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // 阻止默认关闭行为
                        api.prevent_close();
                        // 隐藏窗口到托盘
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                });
            }

            // 在后台启动 Local Server（不阻塞 setup）
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state: tauri::State<'_, LocalServerState> = app_handle.state();
                let mut manager = state.manager.lock().await;
                match manager.start().await {
                    Ok(_) => {
                        println!("[Mantra] Local API Server started on port {}", manager.current_port());
                    }
                    Err(e) => {
                        eprintln!("[Mantra] Failed to start Local API Server: {}", e);
                    }
                }
            });

            // Story 11.1: 如果 Gateway 配置为自动启动，则启动 Gateway Server
            // Story 11.17: 复用 start_gateway 命令逻辑，确保 aggregator 和 warmup 被正确执行
            if gateway_config.auto_start && gateway_config.enabled {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let gateway_state: tauri::State<'_, GatewayServerState> = app_handle.state();
                    let app_state: tauri::State<'_, AppState> = app_handle.state();
                    let mcp_state: tauri::State<'_, McpState> = app_handle.state();

                    // Story 11.27: 传入 app_handle 用于 LPM 查询服务
                    match start_gateway(app_handle.clone(), gateway_state, app_state, mcp_state).await {
                        Ok(status) => {
                            println!("[Mantra] Gateway Server started on port {}", status.port.unwrap_or(0));
                        }
                        Err(e) => {
                            eprintln!("[Mantra] Failed to start Gateway Server: {:?}", e);
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            parse_claude_log,
            parse_claude_log_string,
            parse_cursor_log,
            parse_cursor_all,
            parse_gemini_log,
            parse_gemini_log_string,
            parse_gemini_all,
            parse_gemini_project,
            parse_codex_log,
            parse_codex_log_string,
            parse_codex_all,
            parse_codex_project,
            get_file_snapshot,
            get_file_at_head,
            get_snapshot_at_time,
            get_snapshot_with_fallback,
            detect_git_repo,
            find_commit_at_time,
            get_commit_info,
            list_projects,
            get_project,
            get_project_by_cwd,
            get_project_by_session,
            get_project_sessions,
            get_session,
            import_sessions,
            import_parsed_sessions,
            scan_log_directory,
            scan_custom_directory,
            parse_log_files,
            get_representative_file,
            list_tree_at_commit,
            list_files_at_commit,
            sanitize_text,
            sanitize_session,
            validate_regex,
            // Story 3-5: Builtin rules
            get_builtin_rules,
            // Story 3-6: Privacy scanner
            scan_text_for_privacy,
            // Story 3.10: Privacy rules management
            get_privacy_rules,
            update_privacy_rules,
            validate_regex_v2,
            // Story 3.7: Interception records
            save_interception_record,
            get_interception_records,
            get_interception_stats,
            delete_interception_records,
            // Story 2.19: Project management
            sync_project,
            remove_project,
            rename_project,
            // Story 1.9: Project cwd update
            update_project_cwd,
            // Story 1.12: View-based project aggregation
            add_project_path,
            remove_project_path,
            get_project_paths,
            bind_session_to_project,
            unbind_session,
            get_unassigned_sessions,
            set_project_primary_path,
            get_logical_project_stats,
            get_sessions_by_physical_path,
            get_projects_by_physical_path,
            // Story 1.13: Logical project rename
            rename_logical_project,
            reset_logical_project_name,
            // Story 2.20: Import wizard enhancement
            get_imported_session_ids,
            // Story 2.23: Import with progress events
            import_sessions_with_progress,
            cancel_import,
            // Story 2.10: Global search
            search_sessions,
            // Platform-specific default paths
            get_default_paths,
            // Story 2.32: Git commits in time range
            get_commits_in_range,
            // Story 2.34: Analytics
            get_project_analytics,
            get_session_metrics,
            get_session_stats_view,
            // Story 3.11: Local API Server
            get_local_server_status,
            get_local_server_config,
            update_local_server_port,
            start_local_server,
            stop_local_server,
            // Story 11.1: Gateway Server
            get_gateway_status,
            get_gateway_config,
            update_gateway_config,
            start_gateway,
            stop_gateway,
            restart_gateway,
            regenerate_gateway_token,
            // Story 11.5: Context Routing
            gateway_set_project_context,
            gateway_clear_project_context,
            gateway_get_session_context,
            gateway_list_sessions,
            // Story 11.2: MCP Service
            list_mcp_services,
            list_mcp_services_by_source,
            get_mcp_service,
            get_mcp_service_by_name,
            create_mcp_service,
            update_mcp_service,
            delete_mcp_service,
            toggle_mcp_service,
            link_mcp_service_to_project,
            unlink_mcp_service_from_project,
            get_project_mcp_services,
            get_mcp_service_projects,
            update_project_mcp_service_override,
            set_env_variable,
            list_env_variables,
            delete_env_variable,
            env_variable_exists,
            // Story 11.4: Env Variable Management
            get_env_variable_decrypted,
            get_affected_mcp_services,
            batch_set_env_variables,
            validate_env_variable_name,
            // Story 11.3: MCP Config Import
            scan_mcp_configs_cmd,
            preview_mcp_import,
            execute_mcp_import,
            rollback_mcp_import,
            // Story 11.15: MCP Takeover Restore
            list_active_takeovers,
            restore_takeover,
            restore_takeover_by_tool,
            get_active_takeover,
            // Story 11.16: Takeover scope commands
            get_active_takeovers_by_project,
            read_config_file_content,
            // Story 11.9: Project Detail MCP Integration
            check_project_mcp_status,
            // Story 11.10: Project-Level Tool Management
            get_project_tool_policy,
            update_project_tool_policy,
            fetch_service_tools,
            // Story 11.9 Phase 2: Service-Level Default Tool Policy
            get_service_default_policy,
            update_service_default_policy,
            // Story 11.19: MCP Smart Takeover Merge Engine
            preview_smart_takeover,
            execute_smart_takeover_cmd,
            // Story 11.20: Full Tool Takeover
            preview_full_tool_takeover,
            detect_installed_tools,
            scan_all_tool_configs,
            execute_full_tool_takeover_cmd,
            // Story 11.21: Local Scope
            scan_local_scopes,
            restore_local_scope_takeover_cmd,
            restore_all_local_scope_takeovers_cmd,
            get_active_local_scope_takeovers,
            // Story 11.22: Atomic Backup Integrity
            list_active_takeovers_with_integrity,
            delete_invalid_takeover_backups,
            // Story 11.23: Backup Version Management
            cleanup_old_takeover_backups,
            cleanup_all_old_takeover_backups,
            get_backup_stats,
            list_takeover_backups_with_version,
            delete_single_takeover_backup,
            // Story 11.7: Tray
            get_tray_status,
            update_tray_gateway_status,
            update_tray_project,
            set_tray_error,
            // Story 11.12: OAuth
            oauth_start_flow,
            oauth_get_status,
            oauth_disconnect,
            oauth_refresh_token,
            // Story 11.11: MCP Inspector
            mcp_get_service_capabilities,
            mcp_call_tool,
            mcp_read_resource,
            mcp_stop_service,
            mcp_list_running_services,
            // Story 11.17: MCP Aggregator Refresh
            gateway_refresh_service,
            gateway_refresh_all
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod debug_sync_test {
    use std::path::PathBuf;
    use crate::parsers::{ClaudeParser, LogParser};

    #[test]
    fn test_sync_scanning_logic() {
        let target_cwd = "/mnt/disk0/project/newx/nextalk/voice_capsule";
        let home = std::env::var("HOME").unwrap();
        let claude_projects_dir = PathBuf::from(&home).join(".claude").join("projects");
        let claude_parser = ClaudeParser::new();

        println!("\n=== Scanning Claude projects directory ===");
        println!("Looking for sessions with cwd: {}\n", target_cwd);

        let mut all_sessions: Vec<(String, String, usize)> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&claude_projects_dir) {
            for entry in entries.flatten() {
                let project_dir = entry.path();
                if project_dir.is_dir() {
                    if let Ok(session_files) = std::fs::read_dir(&project_dir) {
                        for session_file in session_files.flatten() {
                            let path = session_file.path();
                            let is_jsonl = path.extension()
                                .map(|e| e == "jsonl")
                                .unwrap_or(false);

                            if !is_jsonl {
                                continue;
                            }

                            let file_name = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_string();

                            // 检查是否以 agent- 开头 (模拟 sync_project 的逻辑)
                            let is_agent = file_name.starts_with("agent-");

                            if is_agent {
                                println!("SKIP: {} (agent file)", file_name);
                                continue;
                            }

                            // 解析文件
                            match claude_parser.parse_file(path.to_string_lossy().as_ref()) {
                                Ok(session) => {
                                    if session.cwd == target_cwd {
                                        println!("FOUND: {} -> session={}, messages={}", 
                                            file_name, session.id, session.messages.len());
                                        all_sessions.push((session.id.clone(), file_name, session.messages.len()));
                                    }
                                }
                                Err(e) => {
                                    println!("ERROR parsing {}: {:?}", file_name, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("\n=== Sessions found for cwd {} ===", target_cwd);
        for (session_id, file_name, message_count) in &all_sessions {
            println!("  {}: {} ({} messages)", session_id, file_name, message_count);
        }

        // 检查 4fe9325e session
        let target_session: Vec<_> = all_sessions.iter()
            .filter(|(sid, _, _)| sid == "4fe9325e-4c69-4633-ac6f-d879ca16d6c5")
            .collect();
        
        println!("\n=== Target session 4fe9325e... ===");
        for (session_id, file_name, message_count) in &target_session {
            println!("  {} from {} ({} messages)", session_id, file_name, message_count);
        }
        
        assert_eq!(target_session.len(), 1, "Should only have 1 entry for session 4fe9325e");
        assert_eq!(target_session[0].2, 12, "Should have 12 messages");
    }
}

#[cfg(test)]
mod debug_full_sync_test {
    use std::path::PathBuf;
    use crate::parsers::{ClaudeParser, LogParser};
    use crate::models::MantraSession;
    use std::collections::HashMap;

    #[test]
    fn test_full_sync_flow() {
        // 模拟完整的 sync_project 流程
        let target_cwd = "/mnt/disk0/project/newx/nextalk/voice_capsule";
        let target_session_id = "4fe9325e-4c69-4633-ac6f-d879ca16d6c5";
        let force = true;

        // 步骤 1: 扫描并解析文件（与 sync_project 相同的逻辑）
        let home = std::env::var("HOME").unwrap();
        let claude_projects_dir = PathBuf::from(&home).join(".claude").join("projects");
        let claude_parser = ClaudeParser::new();
        let mut all_sessions: Vec<MantraSession> = Vec::new();

        println!("\n=== Step 1: Scanning and parsing files ===");
        if let Ok(entries) = std::fs::read_dir(&claude_projects_dir) {
            for entry in entries.flatten() {
                let project_dir = entry.path();
                if project_dir.is_dir() {
                    if let Ok(session_files) = std::fs::read_dir(&project_dir) {
                        for session_file in session_files.flatten() {
                            let path = session_file.path();
                            if path.extension().is_some_and(|e| e == "jsonl") {
                                let file_name = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("");

                                // 跳过 agent 文件（与 sync_project 相同）
                                if file_name.starts_with("agent-") {
                                    continue;
                                }

                                match claude_parser.parse_file(path.to_string_lossy().as_ref()) {
                                    Ok(session) => {
                                        if session.cwd == target_cwd {
                                            println!("Parsed: {} -> {} messages", 
                                                session.id, session.messages.len());
                                            all_sessions.push(session);
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        // 步骤 2: 模拟数据库状态（假设数据库中有错误的 2 条消息）
        println!("\n=== Step 2: Simulating database state ===");
        let mut existing_session_map: HashMap<String, u32> = HashMap::new();
        existing_session_map.insert(target_session_id.to_string(), 2); // 模拟错误状态
        println!("Database has session {} with 2 messages", target_session_id);

        // 步骤 3: 模拟更新逻辑
        println!("\n=== Step 3: Simulating update logic ===");
        for session in &all_sessions {
            if session.id == target_session_id {
                if let Some(&old_count) = existing_session_map.get(&session.id) {
                    let new_count = session.messages.len() as u32;
                    println!("Session {}: old_count={}, new_count={}, force={}", 
                        session.id, old_count, new_count, force);

                    if new_count > old_count || force {
                        println!("  -> Should call update_session with {} messages", 
                            session.messages.len());
                        
                        // 检查 session 内容
                        println!("  -> Session content preview:");
                        for (i, msg) in session.messages.iter().take(3).enumerate() {
                            let preview = match &msg.content_blocks.first() {
                                Some(crate::models::ContentBlock::Text { text, .. }) =>
                                    text.chars().take(50).collect::<String>(),
                                _ => "non-text".to_string(),
                            };
                            println!("     Msg {}: {:?} - {}", i+1, msg.role, preview);
                        }
                    }
                }
            }
        }

        // 验证目标 session 只有一个，且有 12 条消息
        let target_sessions: Vec<_> = all_sessions.iter()
            .filter(|s| s.id == target_session_id)
            .collect();

        println!("\n=== Verification ===");
        assert_eq!(target_sessions.len(), 1, "Should only have 1 session");
        assert_eq!(target_sessions[0].messages.len(), 12, "Should have 12 messages");
        println!("PASS: Found exactly 1 session with 12 messages");

        // 检查消息内容是否正确（应该不是 "Warmup"）
        let first_msg = &target_sessions[0].messages[0];
        let first_text = match &first_msg.content_blocks.first() {
            Some(crate::models::ContentBlock::Text { text, .. }) => text.clone(),
            _ => String::new(),
        };
        println!("First message content: {}", first_text.chars().take(100).collect::<String>());
        assert!(!first_text.contains("Warmup"), "First message should NOT be 'Warmup' (agent file content)");
        println!("PASS: First message is NOT agent file content");
    }
}
