use super::*;
use tempfile::tempdir;

#[test]
fn test_database_creation() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Database::new(&db_path);
    assert!(db.is_ok(), "Database creation failed: {:?}", db.err());

    // Verify database file exists
    assert!(db_path.exists());
}

#[test]
fn test_in_memory_database() {
    let db = Database::new_in_memory();
    assert!(db.is_ok(), "In-memory database creation failed: {:?}", db.err());
}

#[test]
fn test_schema_initialization() {
    let db = Database::new_in_memory().unwrap();

    // Verify projects table exists
    let result = db.connection().execute(
        "SELECT 1 FROM projects LIMIT 1",
        [],
    );
    // Table exists but is empty, so query should succeed
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify sessions table exists
    let result = db.connection().execute(
        "SELECT 1 FROM sessions LIMIT 1",
        [],
    );
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));
}

#[test]
fn test_foreign_keys_enabled() {
    let db = Database::new_in_memory().unwrap();

    let fk_enabled: i32 = db
        .connection()
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();

    assert_eq!(fk_enabled, 1, "Foreign keys should be enabled");
}

#[test]
fn test_interception_records_table_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify interception_records table exists
    let result = db.connection().execute(
        "SELECT 1 FROM interception_records LIMIT 1",
        [],
    );
    // Table exists but is empty, so query should succeed
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(interception_records)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"timestamp".to_string()));
    assert!(columns.contains(&"source_type".to_string()));
    assert!(columns.contains(&"source_context".to_string()));
    assert!(columns.contains(&"matches".to_string()));
    assert!(columns.contains(&"user_action".to_string()));
    assert!(columns.contains(&"original_text_hash".to_string()));
    assert!(columns.contains(&"project_name".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
}

#[test]
fn test_interception_records_indexes_exist() {
    let db = Database::new_in_memory().unwrap();

    // Verify indexes exist
    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='interception_records'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_records_timestamp".to_string()));
    assert!(indexes.contains(&"idx_records_source".to_string()));
    assert!(indexes.contains(&"idx_records_project".to_string()));
}

// ===== Story 1.12: View-based Project Aggregation Tests =====

#[test]
fn test_project_paths_table_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify project_paths table exists
    let result = db.connection().execute(
        "SELECT 1 FROM project_paths LIMIT 1",
        [],
    );
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(project_paths)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"project_id".to_string()));
    assert!(columns.contains(&"path".to_string()));
    assert!(columns.contains(&"is_primary".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
}

#[test]
fn test_project_paths_indexes_exist() {
    let db = Database::new_in_memory().unwrap();

    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='project_paths'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_project_paths_project".to_string()));
    assert!(indexes.contains(&"idx_project_paths_path".to_string()));
}

#[test]
fn test_session_project_bindings_table_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify session_project_bindings table exists
    let result = db.connection().execute(
        "SELECT 1 FROM session_project_bindings LIMIT 1",
        [],
    );
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(session_project_bindings)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"session_id".to_string()));
    assert!(columns.contains(&"project_id".to_string()));
    assert!(columns.contains(&"bound_at".to_string()));
}

#[test]
fn test_session_project_bindings_index_exists() {
    let db = Database::new_in_memory().unwrap();

    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='session_project_bindings'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_session_bindings_project".to_string()));
}

#[test]
fn test_sessions_original_cwd_column_exists() {
    let db = Database::new_in_memory().unwrap();

    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(sessions)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"original_cwd".to_string()));
    assert!(columns.contains(&"source_context".to_string()));
}

#[test]
fn test_sessions_original_cwd_index_exists() {
    let db = Database::new_in_memory().unwrap();

    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='sessions'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_sessions_original_cwd".to_string()));
}

// ===== Story 1.12: Constraint Migration Tests =====

#[test]
fn test_project_paths_allows_same_path_different_projects() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create two projects
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES ('proj1', 'Project 1', '/path/to/proj1', ?1, ?1)",
            [&now],
        )
        .unwrap();
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES ('proj2', 'Project 2', '/path/to/proj2', ?1, ?1)",
            [&now],
        )
        .unwrap();

    // Add the same path to both projects - should succeed with new constraint
    db.connection()
        .execute(
            "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES ('path1', 'proj1', '/shared/path', 1, ?1)",
            [&now],
        )
        .unwrap();

    let result = db.connection().execute(
        "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES ('path2', 'proj2', '/shared/path', 1, ?1)",
        [&now],
    );

    // With UNIQUE(project_id, path), same path in different projects should succeed
    assert!(result.is_ok(), "Same path should be allowed in different projects");
}

#[test]
fn test_project_paths_rejects_duplicate_path_same_project() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create a project
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES ('proj1', 'Project 1', '/path/to/proj1', ?1, ?1)",
            [&now],
        )
        .unwrap();

    // Add a path
    db.connection()
        .execute(
            "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES ('path1', 'proj1', '/shared/path', 1, ?1)",
            [&now],
        )
        .unwrap();

    // Try to add the same path to the same project - should fail
    let result = db.connection().execute(
        "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES ('path2', 'proj1', '/shared/path', 0, ?1)",
        [&now],
    );

    // With UNIQUE(project_id, path), duplicate path in same project should fail
    assert!(result.is_err(), "Duplicate path in same project should be rejected");
}

#[test]
fn test_project_paths_composite_unique_constraint() {
    let db = Database::new_in_memory().unwrap();

    // Check that the table has the composite unique constraint
    let table_sql: String = db
        .connection()
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='project_paths'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    // Verify the constraint is UNIQUE(project_id, path) not just UNIQUE on path
    assert!(
        table_sql.contains("UNIQUE(project_id, path)") || table_sql.contains("UNIQUE (project_id, path)"),
        "Table should have UNIQUE(project_id, path) constraint, got: {}",
        table_sql
    );
    assert!(
        !table_sql.contains("path TEXT NOT NULL UNIQUE"),
        "Table should NOT have path UNIQUE constraint, got: {}",
        table_sql
    );
}

// ===== Story 1.13: Logical Project Names Migration Tests =====

#[test]
fn test_logical_project_names_table_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify logical_project_names table exists
    let result = db.connection().execute(
        "SELECT 1 FROM logical_project_names LIMIT 1",
        [],
    );
    // Table exists but is empty, so query should succeed
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(logical_project_names)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"physical_path".to_string()));
    assert!(columns.contains(&"custom_name".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
    assert!(columns.contains(&"updated_at".to_string()));
}

#[test]
fn test_logical_project_names_index_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify index exists
    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='logical_project_names'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_logical_project_names_path".to_string()));
}

#[test]
fn test_logical_project_names_primary_key() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert a record
    db.connection()
        .execute(
            "INSERT INTO logical_project_names (physical_path, custom_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            ["/path/to/project", "My Custom Name", &now],
        )
        .unwrap();

    // Try to insert with the same physical_path - should fail due to PRIMARY KEY
    let result = db.connection().execute(
        "INSERT INTO logical_project_names (physical_path, custom_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        ["/path/to/project", "Another Name", &now],
    );

    assert!(result.is_err(), "Duplicate physical_path should be rejected");
}

#[test]
fn test_logical_project_names_crud_operations() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // CREATE
    db.connection()
        .execute(
            "INSERT INTO logical_project_names (physical_path, custom_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            ["/path/to/project", "My Project", &now],
        )
        .unwrap();

    // READ
    let custom_name: String = db
        .connection()
        .query_row(
            "SELECT custom_name FROM logical_project_names WHERE physical_path = ?1",
            ["/path/to/project"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(custom_name, "My Project");

    // UPDATE
    let new_now = chrono::Utc::now().to_rfc3339();
    db.connection()
        .execute(
            "UPDATE logical_project_names SET custom_name = ?1, updated_at = ?2 WHERE physical_path = ?3",
            ["Renamed Project", &new_now, "/path/to/project"],
        )
        .unwrap();

    let updated_name: String = db
        .connection()
        .query_row(
            "SELECT custom_name FROM logical_project_names WHERE physical_path = ?1",
            ["/path/to/project"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(updated_name, "Renamed Project");

    // DELETE
    db.connection()
        .execute(
            "DELETE FROM logical_project_names WHERE physical_path = ?1",
            ["/path/to/project"],
        )
        .unwrap();

    let result: Option<String> = db
        .connection()
        .query_row(
            "SELECT custom_name FROM logical_project_names WHERE physical_path = ?1",
            ["/path/to/project"],
            |row| row.get(0),
        )
        .ok();
    assert!(result.is_none(), "Record should be deleted");
}

// ===== Story 11.2: MCP Services Migration Tests =====

#[test]
fn test_mcp_services_table_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify mcp_services table exists
    let result = db.connection().execute(
        "SELECT 1 FROM mcp_services LIMIT 1",
        [],
    );
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(mcp_services)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"name".to_string()));
    assert!(columns.contains(&"command".to_string()));
    assert!(columns.contains(&"args".to_string()));
    assert!(columns.contains(&"env".to_string()));
    assert!(columns.contains(&"source".to_string()));
    assert!(columns.contains(&"source_file".to_string()));
    assert!(columns.contains(&"enabled".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
    assert!(columns.contains(&"updated_at".to_string()));
}

#[test]
fn test_mcp_services_indexes_exist() {
    let db = Database::new_in_memory().unwrap();

    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='mcp_services'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_mcp_services_name".to_string()));
    assert!(indexes.contains(&"idx_mcp_services_source".to_string()));
}

#[test]
fn test_mcp_services_source_check_constraint() {
    let db = Database::new_in_memory().unwrap();

    // Valid source values should work
    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source) VALUES ('s1', 'test', 'npx', 'imported')",
            [],
        )
        .unwrap();

    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source) VALUES ('s2', 'test2', 'uvx', 'manual')",
            [],
        )
        .unwrap();

    // Invalid source value should fail
    let result = db.connection().execute(
        "INSERT INTO mcp_services (id, name, command, source) VALUES ('s3', 'test3', 'npx', 'invalid')",
        [],
    );
    assert!(result.is_err(), "Invalid source value should be rejected");
}

#[test]
fn test_project_mcp_services_table_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify project_mcp_services table exists
    let result = db.connection().execute(
        "SELECT 1 FROM project_mcp_services LIMIT 1",
        [],
    );
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(project_mcp_services)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"project_id".to_string()));
    assert!(columns.contains(&"service_id".to_string()));
    assert!(columns.contains(&"config_override".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
}

#[test]
fn test_project_mcp_services_indexes_exist() {
    let db = Database::new_in_memory().unwrap();

    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='project_mcp_services'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_project_mcp_services_project".to_string()));
    assert!(indexes.contains(&"idx_project_mcp_services_service".to_string()));
}

#[test]
fn test_project_mcp_services_composite_primary_key() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create a project and a service
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES ('proj1', 'Project 1', '/path', ?1, ?1)",
            [&now],
        )
        .unwrap();

    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source) VALUES ('svc1', 'Service 1', 'npx', 'manual')",
            [],
        )
        .unwrap();

    // Insert a link
    db.connection()
        .execute(
            "INSERT INTO project_mcp_services (project_id, service_id) VALUES ('proj1', 'svc1')",
            [],
        )
        .unwrap();

    // Duplicate should fail
    let result = db.connection().execute(
        "INSERT INTO project_mcp_services (project_id, service_id) VALUES ('proj1', 'svc1')",
        [],
    );
    assert!(result.is_err(), "Duplicate (project_id, service_id) should be rejected");
}

#[test]
fn test_project_mcp_services_cascade_delete_project() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create a project and a service
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES ('proj1', 'Project 1', '/path', ?1, ?1)",
            [&now],
        )
        .unwrap();

    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source) VALUES ('svc1', 'Service 1', 'npx', 'manual')",
            [],
        )
        .unwrap();

    // Insert a link
    db.connection()
        .execute(
            "INSERT INTO project_mcp_services (project_id, service_id) VALUES ('proj1', 'svc1')",
            [],
        )
        .unwrap();

    // Delete the project
    db.connection()
        .execute("DELETE FROM projects WHERE id = 'proj1'", [])
        .unwrap();

    // Link should be deleted due to CASCADE
    let count: i32 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM project_mcp_services WHERE project_id = 'proj1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "Link should be deleted when project is deleted");
}

#[test]
fn test_project_mcp_services_cascade_delete_service() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create a project and a service
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES ('proj1', 'Project 1', '/path', ?1, ?1)",
            [&now],
        )
        .unwrap();

    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source) VALUES ('svc1', 'Service 1', 'npx', 'manual')",
            [],
        )
        .unwrap();

    // Insert a link
    db.connection()
        .execute(
            "INSERT INTO project_mcp_services (project_id, service_id) VALUES ('proj1', 'svc1')",
            [],
        )
        .unwrap();

    // Delete the service
    db.connection()
        .execute("DELETE FROM mcp_services WHERE id = 'svc1'", [])
        .unwrap();

    // Link should be deleted due to CASCADE
    let count: i32 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM project_mcp_services WHERE service_id = 'svc1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "Link should be deleted when service is deleted");
}

#[test]
fn test_env_variables_table_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify env_variables table exists
    let result = db.connection().execute(
        "SELECT 1 FROM env_variables LIMIT 1",
        [],
    );
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(env_variables)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"name".to_string()));
    assert!(columns.contains(&"encrypted_value".to_string()));
    assert!(columns.contains(&"description".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
    assert!(columns.contains(&"updated_at".to_string()));
}

#[test]
fn test_env_variables_name_unique_constraint() {
    let db = Database::new_in_memory().unwrap();

    // Insert first variable
    db.connection()
        .execute(
            "INSERT INTO env_variables (id, name, encrypted_value) VALUES ('e1', 'API_KEY', X'0102030405')",
            [],
        )
        .unwrap();

    // Duplicate name should fail
    let result = db.connection().execute(
        "INSERT INTO env_variables (id, name, encrypted_value) VALUES ('e2', 'API_KEY', X'0607080910')",
        [],
    );
    assert!(result.is_err(), "Duplicate name should be rejected");
}

#[test]
fn test_env_variables_index_exists() {
    let db = Database::new_in_memory().unwrap();

    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='env_variables'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_env_variables_name".to_string()));
}

#[test]
fn test_mcp_services_allows_duplicate_names() {
    let db = Database::new_in_memory().unwrap();

    // Insert two services with the same name (different sources)
    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source, source_file) VALUES ('s1', 'git-mcp', 'npx', 'imported', '/home/user/.claude/mcp.json')",
            [],
        )
        .unwrap();

    let result = db.connection().execute(
        "INSERT INTO mcp_services (id, name, command, source) VALUES ('s2', 'git-mcp', 'npx', 'manual')",
        [],
    );

    // Should succeed - name is not unique
    assert!(result.is_ok(), "Duplicate service names should be allowed");
}

// ===== Story 11.15: MCP Takeover Backups Migration Tests =====

#[test]
fn test_mcp_takeover_backups_table_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify mcp_takeover_backups table exists
    let result = db.connection().execute(
        "SELECT 1 FROM mcp_takeover_backups LIMIT 1",
        [],
    );
    assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

    // Verify columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(mcp_takeover_backups)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"tool_type".to_string()));
    assert!(columns.contains(&"original_path".to_string()));
    assert!(columns.contains(&"backup_path".to_string()));
    assert!(columns.contains(&"taken_over_at".to_string()));
    assert!(columns.contains(&"restored_at".to_string()));
    assert!(columns.contains(&"status".to_string()));
}

#[test]
fn test_mcp_takeover_backups_indexes_exist() {
    let db = Database::new_in_memory().unwrap();

    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='mcp_takeover_backups'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_takeover_status".to_string()));
    assert!(indexes.contains(&"idx_takeover_tool".to_string()));
}

#[test]
fn test_mcp_takeover_backups_tool_type_check_constraint() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Valid tool_type values should work
    for tool_type in &["claude_code", "cursor", "codex", "gemini_cli"] {
        let id = format!("backup_{}", tool_type);
        let result = db.connection().execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at) VALUES (?1, ?2, '/original', '/backup', ?3)",
            [&id, *tool_type, &now],
        );
        assert!(result.is_ok(), "Valid tool_type '{}' should be accepted", tool_type);
    }

    // Invalid tool_type should fail
    let result = db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at) VALUES ('invalid', 'vscode', '/original', '/backup', ?1)",
        [&now],
    );
    assert!(result.is_err(), "Invalid tool_type should be rejected");
}

#[test]
fn test_mcp_takeover_backups_status_check_constraint() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Valid status values should work
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status) VALUES ('b1', 'claude_code', '/original', '/backup', ?1, 'active')",
            [&now],
        )
        .unwrap();

    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status) VALUES ('b2', 'cursor', '/original2', '/backup2', ?1, 'restored')",
            [&now],
        )
        .unwrap();

    // Invalid status should fail
    let result = db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status) VALUES ('b3', 'codex', '/original3', '/backup3', ?1, 'invalid')",
        [&now],
    );
    assert!(result.is_err(), "Invalid status should be rejected");
}

#[test]
fn test_mcp_takeover_backups_default_status() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert without specifying status
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at) VALUES ('b1', 'claude_code', '/original', '/backup', ?1)",
            [&now],
        )
        .unwrap();

    // Verify default status is 'active'
    let status: String = db
        .connection()
        .query_row(
            "SELECT status FROM mcp_takeover_backups WHERE id = 'b1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "active", "Default status should be 'active'");
}

#[test]
fn test_mcp_takeover_backups_crud_operations() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // CREATE
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at) VALUES ('b1', 'claude_code', '/home/user/.claude.json', '/home/user/.claude.json.mantra-backup.20260201', ?1)",
            [&now],
        )
        .unwrap();

    // READ
    let (tool_type, original_path, status): (String, String, String) = db
        .connection()
        .query_row(
            "SELECT tool_type, original_path, status FROM mcp_takeover_backups WHERE id = 'b1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(tool_type, "claude_code");
    assert_eq!(original_path, "/home/user/.claude.json");
    assert_eq!(status, "active");

    // UPDATE (restore)
    let restored_at = chrono::Utc::now().to_rfc3339();
    db.connection()
        .execute(
            "UPDATE mcp_takeover_backups SET status = 'restored', restored_at = ?1 WHERE id = 'b1'",
            [&restored_at],
        )
        .unwrap();

    let (new_status, restored): (String, Option<String>) = db
        .connection()
        .query_row(
            "SELECT status, restored_at FROM mcp_takeover_backups WHERE id = 'b1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(new_status, "restored");
    assert!(restored.is_some());

    // DELETE
    db.connection()
        .execute("DELETE FROM mcp_takeover_backups WHERE id = 'b1'", [])
        .unwrap();

    let count: i32 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM mcp_takeover_backups WHERE id = 'b1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "Record should be deleted");
}

// ===== Story 11.16: MCP Takeover Scope Migration Tests =====

#[test]
fn test_mcp_takeover_backups_scope_column_exists() {
    let db = Database::new_in_memory().unwrap();

    // Verify scope column exists
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(mcp_takeover_backups)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"scope".to_string()));
    assert!(columns.contains(&"project_path".to_string()));
}

#[test]
fn test_mcp_takeover_backups_scope_default_value() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert without specifying scope
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at) VALUES ('b1', 'claude_code', '/original', '/backup', ?1)",
            [&now],
        )
        .unwrap();

    // Verify default scope is 'user'
    let scope: String = db
        .connection()
        .query_row(
            "SELECT scope FROM mcp_takeover_backups WHERE id = 'b1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(scope, "user", "Default scope should be 'user'");
}

#[test]
fn test_mcp_takeover_backups_scope_check_constraint() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Valid scope values should work
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope) VALUES ('b1', 'claude_code', '/original1', '/backup1', ?1, 'user')",
            [&now],
        )
        .unwrap();

    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope, project_path) VALUES ('b2', 'cursor', '/original2', '/backup2', ?1, 'project', '/path/to/project')",
            [&now],
        )
        .unwrap();

    // Invalid scope should fail
    let result = db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope) VALUES ('b3', 'codex', '/original3', '/backup3', ?1, 'invalid')",
        [&now],
    );
    assert!(result.is_err(), "Invalid scope should be rejected");
}

#[test]
fn test_mcp_takeover_backups_scope_indexes_exist() {
    let db = Database::new_in_memory().unwrap();

    let indexes: Vec<String> = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='mcp_takeover_backups'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_takeover_scope".to_string()));
    assert!(indexes.contains(&"idx_takeover_project_path".to_string()));
}

#[test]
fn test_mcp_takeover_backups_project_scope_with_path() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create project-level takeover
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope, project_path) VALUES ('b1', 'claude_code', '/project/.mcp.json', '/project/.mcp.json.backup', ?1, 'project', '/home/user/my-project')",
            [&now],
        )
        .unwrap();

    // Verify
    let (scope, project_path): (String, Option<String>) = db
        .connection()
        .query_row(
            "SELECT scope, project_path FROM mcp_takeover_backups WHERE id = 'b1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(scope, "project");
    assert_eq!(project_path, Some("/home/user/my-project".to_string()));
}

#[test]
fn test_mcp_takeover_backups_user_scope_null_project_path() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create user-level takeover (project_path should be NULL)
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope) VALUES ('b1', 'claude_code', '~/.claude.json', '~/.claude.json.backup', ?1, 'user')",
            [&now],
        )
        .unwrap();

    // Verify project_path is NULL
    let project_path: Option<String> = db
        .connection()
        .query_row(
            "SELECT project_path FROM mcp_takeover_backups WHERE id = 'b1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(project_path.is_none(), "User-level takeover should have NULL project_path");
}

#[test]
fn test_mcp_takeover_backups_query_by_scope() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create mixed scope takeovers
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope) VALUES ('u1', 'claude_code', '/u1', '/b1', ?1, 'user')",
            [&now],
        )
        .unwrap();
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope) VALUES ('u2', 'cursor', '/u2', '/b2', ?1, 'user')",
            [&now],
        )
        .unwrap();
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope, project_path) VALUES ('p1', 'claude_code', '/p1', '/b3', ?1, 'project', '/project1')",
            [&now],
        )
        .unwrap();

    // Query by scope
    let user_count: i32 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM mcp_takeover_backups WHERE scope = 'user'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(user_count, 2);

    let project_count: i32 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM mcp_takeover_backups WHERE scope = 'project'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(project_count, 1);
}

#[test]
fn test_mcp_takeover_backups_query_by_project_path() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create project-level takeovers for different projects
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope, project_path) VALUES ('p1', 'claude_code', '/p1', '/b1', ?1, 'project', '/home/user/project-a')",
            [&now],
        )
        .unwrap();
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope, project_path) VALUES ('p2', 'cursor', '/p2', '/b2', ?1, 'project', '/home/user/project-a')",
            [&now],
        )
        .unwrap();
    db.connection()
        .execute(
            "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, scope, project_path) VALUES ('p3', 'codex', '/p3', '/b3', ?1, 'project', '/home/user/project-b')",
            [&now],
        )
        .unwrap();

    // Query by project_path
    let project_a_count: i32 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM mcp_takeover_backups WHERE project_path = '/home/user/project-a'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(project_a_count, 2);

    let project_b_count: i32 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM mcp_takeover_backups WHERE project_path = '/home/user/project-b'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(project_b_count, 1);
}

// ===== Story 11.19: Smart Takeover Migration Tests =====

#[test]
fn test_mcp_services_smart_takeover_columns_exist() {
    let db = Database::new_in_memory().unwrap();

    // Verify source_adapter_id and source_scope columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(mcp_services)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"source_adapter_id".to_string()));
    assert!(columns.contains(&"source_scope".to_string()));
}

#[test]
fn test_project_mcp_services_smart_takeover_columns_exist() {
    let db = Database::new_in_memory().unwrap();

    // Verify detected_adapter_id and detected_config_path columns exist
    let columns: Vec<String> = db
        .connection()
        .prepare("PRAGMA table_info(project_mcp_services)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(columns.contains(&"detected_adapter_id".to_string()));
    assert!(columns.contains(&"detected_config_path".to_string()));
}

#[test]
fn test_mcp_services_source_adapter_crud() {
    let db = Database::new_in_memory().unwrap();

    // Insert a service with source_adapter_id and source_scope
    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source, source_adapter_id, source_scope) VALUES ('s1', 'git-mcp', 'npx', 'imported', 'claude', 'project')",
            [],
        )
        .unwrap();

    // Verify the values
    let (adapter_id, scope): (Option<String>, Option<String>) = db
        .connection()
        .query_row(
            "SELECT source_adapter_id, source_scope FROM mcp_services WHERE id = 's1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(adapter_id, Some("claude".to_string()));
    assert_eq!(scope, Some("project".to_string()));
}

#[test]
fn test_mcp_services_source_adapter_nullable() {
    let db = Database::new_in_memory().unwrap();

    // Insert a service without source_adapter_id and source_scope (should be NULL)
    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source) VALUES ('s1', 'manual-service', 'npx', 'manual')",
            [],
        )
        .unwrap();

    // Verify the values are NULL
    let (adapter_id, scope): (Option<String>, Option<String>) = db
        .connection()
        .query_row(
            "SELECT source_adapter_id, source_scope FROM mcp_services WHERE id = 's1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert!(adapter_id.is_none());
    assert!(scope.is_none());
}

#[test]
fn test_project_mcp_services_detected_adapter_crud() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create a project and a service
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES ('proj1', 'Project 1', '/path', ?1, ?1)",
            [&now],
        )
        .unwrap();

    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source) VALUES ('svc1', 'Service 1', 'npx', 'imported')",
            [],
        )
        .unwrap();

    // Insert a link with detected_adapter_id and detected_config_path
    db.connection()
        .execute(
            "INSERT INTO project_mcp_services (project_id, service_id, detected_adapter_id, detected_config_path) VALUES ('proj1', 'svc1', 'cursor', '/project/.mcp.json')",
            [],
        )
        .unwrap();

    // Verify the values
    let (adapter_id, config_path): (Option<String>, Option<String>) = db
        .connection()
        .query_row(
            "SELECT detected_adapter_id, detected_config_path FROM project_mcp_services WHERE project_id = 'proj1' AND service_id = 'svc1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(adapter_id, Some("cursor".to_string()));
    assert_eq!(config_path, Some("/project/.mcp.json".to_string()));
}

#[test]
fn test_project_mcp_services_detected_adapter_nullable() {
    let db = Database::new_in_memory().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Create a project and a service
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES ('proj1', 'Project 1', '/path', ?1, ?1)",
            [&now],
        )
        .unwrap();

    db.connection()
        .execute(
            "INSERT INTO mcp_services (id, name, command, source) VALUES ('svc1', 'Service 1', 'npx', 'manual')",
            [],
        )
        .unwrap();

    // Insert a link without detected fields (should be NULL)
    db.connection()
        .execute(
            "INSERT INTO project_mcp_services (project_id, service_id) VALUES ('proj1', 'svc1')",
            [],
        )
        .unwrap();

    // Verify the values are NULL
    let (adapter_id, config_path): (Option<String>, Option<String>) = db
        .connection()
        .query_row(
            "SELECT detected_adapter_id, detected_config_path FROM project_mcp_services WHERE project_id = 'proj1' AND service_id = 'svc1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert!(adapter_id.is_none());
    assert!(config_path.is_none());
}

// ===== Story 11.21: Local Scope 迁移测试 =====

#[test]
fn test_mcp_takeover_backups_local_scope_check_constraint() {
    let db = Database::new_in_memory().unwrap();

    // 验证可以插入 'local' scope
    let result = db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status, scope, project_path) VALUES ('test-local-1', 'claude_code', '/home/user/.claude.json', '/home/user/.mantra/backups/test.backup', '2026-02-03T10:00:00Z', 'active', 'local', '/home/user/project-a')",
        [],
    );
    assert!(result.is_ok(), "Should be able to insert 'local' scope: {:?}", result.err());

    // 验证读取
    let scope: String = db
        .connection()
        .query_row(
            "SELECT scope FROM mcp_takeover_backups WHERE id = 'test-local-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(scope, "local");

    // 验证无效 scope 仍然被拒绝
    let invalid = db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status, scope) VALUES ('test-invalid', 'claude_code', '/path', '/backup', '2026-02-03T10:00:00Z', 'active', 'invalid_scope')",
        [],
    );
    assert!(invalid.is_err(), "Should reject invalid scope value");
}

#[test]
fn test_mcp_takeover_backups_query_by_local_scope() {
    let db = Database::new_in_memory().unwrap();

    // 插入多种 scope 的备份
    db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status, scope, project_path) VALUES ('user-1', 'claude_code', '~/.claude.json', '/backup/user.backup', '2026-02-03T10:00:00Z', 'active', 'user', NULL)",
        [],
    ).unwrap();
    db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status, scope, project_path) VALUES ('project-1', 'claude_code', '/proj/.mcp.json', '/backup/proj.backup', '2026-02-03T10:01:00Z', 'active', 'project', '/proj')",
        [],
    ).unwrap();
    db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status, scope, project_path) VALUES ('local-1', 'claude_code', '~/.claude.json', '/backup/local-a.backup', '2026-02-03T10:02:00Z', 'active', 'local', '/home/user/project-a')",
        [],
    ).unwrap();
    db.connection().execute(
        "INSERT INTO mcp_takeover_backups (id, tool_type, original_path, backup_path, taken_over_at, status, scope, project_path) VALUES ('local-2', 'claude_code', '~/.claude.json', '/backup/local-b.backup', '2026-02-03T10:03:00Z', 'active', 'local', '/home/user/project-b')",
        [],
    ).unwrap();

    // 只查询 local scope
    let local_count: i32 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM mcp_takeover_backups WHERE scope = 'local' AND status = 'active'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(local_count, 2);

    // 按 project_path 查询 local scope
    let local_a: Option<String> = db
        .connection()
        .query_row(
            "SELECT id FROM mcp_takeover_backups WHERE scope = 'local' AND project_path = '/home/user/project-a'",
            [],
            |row| row.get(0),
        )
        .ok();
    assert_eq!(local_a, Some("local-1".to_string()));
}
