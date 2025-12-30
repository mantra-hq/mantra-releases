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
        }
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

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new(
            "proj_123".to_string(),
            "/home/user/test".to_string(),
        );
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains(r#""id":"proj_123""#));
        assert!(json.contains(r#""name":"test""#));
        assert!(json.contains(r#""cwd":"/home/user/test""#));

        let deserialized: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, project.id);
        assert_eq!(deserialized.name, project.name);
    }

    #[test]
    fn test_session_summary_serialization() {
        let summary = SessionSummary {
            id: "sess_123".to_string(),
            source: SessionSource::Claude,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            message_count: 10,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains(r#""id":"sess_123""#));
        assert!(json.contains(r#""source":"claude""#));
        assert!(json.contains(r#""message_count":10"#));

        let deserialized: SessionSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, summary.id);
        assert_eq!(deserialized.message_count, 10);
    }

    #[test]
    fn test_import_result_default() {
        let result = ImportResult::default();
        assert_eq!(result.imported_count, 0);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.new_projects_count, 0);
        assert!(result.errors.is_empty());
    }
}
