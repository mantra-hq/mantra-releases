//! Cross-platform Cursor path resolution and workspace scanning
//!
//! Handles locating Cursor's data storage directories across different
//! operating systems and resolving workspace folder hashes to project paths.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::parsers::ParseError;

/// Cursor data storage paths
#[derive(Debug, Clone)]
pub struct CursorPaths {
    /// Path to globalStorage (contains composer data and bubble content)
    pub global_storage: PathBuf,
    /// Path to workspaceStorage (contains workspace-specific data)
    pub workspace_storage: PathBuf,
}

/// Workspace metadata from workspace.json
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// Workspace folder hash (directory name under workspaceStorage)
    pub id: String,
    /// Real project path
    pub folder_path: PathBuf,
    /// Path to state.vscdb for this workspace
    pub state_db_path: PathBuf,
}

/// Internal structure for parsing workspace.json
#[derive(Debug, Deserialize)]
struct WorkspaceJson {
    /// Folder URI (e.g., "file:///path/to/project")
    folder: Option<String>,
}

impl CursorPaths {
    /// Get Cursor paths for the current platform
    pub fn detect() -> Result<Self, ParseError> {
        let user_data_dir = get_user_data_dir()?;

        let global_storage = user_data_dir.join("globalStorage");
        let workspace_storage = user_data_dir.join("workspaceStorage");

        // Validate paths exist
        if !global_storage.exists() {
            return Err(ParseError::invalid_format(format!(
                "Cursor globalStorage not found: {}",
                global_storage.display()
            )));
        }

        if !workspace_storage.exists() {
            return Err(ParseError::invalid_format(format!(
                "Cursor workspaceStorage not found: {}",
                workspace_storage.display()
            )));
        }

        Ok(Self {
            global_storage,
            workspace_storage,
        })
    }

    /// Get the path to the global state.vscdb database
    pub fn global_state_db(&self) -> PathBuf {
        self.global_storage.join("state.vscdb")
    }

    /// Scan all workspace folders and build a mapping of project paths to workspace IDs
    pub fn scan_workspaces(&self) -> Result<Vec<WorkspaceInfo>, ParseError> {
        scan_workspace_folders(&self.workspace_storage)
    }

    /// Find workspace ID for a given project path
    pub fn find_workspace_id(&self, project_path: &Path) -> Result<Option<WorkspaceInfo>, ParseError> {
        let workspaces = self.scan_workspaces()?;
        let normalized_target = normalize_path(project_path);

        Ok(workspaces.into_iter().find(|ws| {
            normalize_path(&ws.folder_path) == normalized_target
        }))
    }
}

/// Get platform-specific Cursor user data directory
fn get_user_data_dir() -> Result<PathBuf, ParseError> {
    #[cfg(target_os = "linux")]
    {
        // Linux: ~/.config/Cursor/User/
        if let Some(home) = dirs::home_dir() {
            return Ok(home.join(".config").join("Cursor").join("User"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Application Support/Cursor/User/
        if let Some(home) = dirs::home_dir() {
            return Ok(home
                .join("Library")
                .join("Application Support")
                .join("Cursor")
                .join("User"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: %APPDATA%\Cursor\User\
        if let Some(app_data) = dirs::data_dir() {
            return Ok(app_data.join("Cursor").join("User"));
        }
    }

    Err(ParseError::invalid_format("Could not determine home directory"))
}

/// Scan workspaceStorage folder for all workspace directories
fn scan_workspace_folders(workspace_storage: &Path) -> Result<Vec<WorkspaceInfo>, ParseError> {
    let mut workspaces = Vec::new();

    let entries = fs::read_dir(workspace_storage).map_err(|e| {
        ParseError::invalid_format(format!("Failed to read workspaceStorage: {}", e))
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Get the folder hash (directory name)
        let id = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Try to read workspace.json
        let workspace_json_path = path.join("workspace.json");
        if let Some(folder_path) = read_workspace_json(&workspace_json_path) {
            let state_db_path = path.join("state.vscdb");

            workspaces.push(WorkspaceInfo {
                id,
                folder_path,
                state_db_path,
            });
        }
    }

    Ok(workspaces)
}

/// Read and parse workspace.json to extract the project folder path
fn read_workspace_json(path: &Path) -> Option<PathBuf> {
    let content = fs::read_to_string(path).ok()?;
    let workspace: WorkspaceJson = serde_json::from_str(&content).ok()?;

    workspace.folder.and_then(|uri| parse_file_uri(&uri))
}

/// Parse a file:// URI to a filesystem path
fn parse_file_uri(uri: &str) -> Option<PathBuf> {
    if !uri.starts_with("file://") {
        return None;
    }

    let path_str = &uri[7..]; // Remove "file://"

    // Handle URL encoding
    let decoded = urlencoding::decode(path_str).ok()?;

    // Handle Windows paths (file:///C:/path -> C:/path)
    #[cfg(target_os = "windows")]
    {
        // On Windows, file:///C:/path/to/folder
        // After removing file://, we get /C:/path/to/folder
        if decoded.starts_with('/') && decoded.len() > 2 && decoded.chars().nth(2) == Some(':') {
            return Some(PathBuf::from(&decoded[1..]));
        }
    }

    // Handle WSL paths: /mnt/c/Users/... -> C:\Users\...
    // But we want to keep them as-is for Linux processing

    Some(PathBuf::from(decoded.as_ref()))
}

/// Normalize a path for comparison
fn normalize_path(path: &Path) -> PathBuf {
    // Attempt to canonicalize, but fall back to the original path if it fails
    // (e.g., if the path doesn't exist)
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// Convert a WSL path to Windows path (for remote workspace matching)
#[allow(dead_code)]
pub fn wsl_to_windows_path(wsl_path: &Path) -> Option<PathBuf> {
    let path_str = wsl_path.to_string_lossy();

    // Pattern: /mnt/{drive}/... -> {Drive}:\...
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

/// Convert a Windows path to WSL path (for remote workspace matching)
#[allow(dead_code)]
pub fn windows_to_wsl_path(windows_path: &Path) -> Option<PathBuf> {
    let path_str = windows_path.to_string_lossy();

    // Pattern: {Drive}:\... -> /mnt/{drive}/...
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
    fn test_parse_file_uri_linux() {
        let uri = "file:///home/user/project";
        let path = parse_file_uri(uri);
        assert_eq!(path, Some(PathBuf::from("/home/user/project")));
    }

    #[test]
    fn test_parse_file_uri_with_spaces() {
        let uri = "file:///home/user/my%20project";
        let path = parse_file_uri(uri);
        assert_eq!(path, Some(PathBuf::from("/home/user/my project")));
    }

    #[test]
    fn test_parse_file_uri_invalid() {
        let uri = "http://example.com/path";
        let path = parse_file_uri(uri);
        assert_eq!(path, None);
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
        let wsl_path = PathBuf::from("/home/user/project");
        let windows_path = wsl_to_windows_path(&wsl_path);
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
}
