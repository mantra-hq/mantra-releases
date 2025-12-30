// Mantra Client Library
// Provides Tauri IPC commands for parsing AI conversation logs

pub mod commands;
pub mod error;
pub mod git;
pub mod models;
pub mod parsers;

use commands::{find_commit_at_time, get_commit_info, get_file_snapshot, parse_claude_log, parse_claude_log_string};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            parse_claude_log,
            parse_claude_log_string,
            get_file_snapshot,
            find_commit_at_time,
            get_commit_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
