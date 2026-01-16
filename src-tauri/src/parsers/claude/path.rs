//! Cross-platform Claude Code path resolution and session scanning
//!
//! Handles locating Claude Code's data storage directories across different
//! operating systems and scanning for session files.
//!
//! ## Storage Structure
//!
//! ```text
//! ~/.claude/
//! ├── projects/
//! │   ├── {encoded-project-path}/    # e.g., -mnt-disk0-project-foo
//! │   │   ├── {session-uuid}.jsonl
//! │   │   └── ...
//! │   └── ...
//! ├── settings.json
//! └── ...
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::parsers::ParseError;

/// Claude Code data paths
#[derive(Debug, Clone)]
pub struct ClaudePaths {
    /// Root `.claude` directory
    pub claude_dir: PathBuf,
    /// Projects directory containing session files
    pub projects_dir: PathBuf,
}

/// Discovered session file
#[derive(Debug, Clone)]
pub struct ClaudeSessionFile {
    /// Encoded project path (directory name)
    pub project_path_encoded: String,
    /// Decoded project path (actual filesystem path)
    pub project_path: String,
    /// Full path to session JSONL file
    pub path: PathBuf,
    /// Session ID (filename without extension)
    pub session_id: String,
}

impl ClaudePaths {
    /// Detect Claude Code paths for the current platform
    pub fn detect() -> Result<Self, ParseError> {
        // Try standard home directory first
        if let Some(claude_dir) = get_claude_dir() {
            if claude_dir.exists() {
                let projects_dir = claude_dir.join("projects");
                return Ok(Self { claude_dir, projects_dir });
            }
        }

        // On Linux, also try WSL Windows home
        #[cfg(target_os = "linux")]
        {
            if let Some(claude_dir) = detect_wsl_windows_claude() {
                if claude_dir.exists() {
                    let projects_dir = claude_dir.join("projects");
                    return Ok(Self { claude_dir, projects_dir });
                }
            }
        }

        Err(ParseError::invalid_format(
            "Claude Code directory not found. Please ensure Claude Code is installed.",
        ))
    }

    /// Create ClaudePaths from a specific directory (for testing or custom paths)
    pub fn from_dir(claude_dir: PathBuf) -> Self {
        let projects_dir = claude_dir.join("projects");
        Self { claude_dir, projects_dir }
    }

    /// Scan all project directories
    pub fn scan_project_dirs(&self) -> Result<Vec<String>, ParseError> {
        if !self.projects_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.projects_dir).map_err(|e| {
            ParseError::invalid_format(format!("Failed to read Claude projects directory: {}", e))
        })?;

        let mut project_paths = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check if directory has any .jsonl files
                    if has_jsonl_files(&path) {
                        project_paths.push(name.to_string());
                    }
                }
            }
        }

        Ok(project_paths)
    }

    /// Scan session files for a specific project
    pub fn scan_sessions(&self, project_path_encoded: &str) -> Result<Vec<ClaudeSessionFile>, ParseError> {
        let project_dir = self.projects_dir.join(project_path_encoded);
        if !project_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&project_dir).map_err(|e| {
            ParseError::invalid_format(format!("Failed to read project directory: {}", e))
        })?;

        let project_path = decode_claude_path(project_path_encoded);
        let mut sessions = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Match *.jsonl pattern (UUID.jsonl)
                    if filename.ends_with(".jsonl") {
                        let session_id = filename.trim_end_matches(".jsonl").to_string();
                        sessions.push(ClaudeSessionFile {
                            project_path_encoded: project_path_encoded.to_string(),
                            project_path: project_path.clone(),
                            path: path.clone(),
                            session_id,
                        });
                    }
                }
            }
        }

        // Sort by session ID
        sessions.sort_by(|a, b| a.session_id.cmp(&b.session_id));
        Ok(sessions)
    }

    /// Scan all sessions across all projects
    pub fn scan_all_sessions(&self) -> Result<Vec<ClaudeSessionFile>, ParseError> {
        let project_dirs = self.scan_project_dirs()?;
        let mut all_sessions = Vec::new();

        for encoded in project_dirs {
            let sessions = self.scan_sessions(&encoded)?;
            all_sessions.extend(sessions);
        }

        Ok(all_sessions)
    }

    /// Get the path to a specific session file
    pub fn session_path(&self, project_path_encoded: &str, session_id: &str) -> PathBuf {
        self.projects_dir
            .join(project_path_encoded)
            .join(format!("{}.jsonl", session_id))
    }
}

/// Get the standard Claude Code directory for the current platform
pub fn get_claude_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

/// Get the Claude Code projects directory
pub fn get_claude_projects_dir() -> Option<PathBuf> {
    get_claude_dir().map(|c| c.join("projects"))
}

/// Decode Claude's encoded project path
/// Claude encodes paths by replacing / with -
/// e.g., -mnt-disk0-project-foo -> /mnt/disk0/project/foo
///
/// Note: This simple replacement works because Claude's encoding is straightforward.
/// Project names with hyphens will be decoded incorrectly, but since we primarily
/// need this for matching with existing sessions that have the real cwd, this is
/// acceptable - the key is consistency within the same project folder.
pub fn decode_claude_path(encoded_path: &str) -> String {
    if !encoded_path.starts_with('-') {
        return encoded_path.to_string();
    }
    
    // Claude encodes paths by replacing / with -
    // Simply replace all - with / to decode
    encoded_path.replace('-', "/")
}

/// Check if a directory contains any .jsonl files
fn has_jsonl_files(dir: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".jsonl") {
                    return true;
                }
            }
        }
    }
    false
}

/// Detect Claude Code directory in WSL accessing Windows home
/// Returns path like /mnt/c/Users/{user}/.claude
#[cfg(target_os = "linux")]
fn detect_wsl_windows_claude() -> Option<PathBuf> {
    // Check if we're in WSL by looking for /mnt/c/Users
    let mnt_c_users = Path::new("/mnt/c/Users");
    if !mnt_c_users.exists() {
        return None;
    }

    // Try to find Windows username from directory listing
    if let Ok(entries) = fs::read_dir(mnt_c_users) {
        for entry in entries.flatten() {
            let user_dir = entry.path();
            if user_dir.is_dir() {
                let claude_dir = user_dir.join(".claude");
                if claude_dir.exists() {
                    return Some(claude_dir);
                }
            }
        }
    }

    None
}

/// Try to extract cwd from file content by reading the first few lines
pub fn extract_cwd_from_file_content(path: &str) -> Option<String> {
    use std::io::{BufRead, BufReader};
    
    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    
    for line in reader.lines().take(20).flatten() {
        if let Ok(record) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(cwd) = record.get("cwd").and_then(|v| v.as_str()) {
                if !cwd.is_empty() {
                    return Some(cwd.to_string());
                }
            }
        }
    }
    None
}

/// Try to extract cwd from sibling session files in the same directory
///
/// When a session file doesn't contain cwd (e.g., only has system events),
/// we try to find it from other session files in the same project directory.
/// This is more reliable than `decode_claude_path` for project names with hyphens.
pub fn extract_cwd_from_sibling_sessions(path: &str) -> Option<String> {
    use std::path::Path;
    
    let path_buf = Path::new(path);
    let parent = path_buf.parent()?;
    let current_filename = path_buf.file_name()?.to_str()?;
    
    // Read directory entries
    let entries = fs::read_dir(parent).ok()?;
    
    // Try to find cwd from other .jsonl files in the same directory
    for entry in entries.flatten() {
        let entry_path = entry.path();
        
        // Skip current file and non-jsonl files
        if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
            if name == current_filename || !name.ends_with(".jsonl") {
                continue;
            }
        } else {
            continue;
        }
        
        // Try to extract cwd from this sibling file
        if let Some(cwd) = extract_cwd_from_file_content(entry_path.to_str()?) {
            return Some(cwd);
        }
    }
    
    None
}


#[cfg(test)]
#[path = "path_tests.rs"]
mod tests;
