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
use crate::models::{ImportResult, MantraSession, Project, SessionSummary};
use crate::parsers::{ClaudeParser, CodexParser, CursorParser, GeminiParser, LogParser};
use crate::scanner::ProjectScanner;
use crate::storage::{Database, SearchResult};

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
/// Scans the project's cwd directory for new session files and checks
/// existing sessions for message count changes.
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

    // Build a map of existing session IDs to (message_count, source)
    // This allows us to only update sessions from the same source
    let existing_session_map: std::collections::HashMap<String, (u32, String)> = existing_sessions
        .iter()
        .map(|s| (s.id.clone(), (s.message_count, s.source.clone())))
        .collect();

    // Scan for sessions in the project directory
    let claude_parser = ClaudeParser::new();
    let gemini_parser = GeminiParser::new();
    let cursor_parser = CursorParser::new();

    let cwd_path = PathBuf::from(&cwd);
    let mut all_sessions: Vec<MantraSession> = Vec::new();

    // Try different session file locations based on tool type
    // Claude: ~/.claude/projects/{project-hash}/*.jsonl
    // Gemini: {cwd}/.gemini/history/
    // Cursor: {cwd}/.cursor/ directory

    // Detect Claude sessions
    if let Some(home) = dirs::home_dir() {
        let claude_projects_dir = home.join(".claude").join("projects");
        if claude_projects_dir.exists() {
            // Search for session files that match this project's cwd
            if let Ok(entries) = std::fs::read_dir(&claude_projects_dir) {
                for entry in entries.flatten() {
                    let project_dir = entry.path();
                    // Claude Code stores JSONL files directly in the project directory
                    // (not in a sessions/ subdirectory)
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
                                            if session.cwd == cwd {
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

    // Process sessions
    let mut new_sessions: Vec<SessionSummary> = Vec::new();
    let mut updated_sessions: Vec<UpdatedSession> = Vec::new();
    let mut unchanged_count: u32 = 0;

    eprintln!(
        "[sync_project] Processing {} sessions for project {} (force={})",
        all_sessions.len(),
        project_id,
        force
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

    // Pre-scan: Build directory -> (cwd, project_id) mapping
    // This ensures all files from the same directory belong to the same project
    let mut dir_project_map: HashMap<String, (String, String, String)> = HashMap::new(); // dir -> (cwd, project_id, project_name)

    // First pass: collect cwd from files that have it
    let mut dir_cwd_map: HashMap<String, String> = HashMap::new();
    for path in &paths {
        let path_buf = PathBuf::from(path);
        if path_buf.is_file() {
            if let Some(parent) = path_buf.parent() {
                let dir_key = parent.to_string_lossy().to_string();
                if !dir_cwd_map.contains_key(&dir_key) {
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
                            dir_cwd_map.insert(dir_key, session.cwd);
                        }
                    }
                }
            }
        }
    }

    // Second pass: create/get projects for each directory
    {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        for (dir_key, cwd) in &dir_cwd_map {
            let (project, is_new) = db.get_or_create_project(cwd)?;
            if is_new {
                total_new_projects += 1;
            }
            dir_project_map.insert(dir_key.clone(), (cwd.clone(), project.id.clone(), project.name.clone()));
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

        if path_buf.is_dir() {
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
                                let db = state.db.lock().map_err(|_| AppError::LockError)?;
                                let (project, is_new) = db.get_or_create_project(&cwd)?;
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

                        if let Some((_, project_id, project_name)) = dir_project_map.get(&dir_key) {
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

                        if let Some((_, project_id, project_name)) = dir_project_map.get(&dir_key) {
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
// Story 2.10: Global Search Command
// ============================================================================

/// Search sessions by content
///
/// Searches through all session messages for the given query.
/// Returns matching results with snippets and highlight positions.
#[tauri::command]
pub async fn search_sessions(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>, AppError> {
    let limit = limit.unwrap_or(50);

    eprintln!("[search_sessions] Query: '{}', limit: {}", query, limit);

    if query.trim().is_empty() {
        eprintln!("[search_sessions] Empty query, returning empty results");
        return Ok(Vec::new());
    }

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let results = db.search_sessions(&query, limit)?;

    eprintln!("[search_sessions] Found {} results", results.len());

    Ok(results)
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::models::sources;

    fn create_test_state() -> AppState {
        AppState {
            db: Mutex::new(Database::new_in_memory().unwrap()),
        }
    }

    fn create_test_session(id: &str, cwd: &str) -> MantraSession {
        MantraSession::new(id.to_string(), sources::CLAUDE.to_string(), cwd.to_string())
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
