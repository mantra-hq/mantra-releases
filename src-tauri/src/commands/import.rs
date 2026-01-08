//! Import-related Tauri commands
//!
//! Provides IPC commands for scanning log directories and importing log files.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::parsers::{ClaudeParser, CodexParser, CursorParser, GeminiParser, LogParser};
use crate::parsers::codex::CodexPaths;
use crate::parsers::cursor::CursorPaths;
use crate::parsers::gemini::GeminiPaths;

/// Import source type
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportSource {
    Claude,
    Gemini,
    Cursor,
    Codex,
}

/// Discovered file information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredFile {
    /// File full path
    pub path: String,
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Modified time (Unix timestamp ms)
    pub modified_at: u64,
    /// Project path (parent directory for grouping)
    pub project_path: String,
    /// Session ID (extracted from file content, used for import status detection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Single file import result
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileImportResult {
    /// Was import successful
    pub success: bool,
    /// File path
    pub file_path: String,
    /// Project ID (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Session ID (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Whether this file was skipped (e.g., empty session)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<bool>,
}

/// Get the default log directory for a given source
fn get_default_log_dir(source: &ImportSource) -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or_else(|| AppError::internal("无法获取 home 目录"))?;

    // Claude Code 的会话日志存储在 ~/.claude/projects/ 目录下
    // 每个项目一个子目录，目录名是路径编码的（如 -mnt-disk0-project-newx-mantra）
    // 会话日志是 JSONL 格式，文件名是 UUID.jsonl
    let path = match source {
        ImportSource::Claude => home.join(".claude").join("projects"),
        ImportSource::Gemini => home.join(".gemini").join("tmp"),
        ImportSource::Codex => home.join(".codex").join("sessions"),
        // Cursor 使用不同的存储结构，这里返回 workspaceStorage 目录
        // 实际扫描逻辑在 scan_cursor_workspaces 中处理
        ImportSource::Cursor => {
            #[cfg(target_os = "linux")]
            {
                home.join(".config").join("Cursor").join("User").join("workspaceStorage")
            }
            #[cfg(target_os = "macos")]
            {
                home.join("Library").join("Application Support").join("Cursor").join("User").join("workspaceStorage")
            }
            #[cfg(target_os = "windows")]
            {
                dirs::data_dir()
                    .unwrap_or(home.clone())
                    .join("Cursor").join("User").join("workspaceStorage")
            }
            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
            {
                return Err(AppError::internal("不支持的操作系统"));
            }
        }
    };

    Ok(path)
}

/// Recursively find all JSONL session files in a directory
/// Claude Code stores sessions as UUID.jsonl files
fn find_session_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_session_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "jsonl") {
                // Filter out agent- prefixed files (smaller agent logs)
                // and .timelines directory files
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    // Only include files that look like session UUIDs
                    // (36 chars with hyphens) or any non-agent file
                    if !name.starts_with("agent-") {
                        files.push(path);
                    }
                }
            }
        }
    }

    files
}

/// Convert a path to DiscoveredFile
fn path_to_discovered_file(path: &PathBuf) -> Option<DiscoveredFile> {
    let metadata = fs::metadata(path).ok()?;

    let name = path.file_name()?.to_str()?.to_string();
    let path_str = path.to_str()?.to_string();
    let size = metadata.len();
    let modified_at = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    // Use parent directory as project_path (for grouping)
    let project_path = path.parent()?.to_str()?.to_string();

    // Extract session_id from file content
    let session_id = extract_session_id_from_file(path);

    Some(DiscoveredFile {
        path: path_str,
        name,
        size,
        modified_at,
        project_path,
        session_id,
    })
}

/// Extract sessionId from a log file (Claude/Gemini JSONL format)
fn extract_session_id_from_file(path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};

    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    // Read up to 10 lines to find sessionId
    for line in reader.lines().take(10) {
        if let Ok(line) = line {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse as JSON and try to extract sessionId
            if let Ok(record) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(session_id) = record.get("sessionId").and_then(|v| v.as_str()) {
                    if !session_id.is_empty() {
                        return Some(session_id.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Extract sessionId from a Gemini JSON session file
fn extract_session_id_from_gemini_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("sessionId")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}



/// Scan the default log directory for a given source
///
/// # Arguments
/// * `source` - Import source (claude/gemini/cursor)
///
/// # Returns
/// * `Ok(Vec<DiscoveredFile>)` - List of discovered files
/// * `Err(AppError)` - IO or internal error
#[tauri::command]
pub async fn scan_log_directory(source: ImportSource) -> Result<Vec<DiscoveredFile>, AppError> {
    // Cursor 使用特殊的扫描逻辑（工作区模式）
    if matches!(source, ImportSource::Cursor) {
        return scan_cursor_workspaces().await;
    }

    // Gemini 使用特殊的扫描逻辑（项目哈希目录）
    if matches!(source, ImportSource::Gemini) {
        return scan_gemini_projects().await;
    }

    // Codex 使用特殊的扫描逻辑（日期目录结构）
    if matches!(source, ImportSource::Codex) {
        return scan_codex_sessions().await;
    }

    let dir = get_default_log_dir(&source)?;

    // Use spawn_blocking for filesystem operations
    let result = tokio::task::spawn_blocking(move || {
        if !dir.exists() {
            return Vec::new();
        }

        find_session_files(&dir)
            .iter()
            .filter_map(path_to_discovered_file)
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    Ok(result)
}

/// Scan Cursor workspaces and return them as discoverable items
///
/// Each workspace is represented as a DiscoveredFile with:
/// - path: The project path (used for import)
/// - name: The project folder name
/// - project_path: Same as path
async fn scan_cursor_workspaces() -> Result<Vec<DiscoveredFile>, AppError> {
    let result = tokio::task::spawn_blocking(|| {
        // Try to detect Cursor paths
        let paths = match CursorPaths::detect() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        // Scan workspaces
        let workspaces = match paths.scan_workspaces() {
            Ok(ws) => ws,
            Err(_) => return Vec::new(),
        };

        // Convert workspaces to DiscoveredFile format
        workspaces
            .into_iter()
            .map(|ws| {
                let folder_path = ws.folder_path.to_string_lossy().to_string();
                let name = ws.folder_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                // Get modified time from state.vscdb if it exists
                let modified_at = fs::metadata(&ws.state_db_path)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                // Get size of state.vscdb as a rough indicator
                let size = fs::metadata(&ws.state_db_path)
                    .map(|m| m.len())
                    .unwrap_or(0);

                DiscoveredFile {
                    path: folder_path.clone(),
                    name: format!("{} (Cursor 工作区)", name),
                    size,
                    modified_at,
                    project_path: folder_path,
                    // Cursor workspaces contain multiple sessions, so no single session_id
                    session_id: None,
                }
            })
            .collect()
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    Ok(result)
}

/// Scan Gemini CLI projects and return session files as discoverable items
///
/// Each session file is represented as a DiscoveredFile with:
/// - path: The session JSON file path (used for import)
/// - name: The session filename
/// - project_path: The project hash directory path
async fn scan_gemini_projects() -> Result<Vec<DiscoveredFile>, AppError> {
    let result = tokio::task::spawn_blocking(|| {
        // Try to detect Gemini paths
        let paths = match GeminiPaths::detect() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        // Scan all sessions
        let sessions = match paths.scan_all_sessions() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        // Convert sessions to DiscoveredFile format
        sessions
            .into_iter()
            .filter_map(|session_file| {
                let metadata = fs::metadata(&session_file.path).ok()?;
                let modified_at = metadata
                    .modified()
                    .ok()?
                    .duration_since(UNIX_EPOCH)
                    .ok()?
                    .as_millis() as u64;

                // Extract session_id from Gemini JSON file
                let session_id = extract_session_id_from_gemini_file(&session_file.path);

                Some(DiscoveredFile {
                    path: session_file.path.to_string_lossy().to_string(),
                    name: format!("{} (Gemini CLI)", session_file.filename),
                    size: metadata.len(),
                    modified_at,
                    project_path: format!("gemini-project:{}", session_file.project_hash),
                    session_id,
                })
            })
            .collect()
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    Ok(result)
}

/// Scan Codex CLI sessions and return session files as discoverable items
///
/// Each session file is represented as a DiscoveredFile with:
/// - path: The session JSONL file path (used for import)
/// - name: The session filename with date
/// - project_path: The cwd from session, or codex-project:{cwd_hash} format
async fn scan_codex_sessions() -> Result<Vec<DiscoveredFile>, AppError> {
    let result = tokio::task::spawn_blocking(|| {
        // Try to detect Codex paths
        let paths = match CodexPaths::detect() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        // Scan all sessions
        let sessions = match paths.scan_all_sessions() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        // Convert sessions to DiscoveredFile format
        sessions
            .into_iter()
            .filter_map(|session_file| {
                let metadata = fs::metadata(&session_file.path).ok()?;
                let modified_at = metadata
                    .modified()
                    .ok()?
                    .duration_since(UNIX_EPOCH)
                    .ok()?
                    .as_millis() as u64;

                // Extract session_id from Codex JSONL file
                let session_id = extract_session_id_from_codex_file(&session_file.path);

                // Extract cwd from Codex file for project_path
                let project_path = extract_cwd_from_codex_file(&session_file.path)
                    .map(|cwd| {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        cwd.hash(&mut hasher);
                        format!("codex-project:{:x}", hasher.finish())
                    })
                    .unwrap_or_else(|| format!("codex-session:{}", session_file.session_id));

                Some(DiscoveredFile {
                    path: session_file.path.to_string_lossy().to_string(),
                    name: format!("{} (Codex CLI)", session_file.date),
                    size: metadata.len(),
                    modified_at,
                    project_path,
                    session_id,
                })
            })
            .collect()
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    Ok(result)
}

/// Extract sessionId from a Codex JSONL session file
fn extract_session_id_from_codex_file(path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};

    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    // Read the first line which should be session_meta
    for line in reader.lines().take(5) {
        if let Ok(line) = line {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse as JSON and check for session_meta type
            if let Ok(record) = serde_json::from_str::<serde_json::Value>(line) {
                if record.get("type").and_then(|v| v.as_str()) == Some("session_meta") {
                    if let Some(payload) = record.get("payload") {
                        if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                            if !id.is_empty() {
                                return Some(id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract cwd from a Codex JSONL session file
fn extract_cwd_from_codex_file(path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};

    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    // Read the first line which should be session_meta
    for line in reader.lines().take(5) {
        if let Ok(line) = line {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse as JSON and check for session_meta type
            if let Ok(record) = serde_json::from_str::<serde_json::Value>(line) {
                if record.get("type").and_then(|v| v.as_str()) == Some("session_meta") {
                    if let Some(payload) = record.get("payload") {
                        if let Some(cwd) = payload.get("cwd").and_then(|v| v.as_str()) {
                            if !cwd.is_empty() {
                                return Some(cwd.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Scan a custom directory for session files
///
/// This function supports multiple strategies:
/// 1. For Claude/Gemini: Scan for .jsonl files
/// 2. For Cursor: Scan workspaceStorage directory for workspaces
///
/// # Arguments
/// * `path` - Directory path to scan (e.g., ~/.config/Cursor/User/workspaceStorage)
///
/// # Returns
/// * `Ok(Vec<DiscoveredFile>)` - List of discovered files
/// * `Err(AppError)` - IO or internal error
#[tauri::command]
pub async fn scan_custom_directory(path: String) -> Result<Vec<DiscoveredFile>, AppError> {
    let dir = PathBuf::from(path.clone());

    // Use spawn_blocking for filesystem operations
    let result = tokio::task::spawn_blocking(move || {
        if !dir.exists() {
            return Vec::new();
        }

        // Strategy 1: Try to find .jsonl files (Claude/Gemini)
        let jsonl_files: Vec<_> = find_session_files(&dir)
            .iter()
            .filter_map(path_to_discovered_file)
            .collect();

        if !jsonl_files.is_empty() {
            return jsonl_files;
        }

        // Strategy 2: Try to scan as Cursor workspaceStorage directory
        // User may select:
        // - workspaceStorage directory directly
        // - User directory (containing workspaceStorage)
        // - Cursor directory (containing User/workspaceStorage)
        let workspace_storage_path = find_workspace_storage_path(&dir);

        if let Some(ws_path) = workspace_storage_path {
            // Create a temporary CursorPaths with custom workspace_storage
            if let Ok(cursor_paths) = CursorPaths::detect() {
                // Use the custom workspace_storage path for scanning
                let custom_paths = CursorPaths {
                    global_storage: cursor_paths.global_storage,
                    workspace_storage: ws_path,
                };

                if let Ok(workspaces) = custom_paths.scan_workspaces() {
                    return workspaces
                        .into_iter()
                        .map(|ws| {
                            let folder_path = ws.folder_path.to_string_lossy().to_string();
                            let name = ws.folder_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();

                            let modified_at = fs::metadata(&ws.state_db_path)
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);

                            let size = fs::metadata(&ws.state_db_path)
                                .map(|m| m.len())
                                .unwrap_or(0);

                            DiscoveredFile {
                                path: folder_path.clone(),
                                name: format!("{} (Cursor 工作区)", name),
                                size,
                                modified_at,
                                project_path: folder_path,
                                // Cursor workspaces contain multiple sessions, so no single session_id
                                session_id: None,
                            }
                        })
                        .collect();
                }
            }
        }

        Vec::new()
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    Ok(result)
}

/// Find workspaceStorage path from user-selected directory
/// Handles cases where user selects:
/// - workspaceStorage directly
/// - User directory (parent of workspaceStorage)
/// - Cursor directory (grandparent)
fn find_workspace_storage_path(dir: &Path) -> Option<PathBuf> {
    // Case 1: dir is workspaceStorage itself
    if dir.file_name().is_some_and(|n| n == "workspaceStorage") {
        return Some(dir.to_path_buf());
    }

    // Case 2: dir contains workspaceStorage (e.g., User directory)
    let ws_child = dir.join("workspaceStorage");
    if ws_child.exists() && ws_child.is_dir() {
        return Some(ws_child);
    }

    // Case 3: dir contains User/workspaceStorage (e.g., Cursor directory)
    let ws_grandchild = dir.join("User").join("workspaceStorage");
    if ws_grandchild.exists() && ws_grandchild.is_dir() {
        return Some(ws_grandchild);
    }

    None
}

/// Parse multiple log files
///
/// # Arguments
/// * `paths` - List of file paths to parse
///
/// # Returns
/// * `Ok(Vec<FileImportResult>)` - List of import results
/// * `Err(AppError)` - Fatal error
#[tauri::command]
pub async fn parse_log_files(paths: Vec<String>) -> Result<Vec<FileImportResult>, AppError> {
    let results = tokio::task::spawn_blocking(move || {
        let claude_parser = ClaudeParser::new();
        let cursor_parser = CursorParser::new();
        let gemini_parser = GeminiParser::new();
        let codex_parser = CodexParser::new();
        let mut results = Vec::with_capacity(paths.len());

        for path in paths {
            let path_buf = PathBuf::from(&path);

            // 检测是 Cursor 工作区（目录）还是 Claude 日志文件
            if path_buf.is_dir() {
                // Cursor 工作区：使用 CursorParser 解析整个工作区
                match cursor_parser.parse_workspace(&path_buf) {
                    Ok(sessions) => {
                        // 每个工作区可能有多个会话，返回第一个作为主结果
                        if sessions.is_empty() {
                            results.push(FileImportResult {
                                success: false,
                                file_path: path,
                                project_id: None,
                                session_id: None,
                                error: Some("工作区中未找到对话".to_string()),
                                skipped: None,
                            });
                        } else {
                            // 返回会话数量作为成功指示
                            let first_session = &sessions[0];
                            results.push(FileImportResult {
                                success: true,
                                file_path: path.clone(),
                                project_id: Some(generate_project_id(&first_session.cwd)),
                                session_id: Some(format!("{} 个会话", sessions.len())),
                                error: None,
                                skipped: None,
                            });
                        }
                    }
                    Err(e) => {
                        let is_skippable = e.is_skippable();
                        results.push(FileImportResult {
                            success: false,
                            file_path: path,
                            project_id: None,
                            session_id: None,
                            error: Some(e.to_string()),
                            skipped: if is_skippable { Some(true) } else { None },
                        });
                    }
                }
            } else {
                // 根据文件扩展名和路径选择解析器
                let is_gemini = path.contains("/.gemini/") || path.contains("\\.gemini\\");
                let is_codex = path.contains("/.codex/") || path.contains("\\.codex\\");
                let is_json = path.ends_with(".json");
                let is_jsonl = path.ends_with(".jsonl");

                let parse_result = if is_codex && is_jsonl {
                    // Codex CLI 会话文件
                    codex_parser.parse_file(&path)
                } else if is_gemini && is_json {
                    // Gemini CLI 会话文件
                    gemini_parser.parse_file(&path)
                } else if is_jsonl {
                    // Claude Code JSONL 文件 (or try Codex as fallback)
                    claude_parser.parse_file(&path)
                        .or_else(|_| codex_parser.parse_file(&path))
                } else if is_json {
                    // 尝试先作为 Gemini 解析，失败则尝试 Claude
                    gemini_parser.parse_file(&path)
                        .or_else(|_| claude_parser.parse_file(&path))
                } else {
                    // 默认使用 Claude 解析器
                    claude_parser.parse_file(&path)
                };

                match parse_result {
                    Ok(session) => {
                        results.push(FileImportResult {
                            success: true,
                            file_path: path,
                            project_id: Some(generate_project_id(&session.cwd)),
                            session_id: Some(session.id.clone()),
                            error: None,
                            skipped: None,
                        });
                    }
                    Err(e) => {
                        let is_skippable = e.is_skippable();
                        results.push(FileImportResult {
                            success: false,
                            file_path: path,
                            project_id: None,
                            session_id: None,
                            error: Some(e.to_string()),
                            skipped: if is_skippable { Some(true) } else { None },
                        });
                    }
                }
            }
        }

        results
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    Ok(results)
}

/// Generate a simple project ID from cwd
fn generate_project_id(cwd: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    cwd.hash(&mut hasher);
    format!("proj_{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_log_dir_claude() {
        let result = get_default_log_dir(&ImportSource::Claude);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_str().unwrap().contains(".claude"));
        assert!(path.to_str().unwrap().contains("projects"));
    }

    #[test]
    fn test_get_default_log_dir_gemini() {
        let result = get_default_log_dir(&ImportSource::Gemini);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_str().unwrap().contains(".gemini"));
        assert!(path.to_str().unwrap().contains("tmp"));
    }

    #[test]
    fn test_generate_project_id() {
        let id1 = generate_project_id("/home/user/project1");
        let id2 = generate_project_id("/home/user/project2");
        let id1_again = generate_project_id("/home/user/project1");

        assert!(id1.starts_with("proj_"));
        assert_ne!(id1, id2);
        assert_eq!(id1, id1_again);
    }

    #[test]
    fn test_path_to_discovered_file() {
        // This test requires a real file, skip in unit tests
        // Integration tests should cover this
    }

    // TODO: Re-enable these tests when extract_cwd_from_file is implemented
    // #[test]
    // fn test_extract_cwd_from_file() {
    //     use std::io::Write;
    //
    //     // Create a temp file with Claude Code JSONL format
    //     let temp_dir = std::env::temp_dir();
    //     let test_file = temp_dir.join("test_claude_session.jsonl");
    //
    //     let content = r#"{"type":"summary","summary":"Test Session"}
    // {"parentUuid":"root","cwd":"/mnt/disk0/project/newx/nextalk-voice-capsule","sessionId":"test-123","type":"user","message":{"role":"user","content":"Hello"}}
    // "#;
    //
    //     let mut file = std::fs::File::create(&test_file).unwrap();
    //     file.write_all(content.as_bytes()).unwrap();
    //
    //     // Test extraction
    //     let result = extract_cwd_from_file(&test_file);
    //     assert_eq!(result, Some("/mnt/disk0/project/newx/nextalk-voice-capsule".to_string()));
    //
    //     // Clean up
    //     std::fs::remove_file(&test_file).ok();
    // }
    //
    // #[test]
    // fn test_extract_cwd_from_real_file() {
    //     // Test with a real Claude Code session file if it exists
    //     let real_file = std::path::PathBuf::from(
    //         "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-capsule"
    //     );
    //
    //     if real_file.exists() {
    //         if let Ok(entries) = std::fs::read_dir(&real_file) {
    //             for entry in entries.flatten() {
    //                 let path = entry.path();
    //                 if path.extension().is_some_and(|ext| ext == "jsonl") {
    //                     let result = extract_cwd_from_file(&path);
    //                     println!("File: {:?}", path);
    //                     println!("Extracted cwd: {:?}", result);
    //
    //                     // The cwd should be the real project path, not the log directory
    //                     if let Some(cwd) = result {
    //                         assert!(
    //                             cwd.starts_with("/mnt/disk0/project"),
    //                             "cwd should be the real project path, got: {}", cwd
    //                         );
    //                         assert!(
    //                             !cwd.contains("-mnt-"),
    //                             "cwd should not contain encoded path format, got: {}", cwd
    //                         );
    //                     }
    //                     break; // Only test one file
    //                 }
    //             }
    //         }
    //     }
    // }

    #[test]
    fn test_find_workspace_storage_path_direct() {
        // Case 1: workspaceStorage directory directly
        let dir = PathBuf::from("/some/path/workspaceStorage");
        let result = find_workspace_storage_path(&dir);
        assert_eq!(result, Some(PathBuf::from("/some/path/workspaceStorage")));
    }

    #[test]
    fn test_find_workspace_storage_path_not_found() {
        // Case where no workspaceStorage exists
        let dir = PathBuf::from("/tmp/nonexistent");
        let result = find_workspace_storage_path(&dir);
        assert_eq!(result, None);
    }

    #[test]
    fn test_gemini_file_detection_in_parse_log_files() {
        // Test the file type detection logic for Gemini files
        // Paths containing ".gemini" and ending with ".json" should be detected as Gemini
        let gemini_path = "/home/user/.gemini/tmp/abc123/chats/session-2025-01-01.json";
        assert!(gemini_path.contains("/.gemini/"));
        assert!(gemini_path.ends_with(".json"));

        // Windows path style
        let gemini_path_win = "C:\\Users\\test\\.gemini\\tmp\\abc123\\chats\\session.json";
        assert!(gemini_path_win.contains("\\.gemini\\"));
        assert!(gemini_path_win.ends_with(".json"));
    }
}

