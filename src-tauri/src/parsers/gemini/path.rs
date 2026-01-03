//! Cross-platform Gemini CLI path resolution and workspace scanning
//!
//! Handles locating Gemini CLI's data storage directories across different
//! operating systems and scanning for session files.
//!
//! ## Storage Structure
//!
//! ```text
//! ~/.gemini/
//! ├── tmp/
//! │   ├── {projectHash}/       # SHA256 of project root
//! │   │   ├── chats/
//! │   │   │   ├── session-{date}-{uuid}.json
//! │   │   │   └── ...
//! │   │   └── logs.json
//! │   └── ...
//! ├── settings.json
//! ├── memory.md
//! └── google_accounts.json
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::parsers::ParseError;

/// Gemini CLI data paths
#[derive(Debug, Clone)]
pub struct GeminiPaths {
    /// Root `.gemini` directory
    pub gemini_dir: PathBuf,
    /// Temp directory containing project hashes
    pub tmp_dir: PathBuf,
}

/// Discovered session file
#[derive(Debug, Clone)]
pub struct GeminiSessionFile {
    /// Project hash (directory name)
    pub project_hash: String,
    /// Full path to session JSON file
    pub path: PathBuf,
    /// Session filename (e.g., "session-2025-12-30T20-11-8c9a7d96.json")
    pub filename: String,
}

impl GeminiPaths {
    /// Detect Gemini CLI paths for the current platform
    pub fn detect() -> Result<Self, ParseError> {
        // Try standard home directory first
        if let Some(gemini_dir) = get_gemini_dir() {
            if gemini_dir.exists() {
                let tmp_dir = gemini_dir.join("tmp");
                return Ok(Self { gemini_dir, tmp_dir });
            }
        }

        // On Linux, also try WSL Windows home
        #[cfg(target_os = "linux")]
        {
            if let Some(gemini_dir) = detect_wsl_windows_gemini() {
                if gemini_dir.exists() {
                    let tmp_dir = gemini_dir.join("tmp");
                    return Ok(Self { gemini_dir, tmp_dir });
                }
            }
        }

        Err(ParseError::invalid_format(
            "Gemini CLI directory not found. Please ensure Gemini CLI is installed.",
        ))
    }

    /// Create GeminiPaths from a specific directory (for testing or custom paths)
    pub fn from_dir(gemini_dir: PathBuf) -> Self {
        let tmp_dir = gemini_dir.join("tmp");
        Self { gemini_dir, tmp_dir }
    }

    /// Scan all project hashes in the tmp directory
    pub fn scan_project_hashes(&self) -> Result<Vec<String>, ParseError> {
        if !self.tmp_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.tmp_dir).map_err(|e| {
            ParseError::invalid_format(format!("Failed to read Gemini tmp directory: {}", e))
        })?;

        let mut project_hashes = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Project hash directories typically look like hex strings
                    // but we don't strictly validate - just check it has a chats subdir
                    let chats_dir = path.join("chats");
                    if chats_dir.exists() && chats_dir.is_dir() {
                        project_hashes.push(name.to_string());
                    }
                }
            }
        }

        Ok(project_hashes)
    }

    /// Scan session files for a specific project hash
    pub fn scan_sessions(&self, project_hash: &str) -> Result<Vec<GeminiSessionFile>, ParseError> {
        let chats_dir = self.tmp_dir.join(project_hash).join("chats");
        if !chats_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&chats_dir).map_err(|e| {
            ParseError::invalid_format(format!("Failed to read chats directory: {}", e))
        })?;

        let mut sessions = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Match session-*.json pattern
                    if filename.starts_with("session-") && filename.ends_with(".json") {
                        sessions.push(GeminiSessionFile {
                            project_hash: project_hash.to_string(),
                            path: path.clone(),
                            filename: filename.to_string(),
                        });
                    }
                }
            }
        }

        // Sort by filename (which includes timestamp)
        sessions.sort_by(|a, b| a.filename.cmp(&b.filename));
        Ok(sessions)
    }

    /// Scan all sessions across all project hashes
    pub fn scan_all_sessions(&self) -> Result<Vec<GeminiSessionFile>, ParseError> {
        let project_hashes = self.scan_project_hashes()?;
        let mut all_sessions = Vec::new();

        for hash in project_hashes {
            let sessions = self.scan_sessions(&hash)?;
            all_sessions.extend(sessions);
        }

        Ok(all_sessions)
    }

    /// Get the path to a specific session file
    pub fn session_path(&self, project_hash: &str, session_id: &str) -> PathBuf {
        self.tmp_dir
            .join(project_hash)
            .join("chats")
            .join(format!("{}.json", session_id))
    }
}

/// Get the standard Gemini CLI directory for the current platform
pub fn get_gemini_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".gemini"))
}

/// Get the Gemini CLI tmp directory
pub fn get_gemini_tmp_dir() -> Option<PathBuf> {
    get_gemini_dir().map(|g| g.join("tmp"))
}

/// Detect Gemini CLI directory in WSL accessing Windows home
/// Returns path like /mnt/c/Users/{user}/.gemini
#[cfg(target_os = "linux")]
fn detect_wsl_windows_gemini() -> Option<PathBuf> {
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
                let gemini_dir = user_dir.join(".gemini");
                if gemini_dir.exists() {
                    return Some(gemini_dir);
                }
            }
        }
    }

    None
}

/// Convert a WSL path to Windows path
/// e.g., /mnt/c/Users/test → C:\Users\test
pub fn wsl_to_windows_path(wsl_path: &Path) -> Option<PathBuf> {
    let path_str = wsl_path.to_string_lossy();

    // Pattern: /mnt/{drive}/... → {Drive}:\...
    if path_str.starts_with("/mnt/") && path_str.len() > 5 {
        let drive = path_str.chars().nth(5)?;
        if drive.is_ascii_alphabetic() && (path_str.len() == 6 || path_str.chars().nth(6) == Some('/')) {
            let rest = if path_str.len() > 6 {
                &path_str[6..]
            } else {
                "\\"
            };
            let windows_path = format!("{}:{}", drive.to_ascii_uppercase(), rest.replace('/', "\\"));
            return Some(PathBuf::from(windows_path));
        }
    }

    None
}

/// Convert a Windows path to WSL path
/// e.g., C:\Users\test → /mnt/c/Users/test
pub fn windows_to_wsl_path(windows_path: &Path) -> Option<PathBuf> {
    let path_str = windows_path.to_string_lossy();

    // Pattern: {Drive}:\... → /mnt/{drive}/...
    if path_str.len() >= 2 && path_str.chars().nth(1) == Some(':') {
        let drive = path_str.chars().next()?;
        if drive.is_ascii_alphabetic() {
            let rest = if path_str.len() > 2 {
                &path_str[2..]
            } else {
                ""
            };
            let wsl_path = format!("/mnt/{}{}", drive.to_ascii_lowercase(), rest.replace('\\', "/"));
            return Some(PathBuf::from(wsl_path));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gemini_dir() {
        // This should return Some on any system with a home directory
        let dir = get_gemini_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.ends_with(".gemini"));
    }

    #[test]
    fn test_get_gemini_tmp_dir() {
        let dir = get_gemini_tmp_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.ends_with("tmp"));
    }

    #[test]
    fn test_wsl_to_windows_path() {
        let wsl_path = PathBuf::from("/mnt/c/Users/test/project");
        let windows_path = wsl_to_windows_path(&wsl_path);
        assert_eq!(windows_path, Some(PathBuf::from("C:\\Users\\test\\project")));
    }

    #[test]
    fn test_wsl_to_windows_path_drive_only() {
        let wsl_path = PathBuf::from("/mnt/d");
        let windows_path = wsl_to_windows_path(&wsl_path);
        assert_eq!(windows_path, Some(PathBuf::from("D:\\")));
    }

    #[test]
    fn test_wsl_to_windows_path_not_wsl() {
        let path = PathBuf::from("/home/user/project");
        let windows_path = wsl_to_windows_path(&path);
        assert_eq!(windows_path, None);
    }

    #[test]
    fn test_windows_to_wsl_path() {
        let windows_path = PathBuf::from("C:\\Users\\test\\project");
        let wsl_path = windows_to_wsl_path(&windows_path);
        assert_eq!(wsl_path, Some(PathBuf::from("/mnt/c/Users/test/project")));
    }

    #[test]
    fn test_windows_to_wsl_path_not_windows() {
        let path = PathBuf::from("/home/user/project");
        let wsl_path = windows_to_wsl_path(&path);
        assert_eq!(wsl_path, None);
    }

    #[test]
    fn test_gemini_paths_from_dir() {
        let paths = GeminiPaths::from_dir(PathBuf::from("/tmp/test/.gemini"));
        assert_eq!(paths.gemini_dir, PathBuf::from("/tmp/test/.gemini"));
        assert_eq!(paths.tmp_dir, PathBuf::from("/tmp/test/.gemini/tmp"));
    }

    #[test]
    fn test_session_path() {
        let paths = GeminiPaths::from_dir(PathBuf::from("/home/user/.gemini"));
        let session_path = paths.session_path("abc123", "session-2025-01-01");
        assert_eq!(
            session_path,
            PathBuf::from("/home/user/.gemini/tmp/abc123/chats/session-2025-01-01.json")
        );
    }
}
