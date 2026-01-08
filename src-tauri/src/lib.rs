// Mantra Client Library
// Provides Tauri IPC commands for parsing AI conversation logs

pub mod commands;
pub mod error;
pub mod git;
pub mod models;
pub mod parsers;
pub mod sanitizer;
pub mod scanner;
pub mod storage;

use std::sync::Mutex;

use tauri::Manager;

use commands::{
    detect_git_repo, find_commit_at_time, get_commit_info, get_file_at_head, get_file_snapshot,
    get_project, get_project_by_cwd, get_project_by_session, get_project_sessions, get_representative_file,
    get_session, get_snapshot_at_time, get_snapshot_with_fallback, import_parsed_sessions,
    import_sessions, list_projects, parse_claude_log, parse_claude_log_string,
    parse_cursor_log, parse_cursor_all, parse_gemini_log, parse_gemini_log_string,
    parse_gemini_all, parse_gemini_project, parse_codex_log, parse_codex_log_string,
    parse_codex_all, parse_codex_project, parse_log_files,
    sanitize_session, sanitize_text, validate_regex, get_builtin_rules, scan_custom_directory, scan_log_directory, AppState,
    list_tree_at_commit, list_files_at_commit,
    // Story 2.19: Project management commands
    sync_project, remove_project, rename_project,
    // Story 1.9: Project cwd update
    update_project_cwd,
    // Story 2.20: Import wizard enhancement
    get_imported_session_ids,
    // Story 2.23: Import with progress events
    import_sessions_with_progress, cancel_import,
    // Story 2.10: Global search
    search_sessions,
    // Platform-specific default paths
    get_default_paths,
};

use storage::Database;

/// Database file name
const DATABASE_FILENAME: &str = "mantra.db";

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
            app.manage(AppState { db: Mutex::new(db) });

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
            // Story 2.19: Project management
            sync_project,
            remove_project,
            rename_project,
            // Story 1.9: Project cwd update
            update_project_cwd,
            // Story 2.20: Import wizard enhancement
            get_imported_session_ids,
            // Story 2.23: Import with progress events
            import_sessions_with_progress,
            cancel_import,
            // Story 2.10: Global search
            search_sessions,
            // Platform-specific default paths
            get_default_paths
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
    use crate::storage::Database;
    use crate::models::{MantraSession, sources};
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
                                Some(crate::models::ContentBlock::Text { text }) => 
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
            Some(crate::models::ContentBlock::Text { text }) => text.clone(),
            _ => String::new(),
        };
        println!("First message content: {}", first_text.chars().take(100).collect::<String>());
        assert!(!first_text.contains("Warmup"), "First message should NOT be 'Warmup' (agent file content)");
        println!("PASS: First message is NOT agent file content");
    }
}
