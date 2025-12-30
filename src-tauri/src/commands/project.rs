//! Project management IPC commands
//!
//! Provides Tauri commands for project and session management.

use std::sync::Mutex;

use tauri::State;

use crate::error::AppError;
use crate::models::{ImportResult, MantraSession, Project, SessionSummary};
use crate::parsers::{ClaudeParser, LogParser};
use crate::scanner::ProjectScanner;
use crate::storage::Database;

/// Application state containing the database connection
pub struct AppState {
    pub db: Mutex<Database>,
}

/// List all projects ordered by last activity
#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.list_projects().map_err(Into::into)
}

/// Get all sessions for a specific project
#[tauri::command]
pub async fn get_project_sessions(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<SessionSummary>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_project_sessions(&project_id).map_err(Into::into)
}

/// Import sessions from file paths
///
/// Parses Claude Code log files and imports them into the database.
#[tauri::command]
pub async fn import_sessions(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<ImportResult, AppError> {
    let mut db = state.db.lock().map_err(|_| AppError::LockError)?;
    let mut scanner = ProjectScanner::new(&mut db);
    let parser = ClaudeParser::new();

    let mut all_sessions = Vec::new();
    let mut parse_errors = Vec::new();

    // Parse all files
    for path in &paths {
        match parser.parse_file(path) {
            Ok(session) => all_sessions.push(session),
            Err(e) => parse_errors.push(format!("{}: {}", path, e)),
        }
    }

    // Import parsed sessions
    let mut result = scanner.scan_and_import(all_sessions)?;

    // Add parse errors to result
    result.errors.extend(parse_errors);

    Ok(result)
}

/// Import sessions from MantraSession objects directly
///
/// Used when sessions are already parsed (e.g., from string parsing).
#[tauri::command]
pub async fn import_parsed_sessions(
    state: State<'_, AppState>,
    sessions: Vec<MantraSession>,
) -> Result<ImportResult, AppError> {
    let mut db = state.db.lock().map_err(|_| AppError::LockError)?;
    let mut scanner = ProjectScanner::new(&mut db);
    scanner.scan_and_import(sessions).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionSource;

    fn create_test_state() -> AppState {
        AppState {
            db: Mutex::new(Database::new_in_memory().unwrap()),
        }
    }

    fn create_test_session(id: &str, cwd: &str) -> MantraSession {
        MantraSession::new(id.to_string(), SessionSource::Claude, cwd.to_string())
    }

    #[test]
    fn test_list_projects_empty() {
        let state = create_test_state();
        let db = state.db.lock().unwrap();
        let projects = db.list_projects().unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn test_import_and_list() {
        let state = create_test_state();
        let mut db = state.db.lock().unwrap();
        let mut scanner = ProjectScanner::new(&mut db);

        let sessions = vec![
            create_test_session("sess_1", "/home/user/project1"),
            create_test_session("sess_2", "/home/user/project2"),
        ];

        let result = scanner.scan_and_import(sessions).unwrap();
        assert_eq!(result.imported_count, 2);
        assert_eq!(result.new_projects_count, 2);

        drop(scanner); // Release mutable borrow
        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_get_project_sessions() {
        let state = create_test_state();
        let mut db = state.db.lock().unwrap();
        let mut scanner = ProjectScanner::new(&mut db);

        let sessions = vec![
            create_test_session("sess_1", "/home/user/test"),
            create_test_session("sess_2", "/home/user/test"),
        ];

        scanner.scan_and_import(sessions).unwrap();

        drop(scanner); // Release mutable borrow
        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 1);

        let project_sessions = db.get_project_sessions(&projects[0].id).unwrap();
        assert_eq!(project_sessions.len(), 2);
    }
}
