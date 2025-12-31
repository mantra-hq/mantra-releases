/**
 * Project Types - 项目和会话类型定义
 * Story 2.8: Dashboard 项目列表
 *
 * 定义项目聚合和会话相关的数据结构
 */

/**
 * 会话来源类型
 */
export type SessionSource = "claude" | "gemini" | "cursor";

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
}

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
  /** 创建时间 (ISO 8601 字符串) */
  created_at: string;
  /** 最后活动时间 (ISO 8601 字符串) */
  last_activity: string;
  /** Git 仓库根路径 (如果检测到) */
  git_repo_path: string | null;
  /** 是否关联 Git 仓库 */
  has_git_repo: boolean;
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
