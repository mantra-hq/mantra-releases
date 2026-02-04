//! Database migrations for Mantra
//!
//! Contains all schema migration functions for evolving the database.

use rusqlite::Connection;

use super::error::StorageError;

/// Run all database migrations for schema updates
pub(super) fn run_all(conn: &Connection) -> Result<(), StorageError> {
    // Migration: Add git_repo_path and has_git_repo columns (Story 2.11)
    run_git_columns_migration(conn)?;

    // Migration: Add deleted_at column (Story 2.19, deprecated)
    run_deleted_at_migration(conn)?;

    // Migration: Add is_empty column to sessions (Story 2.29)
    run_sessions_is_empty_migration(conn)?;

    // Migration: Add git_remote_url column (Story 1.9)
    run_git_remote_url_migration(conn)?;

    // Migration: Add is_empty column to projects (Story 2.29 V2)
    run_projects_is_empty_migration(conn)?;

    // Migration: Add interception_records table (Story 3.7)
    run_interception_records_migration(conn)?;

    // Migration: Add project_paths and session_project_bindings tables (Story 1.12)
    run_view_based_aggregation_migration(conn)?;

    // Migration: Add path_type and path_exists columns to projects (Story 1.12)
    run_path_validation_migration(conn)?;

    // Migration: Add logical_project_names table (Story 1.13)
    run_logical_project_names_migration(conn)?;

    // Migration: Add gateway_config table (Story 11.1)
    run_gateway_config_migration(conn)?;

    // Migration: Add MCP services tables (Story 11.2)
    run_mcp_services_migration(conn)?;

    // Migration: Add MCP service tools cache table (Story 11.10)
    run_mcp_service_tools_migration(conn)?;

    // Migration: Add HTTP transport support to MCP services (Story 11.11)
    run_mcp_http_transport_migration(conn)?;

    // Migration: Add MCP takeover backups table (Story 11.15)
    run_mcp_takeover_backups_migration(conn)?;

    // Migration: Add scope and project_path to takeover backups (Story 11.16)
    run_mcp_takeover_scope_migration(conn)?;

    // Migration: Add 'local' scope support (Story 11.21)
    run_mcp_takeover_local_scope_migration(conn)?;

    // Migration: Add default_tool_policy to mcp_services (Story 11.9 Phase 2)
    run_mcp_default_tool_policy_migration(conn)?;

    // Migration: Add smart takeover fields (Story 11.19)
    run_mcp_smart_takeover_migration(conn)?;

    Ok(())
}

/// Migration: Add git_repo_path and has_git_repo columns (Story 2.11)
fn run_git_columns_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_git_repo_path: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'git_repo_path'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_git_repo_path {
        conn.execute_batch(
            "ALTER TABLE projects ADD COLUMN git_repo_path TEXT;
             ALTER TABLE projects ADD COLUMN has_git_repo INTEGER NOT NULL DEFAULT 0;",
        )?;
    }

    Ok(())
}

/// Migration: Add deleted_at column (Story 2.19, deprecated - kept for backward compatibility)
fn run_deleted_at_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_deleted_at: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'deleted_at'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_deleted_at {
        conn.execute_batch("ALTER TABLE projects ADD COLUMN deleted_at TEXT;")?;
    }

    Ok(())
}

/// Migration: Add is_empty column to sessions (Story 2.29)
fn run_sessions_is_empty_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_is_empty: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name = 'is_empty'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_is_empty {
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN is_empty INTEGER NOT NULL DEFAULT 0;",
        )?;
        conn.execute_batch("UPDATE sessions SET is_empty = 1 WHERE message_count = 0;")?;
    }

    Ok(())
}

/// Migration: Add git_remote_url column (Story 1.9)
fn run_git_remote_url_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_git_remote_url: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'git_remote_url'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_git_remote_url {
        conn.execute_batch("ALTER TABLE projects ADD COLUMN git_remote_url TEXT;")?;
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_projects_git_remote_url ON projects(git_remote_url);",
        )?;
    }

    Ok(())
}

/// Migration: Add is_empty column to projects (Story 2.29 V2)
fn run_projects_is_empty_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_projects_is_empty: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'is_empty'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_projects_is_empty {
        conn.execute_batch(
            "ALTER TABLE projects ADD COLUMN is_empty INTEGER NOT NULL DEFAULT 0;",
        )?;
        conn.execute_batch(
            "UPDATE projects SET is_empty = 1 WHERE id IN (
                SELECT p.id FROM projects p
                LEFT JOIN sessions s ON s.project_id = p.id
                GROUP BY p.id
                HAVING COUNT(s.id) = 0 OR COUNT(s.id) = SUM(CASE WHEN s.is_empty = 1 THEN 1 ELSE 0 END)
            );",
        )?;
    }

    Ok(())
}

/// Migration for interception_records table (Story 3.7)
fn run_interception_records_migration(conn: &Connection) -> Result<(), StorageError> {
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='interception_records'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS interception_records (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_context TEXT,
                matches TEXT NOT NULL,
                user_action TEXT NOT NULL,
                original_text_hash TEXT,
                project_name TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_records_timestamp ON interception_records(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_records_source ON interception_records(source_type);
            CREATE INDEX IF NOT EXISTS idx_records_project ON interception_records(project_name);
            "#,
        )?;
    }

    Ok(())
}

/// Migration for view-based project aggregation (Story 1.12)
fn run_view_based_aggregation_migration(conn: &Connection) -> Result<(), StorageError> {
    let project_paths_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='project_paths'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !project_paths_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS project_paths (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                path TEXT NOT NULL,
                is_primary INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(project_id, path),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_project_paths_project ON project_paths(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_paths_path ON project_paths(path);
            "#,
        )?;

        conn.execute_batch(
            r#"
            INSERT OR IGNORE INTO project_paths (id, project_id, path, is_primary, created_at)
            SELECT
                lower(hex(randomblob(16))),
                id,
                cwd,
                1,
                created_at
            FROM projects
            WHERE cwd IS NOT NULL AND cwd != '';
            "#,
        )?;
    } else {
        migrate_project_paths_constraint(conn)?;
    }

    let bindings_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='session_project_bindings'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !bindings_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS session_project_bindings (
                session_id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                bound_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_session_bindings_project ON session_project_bindings(project_id);
            "#,
        )?;
    }

    let has_original_cwd: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name = 'original_cwd'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_original_cwd {
        conn.execute_batch(
            r#"
            ALTER TABLE sessions ADD COLUMN original_cwd TEXT DEFAULT '';
            UPDATE sessions SET original_cwd = cwd WHERE original_cwd = '' OR original_cwd IS NULL;
            "#,
        )?;
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_sessions_original_cwd ON sessions(original_cwd);",
        )?;
    }

    let has_source_context: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name = 'source_context'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_source_context {
        conn.execute_batch("ALTER TABLE sessions ADD COLUMN source_context TEXT DEFAULT '{}';")?;
    }

    Ok(())
}

/// Migration: Change project_paths constraint from path UNIQUE to UNIQUE(project_id, path)
fn migrate_project_paths_constraint(conn: &Connection) -> Result<(), StorageError> {
    let has_path_unique_index: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='project_paths' AND sql LIKE '%UNIQUE%' AND sql LIKE '%path%' AND sql NOT LIKE '%project_id%'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    let table_sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='project_paths'",
            [],
            |row| row.get(0),
        )
        .ok();

    let needs_migration = has_path_unique_index
        || table_sql
            .as_ref()
            .map(|sql| sql.contains("path TEXT NOT NULL UNIQUE"))
            .unwrap_or(false);

    if !needs_migration {
        return Ok(());
    }

    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = OFF;

        CREATE TABLE project_paths_new (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            path TEXT NOT NULL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            UNIQUE(project_id, path),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        INSERT INTO project_paths_new (id, project_id, path, is_primary, created_at)
        SELECT id, project_id, path, is_primary, created_at FROM project_paths;

        DROP TABLE project_paths;

        ALTER TABLE project_paths_new RENAME TO project_paths;

        CREATE INDEX IF NOT EXISTS idx_project_paths_project ON project_paths(project_id);
        CREATE INDEX IF NOT EXISTS idx_project_paths_path ON project_paths(path);

        PRAGMA foreign_keys = ON;
        "#,
    )?;

    Ok(())
}

/// Migration: Add path_type and path_exists columns to projects table (Story 1.12)
fn run_path_validation_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_path_type: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'path_type'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_path_type {
        conn.execute_batch(
            r#"
            ALTER TABLE projects ADD COLUMN path_type TEXT DEFAULT 'local';
            ALTER TABLE projects ADD COLUMN path_exists INTEGER DEFAULT 1;
            "#,
        )?;
    }

    Ok(())
}

/// Migration for logical_project_names table (Story 1.13)
fn run_logical_project_names_migration(conn: &Connection) -> Result<(), StorageError> {
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='logical_project_names'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS logical_project_names (
                physical_path TEXT PRIMARY KEY,
                custom_name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_logical_project_names_path ON logical_project_names(physical_path);
            "#,
        )?;
    }

    Ok(())
}

/// Migration for gateway_config table (Story 11.1)
fn run_gateway_config_migration(conn: &Connection) -> Result<(), StorageError> {
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='gateway_config'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS gateway_config (
                id INTEGER PRIMARY KEY DEFAULT 1,
                port INTEGER,
                auth_token TEXT NOT NULL,
                enabled INTEGER DEFAULT 0,
                auto_start INTEGER DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now')),
                CHECK (id = 1)
            );
            "#,
        )?;

        let auth_token = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO gateway_config (id, auth_token) VALUES (1, ?1)",
            [&auth_token],
        )?;
    }

    Ok(())
}

/// Migration for MCP services tables (Story 11.2)
fn run_mcp_services_migration(conn: &Connection) -> Result<(), StorageError> {
    let mcp_services_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mcp_services'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !mcp_services_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS mcp_services (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                args TEXT,
                env TEXT,
                source TEXT NOT NULL CHECK(source IN ('imported', 'manual')),
                source_file TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_mcp_services_name ON mcp_services(name);
            CREATE INDEX IF NOT EXISTS idx_mcp_services_source ON mcp_services(source);
            "#,
        )?;
    }

    let project_mcp_services_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='project_mcp_services'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !project_mcp_services_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS project_mcp_services (
                project_id TEXT NOT NULL,
                service_id TEXT NOT NULL,
                config_override TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (project_id, service_id),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (service_id) REFERENCES mcp_services(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_project_mcp_services_project ON project_mcp_services(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_mcp_services_service ON project_mcp_services(service_id);
            "#,
        )?;
    }

    let env_variables_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='env_variables'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !env_variables_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS env_variables (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                encrypted_value BLOB NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_env_variables_name ON env_variables(name);
            "#,
        )?;
    }

    Ok(())
}

/// Migration for MCP service tools cache table (Story 11.10)
fn run_mcp_service_tools_migration(conn: &Connection) -> Result<(), StorageError> {
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mcp_service_tools'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS mcp_service_tools (
                id TEXT PRIMARY KEY,
                service_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                description TEXT,
                input_schema TEXT,
                cached_at TEXT NOT NULL,
                UNIQUE(service_id, tool_name),
                FOREIGN KEY (service_id) REFERENCES mcp_services(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_mcp_service_tools_service ON mcp_service_tools(service_id);
            "#,
        )?;
    }

    Ok(())
}

/// Migration for HTTP transport support in MCP services (Story 11.11)
fn run_mcp_http_transport_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_transport_type: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('mcp_services') WHERE name = 'transport_type'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_transport_type {
        conn.execute_batch(
            r#"
            ALTER TABLE mcp_services ADD COLUMN transport_type TEXT NOT NULL DEFAULT 'stdio'
                CHECK(transport_type IN ('stdio', 'http'));
            ALTER TABLE mcp_services ADD COLUMN url TEXT;
            ALTER TABLE mcp_services ADD COLUMN headers TEXT;
            "#,
        )?;
    }

    Ok(())
}

/// Migration for MCP takeover backups table (Story 11.15)
fn run_mcp_takeover_backups_migration(conn: &Connection) -> Result<(), StorageError> {
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mcp_takeover_backups'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS mcp_takeover_backups (
                id TEXT PRIMARY KEY,
                tool_type TEXT NOT NULL CHECK(tool_type IN ('claude_code', 'cursor', 'codex', 'gemini_cli')),
                original_path TEXT NOT NULL,
                backup_path TEXT NOT NULL,
                taken_over_at TEXT NOT NULL,
                restored_at TEXT,
                status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'restored'))
            );

            CREATE INDEX IF NOT EXISTS idx_takeover_status ON mcp_takeover_backups(status);
            CREATE INDEX IF NOT EXISTS idx_takeover_tool ON mcp_takeover_backups(tool_type);
            "#,
        )?;
    }

    Ok(())
}

/// Migration for MCP takeover scope support (Story 11.16)
fn run_mcp_takeover_scope_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_scope: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('mcp_takeover_backups') WHERE name = 'scope'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_scope {
        conn.execute_batch(
            r#"
            ALTER TABLE mcp_takeover_backups ADD COLUMN scope TEXT NOT NULL DEFAULT 'user'
                CHECK(scope IN ('user', 'project'));
            ALTER TABLE mcp_takeover_backups ADD COLUMN project_path TEXT;
            CREATE INDEX IF NOT EXISTS idx_takeover_scope ON mcp_takeover_backups(scope);
            CREATE INDEX IF NOT EXISTS idx_takeover_project_path ON mcp_takeover_backups(project_path);
            "#,
        )?;
    }

    Ok(())
}

/// Migration for MCP takeover local scope support (Story 11.21)
fn run_mcp_takeover_local_scope_migration(conn: &Connection) -> Result<(), StorageError> {
    let table_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='mcp_takeover_backups'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    if table_sql.contains("'local'") {
        return Ok(());
    }

    let has_scope: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('mcp_takeover_backups') WHERE name = 'scope'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_scope {
        return Ok(());
    }

    conn.execute_batch(
        r#"
        CREATE TABLE mcp_takeover_backups_new (
            id TEXT PRIMARY KEY,
            tool_type TEXT NOT NULL CHECK(tool_type IN ('claude_code', 'cursor', 'codex', 'gemini_cli')),
            original_path TEXT NOT NULL,
            backup_path TEXT NOT NULL,
            taken_over_at TEXT NOT NULL,
            restored_at TEXT,
            status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'restored')),
            scope TEXT NOT NULL DEFAULT 'user' CHECK(scope IN ('user', 'project', 'local')),
            project_path TEXT
        );

        INSERT INTO mcp_takeover_backups_new
            SELECT id, tool_type, original_path, backup_path, taken_over_at, restored_at, status, scope, project_path
            FROM mcp_takeover_backups;

        DROP TABLE mcp_takeover_backups;

        ALTER TABLE mcp_takeover_backups_new RENAME TO mcp_takeover_backups;

        CREATE INDEX IF NOT EXISTS idx_takeover_status ON mcp_takeover_backups(status);
        CREATE INDEX IF NOT EXISTS idx_takeover_tool ON mcp_takeover_backups(tool_type);
        CREATE INDEX IF NOT EXISTS idx_takeover_scope ON mcp_takeover_backups(scope);
        CREATE INDEX IF NOT EXISTS idx_takeover_project_path ON mcp_takeover_backups(project_path);
        "#,
    )?;

    Ok(())
}

/// Migration for MCP service default tool policy (Story 11.9 Phase 2)
fn run_mcp_default_tool_policy_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_default_tool_policy: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('mcp_services') WHERE name = 'default_tool_policy'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_default_tool_policy {
        conn.execute_batch(
            "ALTER TABLE mcp_services ADD COLUMN default_tool_policy TEXT;",
        )?;
    }

    Ok(())
}

/// Migration for MCP smart takeover fields (Story 11.19)
fn run_mcp_smart_takeover_migration(conn: &Connection) -> Result<(), StorageError> {
    let has_source_adapter_id: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('mcp_services') WHERE name = 'source_adapter_id'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_source_adapter_id {
        conn.execute_batch(
            r#"
            ALTER TABLE mcp_services ADD COLUMN source_adapter_id TEXT;
            ALTER TABLE mcp_services ADD COLUMN source_scope TEXT;
            "#,
        )?;
    }

    let has_detected_adapter_id: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('project_mcp_services') WHERE name = 'detected_adapter_id'",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_detected_adapter_id {
        conn.execute_batch(
            r#"
            ALTER TABLE project_mcp_services ADD COLUMN detected_adapter_id TEXT;
            ALTER TABLE project_mcp_services ADD COLUMN detected_config_path TEXT;
            "#,
        )?;
    }

    Ok(())
}
