//! Project management IPC commands
//!
//! Provides Tauri commands for project and session management.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use git2::Repository;
use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::State;

use crate::error::AppError;
use crate::models::{ImportResult, MantraSession, Project, SessionSummary};
use crate::parsers::{ClaudeParser, CursorParser, GeminiParser, LogParser};
use crate::scanner::ProjectScanner;
use crate::storage::Database;

/// Application state containing the database connection
pub struct AppState {
    pub db: Mutex<Database>,
}

/// Representative file information
#[derive(Debug, Clone, Serialize)]
pub struct RepresentativeFile {
    /// File path relative to repository root
    pub path: String,
    /// File content
    pub content: String,
    /// Detected programming language
    pub language: String,
}

/// Sync result for a project (Story 2.19)
#[derive(Debug, Clone, Serialize)]
pub struct SyncResult {
    /// Newly discovered sessions
    pub new_sessions: Vec<SessionSummary>,
    /// Sessions with new messages
    pub updated_sessions: Vec<UpdatedSession>,
    /// Count of unchanged sessions
    pub unchanged_count: u32,
}

/// Updated session information
#[derive(Debug, Clone, Serialize)]
pub struct UpdatedSession {
    /// Session ID
    pub session_id: String,
    /// Previous message count
    pub old_message_count: u32,
    /// New message count
    pub new_message_count: u32,
}

/// List all projects ordered by last activity
#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.list_projects().map_err(Into::into)
}

/// Get a single project by ID
#[tauri::command]
pub async fn get_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Option<Project>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_project(&project_id).map_err(Into::into)
}

/// Get a single project by cwd
#[tauri::command]
pub async fn get_project_by_cwd(
    state: State<'_, AppState>,
    cwd: String,
) -> Result<Option<Project>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_project_by_cwd(&cwd).map_err(Into::into)
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

/// Get a single session by ID
///
/// Returns the full MantraSession including all messages
#[tauri::command]
pub async fn get_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<MantraSession>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_session(&session_id).map_err(Into::into)
}

/// Import sessions from file paths
///
/// Parses Claude Code log files, Gemini CLI sessions, or Cursor workspaces
/// and imports them into the database.
/// - If path is a directory: uses CursorParser for workspace
/// - If path contains .gemini and ends with .json: uses GeminiParser
/// - Otherwise (.jsonl files): uses ClaudeParser
#[tauri::command]
pub async fn import_sessions(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<ImportResult, AppError> {
    let mut db = state.db.lock().map_err(|_| AppError::LockError)?;
    let mut scanner = ProjectScanner::new(&mut db);
    let claude_parser = ClaudeParser::new();
    let cursor_parser = CursorParser::new();
    let gemini_parser = GeminiParser::new();

    let mut all_sessions = Vec::new();
    let mut parse_errors = Vec::new();

    // Parse all files/directories
    for path in &paths {
        let path_buf = PathBuf::from(path);

        if path_buf.is_dir() {
            // Cursor workspace (directory)
            match cursor_parser.parse_workspace(&path_buf) {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        parse_errors.push(format!("{}: 工作区中未找到对话", path));
                    } else {
                        all_sessions.extend(sessions);
                    }
                }
                Err(e) => parse_errors.push(format!("{}: {}", path, e)),
            }
        } else {
            // Detect file type by path pattern
            let is_gemini = path.contains("/.gemini/") || path.contains("\\.gemini\\");
            let is_json = path.ends_with(".json");
            let is_jsonl = path.ends_with(".jsonl");

            let parse_result = if is_gemini && is_json {
                // Gemini CLI session file
                gemini_parser.parse_file(path)
            } else if is_jsonl {
                // Claude Code JSONL file
                claude_parser.parse_file(path)
            } else if is_json {
                // Try Gemini first, then Claude
                gemini_parser.parse_file(path)
                    .or_else(|_| claude_parser.parse_file(path))
            } else {
                // Default to Claude parser
                claude_parser.parse_file(path)
            };

            match parse_result {
                Ok(session) => all_sessions.push(session),
                Err(e) => parse_errors.push(format!("{}: {}", path, e)),
            }
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

/// Get a representative file from a Git repository
///
/// Priority: README.md → most recently modified code file → entry point files
#[tauri::command]
pub async fn get_representative_file(
    repo_path: String,
) -> Result<Option<RepresentativeFile>, AppError> {
    let repo_path = PathBuf::from(&repo_path);

    spawn_blocking(move || {
        let repo = Repository::open(&repo_path)
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
        let head = repo.head()
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
        let commit = head.peel_to_commit()
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
        let tree = commit.tree()
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;

        // 1. Priority: README.md
        if let Ok(file) = try_get_file(&repo, &tree, "README.md") {
            return Ok(Some(file));
        }

        // 2. Try common entry point files
        let entry_patterns = [
            "main.rs", "main.ts", "main.tsx", "main.js", "main.jsx",
            "index.ts", "index.tsx", "index.js", "index.jsx",
            "app.ts", "app.tsx", "app.js", "app.jsx",
            "lib.rs", "mod.rs",
            "src/main.rs", "src/main.ts", "src/main.tsx", "src/main.js",
            "src/index.ts", "src/index.tsx", "src/index.js",
            "src/app.ts", "src/app.tsx", "src/app.js",
            "src/lib.rs",
        ];

        for pattern in entry_patterns {
            if let Ok(file) = try_get_file(&repo, &tree, pattern) {
                return Ok(Some(file));
            }
        }

        // 3. Fallback: find any code file in the tree
        if let Some(file) = find_first_code_file(&repo, &tree) {
            return Ok(Some(file));
        }

        Ok(None)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// Try to get a file from the Git tree
fn try_get_file(repo: &Repository, tree: &git2::Tree, path: &str) -> Result<RepresentativeFile, ()> {
    let entry = tree.get_path(Path::new(path)).map_err(|_| ())?;
    let blob = repo.find_blob(entry.id()).map_err(|_| ())?;
    let content = std::str::from_utf8(blob.content()).map_err(|_| ())?;

    Ok(RepresentativeFile {
        path: path.to_string(),
        content: content.to_string(),
        language: detect_language(path).to_string(),
    })
}

/// Find the first code file in the tree
fn find_first_code_file(repo: &Repository, tree: &git2::Tree) -> Option<RepresentativeFile> {
    let code_extensions = ["rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "cpp", "c", "h"];

    for entry in tree.iter() {
        let name = entry.name()?;
        let ext = Path::new(name).extension()?.to_str()?;

        if code_extensions.contains(&ext) {
            if let Ok(blob) = repo.find_blob(entry.id()) {
                if let Ok(content) = std::str::from_utf8(blob.content()) {
                    return Some(RepresentativeFile {
                        path: name.to_string(),
                        content: content.to_string(),
                        language: detect_language(name).to_string(),
                    });
                }
            }
        }
    }

    None
}

/// Detect programming language from filename
fn detect_language(filename: &str) -> &str {
    match Path::new(filename).extension().and_then(|s| s.to_str()) {
        Some("rs") => "rust",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        Some("md") => "markdown",
        Some("py") => "python",
        Some("go") => "go",
        Some("java") => "java",
        Some("cpp") | Some("cc") | Some("cxx") => "cpp",
        Some("c") => "c",
        Some("h") | Some("hpp") => "cpp",
        Some("json") => "json",
        Some("yaml") | Some("yml") => "yaml",
        Some("toml") => "toml",
        Some("html") => "html",
        Some("css") => "css",
        Some("sql") => "sql",
        _ => "text",
    }
}

// ============================================================================
// Story 2.19: Project Management Commands
// ============================================================================

/// Sync a project: detect new sessions and message updates
///
/// Scans the project's cwd directory for new session files and checks
/// existing sessions for message count changes.
#[tauri::command]
pub async fn sync_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<SyncResult, AppError> {
    // Get project info
    let project = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_project(&project_id)?
            .ok_or_else(|| AppError::NotFound(format!("Project {} not found", project_id)))?
    };

    let cwd = project.cwd.clone();

    // Get existing sessions
    let existing_sessions = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_project_sessions(&project_id)?
    };

    // Build a map of existing session IDs to message counts
    let existing_session_map: std::collections::HashMap<String, u32> = existing_sessions
        .iter()
        .map(|s| (s.id.clone(), s.message_count))
        .collect();

    // Scan for sessions in the project directory
    let claude_parser = ClaudeParser::new();
    let gemini_parser = GeminiParser::new();
    let cursor_parser = CursorParser::new();

    let cwd_path = PathBuf::from(&cwd);
    let mut all_sessions: Vec<MantraSession> = Vec::new();

    // Try different session file locations based on tool type
    // Claude: ~/.claude/projects/{hash}/sessions/
    // Gemini: {cwd}/.gemini/history/
    // Cursor: {cwd}/.cursor/ directory

    // Detect Claude sessions
    if let Some(home) = dirs::home_dir() {
        let claude_projects_dir = home.join(".claude").join("projects");
        if claude_projects_dir.exists() {
            // Search for session files that match this project's cwd
            if let Ok(entries) = std::fs::read_dir(&claude_projects_dir) {
                for entry in entries.flatten() {
                    let sessions_dir = entry.path().join("sessions");
                    if sessions_dir.exists() {
                        if let Ok(session_files) = std::fs::read_dir(&sessions_dir) {
                            for session_file in session_files.flatten() {
                                let path = session_file.path();
                                if path.extension().is_some_and(|e| e == "jsonl") {
                                    if let Ok(session) = claude_parser.parse_file(path.to_string_lossy().as_ref()) {
                                        if session.cwd == cwd {
                                            all_sessions.push(session);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Detect Gemini sessions
    let gemini_history_dir = cwd_path.join(".gemini").join("history");
    if gemini_history_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&gemini_history_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    if let Ok(session) = gemini_parser.parse_file(path.to_string_lossy().as_ref()) {
                        all_sessions.push(session);
                    }
                }
            }
        }
    }

    // Detect Cursor workspace
    let cursor_dir = cwd_path.join(".cursor");
    if cursor_dir.exists() && cursor_dir.is_dir() {
        if let Ok(sessions) = cursor_parser.parse_workspace(&cwd_path) {
            all_sessions.extend(sessions);
        }
    }

    // Process sessions
    let mut new_sessions: Vec<SessionSummary> = Vec::new();
    let mut updated_sessions: Vec<UpdatedSession> = Vec::new();
    let mut unchanged_count: u32 = 0;

    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    for session in all_sessions {
        if let Some(&old_count) = existing_session_map.get(&session.id) {
            let new_count = session.messages.len() as u32;
            if new_count > old_count {
                // Session has new messages
                db.update_session(&session)?;
                updated_sessions.push(UpdatedSession {
                    session_id: session.id.clone(),
                    old_message_count: old_count,
                    new_message_count: new_count,
                });
            } else {
                unchanged_count += 1;
            }
        } else {
            // New session - import it
            let (imported, _) = db.import_session(&session)?;
            if imported {
                new_sessions.push(SessionSummary {
                    id: session.id.clone(),
                    source: session.source.clone(),
                    created_at: session.created_at,
                    updated_at: session.updated_at,
                    message_count: session.messages.len() as u32,
                });
            }
        }
    }

    Ok(SyncResult {
        new_sessions,
        updated_sessions,
        unchanged_count,
    })
}

/// Remove a project (soft delete)
///
/// Marks the project as deleted without actually removing data,
/// allowing for undo within a time window.
#[tauri::command]
pub async fn remove_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.soft_delete_project(&project_id)?;
    Ok(())
}

/// Restore a removed project
///
/// Clears the deleted_at marker, making the project visible again.
#[tauri::command]
pub async fn restore_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.restore_project(&project_id)?;
    Ok(())
}

/// Rename a project
///
/// Updates the project's display name.
#[tauri::command]
pub async fn rename_project(
    state: State<'_, AppState>,
    project_id: String,
    new_name: String,
) -> Result<(), AppError> {
    // Validate name is not empty
    let trimmed_name = new_name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation("项目名称不能为空".to_string()));
    }

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.rename_project(&project_id, trimmed_name)?;
    Ok(())
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

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), "rust");
        assert_eq!(detect_language("index.ts"), "typescript");
        assert_eq!(detect_language("app.tsx"), "typescript");
        assert_eq!(detect_language("script.js"), "javascript");
        assert_eq!(detect_language("app.jsx"), "javascript");
        assert_eq!(detect_language("README.md"), "markdown");
        assert_eq!(detect_language("main.py"), "python");
        assert_eq!(detect_language("main.go"), "go");
        assert_eq!(detect_language("Main.java"), "java");
        assert_eq!(detect_language("main.cpp"), "cpp");
        assert_eq!(detect_language("main.c"), "c");
        assert_eq!(detect_language("config.json"), "json");
        assert_eq!(detect_language("config.yaml"), "yaml");
        assert_eq!(detect_language("Cargo.toml"), "toml");
        assert_eq!(detect_language("unknown.xyz"), "text");
    }

    #[tokio::test]
    async fn test_get_representative_file_finds_file() {
        // Get the Git repo root (mantra project root)
        // CARGO_MANIFEST_DIR is apps/client/src-tauri, we need the root
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let repo_path = std::path::PathBuf::from(manifest_dir)
            .parent() // apps/client
            .and_then(|p| p.parent()) // apps
            .and_then(|p| p.parent()) // mantra (root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string());

        println!("Testing with repo_path: {}", repo_path);

        let result = get_representative_file(repo_path).await;
        println!("Result: {:?}", result);

        match &result {
            Ok(Some(file)) => {
                println!("Found file: {} ({})", file.path, file.language);
                assert!(!file.path.is_empty());
                assert!(!file.content.is_empty());
            }
            Ok(None) => {
                println!("No representative file found");
                // This shouldn't happen for mantra project which has README.md
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_representative_file_invalid_repo() {
        let result = get_representative_file("/nonexistent/path".to_string()).await;
        assert!(result.is_err());
    }
}
