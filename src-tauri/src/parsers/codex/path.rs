//! Cross-platform Codex CLI path resolution and session scanning
//!
//! Handles locating Codex CLI's data storage directories across different
//! operating systems and scanning for session files.
//!
//! ## Storage Structure
//!
//! ```text
//! ~/.codex/
//! └── sessions/
//!     └── YYYY/
//!         └── MM/
//!             └── DD/
//!                 └── rollout-{timestamp}-{session_id}.jsonl
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::parsers::ParseError;

/// Codex CLI data paths
#[derive(Debug, Clone)]
pub struct CodexPaths {
    /// Root `.codex` directory
    pub codex_dir: PathBuf,
    /// Sessions directory
    pub sessions_dir: PathBuf,
}

/// Discovered session file
#[derive(Debug, Clone)]
pub struct CodexSessionFile {
    /// Full path to session JSONL file
    pub path: PathBuf,
    /// Session ID extracted from filename
    pub session_id: String,
    /// Date extracted from directory structure (YYYY-MM-DD)
    pub date: String,
    /// Filename
    pub filename: String,
}

impl CodexPaths {
    /// Detect Codex CLI paths for the current platform
    pub fn detect() -> Result<Self, ParseError> {
        // Try standard home directory first
        if let Some(codex_dir) = get_codex_dir() {
            if codex_dir.exists() {
                let sessions_dir = codex_dir.join("sessions");
                return Ok(Self { codex_dir, sessions_dir });
            }
        }

        // On Linux, also try WSL Windows home
        #[cfg(target_os = "linux")]
        {
            if let Some(codex_dir) = detect_wsl_windows_codex() {
                if codex_dir.exists() {
                    let sessions_dir = codex_dir.join("sessions");
                    return Ok(Self { codex_dir, sessions_dir });
                }
            }
        }

        Err(ParseError::invalid_format(
            "Codex CLI directory not found. Please ensure Codex CLI is installed.",
        ))
    }

    /// Create CodexPaths from a specific directory (for testing or custom paths)
    pub fn from_dir(codex_dir: PathBuf) -> Self {
        let sessions_dir = codex_dir.join("sessions");
        Self { codex_dir, sessions_dir }
    }

    /// Scan all session files in the sessions directory
    pub fn scan_all_sessions(&self) -> Result<Vec<CodexSessionFile>, ParseError> {
        if !self.sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        self.scan_directory(&self.sessions_dir, &mut sessions)?;

        // Sort by date and filename (newest first)
        sessions.sort_by(|a, b| {
            let date_cmp = b.date.cmp(&a.date);
            if date_cmp == std::cmp::Ordering::Equal {
                b.filename.cmp(&a.filename)
            } else {
                date_cmp
            }
        });

        Ok(sessions)
    }

    /// Recursively scan directory for JSONL files
    fn scan_directory(&self, dir: &Path, sessions: &mut Vec<CodexSessionFile>) -> Result<(), ParseError> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(dir).map_err(|e| {
            ParseError::invalid_format(format!("Failed to read directory {}: {}", dir.display(), e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Recurse into subdirectories (YYYY/MM/DD structure)
                self.scan_directory(&path, sessions)?;
            } else if path.is_file() {
                // Check for rollout-*.jsonl files
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("rollout-") && filename.ends_with(".jsonl") {
                        if let Some(session_file) = self.parse_session_file(&path, filename) {
                            sessions.push(session_file);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse session file path to extract metadata
    fn parse_session_file(&self, path: &Path, filename: &str) -> Option<CodexSessionFile> {
        // Extract session ID from filename: rollout-{timestamp}-{session_id}.jsonl
        // Example: rollout-2026-01-05T19-04-21-019b8dd4-539f-7393-acb6-7a856d6892ca.jsonl
        let session_id = extract_session_id(filename)?;

        // Extract date from directory structure: sessions/YYYY/MM/DD/
        let date = extract_date_from_path(path, &self.sessions_dir)?;

        Some(CodexSessionFile {
            path: path.to_path_buf(),
            session_id,
            date,
            filename: filename.to_string(),
        })
    }
}

/// Extract session ID from filename
/// Format: rollout-{timestamp}-{session_id}.jsonl
fn extract_session_id(filename: &str) -> Option<String> {
    // Remove "rollout-" prefix and ".jsonl" suffix
    let name = filename.strip_prefix("rollout-")?.strip_suffix(".jsonl")?;

    // The session ID is the UUID at the end
    // Format: YYYY-MM-DDTHH-MM-SS-{uuid}
    // We need to find the last UUID-like segment
    // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx

    // Split by '-' and look for UUID pattern
    let parts: Vec<&str> = name.split('-').collect();

    // A UUID has 5 groups: 8-4-4-4-12 chars
    // The filename has: YYYY-MM-DDTHH-MM-SS-{8}-{4}-{4}-{4}-{12}
    // That's 6 parts for timestamp + 5 parts for UUID = 11+ parts minimum
    if parts.len() >= 5 {
        // Take the last 5 parts as the UUID
        let uuid_parts = &parts[parts.len() - 5..];
        let session_id = uuid_parts.join("-");

        // Validate UUID format (8-4-4-4-12)
        if uuid_parts.len() == 5
            && uuid_parts[0].len() == 8
            && uuid_parts[1].len() == 4
            && uuid_parts[2].len() == 4
            && uuid_parts[3].len() == 4
            && uuid_parts[4].len() == 12
        {
            return Some(session_id);
        }
    }

    // Fallback: just return the whole name after rollout-
    Some(name.to_string())
}

/// Extract date from path structure: sessions/YYYY/MM/DD/
fn extract_date_from_path(path: &Path, sessions_dir: &Path) -> Option<String> {
    // Get relative path from sessions_dir
    let relative = path.strip_prefix(sessions_dir).ok()?;
    let components: Vec<_> = relative.components().collect();

    // Expect: YYYY/MM/DD/filename.jsonl
    if components.len() >= 4 {
        let year = components[0].as_os_str().to_str()?;
        let month = components[1].as_os_str().to_str()?;
        let day = components[2].as_os_str().to_str()?;

        // Validate format
        if year.len() == 4 && month.len() == 2 && day.len() == 2 {
            return Some(format!("{}-{}-{}", year, month, day));
        }
    }

    None
}

/// Get the standard Codex CLI directory for the current platform
pub fn get_codex_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codex"))
}

/// Get the Codex CLI sessions directory
pub fn get_codex_sessions_dir() -> Option<PathBuf> {
    get_codex_dir().map(|c| c.join("sessions"))
}

/// Detect Codex CLI directory in WSL accessing Windows home
/// Returns path like /mnt/c/Users/{user}/.codex
#[cfg(target_os = "linux")]
fn detect_wsl_windows_codex() -> Option<PathBuf> {
    // Check if we're in WSL by looking for /mnt/c/Users
    let mnt_c_users = Path::new("/mnt/c/Users");
    if !mnt_c_users.exists() {
        return None;
    }

    // Try to find Windows username from current user
    if let Ok(entries) = fs::read_dir(mnt_c_users) {
        for entry in entries.flatten() {
            let user_dir = entry.path();
            if user_dir.is_dir() {
                let codex_dir = user_dir.join(".codex");
                if codex_dir.exists() {
                    return Some(codex_dir);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_codex_dir() {
        let dir = get_codex_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.ends_with(".codex"));
    }

    #[test]
    fn test_get_codex_sessions_dir() {
        let dir = get_codex_sessions_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.ends_with("sessions"));
    }

    #[test]
    fn test_codex_paths_from_dir() {
        let paths = CodexPaths::from_dir(PathBuf::from("/tmp/test/.codex"));
        assert_eq!(paths.codex_dir, PathBuf::from("/tmp/test/.codex"));
        assert_eq!(paths.sessions_dir, PathBuf::from("/tmp/test/.codex/sessions"));
    }

    #[test]
    fn test_extract_session_id() {
        // Standard format
        let id = extract_session_id("rollout-2026-01-05T19-04-21-019b8dd4-539f-7393-acb6-7a856d6892ca.jsonl");
        assert_eq!(id, Some("019b8dd4-539f-7393-acb6-7a856d6892ca".to_string()));

        // Another format
        let id = extract_session_id("rollout-2025-10-19T09-08-27-0199fa02-be3a-7021-8051-9b891b6d0eb2.jsonl");
        assert_eq!(id, Some("0199fa02-be3a-7021-8051-9b891b6d0eb2".to_string()));
    }

    #[test]
    fn test_extract_date_from_path() {
        let sessions_dir = PathBuf::from("/home/user/.codex/sessions");
        let path = PathBuf::from("/home/user/.codex/sessions/2026/01/05/rollout-test.jsonl");

        let date = extract_date_from_path(&path, &sessions_dir);
        assert_eq!(date, Some("2026-01-05".to_string()));
    }

    #[test]
    fn test_extract_date_from_path_invalid() {
        let sessions_dir = PathBuf::from("/home/user/.codex/sessions");
        let path = PathBuf::from("/home/user/.codex/sessions/invalid/rollout-test.jsonl");

        let date = extract_date_from_path(&path, &sessions_dir);
        assert_eq!(date, None);
    }
}
