//! Import-related Tauri commands
//!
//! Provides IPC commands for scanning log directories and importing log files.

use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::parsers::{ClaudeParser, LogParser};

/// Import source type
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportSource {
    Claude,
    Gemini,
    Cursor,
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
    /// Project path (inferred from cwd)
    pub project_path: String,
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
}

/// Get the default log directory for a given source
fn get_default_log_dir(source: &ImportSource) -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or_else(|| AppError::internal("无法获取 home 目录"))?;

    // Claude Code 的会话日志存储在 ~/.claude/projects/ 目录下
    // 每个项目一个子目录，目录名是路径编码的（如 -mnt-disk0-project-newx-mantra）
    // 会话日志是 JSONL 格式，文件名是 UUID.jsonl
    let path = match source {
        ImportSource::Claude => home.join(".claude").join("projects"),
        ImportSource::Gemini => home
            .join(".gemini")
            .join("project_temp")
            .join("chats"),
        ImportSource::Cursor => home.join(".cursor").join("projects"),
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
            } else if path.extension().map_or(false, |ext| ext == "jsonl") {
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

    // Extract project path from parent
    let project_path = path.parent()?.to_str()?.to_string();

    Some(DiscoveredFile {
        path: path_str,
        name,
        size,
        modified_at,
        project_path,
    })
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

/// Scan a custom directory for session files
///
/// # Arguments
/// * `path` - Directory path to scan
///
/// # Returns
/// * `Ok(Vec<DiscoveredFile>)` - List of discovered files
/// * `Err(AppError)` - IO or internal error
#[tauri::command]
pub async fn scan_custom_directory(path: String) -> Result<Vec<DiscoveredFile>, AppError> {
    let dir = PathBuf::from(path);

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
        let parser = ClaudeParser::new();
        let mut results = Vec::with_capacity(paths.len());

        for path in paths {
            let result = match parser.parse_file(&path) {
                Ok(session) => FileImportResult {
                    success: true,
                    file_path: path,
                    project_id: Some(generate_project_id(&session.cwd)),
                    session_id: Some(session.id.clone()),
                    error: None,
                },
                Err(e) => FileImportResult {
                    success: false,
                    file_path: path,
                    project_id: None,
                    session_id: None,
                    error: Some(e.to_string()),
                },
            };
            results.push(result);
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
}
