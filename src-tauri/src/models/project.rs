//! Project and session summary data models
//!
//! Defines the Project and SessionSummary structures for representing
//! aggregated project data and lightweight session listings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::session::SessionSource;

/// Path type classification (Story 1.12)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PathType {
    /// Local filesystem path (e.g., /home/user/project)
    #[default]
    Local,
    /// Virtual placeholder path (e.g., gemini-project:xxx, placeholder:xxx)
    Virtual,
    /// Remote path (e.g., github.com/user/repo)
    Remote,
}

impl PathType {
    /// Convert from string representation
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "virtual" => PathType::Virtual,
            "remote" => PathType::Remote,
            _ => PathType::Local,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PathType::Local => "local",
            PathType::Virtual => "virtual",
            PathType::Remote => "remote",
        }
    }
}

/// Project data representing an aggregated working directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Project {
    /// Project unique ID (UUID)
    pub id: String,
    /// Project name (directory name extracted from cwd)
    pub name: String,
    /// Working directory absolute path (primary path, kept for backward compatibility)
    pub cwd: String,
    /// Number of sessions in this project
    pub session_count: u32,
    /// Number of non-empty sessions in this project (Story 2.29 V2)
    /// Used for displaying filtered count when "hide empty sessions" is enabled
    #[serde(default)]
    pub non_empty_session_count: u32,
    /// First import time
    pub created_at: DateTime<Utc>,
    /// Last activity time (latest session's updated_at)
    pub last_activity: DateTime<Utc>,
    /// Git repository root path (if detected)
    pub git_repo_path: Option<String>,
    /// Whether this project has an associated Git repository
    pub has_git_repo: bool,
    /// Git remote URL for project identification (Story 1.9)
    /// Used for cross-path project aggregation (same repo, different paths)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_remote_url: Option<String>,
    /// Whether this project is empty (all sessions are empty)
    /// Story 2.29 V2: Used for filtering empty projects in the UI
    #[serde(default)]
    pub is_empty: bool,
    /// Path type classification (Story 1.12)
    /// Determines how the path should be validated and displayed
    #[serde(default)]
    pub path_type: PathType,
    /// Whether the local path exists on the filesystem (Story 1.12)
    /// Only meaningful for PathType::Local
    #[serde(default = "default_path_exists")]
    pub path_exists: bool,
}

fn default_path_exists() -> bool {
    true
}

/// Project path mapping (Story 1.12)
///
/// A project can have multiple paths associated with it.
/// This enables flexible project aggregation from different sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectPath {
    /// Unique ID (UUID)
    pub id: String,
    /// Project ID this path belongs to
    pub project_id: String,
    /// The path (normalized)
    pub path: String,
    /// Whether this is the primary path for the project
    pub is_primary: bool,
    /// When this path was associated with the project
    pub created_at: DateTime<Utc>,
}

/// Session-to-project manual binding (Story 1.12)
///
/// Allows users to manually bind a session to a specific project,
/// overriding the automatic path-based matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SessionBinding {
    /// Session ID
    pub session_id: String,
    /// Project ID the session is bound to
    pub project_id: String,
    /// When the binding was created
    pub bound_at: DateTime<Utc>,
}

/// Source context for session import (Story 1.12)
///
/// Stores source-specific metadata that helps identify the session origin.
/// This is immutable after import and used for debugging/auditing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SourceContext {
    /// Original file path from the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,

    /// Claude Code: Encoded project path from directory name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path_encoded: Option<String>,

    /// Gemini CLI: Project hash from the tmp directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_hash: Option<String>,

    /// Gemini CLI: Session filename
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_filename: Option<String>,

    /// Cursor: Workspace ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,

    /// Cursor: Workspace storage path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
}

/// Lightweight session summary for listings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SessionSummary {
    /// Session ID
    pub id: String,
    /// Source (Claude/Gemini/Cursor)
    pub source: SessionSource,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Session update time
    pub updated_at: DateTime<Utc>,
    /// Message count
    pub message_count: u32,
    /// Whether this session is empty (no user AND no assistant messages)
    /// Story 2.29: Used for filtering empty projects in the UI
    pub is_empty: bool,
    /// Session title (from metadata, if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Original working directory at import time (Story 1.12)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_cwd: Option<String>,
}

/// Import result statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImportResult {
    /// Number of successfully imported sessions
    pub imported_count: u32,
    /// Number of skipped duplicate sessions
    pub skipped_count: u32,
    /// Number of newly created projects
    pub new_projects_count: u32,
    /// Import error list
    pub errors: Vec<String>,
}

impl Project {
    /// Create a new project with the given cwd
    pub fn new(id: String, cwd: String) -> Self {
        let name = extract_project_name(&cwd);
        let path_type = classify_path_type(&cwd);
        let path_exists = match path_type {
            PathType::Local => check_path_exists(&cwd),
            _ => true, // Virtual and remote paths don't need existence check
        };
        let now = Utc::now();
        Self {
            id,
            name,
            cwd,
            session_count: 0,
            non_empty_session_count: 0,
            created_at: now,
            last_activity: now,
            git_repo_path: None,
            has_git_repo: false,
            git_remote_url: None,
            is_empty: true, // New project starts as empty until sessions are added
            path_type,
            path_exists,
        }
    }

    /// Set Git repository information
    pub fn set_git_repo(&mut self, repo_path: Option<String>) {
        self.has_git_repo = repo_path.is_some();
        self.git_repo_path = repo_path;
    }

    /// Set Git remote URL (Story 1.9)
    pub fn set_git_remote_url(&mut self, url: Option<String>) {
        self.git_remote_url = url;
    }
}

/// Extract project name from cwd path
///
/// # Examples
/// ```
/// use client_lib::models::project::extract_project_name;
/// assert_eq!(extract_project_name("/Users/decker/projects/mantra"), "mantra");
/// assert_eq!(extract_project_name("/home/user/code/my-app"), "my-app");
/// ```
pub fn extract_project_name(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Project")
        .to_string()
}

/// Classify path type (Story 1.12)
///
/// Determines whether a path is local, virtual, or remote.
///
/// # Examples
/// ```
/// use client_lib::models::project::{classify_path_type, PathType};
/// assert_eq!(classify_path_type("/home/user/project"), PathType::Local);
/// assert_eq!(classify_path_type("gemini-project:abc123"), PathType::Virtual);
/// assert_eq!(classify_path_type("github.com/user/repo"), PathType::Remote);
/// ```
pub fn classify_path_type(path: &str) -> PathType {
    let path = path.trim();

    // Virtual paths: placeholders and special identifiers
    if path.is_empty()
        || path == "unknown"
        || path.starts_with("gemini-project:")
        || path.starts_with("placeholder:")
    {
        return PathType::Virtual;
    }

    // Remote paths: URLs and git references
    if path.starts_with("github.com/")
        || path.starts_with("gitlab.com/")
        || path.starts_with("bitbucket.org/")
        || path.starts_with("https://")
        || path.starts_with("http://")
        || path.starts_with("git@")
    {
        return PathType::Remote;
    }

    // Default to local path
    PathType::Local
}

/// Check if a local path exists on the filesystem (Story 1.12)
///
/// # Examples
/// ```
/// use client_lib::models::project::check_path_exists;
/// // This will return true for existing paths
/// assert!(check_path_exists("/tmp"));
/// // This will return false for non-existing paths
/// assert!(!check_path_exists("/nonexistent/path/12345"));
/// ```
pub fn check_path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

/// Normalize cwd path for consistent aggregation (Story 2.25)
///
/// - Removes trailing slashes
/// - Converts backslashes to forward slashes (cross-platform)
/// - Trims whitespace
///
/// # Examples
/// ```
/// use client_lib::models::project::normalize_cwd;
/// assert_eq!(normalize_cwd("/home/user/project/"), "/home/user/project");
/// assert_eq!(normalize_cwd("C:\\Users\\test\\project"), "C:/Users/test/project");
/// ```
pub fn normalize_cwd(cwd: &str) -> String {
    let normalized = cwd
        .trim()
        .replace('\\', "/"); // Cross-platform: backslashes to forward slashes

    // Remove trailing slashes (but keep root "/" or "C:/")
    let trimmed = normalized.trim_end_matches('/');

    // Handle edge cases: root paths
    if trimmed.is_empty() {
        "/".to_string()
    } else if trimmed.ends_with(':') {
        // Windows drive letter like "C:" -> "C:/"
        format!("{}/", trimmed)
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests;
