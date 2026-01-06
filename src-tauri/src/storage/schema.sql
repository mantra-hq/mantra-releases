-- Mantra 本地数据库 Schema
-- 用于存储项目和会话数据

-- projects 表: 存储项目信息
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cwd TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,     -- ISO 8601 格式
    last_activity TEXT NOT NULL,
    git_repo_path TEXT,           -- Git 仓库根路径 (Story 2.11)
    has_git_repo INTEGER NOT NULL DEFAULT 0,  -- 是否关联 Git 仓库 (Story 2.11)
    git_remote_url TEXT,          -- Git remote URL (Story 1.9: 用于跨路径项目识别)
    is_empty INTEGER NOT NULL DEFAULT 0  -- 项目是否为空 (所有会话都是空会话)
);

CREATE INDEX IF NOT EXISTS idx_projects_cwd ON projects(cwd);
CREATE INDEX IF NOT EXISTS idx_projects_last_activity ON projects(last_activity DESC);
CREATE INDEX IF NOT EXISTS idx_projects_git_remote_url ON projects(git_remote_url);

-- sessions 表: 存储会话信息
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    source TEXT NOT NULL,         -- 'claude' | 'gemini' | 'cursor'
    cwd TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0,
    is_empty INTEGER NOT NULL DEFAULT 0,  -- Story 2.29: 空会话标记 (无用户消息且无助手消息)
    raw_data TEXT NOT NULL,       -- JSON 序列化的完整 MantraSession
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE INDEX IF NOT EXISTS idx_sessions_project_id ON sessions(project_id);
CREATE INDEX IF NOT EXISTS idx_sessions_cwd ON sessions(cwd);
CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at DESC);
