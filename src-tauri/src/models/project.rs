//! Project and session summary data models
//!
//! Defines the Project and SessionSummary structures for representing
//! aggregated project data and lightweight session listings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::session::SessionSource;

/// Project data representing an aggregated working directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Project {
    /// Project unique ID (UUID)
    pub id: String,
    /// Project name (directory name extracted from cwd)
    pub name: String,
    /// Working directory absolute path
    pub cwd: String,
    /// Number of sessions in this project
    pub session_count: u32,
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
        let now = Utc::now();
        Self {
            id,
            name,
            cwd,
            session_count: 0,
            created_at: now,
            last_activity: now,
            git_repo_path: None,
            has_git_repo: false,
            git_remote_url: None,
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
mod tests {
    use super::*;
    use crate::models::sources;

    #[test]
    fn test_extract_project_name() {
        assert_eq!(
            extract_project_name("/Users/decker/projects/mantra"),
            "mantra"
        );
        assert_eq!(extract_project_name("/home/user/code/my-app"), "my-app");
        assert_eq!(extract_project_name("/single"), "single");
        assert_eq!(extract_project_name("relative/path/project"), "project");
    }

    #[test]
    fn test_extract_project_name_edge_cases() {
        // Root path
        assert_eq!(extract_project_name("/"), "Unknown Project");
        // Empty string
        assert_eq!(extract_project_name(""), "Unknown Project");
        // Trailing slash
        assert_eq!(extract_project_name("/path/to/project/"), "project");
    }

    #[test]
    fn test_project_new() {
        let project = Project::new(
            "test-id".to_string(),
            "/home/user/myproject".to_string(),
        );
        assert_eq!(project.id, "test-id");
        assert_eq!(project.name, "myproject");
        assert_eq!(project.cwd, "/home/user/myproject");
        assert_eq!(project.session_count, 0);
        assert!(project.git_repo_path.is_none());
        assert!(!project.has_git_repo);
    }

    #[test]
    fn test_project_set_git_repo() {
        let mut project = Project::new(
            "test-id".to_string(),
            "/home/user/myproject".to_string(),
        );

        // Initially no Git repo
        assert!(!project.has_git_repo);
        assert!(project.git_repo_path.is_none());

        // Set Git repo
        project.set_git_repo(Some("/home/user/myproject".to_string()));
        assert!(project.has_git_repo);
        assert_eq!(project.git_repo_path, Some("/home/user/myproject".to_string()));

        // Clear Git repo
        project.set_git_repo(None);
        assert!(!project.has_git_repo);
        assert!(project.git_repo_path.is_none());
    }

    #[test]
    fn test_project_serialization() {
        let mut project = Project::new(
            "proj_123".to_string(),
            "/home/user/test".to_string(),
        );
        project.set_git_repo(Some("/home/user/test".to_string()));

        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains(r#""id":"proj_123""#));
        assert!(json.contains(r#""name":"test""#));
        assert!(json.contains(r#""cwd":"/home/user/test""#));
        assert!(json.contains(r#""git_repo_path":"/home/user/test""#));
        assert!(json.contains(r#""has_git_repo":true"#));

        let deserialized: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, project.id);
        assert_eq!(deserialized.name, project.name);
        assert_eq!(deserialized.git_repo_path, project.git_repo_path);
        assert_eq!(deserialized.has_git_repo, project.has_git_repo);
    }

    #[test]
    fn test_session_summary_serialization() {
        let summary = SessionSummary {
            id: "sess_123".to_string(),
            source: sources::CLAUDE.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            message_count: 10,
            is_empty: false,
            title: Some("Test Session".to_string()),
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains(r#""id":"sess_123""#));
        assert!(json.contains(r#""source":"claude""#));
        assert!(json.contains(r#""message_count":10"#));
        assert!(json.contains(r#""title":"Test Session""#));

        let deserialized: SessionSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, summary.id);
        assert_eq!(deserialized.message_count, 10);
        assert_eq!(deserialized.title, Some("Test Session".to_string()));
    }

    #[test]
    fn test_import_result_default() {
        let result = ImportResult::default();
        assert_eq!(result.imported_count, 0);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.new_projects_count, 0);
        assert!(result.errors.is_empty());
    }

    // Story 2.25: normalize_cwd tests
    #[test]
    fn test_normalize_cwd_trailing_slash() {
        assert_eq!(normalize_cwd("/home/user/project/"), "/home/user/project");
        assert_eq!(normalize_cwd("/home/user/project"), "/home/user/project");
        assert_eq!(normalize_cwd("/path/to/dir///"), "/path/to/dir");
    }

    #[test]
    fn test_normalize_cwd_backslashes() {
        assert_eq!(normalize_cwd("C:\\Users\\test\\project"), "C:/Users/test/project");
        assert_eq!(normalize_cwd("C:\\Users\\test\\project\\"), "C:/Users/test/project");
    }

    #[test]
    fn test_normalize_cwd_whitespace() {
        assert_eq!(normalize_cwd("  /home/user/project  "), "/home/user/project");
        assert_eq!(normalize_cwd("\t/path/to/dir\n"), "/path/to/dir");
    }

    #[test]
    fn test_normalize_cwd_edge_cases() {
        // Root paths
        assert_eq!(normalize_cwd("/"), "/");
        assert_eq!(normalize_cwd("C:"), "C:/");
        assert_eq!(normalize_cwd("C:\\"), "C:/");
        // Empty/whitespace
        assert_eq!(normalize_cwd(""), "/");
        assert_eq!(normalize_cwd("   "), "/");
    }

    #[test]
    fn test_normalize_cwd_aggregation_scenario() {
        // Different formats of the same path should normalize to the same value
        let paths = vec![
            "/home/user/myproject",
            "/home/user/myproject/",
            "/home/user/myproject//",
        ];
        let normalized: Vec<String> = paths.iter().map(|p| normalize_cwd(p)).collect();
        assert!(normalized.iter().all(|p| p == "/home/user/myproject"));
    }
}
