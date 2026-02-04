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
    assert!(project.is_empty); // New projects start as empty
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
        original_cwd: Some("/home/user/project".to_string()),
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

// ===== Story 1.12: View-based Project Aggregation Tests =====

#[test]
fn test_project_path_serialization() {
    let path = ProjectPath {
        id: "path_123".to_string(),
        project_id: "proj_456".to_string(),
        path: "/home/user/myproject".to_string(),
        is_primary: true,
        created_at: Utc::now(),
    };

    let json = serde_json::to_string(&path).unwrap();
    assert!(json.contains(r#""id":"path_123""#));
    assert!(json.contains(r#""project_id":"proj_456""#));
    assert!(json.contains(r#""path":"/home/user/myproject""#));
    assert!(json.contains(r#""is_primary":true"#));

    let deserialized: ProjectPath = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, path.id);
    assert_eq!(deserialized.project_id, path.project_id);
    assert_eq!(deserialized.path, path.path);
    assert_eq!(deserialized.is_primary, path.is_primary);
}

#[test]
fn test_session_binding_serialization() {
    let binding = SessionBinding {
        session_id: "sess_123".to_string(),
        project_id: "proj_456".to_string(),
        bound_at: Utc::now(),
    };

    let json = serde_json::to_string(&binding).unwrap();
    assert!(json.contains(r#""session_id":"sess_123""#));
    assert!(json.contains(r#""project_id":"proj_456""#));
    assert!(json.contains(r#""bound_at""#));

    let deserialized: SessionBinding = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.session_id, binding.session_id);
    assert_eq!(deserialized.project_id, binding.project_id);
}

#[test]
fn test_source_context_default() {
    let ctx = SourceContext::default();
    assert!(ctx.file_path.is_none());
    assert!(ctx.project_path_encoded.is_none());
    assert!(ctx.project_hash.is_none());
    assert!(ctx.session_filename.is_none());
    assert!(ctx.workspace_id.is_none());
    assert!(ctx.workspace_path.is_none());
}

#[test]
fn test_source_context_claude() {
    let ctx = SourceContext {
        file_path: Some("~/.claude/projects/-mnt-disk0-project-foo/abc.jsonl".to_string()),
        project_path_encoded: Some("-mnt-disk0-project-foo".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&ctx).unwrap();
    assert!(json.contains(r#""file_path""#));
    assert!(json.contains(r#""project_path_encoded""#));
    // Should not contain unset fields
    assert!(!json.contains(r#""project_hash""#));
    assert!(!json.contains(r#""workspace_id""#));

    let deserialized: SourceContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.project_path_encoded, ctx.project_path_encoded);
}

#[test]
fn test_source_context_gemini() {
    let ctx = SourceContext {
        file_path: Some("~/.gemini/tmp/abc123/chats/session-xxx.json".to_string()),
        project_hash: Some("abc123def456".to_string()),
        session_filename: Some("session-2025-12-30-xxx.json".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&ctx).unwrap();
    assert!(json.contains(r#""project_hash""#));
    assert!(json.contains(r#""session_filename""#));

    let deserialized: SourceContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.project_hash, ctx.project_hash);
    assert_eq!(deserialized.session_filename, ctx.session_filename);
}

#[test]
fn test_source_context_cursor() {
    let ctx = SourceContext {
        workspace_id: Some("a1b2c3d4".to_string()),
        workspace_path: Some("~/.config/Cursor/User/workspaceStorage/a1b2c3d4/".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&ctx).unwrap();
    assert!(json.contains(r#""workspace_id""#));
    assert!(json.contains(r#""workspace_path""#));

    let deserialized: SourceContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.workspace_id, ctx.workspace_id);
    assert_eq!(deserialized.workspace_path, ctx.workspace_path);
}

// ===== Story 1.12: Path Type Classification Tests =====

#[test]
fn test_classify_path_type_local() {
    assert_eq!(classify_path_type("/home/user/project"), PathType::Local);
    assert_eq!(classify_path_type("/mnt/disk0/project/foo"), PathType::Local);
    assert_eq!(classify_path_type("C:/Users/test/project"), PathType::Local);
    assert_eq!(classify_path_type("/tmp"), PathType::Local);
}

#[test]
fn test_classify_path_type_virtual() {
    assert_eq!(classify_path_type("gemini-project:abc123"), PathType::Virtual);
    assert_eq!(classify_path_type("placeholder:unknown"), PathType::Virtual);
    assert_eq!(classify_path_type(""), PathType::Virtual);
    assert_eq!(classify_path_type("unknown"), PathType::Virtual);
    assert_eq!(classify_path_type("  "), PathType::Virtual);
}

#[test]
fn test_classify_path_type_remote() {
    assert_eq!(classify_path_type("github.com/user/repo"), PathType::Remote);
    assert_eq!(classify_path_type("gitlab.com/user/repo"), PathType::Remote);
    assert_eq!(classify_path_type("bitbucket.org/user/repo"), PathType::Remote);
    assert_eq!(classify_path_type("https://github.com/user/repo"), PathType::Remote);
    assert_eq!(classify_path_type("git@github.com:user/repo.git"), PathType::Remote);
}

#[test]
fn test_check_path_exists() {
    // /tmp should exist on most systems
    assert!(check_path_exists("/tmp"));
    // Non-existent path
    assert!(!check_path_exists("/nonexistent/path/12345/67890"));
}

#[test]
fn test_path_type_serialization() {
    assert_eq!(PathType::Local.as_str(), "local");
    assert_eq!(PathType::Virtual.as_str(), "virtual");
    assert_eq!(PathType::Remote.as_str(), "remote");

    assert_eq!(PathType::from_str("local"), PathType::Local);
    assert_eq!(PathType::from_str("virtual"), PathType::Virtual);
    assert_eq!(PathType::from_str("remote"), PathType::Remote);
    assert_eq!(PathType::from_str("LOCAL"), PathType::Local);
    assert_eq!(PathType::from_str("unknown"), PathType::Local); // Default
}

#[test]
fn test_project_new_with_path_type() {
    // Local path
    let project = Project::new("id1".to_string(), "/home/user/project".to_string());
    assert_eq!(project.path_type, PathType::Local);

    // Virtual path
    let project = Project::new("id2".to_string(), "gemini-project:abc123".to_string());
    assert_eq!(project.path_type, PathType::Virtual);
    assert!(project.path_exists); // Virtual paths always "exist"

    // Remote path
    let project = Project::new("id3".to_string(), "github.com/user/repo".to_string());
    assert_eq!(project.path_type, PathType::Remote);
    assert!(project.path_exists); // Remote paths always "exist"
}

#[test]
fn test_project_serialization_with_path_type() {
    let mut project = Project::new("proj_123".to_string(), "/home/user/test".to_string());
    project.path_type = PathType::Virtual;
    project.path_exists = false;

    let json = serde_json::to_string(&project).unwrap();
    assert!(json.contains(r#""path_type":"virtual""#));
    assert!(json.contains(r#""path_exists":false"#));

    let deserialized: Project = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.path_type, PathType::Virtual);
    assert!(!deserialized.path_exists);
}
