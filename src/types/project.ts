/**
 * Project Types - 项目和会话类型定义
 * Story 2.8: Dashboard 项目列表
 *
 * 定义项目聚合和会话相关的数据结构
 */

/**
 * 会话来源类型
 */
export type SessionSource = "claude" | "gemini" | "cursor" | "unknown";

/**
 * 会话摘要信息 (Rust: SessionSummary)
 * 用于项目列表中的会话展示
 */
export interface Session {
  /** 会话唯一标识 */
  id: string;
  /** 会话来源 */
  source: SessionSource;
  /** 创建时间 (ISO 8601 字符串) */
  created_at: string;
  /** 更新时间 (ISO 8601 字符串) */
  updated_at: string;
  /** 消息数量 */
  message_count: number;
  /** 会话是否为空 (Story 2.29) */
  is_empty?: boolean;
  /** 会话标题 (Story 1.12) */
  title?: string;
  /** 原始工作目录 (Story 1.12: 导入时记录，不可修改) */
  original_cwd?: string;
  /** 来源上下文 (Story 1.12: 导入源特定的元数据) */
  source_context?: SourceContext;
}

/**
 * 路径类型 (Story 1.12)
 *
 * 用于分类路径的类型，决定如何验证和显示路径。
 */
export type PathType = "local" | "virtual" | "remote";

/**
 * 项目信息 (匹配 Rust: Project，snake_case)
 * 从 list_projects 返回的数据
 */
export interface Project {
  /** 项目唯一标识 */
  id: string;
  /** 项目名称 (目录名) */
  name: string;
  /** 项目路径 (工作目录) */
  cwd: string;
  /** 会话数量 */
  session_count: number;
  /** 非空会话数量 - Story 2.29 V2 */
  non_empty_session_count?: number;
  /** 创建时间 (ISO 8601 字符串) */
  created_at: string;
  /** 最后活动时间 (ISO 8601 字符串) */
  last_activity: string;
  /** Git 仓库根路径 (如果检测到) */
  git_repo_path: string | null;
  /** 是否关联 Git 仓库 */
  has_git_repo: boolean;
  /** Git 远程 URL (Story 1.9: 用于跨路径项目聚合) */
  git_remote_url: string | null;
  /** 项目是否为空 (所有会话都是空会话) - Story 2.29 V2 */
  is_empty?: boolean;
  /** 路径类型 (Story 1.12: local/virtual/remote) */
  path_type?: PathType;
  /** 本机路径是否存在 (Story 1.12: 仅对 local 类型有效) */
  path_exists?: boolean;
}

/**
 * 代表性文件信息 (匹配 Rust: RepresentativeFile)
 * Story 2.11: 用于显示项目初始代码状态
 */
export interface RepresentativeFile {
  /** 文件路径 (相对于仓库根目录) */
  path: string;
  /** 文件内容 */
  content: string;
  /** 检测到的编程语言 */
  language: string;
}

/**
 * 快照结果 (匹配 Rust: SnapshotResult)
 * 用于获取 Git HEAD 版本的文件内容
 */
export interface SnapshotResult {
  /** 文件内容 */
  content: string;
  /** Commit Hash */
  commit_hash: string;
  /** Commit 消息 */
  commit_message: string;
  /** Commit 时间戳 (Unix seconds) */
  commit_timestamp: number;
}

/**
 * 导入结果 (匹配 Rust: ImportResult)
 */
export interface ImportResult {
  /** 成功导入的会话数 */
  imported_count: number;
  /** 跳过的重复会话数 */
  skipped_count: number;
  /** 新创建的项目数 */
  new_projects_count: number;
  /** 错误列表 */
  errors: string[];
}

// ============================================================================
// Story 1.12: View-based Project Aggregation Types
// ============================================================================

/**
 * 项目路径映射 (匹配 Rust: ProjectPath)
 *
 * 一个项目可以关联多个路径，实现灵活的项目聚合。
 */
export interface ProjectPath {
  /** 唯一标识 (UUID) */
  id: string;
  /** 所属项目 ID */
  project_id: string;
  /** 路径 (已标准化) */
  path: string;
  /** 是否为主路径 */
  is_primary: boolean;
  /** 创建时间 (ISO 8601 字符串) */
  created_at: string;
}

/**
 * 会话手动绑定 (匹配 Rust: SessionBinding)
 *
 * 允许用户手动将会话绑定到特定项目，
 * 优先级高于基于路径的自动匹配。
 */
export interface SessionBinding {
  /** 会话 ID */
  session_id: string;
  /** 绑定的项目 ID */
  project_id: string;
  /** 绑定时间 (ISO 8601 字符串) */
  bound_at: string;
}

/**
 * 来源上下文 (匹配 Rust: SourceContext)
 *
 * 存储导入源特定的元数据，用于标识会话来源。
 * 导入后不可变，用于调试/审计。
 */
export interface SourceContext {
  /** 原始文件路径 */
  file_path?: string;
  /** Claude Code: 编码的项目路径 */
  project_path_encoded?: string;
  /** Gemini CLI: 项目哈希 */
  project_hash?: string;
  /** Gemini CLI: 会话文件名 */
  session_filename?: string;
  /** Cursor: 工作区 ID */
  workspace_id?: string;
  /** Cursor: 工作区存储路径 */
  workspace_path?: string;
}

/**
 * 逻辑项目统计 (匹配 Rust: LogicalProjectStats)
 *
 * 按物理路径聚合的项目统计信息。
 * 用于视图层显示"逻辑项目"，将来自不同导入源的会话聚合在一起。
 *
 * Story 1.12 Phase 5: 增强数据结构
 */
export interface LogicalProjectStats {
  /** 物理路径 (已标准化) */
  physical_path: string;
  /** 关联此路径的项目数量 */
  project_count: number;
  /** 关联此路径的所有项目 ID */
  project_ids: string[];
  /** 所有项目的会话总数 */
  total_sessions: number;
  /** 最近活动时间 (ISO 8601 字符串) */
  last_activity: string;
  /** 显示名称 (从路径提取) - Task 8.1 */
  display_name: string;
  /** 路径类型: local/virtual/remote - Task 8.2 */
  path_type: PathType;
  /** 本机路径是否存在 (仅 local 类型有效) - Task 8.3 */
  path_exists: boolean;
  /** 是否需要关联真实路径 - Task 8.4
   * True if path_type is "virtual" or (path_type is "local" AND path_exists is false)
   */
  needs_association: boolean;
  /** 是否关联 Git 仓库 (聚合自关联项目) - Task 17 */
  has_git_repo: boolean;
}