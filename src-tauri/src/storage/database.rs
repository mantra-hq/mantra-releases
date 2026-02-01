//! SQLite database connection management
//!
//! Provides database initialization and connection management for Mantra.

use std::path::Path;

use rusqlite::Connection;

use super::error::StorageError;

/// Database wrapper for SQLite connection management
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection and initialize schema
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file
    ///
    /// # Returns
    /// A new Database instance with initialized schema
    pub fn new(path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;

        // Enable foreign key support
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Execute schema migration
        conn.execute_batch(include_str!("schema.sql"))?;

        // Run migrations for existing databases
        Self::run_migrations(&conn)?;

        Ok(Self { conn })
    }

    /// Run database migrations for schema updates
    fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
        // Migration: Add git_repo_path and has_git_repo columns (Story 2.11)
        // SQLite ignores ALTER TABLE if column already exists, but we check to avoid errors
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

        // Migration: Add deleted_at column (Story 2.19, deprecated - kept for backward compatibility)
        // This column is no longer used after removing soft-delete logic
        let has_deleted_at: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'deleted_at'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_deleted_at {
            conn.execute_batch(
                "ALTER TABLE projects ADD COLUMN deleted_at TEXT;",
            )?;
        }

        // Migration: Add is_empty column (Story 2.29)
        let has_is_empty: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name = 'is_empty'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_is_empty {
            // Add column with default value
            conn.execute_batch(
                "ALTER TABLE sessions ADD COLUMN is_empty INTEGER NOT NULL DEFAULT 0;",
            )?;

            // Backfill: Mark sessions as empty if they have no messages
            // Empty session = no user messages AND no assistant messages (message_count = 0)
            conn.execute_batch(
                "UPDATE sessions SET is_empty = 1 WHERE message_count = 0;",
            )?;
        }

        // Migration: Add git_remote_url column (Story 1.9)
        let has_git_remote_url: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'git_remote_url'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_git_remote_url {
            conn.execute_batch(
                "ALTER TABLE projects ADD COLUMN git_remote_url TEXT;",
            )?;
            // Create index for git_remote_url
            conn.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_projects_git_remote_url ON projects(git_remote_url);",
            )?;
        }

        // Migration: Add is_empty column to projects table (Story 2.29 V2)
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

            // Backfill: Mark projects as empty if all their sessions are empty
            // A project is empty if it has no sessions OR all sessions have is_empty = 1
            conn.execute_batch(
                "UPDATE projects SET is_empty = 1 WHERE id IN (
                    SELECT p.id FROM projects p
                    LEFT JOIN sessions s ON s.project_id = p.id
                    GROUP BY p.id
                    HAVING COUNT(s.id) = 0 OR COUNT(s.id) = SUM(CASE WHEN s.is_empty = 1 THEN 1 ELSE 0 END)
                );",
            )?;
        }

        // Migration: Add interception_records table (Story 3.7)
        Self::run_interception_records_migration(conn)?;

        // Migration: Add project_paths and session_project_bindings tables (Story 1.12)
        Self::run_view_based_aggregation_migration(conn)?;

        // Migration: Add path_type and path_exists columns to projects (Story 1.12)
        Self::run_path_validation_migration(conn)?;

        // Migration: Add logical_project_names table (Story 1.13)
        Self::run_logical_project_names_migration(conn)?;

        // Migration: Add gateway_config table (Story 11.1)
        Self::run_gateway_config_migration(conn)?;

        // Migration: Add MCP services tables (Story 11.2)
        Self::run_mcp_services_migration(conn)?;

        // Migration: Add MCP service tools cache table (Story 11.10)
        Self::run_mcp_service_tools_migration(conn)?;

        // Migration: Add HTTP transport support to MCP services (Story 11.11)
        Self::run_mcp_http_transport_migration(conn)?;

        // Migration: Add MCP takeover backups table (Story 11.15)
        Self::run_mcp_takeover_backups_migration(conn)?;

        // Migration: Add scope and project_path to takeover backups (Story 11.16)
        Self::run_mcp_takeover_scope_migration(conn)?;

        Ok(())
    }

    /// Migration for interception_records table (Story 3.7)
    fn run_interception_records_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if table already exists
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
                -- 拦截记录表 (Story 3.7)
                CREATE TABLE IF NOT EXISTS interception_records (
                    id TEXT PRIMARY KEY,
                    timestamp TEXT NOT NULL,
                    source_type TEXT NOT NULL,          -- 'pre_upload' | 'claude_code_hook' | 'external_hook'
                    source_context TEXT,                -- JSON: session_id, tool_name 等
                    matches TEXT NOT NULL,              -- JSON: ScanMatch[]
                    user_action TEXT NOT NULL,          -- 'redacted' | 'ignored' | 'cancelled' | 'rule_disabled'
                    original_text_hash TEXT,
                    project_name TEXT,
                    created_at TEXT DEFAULT (datetime('now'))
                );

                -- 时间戳索引 (按时间倒序查询)
                CREATE INDEX IF NOT EXISTS idx_records_timestamp ON interception_records(timestamp DESC);
                -- 来源类型索引 (按来源筛选)
                CREATE INDEX IF NOT EXISTS idx_records_source ON interception_records(source_type);
                -- 项目名索引 (按项目筛选)
                CREATE INDEX IF NOT EXISTS idx_records_project ON interception_records(project_name);
                "#,
            )?;
        }

        Ok(())
    }

    /// Migration for view-based project aggregation (Story 1.12)
    ///
    /// Creates:
    /// - project_paths table: Maps multiple paths to a single project
    /// - session_project_bindings table: Manual session-to-project bindings
    /// - original_cwd and source_context columns on sessions table
    fn run_view_based_aggregation_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if project_paths table already exists
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
                -- project_paths 表: 项目路径映射 (Story 1.12)
                -- 同一路径可以属于多个项目（不同导入源），通过视图层聚合显示
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

            // Migrate existing projects.cwd to project_paths
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
            // Migration: Remove old path UNIQUE constraint and add UNIQUE(project_id, path)
            // SQLite doesn't support ALTER TABLE to modify constraints, so we need to recreate the table
            Self::migrate_project_paths_constraint(conn)?;
        }

        // Check if session_project_bindings table already exists
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
                -- session_project_bindings 表: 会话手动绑定 (Story 1.12)
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

        // Check if sessions.original_cwd column exists
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
                -- Add original_cwd column (Story 1.12)
                ALTER TABLE sessions ADD COLUMN original_cwd TEXT DEFAULT '';

                -- Backfill: Copy existing cwd to original_cwd
                UPDATE sessions SET original_cwd = cwd WHERE original_cwd = '' OR original_cwd IS NULL;
                "#,
            )?;

            // Create index for original_cwd
            conn.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_sessions_original_cwd ON sessions(original_cwd);",
            )?;
        }

        // Check if sessions.source_context column exists
        let has_source_context: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name = 'source_context'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_source_context {
            conn.execute_batch(
                "ALTER TABLE sessions ADD COLUMN source_context TEXT DEFAULT '{}';",
            )?;
        }

        Ok(())
    }

    /// Migration: Add path_type and path_exists columns to projects table (Story 1.12)
    fn run_path_validation_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if path_type column exists
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
                -- Add path_type column (Story 1.12)
                ALTER TABLE projects ADD COLUMN path_type TEXT DEFAULT 'local';

                -- Add path_exists column (Story 1.12)
                ALTER TABLE projects ADD COLUMN path_exists INTEGER DEFAULT 1;
                "#,
            )?;
        }

        Ok(())
    }

    /// Migration: Change project_paths constraint from path UNIQUE to UNIQUE(project_id, path)
    ///
    /// This allows the same path to belong to multiple projects (from different import sources),
    /// enabling view-layer aggregation by physical path.
    fn migrate_project_paths_constraint(conn: &Connection) -> Result<(), StorageError> {
        // Check if we need to migrate by looking at the table schema
        // If the table has a unique index on just 'path', we need to migrate
        let has_path_unique_index: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='project_paths' AND sql LIKE '%UNIQUE%' AND sql LIKE '%path%' AND sql NOT LIKE '%project_id%'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        // Also check if the table definition itself has UNIQUE on path column
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

        // SQLite doesn't support ALTER TABLE to modify constraints
        // We need to recreate the table with the new constraint
        conn.execute_batch(
            r#"
            -- Disable foreign keys temporarily
            PRAGMA foreign_keys = OFF;

            -- Create new table with correct constraint
            CREATE TABLE project_paths_new (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                path TEXT NOT NULL,
                is_primary INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(project_id, path),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            -- Copy data from old table
            INSERT INTO project_paths_new (id, project_id, path, is_primary, created_at)
            SELECT id, project_id, path, is_primary, created_at FROM project_paths;

            -- Drop old table
            DROP TABLE project_paths;

            -- Rename new table
            ALTER TABLE project_paths_new RENAME TO project_paths;

            -- Recreate indexes
            CREATE INDEX IF NOT EXISTS idx_project_paths_project ON project_paths(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_paths_path ON project_paths(path);

            -- Re-enable foreign keys
            PRAGMA foreign_keys = ON;
            "#,
        )?;

        Ok(())
    }

    /// Migration for logical_project_names table (Story 1.13)
    ///
    /// Creates the logical_project_names table for storing custom names for logical projects.
    /// This allows users to rename aggregated logical projects without affecting the underlying
    /// storage-layer project names.
    fn run_logical_project_names_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if table already exists
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
                -- logical_project_names 表: 逻辑项目自定义名称 (Story 1.13)
                -- 存储用户为逻辑项目设置的自定义显示名称
                CREATE TABLE IF NOT EXISTS logical_project_names (
                    physical_path TEXT PRIMARY KEY,
                    custom_name TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                -- 物理路径索引 (用于快速查询)
                CREATE INDEX IF NOT EXISTS idx_logical_project_names_path ON logical_project_names(physical_path);
                "#,
            )?;
        }

        Ok(())
    }

    /// Migration for gateway_config table (Story 11.1)
    ///
    /// Creates the gateway_config table for storing MCP Gateway configuration.
    /// Uses a singleton pattern (id = 1) to ensure only one config record exists.
    fn run_gateway_config_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if table already exists
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
                -- gateway_config 表: MCP Gateway 配置 (Story 11.1)
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

            // Insert default config with UUID v4 token
            let auth_token = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO gateway_config (id, auth_token) VALUES (1, ?1)",
                [&auth_token],
            )?;
        }

        Ok(())
    }

    /// Migration for MCP services tables (Story 11.2)
    ///
    /// Creates:
    /// - mcp_services table: MCP 服务配置
    /// - project_mcp_services table: 项目与 MCP 服务的多对多关联
    /// - env_variables table: 加密存储的环境变量
    fn run_mcp_services_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if mcp_services table already exists
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
                -- mcp_services 表: MCP 服务配置 (Story 11.2)
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

                -- 按名称索引，用于快速查找
                CREATE INDEX IF NOT EXISTS idx_mcp_services_name ON mcp_services(name);
                -- 按来源索引，用于筛选导入/手动服务
                CREATE INDEX IF NOT EXISTS idx_mcp_services_source ON mcp_services(source);
                "#,
            )?;
        }

        // Check if project_mcp_services table already exists
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
                -- project_mcp_services 表: 项目与 MCP 服务的多对多关联 (Story 11.2)
                CREATE TABLE IF NOT EXISTS project_mcp_services (
                    project_id TEXT NOT NULL,
                    service_id TEXT NOT NULL,
                    config_override TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    PRIMARY KEY (project_id, service_id),
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                    FOREIGN KEY (service_id) REFERENCES mcp_services(id) ON DELETE CASCADE
                );

                -- 按项目 ID 索引
                CREATE INDEX IF NOT EXISTS idx_project_mcp_services_project ON project_mcp_services(project_id);
                -- 按服务 ID 索引
                CREATE INDEX IF NOT EXISTS idx_project_mcp_services_service ON project_mcp_services(service_id);
                "#,
            )?;
        }

        // Check if env_variables table already exists
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
                -- env_variables 表: 加密存储的环境变量 (Story 11.2)
                CREATE TABLE IF NOT EXISTS env_variables (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    encrypted_value BLOB NOT NULL,
                    description TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                -- 按名称索引（唯一约束已提供）
                CREATE INDEX IF NOT EXISTS idx_env_variables_name ON env_variables(name);
                "#,
            )?;
        }

        Ok(())
    }

    /// Migration for MCP service tools cache table (Story 11.10)
    ///
    /// Creates:
    /// - mcp_service_tools table: 缓存 MCP 服务的工具列表
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
                -- mcp_service_tools 表: MCP 服务工具缓存 (Story 11.10)
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

                -- 按服务 ID 索引
                CREATE INDEX IF NOT EXISTS idx_mcp_service_tools_service ON mcp_service_tools(service_id);
                "#,
            )?;
        }

        Ok(())
    }

    /// Migration for HTTP transport support in MCP services (Story 11.11)
    ///
    /// Adds:
    /// - transport_type column: 'stdio' (default) or 'http'
    /// - url column: HTTP endpoint URL for http transport
    /// - headers column: HTTP headers as JSON for http transport
    fn run_mcp_http_transport_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if transport_type column exists
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
                -- 添加传输类型字段（默认 stdio）
                ALTER TABLE mcp_services ADD COLUMN transport_type TEXT NOT NULL DEFAULT 'stdio'
                    CHECK(transport_type IN ('stdio', 'http'));
                -- 添加 HTTP 端点 URL 字段
                ALTER TABLE mcp_services ADD COLUMN url TEXT;
                -- 添加 HTTP 请求头字段（JSON 格式）
                ALTER TABLE mcp_services ADD COLUMN headers TEXT;
                "#,
            )?;
        }

        Ok(())
    }

    /// Migration for MCP takeover backups table (Story 11.15)
    ///
    /// Creates:
    /// - mcp_takeover_backups table: 记录 MCP 配置接管的备份信息
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
                -- mcp_takeover_backups 表: MCP 配置接管备份记录 (Story 11.15)
                CREATE TABLE IF NOT EXISTS mcp_takeover_backups (
                    id TEXT PRIMARY KEY,
                    tool_type TEXT NOT NULL CHECK(tool_type IN ('claude_code', 'cursor', 'codex', 'gemini_cli')),
                    original_path TEXT NOT NULL,
                    backup_path TEXT NOT NULL,
                    taken_over_at TEXT NOT NULL,
                    restored_at TEXT,
                    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'restored'))
                );

                -- 按状态索引 (快速查询活跃的接管记录)
                CREATE INDEX IF NOT EXISTS idx_takeover_status ON mcp_takeover_backups(status);
                -- 按工具类型索引 (按工具筛选)
                CREATE INDEX IF NOT EXISTS idx_takeover_tool ON mcp_takeover_backups(tool_type);
                "#,
            )?;
        }

        Ok(())
    }

    /// Migration for MCP takeover scope support (Story 11.16)
    ///
    /// Adds:
    /// - scope column: 'user' (default) or 'project'
    /// - project_path column: project path for project-level takeovers
    /// - Indexes for efficient querying by scope and project_path
    fn run_mcp_takeover_scope_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if scope column exists
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
                -- 添加 scope 字段（默认 'user'）
                ALTER TABLE mcp_takeover_backups ADD COLUMN scope TEXT NOT NULL DEFAULT 'user'
                    CHECK(scope IN ('user', 'project'));
                -- 添加 project_path 字段
                ALTER TABLE mcp_takeover_backups ADD COLUMN project_path TEXT;
                -- 创建 scope 索引
                CREATE INDEX IF NOT EXISTS idx_takeover_scope ON mcp_takeover_backups(scope);
                -- 创建 project_path 索引
                CREATE INDEX IF NOT EXISTS idx_takeover_project_path ON mcp_takeover_backups(project_path);
                "#,
            )?;
        }

        Ok(())
    }

    /// Create an in-memory database for testing
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(include_str!("schema.sql"))?;
        Self::run_migrations(&conn)?;
        Ok(Self { conn })
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get a mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
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
}
