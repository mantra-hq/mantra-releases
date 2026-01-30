-- Mantra 本地数据库 Schema
-- 用于存储项目和会话数据

-- projects 表: 存储项目信息
-- Story 1.12: 移除 cwd 字段，改为通过 project_paths 表关联
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cwd TEXT NOT NULL UNIQUE,     -- 保留用于向后兼容，逐步迁移到 project_paths
    created_at TEXT NOT NULL,     -- ISO 8601 格式
    last_activity TEXT NOT NULL,
    git_repo_path TEXT,           -- Git 仓库根路径 (Story 2.11)
    has_git_repo INTEGER NOT NULL DEFAULT 0,  -- 是否关联 Git 仓库 (Story 2.11)
    git_remote_url TEXT,          -- Git remote URL (Story 1.9: 用于跨路径项目识别)
    is_empty INTEGER NOT NULL DEFAULT 0,  -- 项目是否为空 (所有会话都是空会话)
    path_type TEXT DEFAULT 'local',       -- Story 1.12: 路径类型 (local/virtual/remote)
    path_exists INTEGER DEFAULT 1         -- Story 1.12: 本机路径是否存在 (仅对 local 类型有效)
);

CREATE INDEX IF NOT EXISTS idx_projects_cwd ON projects(cwd);
CREATE INDEX IF NOT EXISTS idx_projects_last_activity ON projects(last_activity DESC);
CREATE INDEX IF NOT EXISTS idx_projects_git_remote_url ON projects(git_remote_url);

-- project_paths 表: 项目路径映射 (Story 1.12)
-- 一个项目可以关联多个路径，支持灵活的项目聚合
-- 同一路径可以属于多个项目（不同导入源），通过视图层聚合显示
CREATE TABLE IF NOT EXISTS project_paths (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    path TEXT NOT NULL,           -- 路径可以属于多个项目
    is_primary INTEGER NOT NULL DEFAULT 0,  -- 是否为主路径
    created_at TEXT NOT NULL,
    UNIQUE(project_id, path),     -- 复合唯一约束：同一项目内路径唯一
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_project_paths_project ON project_paths(project_id);
CREATE INDEX IF NOT EXISTS idx_project_paths_path ON project_paths(path);

-- session_project_bindings 表: 会话手动绑定 (Story 1.12)
-- 用户可以手动将会话绑定到任意项目，优先级高于路径匹配
CREATE TABLE IF NOT EXISTS session_project_bindings (
    session_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    bound_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_session_bindings_project ON session_project_bindings(project_id);

-- sessions 表: 存储会话信息
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    source TEXT NOT NULL,         -- 'claude' | 'gemini' | 'cursor' | 'codex'
    cwd TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0,
    is_empty INTEGER NOT NULL DEFAULT 0,  -- Story 2.29: 空会话标记 (无用户消息且无助手消息)
    original_cwd TEXT DEFAULT '',         -- Story 1.12: 原始 cwd，导入时记录，不可修改
    source_context TEXT DEFAULT '{}',     -- Story 1.12: 来源上下文 JSON (project_hash, workspace_id 等)
    raw_data TEXT NOT NULL,       -- JSON 序列化的完整 MantraSession
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE INDEX IF NOT EXISTS idx_sessions_project_id ON sessions(project_id);
CREATE INDEX IF NOT EXISTS idx_sessions_cwd ON sessions(cwd);
CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_original_cwd ON sessions(original_cwd);

-- gateway_config 表: MCP Gateway 配置 (Story 11.1)
-- 存储 Gateway Server 配置信息，使用单例模式 (id = 1)
CREATE TABLE IF NOT EXISTS gateway_config (
    id INTEGER PRIMARY KEY DEFAULT 1,
    port INTEGER,                          -- 监听端口 (NULL 表示自动分配)
    auth_token TEXT NOT NULL,              -- 认证 Token (UUID v4)
    enabled INTEGER DEFAULT 0,             -- 是否启用 Gateway
    auto_start INTEGER DEFAULT 0,          -- 是否随应用自动启动
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    CHECK (id = 1)                         -- 确保只有一条配置记录
);
