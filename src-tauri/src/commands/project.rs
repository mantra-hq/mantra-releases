//! Project management IPC commands
//!
//! Provides Tauri commands for project and session management.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use git2::Repository;
use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Emitter, State};

use crate::error::AppError;
use crate::git::get_git_remote_url;
use crate::models::{ImportResult, MantraSession, Project, SessionSummary};
use crate::parsers::{ClaudeParser, CodexParser, CursorParser, GeminiParser, LogParser};
use crate::scanner::ProjectScanner;
use crate::storage::{Database, SearchFilters, SearchResult};

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

// ============================================================================
// Story 2.23: Import Progress Events and Cancellation
// ============================================================================

/// Global flag for import cancellation
static IMPORT_CANCELLED: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Progress event sent before processing each file
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgressEvent {
    /// Current file index (0-based)
    pub current: usize,
    /// Total number of files
    pub total: usize,
    /// Current file path being processed
    pub current_file: String,
    /// Number of successfully processed files so far
    pub success_count: usize,
    /// Number of failed files so far
    pub failure_count: usize,
}

/// Event sent after each file is processed
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFileDoneEvent {
    /// File path that was processed
    pub file_path: String,
    /// Whether the import was successful
    pub success: bool,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Project ID if successful
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Session ID if successful
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Project name if successful (Story 2.23 fix)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Whether the file was skipped (empty session)
    #[serde(default)]
    pub skipped: bool,
    /// Whether this session was imported to a newly created project
    /// - true: A new project was created for this session
    /// - false: Session was merged into an existing project
    #[serde(default)]
    pub is_new_project: bool,
}

/// Event sent when import is cancelled
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCancelledEvent {
    /// Number of files processed before cancellation
    pub processed_count: usize,
    /// Number of successful imports before cancellation
    pub success_count: usize,
    /// Number of failed imports before cancellation
    pub failure_count: usize,
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

/// Get the project that a session belongs to (Story 1.9)
///
/// This is more reliable than get_project_by_cwd when the project's cwd
/// might have been updated after the session was created.
#[tauri::command]
pub async fn get_project_by_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<Project>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_project_by_session_id(&session_id).map_err(Into::into)
}

/// Get all sessions for a specific project
///
/// Story 1.12: Uses view-based aggregation that combines:
/// 1. Sessions manually bound to the project (highest priority)
/// 2. Sessions matching project paths via original_cwd
#[tauri::command]
pub async fn get_project_sessions(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<SessionSummary>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    // Story 1.12: Use aggregated method for view-based project aggregation
    db.get_project_sessions_aggregated(&project_id).map_err(Into::into)
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
    skip_empty: bool,
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
        // Check if this is a Cursor single composer (path contains #)
        if let Some(hash_pos) = path.find('#') {
            let project_path = &path[..hash_pos];
            let composer_id = &path[hash_pos + 1..];
            let path_buf = PathBuf::from(project_path);

            match cursor_parser.parse_single_composer(&path_buf, composer_id) {
                Ok(session) => all_sessions.push(session),
                Err(e) => parse_errors.push(format!("{}: {}", path, e)),
            }
            continue;
        }

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

    // Filter out empty sessions if requested
    let mut empty_skipped = 0;
    if skip_empty {
        let initial_len = all_sessions.len();
        all_sessions.retain(|s| !s.messages.is_empty());
        empty_skipped = (initial_len - all_sessions.len()) as u32;
    }

    // Import parsed sessions
    let mut result = scanner.scan_and_import(all_sessions)?;

    // Add skipped count
    result.skipped_count += empty_skipped;

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
/// Scans the project's paths for new session files and checks
/// existing sessions for message count changes.
///
/// Story 1.12: Now uses project_paths table to scan all associated paths.
///
/// # Arguments
/// * `project_id` - The project ID to sync
/// * `force` - If true, re-parse all sessions regardless of message count changes.
///             Useful when parser bugs are fixed and data needs to be corrected.
#[tauri::command]
pub async fn sync_project(
    state: State<'_, AppState>,
    project_id: String,
    force: Option<bool>,
) -> Result<SyncResult, AppError> {
    let force = force.unwrap_or(false);

    // Get project info and all associated paths (Story 1.12)
    let (project, project_paths) = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        let project = db.get_project(&project_id)?
            .ok_or_else(|| AppError::NotFound(format!("Project {} not found", project_id)))?;
        let paths = db.get_project_paths(&project_id)?;
        (project, paths)
    };

    // Collect all paths to scan (Story 1.12: use project_paths table)
    let paths_to_scan: Vec<String> = if project_paths.is_empty() {
        // Fallback to legacy cwd if no project_paths exist
        vec![project.cwd.clone()]
    } else {
        project_paths.iter().map(|p| p.path.clone()).collect()
    };

    // Get existing sessions
    let existing_sessions = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_project_sessions(&project_id)?
    };

    // Build a map of existing session IDs to (message_count, source)
    // This allows us to only update sessions from the same source
    let existing_session_map: std::collections::HashMap<String, (u32, String)> = existing_sessions
        .iter()
        .map(|s| (s.id.clone(), (s.message_count, s.source.clone())))
        .collect();

    // Scan for sessions in all project paths
    let claude_parser = ClaudeParser::new();
    let gemini_parser = GeminiParser::new();
    let cursor_parser = CursorParser::new();

    let mut all_sessions: Vec<MantraSession> = Vec::new();

    // Story 1.12: Scan all associated paths
    for path_str in &paths_to_scan {
        let cwd_path = PathBuf::from(path_str);

        // Detect Claude sessions
        if let Some(home) = dirs::home_dir() {
            let claude_projects_dir = home.join(".claude").join("projects");
            if claude_projects_dir.exists() {
                // Search for session files that match this project's path
                if let Ok(entries) = std::fs::read_dir(&claude_projects_dir) {
                    for entry in entries.flatten() {
                        let project_dir = entry.path();
                        if project_dir.is_dir() {
                            if let Ok(session_files) = std::fs::read_dir(&project_dir) {
                                for session_file in session_files.flatten() {
                                    let path = session_file.path();
                                    if path.extension().is_some_and(|e| e == "jsonl") {
                                        // 跳过 agent- 开头的文件
                                        if path.file_name()
                                            .and_then(|n| n.to_str())
                                            .is_some_and(|n| n.starts_with("agent-"))
                                        {
                                            continue;
                                        }

                                        match claude_parser.parse_file(path.to_string_lossy().as_ref()) {
                                            Ok(session) => {
                                                eprintln!(
                                                    "[sync_project] Parsed session {} from {:?}: cwd={}, messages={}",
                                                    session.id,
                                                    path.file_name(),
                                                    session.cwd,
                                                    session.messages.len()
                                                );
                                                // Story 1.12: Match against any of the project's paths
                                                if session.cwd == *path_str {
                                                    all_sessions.push(session);
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "[sync_project] Failed to parse {:?}: {:?}",
                                                    path.file_name(),
                                                    e
                                                );
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
    }

    // Deduplicate sessions by ID (in case same session matched multiple paths)
    let mut seen_ids = std::collections::HashSet::new();
    all_sessions.retain(|s| seen_ids.insert(s.id.clone()));

    // Process sessions
    let mut new_sessions: Vec<SessionSummary> = Vec::new();
    let mut updated_sessions: Vec<UpdatedSession> = Vec::new();
    let mut unchanged_count: u32 = 0;

    eprintln!(
        "[sync_project] Processing {} sessions for project {} (force={}, paths={})",
        all_sessions.len(),
        project_id,
        force,
        paths_to_scan.len()
    );

    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    for session in all_sessions {
        if let Some((old_count, existing_source)) = existing_session_map.get(&session.id) {
            // Only update if the source matches to prevent cross-source contamination
            // This prevents a Claude session from overwriting a Cursor session's data
            if session.source != *existing_source {
                eprintln!(
                    "[sync_project] Skipping session {} - source mismatch (existing: {}, new: {})",
                    session.id,
                    existing_source,
                    session.source
                );
                continue;
            }

            let new_count = session.messages.len() as u32;
            eprintln!(
                "[sync_project] Session {}: old_count={}, new_count={}, force={}, source={}",
                session.id,
                old_count,
                new_count,
                force,
                session.source
            );
            // Update if: new messages added OR force re-parse requested
            if new_count > *old_count || force {
                // Session has updates or force re-parse
                eprintln!(
                    "[sync_project] Updating session {} with {} messages",
                    session.id,
                    session.messages.len()
                );
                db.update_session(&session)?;
                updated_sessions.push(UpdatedSession {
                    session_id: session.id.clone(),
                    old_message_count: *old_count,
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
                    is_empty: session.is_empty(),
                    title: session.metadata.title.clone(),
                    original_cwd: Some(session.cwd.clone()),
                });
            }
        }
    }

    // Story 2.29 V2: Update project is_empty status after sync
    let _ = db.update_project_is_empty(&project_id);

    Ok(SyncResult {
        new_sessions,
        updated_sessions,
        unchanged_count,
    })
}

/// Remove a project and all its sessions
///
/// Permanently deletes the project and all associated session data.
#[tauri::command]
pub async fn remove_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.delete_project(&project_id)?;
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

/// Update a project's working directory (Story 1.9)
///
/// Allows users to manually set a project's cwd for cases where:
/// - Path detection was incorrect
/// - User wants to associate sessions with a different directory
///
/// # Arguments
/// * `project_id` - The project ID to update
/// * `new_cwd` - The new working directory path
#[tauri::command]
pub async fn update_project_cwd(
    state: State<'_, AppState>,
    project_id: String,
    new_cwd: String,
) -> Result<Project, AppError> {
    use crate::git::get_git_remote_url;
    use crate::models::normalize_cwd;

    // Validate cwd is not empty
    let trimmed_cwd = new_cwd.trim();
    if trimmed_cwd.is_empty() {
        return Err(AppError::Validation("工作目录不能为空".to_string()));
    }

    // Normalize the path
    let normalized_cwd = normalize_cwd(trimmed_cwd);

    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // Check if a project with the new cwd already exists
    if let Some(existing) = db.get_project_by_cwd(&normalized_cwd)? {
        if existing.id != project_id {
            return Err(AppError::Validation(format!(
                "路径 {} 已被项目 {} 使用",
                normalized_cwd, existing.name
            )));
        }
    }

    // Update the cwd
    db.update_project_cwd(&project_id, &normalized_cwd)?;

    // Detect Git repository and remote URL for the new path
    let git_repo_path = crate::git::detect_git_repo_sync(&normalized_cwd);
    if let Some(ref repo_path) = git_repo_path {
        db.update_project_git_info(&normalized_cwd, Some(repo_path.clone()))?;

        // Also update Git remote URL
        if let Ok(Some(url)) = get_git_remote_url(std::path::Path::new(repo_path)) {
            db.update_project_git_remote(&project_id, Some(&url))?;
        }
    } else {
        db.update_project_git_info(&normalized_cwd, None)?;
    }

    // Return the updated project
    db.get_project(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("项目 {} 不存在", project_id)))
}

// ============================================================================
// Story 2.20: Import Wizard Enhancement Commands
// ============================================================================

/// Get all imported project cwd paths
///
/// Returns a list of session IDs for all imported sessions.
/// Used by ImportWizard to identify already-imported files.
#[tauri::command]
pub async fn get_imported_session_ids(
    state: State<'_, AppState>,
) -> Result<Vec<String>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_imported_session_ids().map_err(Into::into)
}

// ============================================================================
// Story 2.23: Import with Progress Events
// ============================================================================

/// Import sessions with real-time progress events
///
/// Similar to `import_sessions` but emits events for each file:
/// - `import-progress`: Before processing each file
/// - `import-file-done`: After each file is processed
/// - `import-cancelled`: When import is cancelled by user
#[tauri::command]
pub async fn import_sessions_with_progress(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
    skip_empty: bool,
) -> Result<ImportResult, AppError> {
    use std::collections::HashMap;
    use std::collections::HashSet;

    // Reset cancellation flag at start
    IMPORT_CANCELLED.store(false, Ordering::SeqCst);

    let total = paths.len();
    let mut success_count = 0usize;
    let mut failure_count = 0usize;
    let mut total_imported = 0u32;
    let mut total_skipped = 0u32;
    let mut total_new_projects = 0u32;
    let mut all_errors: Vec<String> = Vec::new();
    let mut affected_project_ids: HashSet<String> = HashSet::new(); // Track affected projects for is_empty update

    let claude_parser = ClaudeParser::new();
    let cursor_parser = CursorParser::new();
    let gemini_parser = GeminiParser::new();
    let codex_parser = CodexParser::new();

    // Pre-scan: Build directory -> (cwd, project_id, project_name, is_new) mapping
    // This ensures all files from the same directory belong to the same project
    let mut dir_project_map: HashMap<String, (String, String, String, bool)> = HashMap::new(); // dir -> (cwd, project_id, project_name, is_new)

    // First pass: collect cwd and git_remote_url from files that have them
    // Story: Use git_remote_url for cross-path project aggregation during import
    let mut dir_cwd_map: HashMap<String, (String, Option<String>)> = HashMap::new(); // dir -> (cwd, git_remote_url)
    for path in &paths {
        let path_buf = PathBuf::from(path);
        if path_buf.is_file() {
            if let Some(parent) = path_buf.parent() {
                let dir_key = parent.to_string_lossy().to_string();
                if let std::collections::hash_map::Entry::Vacant(e) = dir_cwd_map.entry(dir_key) {
                    // Try to extract cwd from this file using appropriate parser
                    let is_codex = path.contains("/.codex/") || path.contains("\\.codex\\");
                    let is_gemini = path.contains("/.gemini/") || path.contains("\\.gemini\\");
                    let is_jsonl = path.ends_with(".jsonl");
                    let is_json = path.ends_with(".json");

                    let parse_result = if is_codex && is_jsonl {
                        codex_parser.parse_file(path)
                    } else if is_gemini && is_json {
                        gemini_parser.parse_file(path)
                    } else if is_jsonl {
                        claude_parser.parse_file(path)
                            .or_else(|_| codex_parser.parse_file(path))
                    } else {
                        claude_parser.parse_file(path)
                    };

                    if let Ok(session) = parse_result {
                        if !session.cwd.is_empty() {
                            // Extract git_remote_url for cross-path project aggregation
                            let git_url = get_git_remote_url(Path::new(&session.cwd)).ok().flatten();
                            e.insert((session.cwd, git_url));
                        }
                    }
                }
            }
        }
    }

    // Second pass: create/get projects for each directory using git_remote_url for cross-path aggregation
    {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        for (dir_key, (cwd, git_url)) in &dir_cwd_map {
            // Use find_or_create_project with git_url for proper cross-path project merging
            let (project, is_new) = db.find_or_create_project(cwd, git_url.as_deref())?;
            if is_new {
                total_new_projects += 1;
            }
            dir_project_map.insert(dir_key.clone(), (cwd.clone(), project.id.clone(), project.name.clone(), is_new));
        }
    }

    for (index, path) in paths.iter().enumerate() {
        // Check for cancellation
        if IMPORT_CANCELLED.load(Ordering::SeqCst) {
            // Emit cancellation event
            let _ = app_handle.emit("import-cancelled", ImportCancelledEvent {
                processed_count: index,
                success_count,
                failure_count,
            });
            break;
        }

        // Emit progress event before processing
        let _ = app_handle.emit("import-progress", ImportProgressEvent {
            current: index,
            total,
            current_file: path.clone(),
            success_count,
            failure_count,
        });

        let path_buf = PathBuf::from(path);
        let mut file_success = false;
        let mut file_error: Option<String> = None;
        let mut file_project_id: Option<String> = None;
        let mut file_session_id: Option<String> = None;
        let mut file_project_name: Option<String> = None;
        let mut file_skipped = false;
        let mut file_is_new_project = false; // Track if session is imported to a new vs existing project

        // Check if this is a Cursor single composer (path contains #)
        if let Some(hash_pos) = path.find('#') {
            let project_path = &path[..hash_pos];
            let composer_id = &path[hash_pos + 1..];
            let path_buf = PathBuf::from(project_path);

            // Cursor single composer
            match cursor_parser.parse_single_composer(&path_buf, composer_id) {
                Ok(session) => {
                    // Get or create project
                    let db = state.db.lock().map_err(|_| AppError::LockError)?;
                    let git_url = get_git_remote_url(Path::new(&session.cwd)).ok().flatten();
                    let (project, is_new) = db.find_or_create_project(&session.cwd, git_url.as_deref())?;
                    if is_new {
                        total_new_projects += 1;
                    }

                    // Check for duplicate
                    match db.get_session(&session.id) {
                        Ok(Some(_)) => {
                            total_skipped += 1;
                            file_skipped = true;
                        }
                        Ok(None) => {
                            if let Ok(_) = db.insert_session(&session, &project.id) {
                                total_imported += 1;
                                let _ = db.update_project_last_activity(&project.id, session.updated_at);
                            }
                        }
                        Err(_) => {}
                    }

                    file_success = true;
                    file_project_id = Some(project.id.clone());
                    file_project_name = Some(project.name.clone());
                    file_session_id = Some(session.id.clone());
                    file_is_new_project = is_new;
                    affected_project_ids.insert(project.id);
                }
                Err(e) => {
                    file_error = Some(e.to_string());
                    all_errors.push(format!("{}: {}", path, e));
                }
            }
        } else if PathBuf::from(path).is_dir() {
            // Cursor workspace (directory)
            match cursor_parser.parse_workspace(&path_buf) {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        file_error = Some("工作区中未找到对话".to_string());
                        all_errors.push(format!("{}: 工作区中未找到对话", path));
                    } else {
                        // Filter empty sessions if needed
                        let mut sessions_to_import = sessions;
                        if skip_empty {
                            sessions_to_import.retain(|s| !s.messages.is_empty());
                        }

                        if sessions_to_import.is_empty() && skip_empty {
                            file_success = true;
                            file_skipped = true;
                            total_skipped += 1;
                        } else if !sessions_to_import.is_empty() {
                            // Get cwd from first session with non-empty cwd
                            let workspace_cwd = sessions_to_import.iter()
                                .find(|s| !s.cwd.is_empty())
                                .map(|s| s.cwd.clone());

                            if let Some(cwd) = workspace_cwd {
                                // Use unified logic: get/create project first, then import all sessions
                                // Extract git_remote_url for cross-path project aggregation
                                let git_url = get_git_remote_url(Path::new(&cwd)).ok().flatten();
                                let db = state.db.lock().map_err(|_| AppError::LockError)?;
                                let (project, is_new) = db.find_or_create_project(&cwd, git_url.as_deref())?;
                                if is_new {
                                    total_new_projects += 1;
                                }

                                // Import all sessions with the same project_id
                                for session in &sessions_to_import {
                                    match db.get_session(&session.id) {
                                        Ok(Some(_)) => {
                                            total_skipped += 1;
                                        }
                                        Ok(None) => {
                                            if let Ok(_) = db.insert_session(session, &project.id) {
                                                total_imported += 1;
                                                let _ = db.update_project_last_activity(&project.id, session.updated_at);
                                            }
                                        }
                                        Err(_) => {}
                                    }
                                }

                                file_success = true;
                                file_project_id = Some(project.id.clone());
                                file_project_name = Some(project.name.clone());
                                file_is_new_project = is_new;
                                if let Some(first_session) = sessions_to_import.first() {
                                    file_session_id = Some(first_session.id.clone());
                                }
                            } else {
                                // Fallback: no valid cwd found, use old logic
                                let mut db = state.db.lock().map_err(|_| AppError::LockError)?;
                                let result = db.import_sessions(&sessions_to_import)?;

                                total_imported += result.imported_count;
                                total_skipped += result.skipped_count;
                                total_new_projects += result.new_projects_count;
                                all_errors.extend(result.errors);

                                file_success = true;
                                if let Some(first_session) = sessions_to_import.first() {
                                    if let Ok(Some(project)) = db.get_project_by_cwd(&first_session.cwd) {
                                        file_project_id = Some(project.id);
                                        file_project_name = Some(project.name);
                                    }
                                    file_session_id = Some(first_session.id.clone());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if skip_empty && e.is_skippable() {
                        file_success = true;
                        file_skipped = true;
                        file_error = Some(e.to_string());
                        total_skipped += 1;
                    } else {
                        file_error = Some(e.to_string());
                        all_errors.push(format!("{}: {}", path, e));
                    }
                }
            }
        } else {
            // Detect file type by path pattern
            let is_codex = path.contains("/.codex/") || path.contains("\\.codex\\");
            let is_gemini = path.contains("/.gemini/") || path.contains("\\.gemini\\");
            let is_json = path.ends_with(".json");
            let is_jsonl = path.ends_with(".jsonl");

            let parse_result = if is_codex && is_jsonl {
                codex_parser.parse_file(path)
            } else if is_gemini && is_json {
                gemini_parser.parse_file(path)
            } else if is_jsonl {
                claude_parser.parse_file(path)
                    .or_else(|_| codex_parser.parse_file(path))
            } else if is_json {
                gemini_parser.parse_file(path)
                    .or_else(|_| claude_parser.parse_file(path))
            } else {
                claude_parser.parse_file(path)
            };

            match parse_result {
                Ok(session) => {
                    if skip_empty && session.messages.is_empty() {
                        file_success = true;
                        file_skipped = true;
                        total_skipped += 1;
                    } else {
                        // Get pre-determined project for this directory
                        let dir_key = PathBuf::from(path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();

                        if let Some((_, project_id, project_name, is_new)) = dir_project_map.get(&dir_key) {
                            // Use pre-determined project_id - session data saved as-is
                            let db = state.db.lock().map_err(|_| AppError::LockError)?;

                            // Check for duplicate
                            match db.get_session(&session.id) {
                                Ok(Some(_)) => {
                                    // Session already exists, skip
                                    total_skipped += 1;
                                    file_skipped = true;
                                }
                                Ok(None) => {
                                    // Insert session with the directory's project_id
                                    match db.insert_session(&session, project_id) {
                                        Ok(_) => {
                                            total_imported += 1;
                                            // Update project last_activity
                                            let _ = db.update_project_last_activity(project_id, session.updated_at);
                                        }
                                        Err(e) => {
                                            all_errors.push(format!("{}: {}", path, e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    all_errors.push(format!("{}: {}", path, e));
                                }
                            }

                            file_success = true;
                            file_session_id = Some(session.id.clone());
                            file_project_id = Some(project_id.clone());
                            file_project_name = Some(project_name.clone());
                            file_is_new_project = *is_new;
                        } else {
                            // Fallback: use import_sessions for files without pre-determined project
                            let mut db = state.db.lock().map_err(|_| AppError::LockError)?;
                            let result = db.import_sessions(&[session.clone()])?;

                            total_imported += result.imported_count;
                            total_skipped += result.skipped_count;
                            total_new_projects += result.new_projects_count;
                            all_errors.extend(result.errors);

                            file_success = true;
                            file_session_id = Some(session.id.clone());

                            if let Ok(Some(project)) = db.get_project_by_cwd(&session.cwd) {
                                file_project_id = Some(project.id);
                                file_project_name = Some(project.name);
                            }
                        }
                    }
                }
                Err(e) => {
                    // Check if this is a skippable error (empty session, system events only, etc.)
                    if e.is_skippable() {
                        // Story 2.29 V2: Import empty session with directory's project
                        let dir_key = PathBuf::from(path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();

                        if let Some((_, project_id, project_name, is_new)) = dir_project_map.get(&dir_key) {
                            // Create empty session from parser and import it
                            if let Ok(empty_session) = claude_parser.parse_file(path) {
                                let db = state.db.lock().map_err(|_| AppError::LockError)?;
                                match db.get_session(&empty_session.id) {
                                    Ok(Some(_)) => {
                                        total_skipped += 1;
                                    }
                                    Ok(None) => {
                                        if let Ok(_) = db.insert_session(&empty_session, project_id) {
                                            total_imported += 1;
                                        }
                                    }
                                    Err(_) => {}
                                }
                                file_project_id = Some(project_id.clone());
                                file_project_name = Some(project_name.clone());
                                file_session_id = Some(empty_session.id.clone());
                                file_is_new_project = *is_new;
                            }
                            file_success = true;
                        } else {
                            file_success = true;
                            file_skipped = true;
                            total_skipped += 1;
                        }
                        file_error = Some(e.to_string());
                    } else {
                        file_error = Some(e.to_string());
                        all_errors.push(format!("{}: {}", path, e));
                    }
                }
            }
        }

        // Update counts
        if file_success {
            success_count += 1;
        } else {
            failure_count += 1;
        }

        // Emit file done event with real project_id and project_name
        let _ = app_handle.emit("import-file-done", ImportFileDoneEvent {
            file_path: path.clone(),
            success: file_success,
            error: file_error,
            project_id: file_project_id.clone(),
            session_id: file_session_id,
            project_name: file_project_name,
            skipped: file_skipped,
            is_new_project: file_is_new_project,
        });

        // Track affected project for is_empty update
        if let Some(pid) = file_project_id {
            affected_project_ids.insert(pid);
        }
    }

    // Story 2.29 V2: Update is_empty status for all affected projects
    {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        for project_id in &affected_project_ids {
            let _ = db.update_project_is_empty(project_id);
        }
    }

    // Return aggregated result
    Ok(ImportResult {
        imported_count: total_imported,
        skipped_count: total_skipped,
        new_projects_count: total_new_projects,
        errors: all_errors,
    })
}


/// Cancel the current import operation
///
/// Sets a flag that will be checked by import_sessions_with_progress
/// to stop processing further files.
#[tauri::command]
pub async fn cancel_import() -> Result<(), AppError> {
    IMPORT_CANCELLED.store(true, Ordering::SeqCst);
    Ok(())
}

// ============================================================================
// ============================================================================
// Story 1.12: View-based Project Aggregation Commands
// ============================================================================

/// Add a path to a project (Story 1.12 - AC1)
///
/// Associates an additional path with a project, enabling multi-path aggregation.
///
/// # Arguments
/// * `project_id` - The project to add the path to
/// * `path` - The path to add (will be normalized)
/// * `is_primary` - Whether this should be the primary path (optional, defaults to false)
#[tauri::command]
pub async fn add_project_path(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
    is_primary: Option<bool>,
) -> Result<crate::models::ProjectPath, AppError> {
    let is_primary = is_primary.unwrap_or(false);
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.add_project_path(&project_id, &path, is_primary)
        .map_err(Into::into)
}

/// Remove a path from a project (Story 1.12 - AC1)
///
/// Removes the specified path association from a project.
///
/// # Arguments
/// * `path_id` - The project_path ID to remove
#[tauri::command]
pub async fn remove_project_path(
    state: State<'_, AppState>,
    path_id: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.remove_project_path(&path_id).map_err(Into::into)
}

/// Get all paths for a project (Story 1.12 - AC1)
///
/// Returns all paths associated with a project, ordered by primary status then creation time.
///
/// # Arguments
/// * `project_id` - The project to get paths for
#[tauri::command]
pub async fn get_project_paths(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<crate::models::ProjectPath>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_project_paths(&project_id).map_err(Into::into)
}

/// Get logical project statistics grouped by physical path (Story 1.12 - AC9)
///
/// Returns aggregated statistics for each unique physical path across all projects.
/// This enables the view layer to display "logical projects" that combine sessions
/// from different import sources (Claude, Gemini, Cursor, etc.) that share the same path.
#[tauri::command]
pub async fn get_logical_project_stats(
    state: State<'_, AppState>,
) -> Result<Vec<crate::storage::LogicalProjectStats>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_logical_project_stats().map_err(Into::into)
}

/// Get all sessions for a physical path across all projects (Story 1.12 - AC9)
///
/// Returns sessions from all projects that have the given physical path associated.
/// This enables the view layer to display all sessions for a "logical project".
#[tauri::command]
pub async fn get_sessions_by_physical_path(
    state: State<'_, AppState>,
    physical_path: String,
) -> Result<Vec<crate::models::SessionSummary>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_sessions_by_physical_path(&physical_path)
        .map_err(Into::into)
}

/// Get all projects that share a physical path (Story 1.12 - AC9)
///
/// Returns all projects that have the given physical path associated.
/// Useful for displaying which import sources contributed to a logical project.
#[tauri::command]
pub async fn get_projects_by_physical_path(
    state: State<'_, AppState>,
    physical_path: String,
) -> Result<Vec<crate::models::Project>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_projects_by_physical_path(&physical_path)
        .map_err(Into::into)
}

/// Bind a session to a project manually (Story 1.12 - AC3)
///
/// Creates a manual binding that takes priority over path-based matching.
///
/// # Arguments
/// * `session_id` - The session to bind
/// * `project_id` - The project to bind the session to
#[tauri::command]
pub async fn bind_session_to_project(
    state: State<'_, AppState>,
    session_id: String,
    project_id: String,
) -> Result<crate::models::SessionBinding, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.bind_session_to_project(&session_id, &project_id)
        .map_err(Into::into)
}

/// Unbind a session from its manual project binding (Story 1.12 - AC3)
///
/// Removes the manual binding, allowing the session to fall back to path-based matching.
///
/// # Arguments
/// * `session_id` - The session to unbind
#[tauri::command]
pub async fn unbind_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.unbind_session(&session_id).map_err(Into::into)
}

/// Get unassigned sessions (Story 1.12 - AC5)
///
/// Returns sessions that have no manual binding and no matching path in project_paths.
/// These sessions should be displayed in an "Unassigned" group in the UI.
#[tauri::command]
pub async fn get_unassigned_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<SessionSummary>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_unassigned_sessions().map_err(Into::into)
}

/// Set the primary path for a project (Story 1.12 - AC1)
///
/// Updates which path is considered the primary path for a project.
/// This replaces the deprecated update_project_cwd functionality.
///
/// # Arguments
/// * `project_id` - The project to update
/// * `path` - The path to set as primary (will be normalized)
#[tauri::command]
pub async fn set_project_primary_path(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
) -> Result<crate::models::ProjectPath, AppError> {
    use crate::models::normalize_cwd;

    // Validate path is not empty
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return Err(AppError::Validation("路径不能为空".to_string()));
    }

    let normalized_path = normalize_cwd(trimmed_path);
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // Check if this path already exists for this project
    let existing_paths = db.get_project_paths(&project_id)?;

    if let Some(existing) = existing_paths.iter().find(|p| p.path == normalized_path) {
        // Path already exists, just set it as primary if not already
        if existing.is_primary {
            return Ok(existing.clone());
        }
        // Demote current primary and promote this one
        db.connection().execute(
            "UPDATE project_paths SET is_primary = 0 WHERE project_id = ?1 AND is_primary = 1",
            rusqlite::params![project_id],
        ).map_err(|e| AppError::Internal(e.to_string()))?;
        db.connection().execute(
            "UPDATE project_paths SET is_primary = 1 WHERE id = ?1",
            rusqlite::params![existing.id],
        ).map_err(|e| AppError::Internal(e.to_string()))?;

        let mut updated = existing.clone();
        updated.is_primary = true;
        return Ok(updated);
    }

    // Add new path as primary
    db.add_project_path(&project_id, &normalized_path, true)
        .map_err(Into::into)
}

// ============================================================================
// Story 1.13: Logical Project Rename Commands
// ============================================================================

/// Rename a logical project by setting a custom display name (Story 1.13 - AC2, AC3)
///
/// Sets a custom name for a logical project identified by its physical path.
/// The custom name is stored in the `logical_project_names` table and takes
/// priority over the default name extracted from the path.
///
/// # Arguments
/// * `physical_path` - The physical path of the logical project
/// * `new_name` - The new display name to set
///
/// # Returns
/// Success or error
#[tauri::command]
pub async fn rename_logical_project(
    state: State<'_, AppState>,
    physical_path: String,
    new_name: String,
) -> Result<(), AppError> {
    // Validate name is not empty
    let trimmed_name = new_name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation("项目名称不能为空".to_string()));
    }

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.set_logical_project_name(&physical_path, trimmed_name)
        .map_err(Into::into)
}

/// Reset a logical project's name to default (Story 1.13 - AC4)
///
/// Deletes the custom name for a logical project, reverting to the default
/// name extracted from the physical path.
///
/// # Arguments
/// * `physical_path` - The physical path of the logical project
///
/// # Returns
/// Success or error (NotFound if no custom name exists)
#[tauri::command]
pub async fn reset_logical_project_name(
    state: State<'_, AppState>,
    physical_path: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.delete_logical_project_name(&physical_path)
        .map_err(Into::into)
}

// Story 2.10: Global Search Command
// Story 2.33: Enhanced with filters support
// ============================================================================

/// Search sessions by content with optional filters
///
/// Searches through all session messages for the given query.
/// Returns matching results with snippets and highlight positions.
///
/// Story 2.33: Added filters parameter for:
/// - Content type filtering (code/conversation/all)
/// - Project filtering
/// - Time range filtering
#[tauri::command]
pub async fn search_sessions(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    filters: Option<SearchFilters>,
) -> Result<Vec<SearchResult>, AppError> {
    let limit = limit.unwrap_or(50);
    let filters = filters.unwrap_or_default();

    eprintln!(
        "[search_sessions] Query: '{}', limit: {}, filters: {:?}",
        query, limit, filters
    );

    if query.trim().is_empty() {
        eprintln!("[search_sessions] Empty query, returning empty results");
        return Ok(Vec::new());
    }

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let results = db.search_sessions_with_filters(&query, limit, &filters)?;

    eprintln!("[search_sessions] Found {} results", results.len());

    Ok(results)
}

#[cfg(test)]
mod tests;
