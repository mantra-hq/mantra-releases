// Mantra Client Library
// Provides Tauri IPC commands for parsing AI conversation logs

pub mod commands;
pub mod error;
pub mod git;
pub mod models;
pub mod parsers;
pub mod scanner;
pub mod storage;

use std::sync::Mutex;

use tauri::Manager;

use commands::{
    detect_git_repo, find_commit_at_time, get_commit_info, get_file_snapshot,
    get_project_sessions, get_session, get_snapshot_at_time, import_parsed_sessions,
    import_sessions, list_projects, parse_claude_log, parse_claude_log_string, parse_log_files,
    scan_custom_directory, scan_log_directory, AppState,
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
            get_file_snapshot,
            get_snapshot_at_time,
            detect_git_repo,
            find_commit_at_time,
            get_commit_info,
            list_projects,
            get_project_sessions,
            get_session,
            import_sessions,
            import_parsed_sessions,
            scan_log_directory,
            scan_custom_directory,
            parse_log_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
