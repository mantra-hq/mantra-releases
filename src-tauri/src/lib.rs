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
    get_project, get_project_by_cwd, get_project_sessions, get_representative_file,
    get_session, get_snapshot_at_time, import_parsed_sessions,
    import_sessions, list_projects, parse_claude_log, parse_claude_log_string,
    parse_cursor_log, parse_cursor_all, parse_gemini_log, parse_gemini_log_string,
    parse_gemini_all, parse_gemini_project, parse_log_files,
    sanitize_session, sanitize_text, validate_regex, scan_custom_directory, scan_log_directory, AppState,
    list_tree_at_commit, list_files_at_commit,
    // Story 2.19: Project management commands
    sync_project, remove_project, restore_project, rename_project,
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
            get_file_snapshot,
            get_file_at_head,
            get_snapshot_at_time,
            detect_git_repo,
            find_commit_at_time,
            get_commit_info,
            list_projects,
            get_project,
            get_project_by_cwd,
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
            // Story 2.19: Project management
            sync_project,
            remove_project,
            restore_project,
            rename_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
