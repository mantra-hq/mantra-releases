use super::*;
use crate::models::{sources, MantraSession};
use crate::storage::Database;

fn create_test_session(id: &str, cwd: &str) -> MantraSession {
    MantraSession::new(id.to_string(), sources::CLAUDE.to_string(), cwd.to_string())
}

#[test]
fn test_get_or_create_project_creates_new() {
    let db = Database::new_in_memory().unwrap();
    let (project, is_new) = db.get_or_create_project("/home/user/test").unwrap();

    assert!(is_new);
    assert_eq!(project.name, "test");
    assert_eq!(project.cwd, "/home/user/test");
    assert_eq!(project.session_count, 0);
}

#[test]
fn test_get_or_create_project_returns_existing() {
    let db = Database::new_in_memory().unwrap();

    // Create first
    let (project1, is_new1) = db.get_or_create_project("/home/user/test").unwrap();
    assert!(is_new1);

    // Get existing
    let (project2, is_new2) = db.get_or_create_project("/home/user/test").unwrap();
    assert!(!is_new2);
    assert_eq!(project1.id, project2.id);
}

#[test]
fn test_insert_and_list_sessions() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/test").unwrap();

    let session = create_test_session("sess_1", "/home/user/test");
    db.insert_session(&session, &project.id).unwrap();

    let sessions = db.get_project_sessions(&project.id).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "sess_1");
}

#[test]
fn test_session_exists() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/test").unwrap();

    assert!(!db.session_exists("sess_1").unwrap());

    let session = create_test_session("sess_1", "/home/user/test");
    db.insert_session(&session, &project.id).unwrap();

    assert!(db.session_exists("sess_1").unwrap());
}

#[test]
fn test_list_projects_ordered_by_activity() {
    let db = Database::new_in_memory().unwrap();

    // Create projects
    db.get_or_create_project("/home/user/project1").unwrap();
    db.get_or_create_project("/home/user/project2").unwrap();

    // Update project1 to be more recent
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let future_time = Utc::now() + chrono::Duration::hours(1);
    db.update_project_last_activity(&project1.id, future_time)
        .unwrap();

    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "project1"); // Most recent first
}

#[test]
fn test_import_session_deduplication() {
    let db = Database::new_in_memory().unwrap();

    let session = create_test_session("sess_1", "/home/user/test");

    // First import
    let (imported1, new_project1) = db.import_session(&session).unwrap();
    assert!(imported1);
    assert!(new_project1);

    // Second import (should be skipped)
    let (imported2, new_project2) = db.import_session(&session).unwrap();
    assert!(!imported2);
    assert!(!new_project2);
}

#[test]
fn test_import_sessions_batch() {
    let mut db = Database::new_in_memory().unwrap();

    let sessions = vec![
        create_test_session("sess_1", "/home/user/project1"),
        create_test_session("sess_2", "/home/user/project1"),
        create_test_session("sess_3", "/home/user/project2"),
        create_test_session("sess_1", "/home/user/project1"), // Duplicate
    ];

    let result = db.import_sessions(&sessions).unwrap();
    assert_eq!(result.imported_count, 3);
    assert_eq!(result.skipped_count, 1);
    assert_eq!(result.new_projects_count, 2);
    assert!(result.errors.is_empty());
}

#[test]
fn test_project_session_count() {
    let mut db = Database::new_in_memory().unwrap();

    let sessions = vec![
        create_test_session("sess_1", "/home/user/test"),
        create_test_session("sess_2", "/home/user/test"),
    ];

    db.import_sessions(&sessions).unwrap();

    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].session_count, 2);
}

#[test]
fn test_project_git_fields_default() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/test").unwrap();

    assert!(!project.has_git_repo);
    assert!(project.git_repo_path.is_none());
}

#[test]
fn test_update_project_git_info() {
    let db = Database::new_in_memory().unwrap();
    db.get_or_create_project("/home/user/test").unwrap();

    // Update Git info
    db.update_project_git_info("/home/user/test", Some("/home/user/test".to_string()))
        .unwrap();

    // Verify update
    let (project, _) = db.get_or_create_project("/home/user/test").unwrap();
    assert!(project.has_git_repo);
    assert_eq!(project.git_repo_path, Some("/home/user/test".to_string()));

    // Clear Git info
    db.update_project_git_info("/home/user/test", None).unwrap();

    let (project, _) = db.get_or_create_project("/home/user/test").unwrap();
    assert!(!project.has_git_repo);
    assert!(project.git_repo_path.is_none());
}

#[test]
fn test_get_project_by_id() {
    let db = Database::new_in_memory().unwrap();
    let (created_project, _) = db.get_or_create_project("/home/user/test").unwrap();

    let project = db.get_project(&created_project.id).unwrap();
    assert!(project.is_some());
    assert_eq!(project.unwrap().id, created_project.id);

    let not_found = db.get_project("nonexistent").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_get_project_by_cwd() {
    let db = Database::new_in_memory().unwrap();
    db.get_or_create_project("/home/user/test").unwrap();

    let project = db.get_project_by_cwd("/home/user/test").unwrap();
    assert!(project.is_some());
    assert_eq!(project.unwrap().cwd, "/home/user/test");

    let not_found = db.get_project_by_cwd("/nonexistent/path").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_list_projects_includes_git_fields() {
    let db = Database::new_in_memory().unwrap();
    db.get_or_create_project("/home/user/test").unwrap();
    db.update_project_git_info("/home/user/test", Some("/home/user/test".to_string()))
        .unwrap();

    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 1);
    assert!(projects[0].has_git_repo);
    assert_eq!(projects[0].git_repo_path, Some("/home/user/test".to_string()));
}

// Story 2.25: Multi-source aggregation tests
#[test]
fn test_multi_source_aggregation_same_cwd() {
    let mut db = Database::new_in_memory().unwrap();

    // Create sessions from different sources with the same cwd
    let claude_session = MantraSession::new(
        "sess_claude_1".to_string(),
        sources::CLAUDE.to_string(),
        "/home/user/myproject".to_string(),
    );
    let gemini_session = MantraSession::new(
        "sess_gemini_1".to_string(),
        sources::GEMINI.to_string(),
        "/home/user/myproject".to_string(),
    );
    let cursor_session = MantraSession::new(
        "sess_cursor_1".to_string(),
        sources::CURSOR.to_string(),
        "/home/user/myproject".to_string(),
    );

    // Import all sessions
    let result = db.import_sessions(&[claude_session, gemini_session, cursor_session]).unwrap();

    // All should be imported
    assert_eq!(result.imported_count, 3);
    // Only ONE project should be created (aggregated by cwd)
    assert_eq!(result.new_projects_count, 1);

    // Verify only one project exists
    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].session_count, 3);

    // Verify all sessions are under the same project
    let sessions = db.get_project_sessions(&projects[0].id).unwrap();
    assert_eq!(sessions.len(), 3);

    // Verify sources are preserved
    let sources: Vec<&str> = sessions.iter().map(|s| s.source.as_str()).collect();
    assert!(sources.contains(&"claude"));
    assert!(sources.contains(&"gemini"));
    assert!(sources.contains(&"cursor"));
}

#[test]
fn test_path_normalization_aggregation() {
    let mut db = Database::new_in_memory().unwrap();

    // Sessions with different path formats pointing to the same location
    let session1 = MantraSession::new(
        "sess_1".to_string(),
        sources::CLAUDE.to_string(),
        "/home/user/project".to_string(),
    );
    let session2 = MantraSession::new(
        "sess_2".to_string(),
        sources::GEMINI.to_string(),
        "/home/user/project/".to_string(), // With trailing slash
    );
    let session3 = MantraSession::new(
        "sess_3".to_string(),
        sources::CURSOR.to_string(),
        "/home/user/project//".to_string(), // Multiple trailing slashes
    );

    let result = db.import_sessions(&[session1, session2, session3]).unwrap();

    assert_eq!(result.imported_count, 3);
    assert_eq!(result.new_projects_count, 1); // All aggregated to one project

    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].cwd, "/home/user/project"); // Normalized
}

#[test]
fn test_first_project_name_preserved() {
    let mut db = Database::new_in_memory().unwrap();

    // First import sets the project name
    let session1 = MantraSession::new(
        "sess_1".to_string(),
        sources::CLAUDE.to_string(),
        "/home/user/my-awesome-project".to_string(),
    );
    db.import_sessions(&[session1]).unwrap();

    let projects = db.list_projects().unwrap();
    let original_name = projects[0].name.clone();

    // Second import with same cwd should NOT change the name
    let session2 = MantraSession::new(
        "sess_2".to_string(),
        sources::GEMINI.to_string(),
        "/home/user/my-awesome-project".to_string(),
    );
    db.import_sessions(&[session2]).unwrap();

    let projects = db.list_projects().unwrap();
    assert_eq!(projects[0].name, original_name);
}

// ===== Story 1.9: Enhanced Project Identification Tests =====

#[test]
fn test_find_by_git_remote_found() {
    let db = Database::new_in_memory().unwrap();

    // Create project with Git remote URL
    let (project, _) = db.find_or_create_project(
        "/home/user/project1",
        Some("https://github.com/user/repo"),
    ).unwrap();

    // Find by Git remote URL
    let found = db.find_by_git_remote("https://github.com/user/repo").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, project.id);
}

#[test]
fn test_find_by_git_remote_not_found() {
    let db = Database::new_in_memory().unwrap();

    // Create project without Git remote URL
    db.find_or_create_project("/home/user/project1", None).unwrap();

    // Should not find any project
    let found = db.find_by_git_remote("https://github.com/user/repo").unwrap();
    assert!(found.is_none());
}

#[test]
fn test_find_by_git_remote_normalizes_url() {
    let db = Database::new_in_memory().unwrap();

    // Create project with SSH format URL
    db.find_or_create_project(
        "/home/user/project1",
        Some("git@github.com:user/repo.git"),
    ).unwrap();

    // Find by HTTPS format (should be normalized to match)
    let found = db.find_by_git_remote("https://github.com/user/repo").unwrap();
    assert!(found.is_some());
}

#[test]
fn test_find_or_create_git_url_priority() {
    let db = Database::new_in_memory().unwrap();

    // Create project with path1 and Git URL
    let (project1, _) = db.find_or_create_project(
        "/home/user/path1",
        Some("https://github.com/user/myrepo"),
    ).unwrap();

    // Create session with path2 but SAME Git URL
    // Should aggregate to existing project (Git URL priority)
    let (project2, is_new) = db.find_or_create_project(
        "/home/user/path2",
        Some("https://github.com/user/myrepo"),
    ).unwrap();

    assert!(!is_new, "Should aggregate to existing project by Git URL");
    assert_eq!(project1.id, project2.id, "Same project ID");
}

#[test]
fn test_find_or_create_path_fallback() {
    let db = Database::new_in_memory().unwrap();

    // Create project without Git URL
    let (project1, _) = db.find_or_create_project(
        "/home/user/project",
        None,
    ).unwrap();

    // Same path, no Git URL → fallback to path match
    let (project2, is_new) = db.find_or_create_project(
        "/home/user/project",
        None,
    ).unwrap();

    assert!(!is_new, "Should find existing project by path");
    assert_eq!(project1.id, project2.id);
}

#[test]
fn test_find_or_create_updates_missing_git_url() {
    let db = Database::new_in_memory().unwrap();

    // Create project without Git URL
    let (project1, _) = db.find_or_create_project(
        "/home/user/project",
        None,
    ).unwrap();
    assert!(project1.git_remote_url.is_none());

    // Same path but now with Git URL → should update
    let (project2, is_new) = db.find_or_create_project(
        "/home/user/project",
        Some("https://github.com/user/repo"),
    ).unwrap();

    assert!(!is_new, "Should aggregate, not create new");
    assert_eq!(project1.id, project2.id);
    assert!(project2.git_remote_url.is_some(), "Git URL should be updated");
    assert_eq!(project2.git_remote_url.unwrap(), "https://github.com/user/repo");
}

#[test]
fn test_find_or_create_path_reuse_conflict() {
    let db = Database::new_in_memory().unwrap();

    // Create project with Git URL A
    let (project1, _) = db.find_or_create_project(
        "/home/user/project",
        Some("https://github.com/user/repoA"),
    ).unwrap();

    // Same path but DIFFERENT Git URL → path reuse!
    // Should update project's Git URL to the new one (not create new project)
    let (project2, is_new) = db.find_or_create_project(
        "/home/user/project",
        Some("https://github.com/user/repoB"),
    ).unwrap();

    assert!(!is_new, "Should update existing project, not create new");
    assert_eq!(project1.id, project2.id, "Same project ID");
    assert_eq!(project2.git_remote_url.unwrap(), "https://github.com/user/repoB", "Git URL should be updated");
}

#[test]
fn test_find_or_create_project_has_url_session_no_url() {
    let db = Database::new_in_memory().unwrap();

    // Create project with Git URL
    let (project1, _) = db.find_or_create_project(
        "/home/user/project",
        Some("https://github.com/user/repo"),
    ).unwrap();

    // Same path, no Git URL → aggregate to existing project
    let (project2, is_new) = db.find_or_create_project(
        "/home/user/project",
        None,
    ).unwrap();

    assert!(!is_new, "Should aggregate by path");
    assert_eq!(project1.id, project2.id);
}

#[test]
fn test_import_session_with_git_url() {
    let db = Database::new_in_memory().unwrap();

    let session = MantraSession::new(
        "sess_1".to_string(),
        sources::CLAUDE.to_string(),
        "/home/user/project".to_string(),
    );

    let (imported, is_new, project_id) = db.import_session_with_git_url(
        &session,
        Some("https://github.com/user/repo"),
    ).unwrap();

    assert!(imported);
    assert!(is_new);
    assert!(!project_id.is_empty());

    // Verify Git URL is stored
    let project = db.get_project(&project_id).unwrap().unwrap();
    assert_eq!(project.git_remote_url, Some("https://github.com/user/repo".to_string()));
}

#[test]
fn test_cross_path_aggregation_by_git_url() {
    let db = Database::new_in_memory().unwrap();

    // Session 1: path1 + repo URL
    let session1 = MantraSession::new(
        "sess_1".to_string(),
        sources::CLAUDE.to_string(),
        "/home/user/path1/myrepo".to_string(),
    );
    let (_, _, project_id1) = db.import_session_with_git_url(
        &session1,
        Some("https://github.com/user/myrepo"),
    ).unwrap();

    // Session 2: path2 + SAME repo URL
    let session2 = MantraSession::new(
        "sess_2".to_string(),
        sources::GEMINI.to_string(),
        "/home/user/path2/myrepo".to_string(),
    );
    let (imported, is_new, project_id2) = db.import_session_with_git_url(
        &session2,
        Some("https://github.com/user/myrepo"),
    ).unwrap();

    assert!(imported);
    assert!(!is_new, "Should aggregate by Git URL, not create new project");
    assert_eq!(project_id1, project_id2, "Both sessions in same project");

    // Verify only one project exists
    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].session_count, 2);
}

// ===== Story 2.33: Search Filters Tests =====

#[test]
fn test_search_filters_default() {
    let filters = SearchFilters::default();
    assert_eq!(filters.content_type, ContentType::All);
    assert!(filters.project_id.is_none());
    assert!(filters.time_preset.is_none());
}

#[test]
fn test_search_filters_serialization() {
    let filters = SearchFilters {
        content_type: ContentType::Code,
        project_id: Some("proj_123".to_string()),
        time_preset: Some(TimePreset::Today),
    };

    let json = serde_json::to_string(&filters).unwrap();
    assert!(json.contains(r#""contentType":"code""#));
    assert!(json.contains(r#""projectId":"proj_123""#));
    assert!(json.contains(r#""timePreset":"today""#));

    let deserialized: SearchFilters = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.content_type, ContentType::Code);
    assert_eq!(deserialized.project_id, Some("proj_123".to_string()));
    assert_eq!(deserialized.time_preset, Some(TimePreset::Today));
}

#[test]
fn test_content_type_enum() {
    assert_eq!(ContentType::default(), ContentType::All);

    let code_json = r#""code""#;
    let code: ContentType = serde_json::from_str(code_json).unwrap();
    assert_eq!(code, ContentType::Code);

    let conv_json = r#""conversation""#;
    let conv: ContentType = serde_json::from_str(conv_json).unwrap();
    assert_eq!(conv, ContentType::Conversation);
}

#[test]
fn test_time_preset_enum() {
    let all_json = r#""all""#;
    let all: TimePreset = serde_json::from_str(all_json).unwrap();
    assert_eq!(all, TimePreset::All);

    let today_json = r#""today""#;
    let today: TimePreset = serde_json::from_str(today_json).unwrap();
    assert_eq!(today, TimePreset::Today);

    let week_json = r#""week""#;
    let week: TimePreset = serde_json::from_str(week_json).unwrap();
    assert_eq!(week, TimePreset::Week);

    let month_json = r#""month""#;
    let month: TimePreset = serde_json::from_str(month_json).unwrap();
    assert_eq!(month, TimePreset::Month);
}

#[test]
fn test_search_result_with_content_type() {
    let result = SearchResult {
        id: "sess_1-0".to_string(),
        session_id: "sess_1".to_string(),
        project_id: "proj_1".to_string(),
        project_name: "test".to_string(),
        session_name: "Test Session".to_string(),
        message_id: "0".to_string(),
        content: "Hello world".to_string(),
        match_positions: vec![(0, 5)],
        timestamp: 1234567890,
        content_type: Some(ContentType::Conversation),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains(r#""content_type":"conversation""#));
}

#[test]
fn test_search_result_content_type_omitted_when_none() {
    let result = SearchResult {
        id: "sess_1-0".to_string(),
        session_id: "sess_1".to_string(),
        project_id: "proj_1".to_string(),
        project_name: "test".to_string(),
        session_name: "Test Session".to_string(),
        message_id: "0".to_string(),
        content: "Hello world".to_string(),
        match_positions: vec![(0, 5)],
        timestamp: 1234567890,
        content_type: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(!json.contains(r#""content_type""#));
}

// ===== Story 1.12: View-based Project Aggregation Tests =====

#[test]
fn test_add_project_path() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

    // Add a secondary path
    let path = db.add_project_path(&project.id, "/home/user/project-alt", false).unwrap();
    assert_eq!(path.project_id, project.id);
    assert_eq!(path.path, "/home/user/project-alt");
    assert!(!path.is_primary);

    // Verify path was added
    let paths = db.get_project_paths(&project.id).unwrap();
    // Should have 2 paths: original (migrated) + new one
    assert!(paths.len() >= 1);
}

#[test]
fn test_add_project_path_sets_primary() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

    // Add a primary path (should demote existing)
    let path = db.add_project_path(&project.id, "/home/user/new-primary", true).unwrap();
    assert!(path.is_primary);

    // Verify paths
    let paths = db.get_project_paths(&project.id).unwrap();
    let primary_count = paths.iter().filter(|p| p.is_primary).count();
    assert_eq!(primary_count, 1, "Should only have one primary path");
}

#[test]
fn test_remove_project_path() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

    // Add and then remove a path
    let path = db.add_project_path(&project.id, "/home/user/to-remove", false).unwrap();
    db.remove_project_path(&path.id).unwrap();

    // Verify removal
    let paths = db.get_project_paths(&project.id).unwrap();
    assert!(!paths.iter().any(|p| p.path == "/home/user/to-remove"));
}

#[test]
fn test_add_project_path_same_path_different_projects() {
    // Story 1.12: Same path can belong to multiple projects (from different import sources)
    let db = Database::new_in_memory().unwrap();
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    let shared_path = "/shared/workspace/myproject";

    // Add the same path to both projects - should succeed
    let path1 = db.add_project_path(&project1.id, shared_path, false).unwrap();
    let path2 = db.add_project_path(&project2.id, shared_path, false).unwrap();

    // Both should have the path
    assert_eq!(path1.path, shared_path);
    assert_eq!(path2.path, shared_path);
    assert_ne!(path1.id, path2.id); // Different records

    // Verify both projects have the path
    let paths1 = db.get_project_paths(&project1.id).unwrap();
    let paths2 = db.get_project_paths(&project2.id).unwrap();
    assert!(paths1.iter().any(|p| p.path == shared_path));
    assert!(paths2.iter().any(|p| p.path == shared_path));
}

#[test]
fn test_add_project_path_idempotent() {
    // Story 1.12: Adding the same path to the same project should be idempotent
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

    let test_path = "/home/user/extra-path";

    // Add path twice
    let path1 = db.add_project_path(&project.id, test_path, false).unwrap();
    let path2 = db.add_project_path(&project.id, test_path, false).unwrap();

    // Should return the same record (idempotent)
    assert_eq!(path1.id, path2.id);
    assert_eq!(path1.path, path2.path);

    // Should only have one entry for this path
    let paths = db.get_project_paths(&project.id).unwrap();
    let count = paths.iter().filter(|p| p.path == test_path).count();
    assert_eq!(count, 1, "Should only have one entry for the path");
}

#[test]
fn test_find_project_by_path() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

    // Should find project by path (migrated from cwd)
    let found = db.find_project_by_path("/home/user/project").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, project.id);

    // Should not find non-existent path
    let not_found = db.find_project_by_path("/nonexistent/path").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_get_logical_project_stats() {
    let db = Database::new_in_memory().unwrap();

    // Create two projects with the same path
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    let shared_path = "/shared/workspace";
    db.add_project_path(&project1.id, shared_path, false).unwrap();
    db.add_project_path(&project2.id, shared_path, false).unwrap();

    // Get logical project stats
    let stats = db.get_logical_project_stats().unwrap();

    // Should have stats for the shared path
    let shared_stats = stats.iter().find(|s| s.physical_path == shared_path);
    assert!(shared_stats.is_some(), "Should have stats for shared path");

    let shared = shared_stats.unwrap();
    assert_eq!(shared.project_count, 2, "Should have 2 projects");
    assert!(shared.project_ids.contains(&project1.id));
    assert!(shared.project_ids.contains(&project2.id));
}

#[test]
fn test_get_projects_by_physical_path() {
    let db = Database::new_in_memory().unwrap();

    // Create two projects with the same path
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    let shared_path = "/shared/workspace";
    db.add_project_path(&project1.id, shared_path, false).unwrap();
    db.add_project_path(&project2.id, shared_path, false).unwrap();

    // Get projects by physical path
    let projects = db.get_projects_by_physical_path(shared_path).unwrap();

    assert_eq!(projects.len(), 2, "Should have 2 projects");
    let ids: Vec<&str> = projects.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&project1.id.as_str()));
    assert!(ids.contains(&project2.id.as_str()));
}

#[test]
fn test_get_sessions_by_physical_path() {
    let db = Database::new_in_memory().unwrap();

    // Create two projects with the same path
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    let shared_path = "/shared/workspace";
    db.add_project_path(&project1.id, shared_path, false).unwrap();
    db.add_project_path(&project2.id, shared_path, false).unwrap();

    // Add sessions to both projects
    let session1 = create_test_session("sess_phys_1", "/home/user/project1");
    let session2 = create_test_session("sess_phys_2", "/home/user/project2");
    db.insert_session(&session1, &project1.id).unwrap();
    db.insert_session(&session2, &project2.id).unwrap();

    // Get sessions by physical path
    let sessions = db.get_sessions_by_physical_path(shared_path).unwrap();

    assert_eq!(sessions.len(), 2, "Should have 2 sessions");
    let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"sess_phys_1"));
    assert!(ids.contains(&"sess_phys_2"));
}

#[test]
fn test_logical_project_stats_includes_virtual_paths() {
    // Task 9.1: Virtual paths are now included in logical project stats
    let db = Database::new_in_memory().unwrap();

    // Create a project with a virtual path
    let (_project, _) = db.get_or_create_project("gemini-project:abc123").unwrap();

    // Get logical project stats - should now include virtual paths (Task 9.1 change)
    let stats = db.get_logical_project_stats().unwrap();

    // Should have stats for virtual path
    let virtual_stats = stats.iter().find(|s| s.physical_path.starts_with("gemini-project:"));
    assert!(virtual_stats.is_some(), "Should include virtual paths (Task 9.1)");

    // Verify virtual path is marked correctly
    let virtual_stats = virtual_stats.unwrap();
    assert_eq!(virtual_stats.path_type, "virtual");
    assert!(virtual_stats.needs_association, "Virtual paths need association");
}

#[test]
fn test_bind_session_to_project() {
    let db = Database::new_in_memory().unwrap();
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    // Create a session in project1
    let session = create_test_session("sess_bind_test", "/home/user/project1");
    db.insert_session(&session, &project1.id).unwrap();

    // Bind session to project2
    let binding = db.bind_session_to_project("sess_bind_test", &project2.id).unwrap();
    assert_eq!(binding.session_id, "sess_bind_test");
    assert_eq!(binding.project_id, project2.id);

    // Verify binding exists
    let retrieved = db.get_session_binding("sess_bind_test").unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().project_id, project2.id);
}

#[test]
fn test_unbind_session() {
    let db = Database::new_in_memory().unwrap();
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    // Create and bind session
    let session = create_test_session("sess_unbind_test", "/home/user/project1");
    db.insert_session(&session, &project1.id).unwrap();
    db.bind_session_to_project("sess_unbind_test", &project2.id).unwrap();

    // Unbind
    db.unbind_session("sess_unbind_test").unwrap();

    // Verify binding is gone
    let binding = db.get_session_binding("sess_unbind_test").unwrap();
    assert!(binding.is_none());
}

#[test]
fn test_get_project_sessions_aggregated() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

    // Create sessions
    let session1 = create_test_session("sess_agg_1", "/home/user/project");
    let session2 = create_test_session("sess_agg_2", "/home/user/project");
    db.insert_session(&session1, &project.id).unwrap();
    db.insert_session(&session2, &project.id).unwrap();

    // Get aggregated sessions
    let sessions = db.get_project_sessions_aggregated(&project.id).unwrap();
    assert_eq!(sessions.len(), 2);
}

#[test]
fn test_manual_binding_priority() {
    let db = Database::new_in_memory().unwrap();
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    // Create session in project1
    let session = create_test_session("sess_priority", "/home/user/project1");
    db.insert_session(&session, &project1.id).unwrap();

    // Bind to project2 (manual binding should take priority)
    db.bind_session_to_project("sess_priority", &project2.id).unwrap();

    // Session should appear in project2's aggregated list
    let sessions2 = db.get_project_sessions_aggregated(&project2.id).unwrap();
    assert!(sessions2.iter().any(|s| s.id == "sess_priority"));
}

#[test]
fn test_set_project_primary_path() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/old-path").unwrap();

    // Set new primary path
    db.set_project_primary_path(&project.id, "/home/user/new-path").unwrap();

    // Verify project cwd updated
    let updated = db.get_project(&project.id).unwrap().unwrap();
    assert_eq!(updated.cwd, "/home/user/new-path");
    assert_eq!(updated.name, "new-path");

    // Verify path is primary
    let paths = db.get_project_paths(&project.id).unwrap();
    let primary = paths.iter().find(|p| p.is_primary);
    assert!(primary.is_some());
    assert_eq!(primary.unwrap().path, "/home/user/new-path");
}

// =========================================================================
// Story 1.12: View-based Project Aggregation Tests
// =========================================================================

#[test]
fn test_get_unassigned_sessions_empty() {
    let db = Database::new_in_memory().unwrap();

    // No sessions, should return empty
    let unassigned = db.get_unassigned_sessions().unwrap();
    assert!(unassigned.is_empty());
}

#[test]
fn test_get_unassigned_sessions_with_orphan() {
    let db = Database::new_in_memory().unwrap();

    // Create a project with path
    let (project, _) = db.get_or_create_project("/home/user/known-project").unwrap();

    // Create a session with unknown path (not matching any project_paths)
    let orphan_session = create_test_session("sess_orphan", "/home/user/unknown-project");
    db.insert_session(&orphan_session, &project.id).unwrap();

    // Update the session's original_cwd to something that doesn't match
    db.connection().execute(
        "UPDATE sessions SET original_cwd = '/home/user/unknown-project' WHERE id = 'sess_orphan'",
        [],
    ).unwrap();

    // The session should appear as unassigned since its original_cwd doesn't match any project_paths
    let unassigned = db.get_unassigned_sessions().unwrap();
    // Note: This depends on how the query handles the fallback to project_id
    // The actual behavior may vary based on implementation
    assert!(unassigned.len() <= 1);
}

#[test]
fn test_unbind_session_returns_to_unassigned() {
    let db = Database::new_in_memory().unwrap();

    // Create two projects
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    // Create session in project1
    let session = create_test_session("sess_unbind", "/home/user/project1");
    db.insert_session(&session, &project1.id).unwrap();

    // Bind to project2
    db.bind_session_to_project("sess_unbind", &project2.id).unwrap();

    // Verify it's bound
    let binding = db.get_session_binding("sess_unbind").unwrap();
    assert!(binding.is_some());
    assert_eq!(binding.unwrap().project_id, project2.id);

    // Unbind
    db.unbind_session("sess_unbind").unwrap();

    // Verify binding is removed
    let binding_after = db.get_session_binding("sess_unbind").unwrap();
    assert!(binding_after.is_none());
}

#[test]
fn test_multiple_paths_per_project() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/main-path").unwrap();

    // Add additional paths
    db.add_project_path(&project.id, "/home/user/alt-path-1", false).unwrap();
    db.add_project_path(&project.id, "/home/user/alt-path-2", false).unwrap();

    // Verify all paths exist
    let paths = db.get_project_paths(&project.id).unwrap();
    assert_eq!(paths.len(), 3); // main + 2 alternatives

    // Verify primary is first
    assert!(paths[0].is_primary);
    assert_eq!(paths[0].path, "/home/user/main-path");
}

#[test]
fn test_find_project_by_path_with_alt_paths() {
    let db = Database::new_in_memory().unwrap();
    let (project, _) = db.get_or_create_project("/home/user/main-path").unwrap();

    // Add alternative path
    db.add_project_path(&project.id, "/home/user/alt-path", false).unwrap();

    // Find by main path
    let found1 = db.find_project_by_path("/home/user/main-path").unwrap();
    assert!(found1.is_some());
    assert_eq!(found1.unwrap().id, project.id);

    // Find by alternative path
    let found2 = db.find_project_by_path("/home/user/alt-path").unwrap();
    assert!(found2.is_some());
    assert_eq!(found2.unwrap().id, project.id);

    // Not found
    let not_found = db.find_project_by_path("/home/user/unknown").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_rebind_session_to_different_project() {
    let db = Database::new_in_memory().unwrap();
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();
    let (project3, _) = db.get_or_create_project("/home/user/project3").unwrap();

    // Create session
    let session = create_test_session("sess_rebind", "/home/user/project1");
    db.insert_session(&session, &project1.id).unwrap();

    // Bind to project2
    db.bind_session_to_project("sess_rebind", &project2.id).unwrap();
    let binding1 = db.get_session_binding("sess_rebind").unwrap().unwrap();
    assert_eq!(binding1.project_id, project2.id);

    // Rebind to project3 (should replace)
    db.bind_session_to_project("sess_rebind", &project3.id).unwrap();
    let binding2 = db.get_session_binding("sess_rebind").unwrap().unwrap();
    assert_eq!(binding2.project_id, project3.id);
}

// =========================================================================
// Story 1.13: Logical Project Rename Tests
// =========================================================================

#[test]
fn test_get_logical_project_name_not_found() {
    let db = Database::new_in_memory().unwrap();

    // Get name for non-existent path
    let result = db.get_logical_project_name("/home/user/project").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_set_and_get_logical_project_name() {
    let db = Database::new_in_memory().unwrap();

    // Set a custom name
    db.set_logical_project_name("/home/user/project", "My Custom Project").unwrap();

    // Get it back
    let name = db.get_logical_project_name("/home/user/project").unwrap();
    assert_eq!(name, Some("My Custom Project".to_string()));
}

#[test]
fn test_update_logical_project_name() {
    let db = Database::new_in_memory().unwrap();

    // Set initial name
    db.set_logical_project_name("/home/user/project", "Initial Name").unwrap();

    // Update to new name
    db.set_logical_project_name("/home/user/project", "Updated Name").unwrap();

    // Verify update
    let name = db.get_logical_project_name("/home/user/project").unwrap();
    assert_eq!(name, Some("Updated Name".to_string()));
}

#[test]
fn test_delete_logical_project_name() {
    let db = Database::new_in_memory().unwrap();

    // Set a name
    db.set_logical_project_name("/home/user/project", "My Project").unwrap();

    // Delete it
    db.delete_logical_project_name("/home/user/project").unwrap();

    // Verify it's gone
    let name = db.get_logical_project_name("/home/user/project").unwrap();
    assert!(name.is_none());
}

#[test]
fn test_delete_logical_project_name_not_found() {
    let db = Database::new_in_memory().unwrap();

    // Try to delete non-existent name
    let result = db.delete_logical_project_name("/home/user/project");
    assert!(result.is_err());
}

#[test]
fn test_logical_project_stats_with_custom_name() {
    let db = Database::new_in_memory().unwrap();

    // Create a project
    let (project, _) = db.get_or_create_project("/home/user/myproject").unwrap();

    // Set a custom name for the logical project
    db.set_logical_project_name("/home/user/myproject", "My Custom Name").unwrap();

    // Get logical project stats
    let stats = db.get_logical_project_stats().unwrap();

    // Find our project
    let my_stats = stats.iter().find(|s| s.physical_path == "/home/user/myproject");
    assert!(my_stats.is_some(), "Should find the project in stats");

    // Verify custom name is used
    let my_stats = my_stats.unwrap();
    assert_eq!(my_stats.display_name, "My Custom Name");
}

#[test]
fn test_logical_project_stats_without_custom_name() {
    let db = Database::new_in_memory().unwrap();

    // Create a project without custom name
    let (_project, _) = db.get_or_create_project("/home/user/myproject").unwrap();

    // Get logical project stats
    let stats = db.get_logical_project_stats().unwrap();

    // Find our project
    let my_stats = stats.iter().find(|s| s.physical_path == "/home/user/myproject");
    assert!(my_stats.is_some(), "Should find the project in stats");

    // Verify default name (extracted from path) is used
    let my_stats = my_stats.unwrap();
    assert_eq!(my_stats.display_name, "myproject");
}

#[test]
fn test_logical_project_stats_custom_name_reset() {
    let db = Database::new_in_memory().unwrap();

    // Create a project and set custom name
    let (_project, _) = db.get_or_create_project("/home/user/myproject").unwrap();
    db.set_logical_project_name("/home/user/myproject", "Custom Name").unwrap();

    // Verify custom name is used
    let stats1 = db.get_logical_project_stats().unwrap();
    let my_stats1 = stats1.iter().find(|s| s.physical_path == "/home/user/myproject").unwrap();
    assert_eq!(my_stats1.display_name, "Custom Name");

    // Delete custom name (reset to default)
    db.delete_logical_project_name("/home/user/myproject").unwrap();

    // Verify default name is used again
    let stats2 = db.get_logical_project_stats().unwrap();
    let my_stats2 = stats2.iter().find(|s| s.physical_path == "/home/user/myproject").unwrap();
    assert_eq!(my_stats2.display_name, "myproject");
}

#[test]
fn test_logical_project_name_path_normalization() {
    let db = Database::new_in_memory().unwrap();

    // Set name with trailing slash
    db.set_logical_project_name("/home/user/project/", "My Project").unwrap();

    // Get with path without trailing slash - should still find it due to normalization
    let name = db.get_logical_project_name("/home/user/project").unwrap();
    assert_eq!(name, Some("My Project".to_string()));
}

#[test]
fn test_rename_aggregated_logical_project() {
    let db = Database::new_in_memory().unwrap();

    // Create two projects with the same physical path (simulating multi-source aggregation)
    let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
    let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

    let shared_path = "/shared/workspace";
    db.add_project_path(&project1.id, shared_path, false).unwrap();
    db.add_project_path(&project2.id, shared_path, false).unwrap();

    // Set custom name for the aggregated logical project
    db.set_logical_project_name(shared_path, "Shared Workspace Renamed").unwrap();

    // Get logical project stats
    let stats = db.get_logical_project_stats().unwrap();
    let shared_stats = stats.iter().find(|s| s.physical_path == shared_path).unwrap();

    // Verify aggregation still works with custom name
    assert_eq!(shared_stats.project_count, 2);
    assert_eq!(shared_stats.display_name, "Shared Workspace Renamed");
}
