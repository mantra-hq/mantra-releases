/**
 * project-ipc - Tauri IPC 项目功能封装
 * Story 2.11: Task 5
 *
 * 提供项目和 Git 相关的 Tauri IPC 调用封装：
 * - getProject - 根据 ID 获取项目
 * - getProjectByCwd - 根据 cwd 获取项目
 * - getRepresentativeFile - 获取代表性文件
 * - getFileAtHead - 获取 HEAD 版本文件内容
 * - detectGitRepo - 检测 Git 仓库
 */

import { invoke } from "@tauri-apps/api/core";
import type { Project, RepresentativeFile, SnapshotResult } from "@/types/project";

/**
 * 根据 ID 获取项目信息
 *
 * @param projectId - 项目 ID
 * @returns 项目信息，如果不存在返回 null
 */
export async function getProject(projectId: string): Promise<Project | null> {
  return invoke<Project | null>("get_project", { projectId });
}

/**
 * 根据 cwd 获取项目信息
 *
 * @param cwd - 项目工作目录路径
 * @returns 项目信息，如果不存在返回 null
 */
export async function getProjectByCwd(cwd: string): Promise<Project | null> {
  return invoke<Project | null>("get_project_by_cwd", { cwd });
}

/**
 * 获取项目的代表性文件
 *
 * 优先级: README.md → 入口文件 → 任意代码文件
 *
 * @param repoPath - Git 仓库路径
 * @returns 代表性文件信息，如果没有找到返回 null
 */
export async function getRepresentativeFile(repoPath: string): Promise<RepresentativeFile | null> {
  return invoke<RepresentativeFile | null>("get_representative_file", { repoPath });
}

/**
 * 获取 HEAD 版本的文件内容
 *
 * @param repoPath - Git 仓库路径
 * @param filePath - 相对于仓库根目录的文件路径
 * @returns 快照结果，包含文件内容和 commit 信息
 */
export async function getFileAtHead(repoPath: string, filePath: string): Promise<SnapshotResult> {
  return invoke<SnapshotResult>("get_file_at_head", { repoPath, filePath });
}

/**
 * 检测目录是否为 Git 仓库
 *
 * @param dirPath - 要检测的目录路径
 * @returns Git 仓库根路径，如果不是 Git 仓库返回 null
 */
export async function detectGitRepo(dirPath: string): Promise<string | null> {
  return invoke<string | null>("detect_git_repo", { dirPath });
}

/**
 * 获取指定时间戳的文件快照
 *
 * @param repoPath - Git 仓库路径
 * @param filePath - 相对于仓库根目录的文件路径
 * @param timestamp - Unix 秒级时间戳
 * @returns 快照结果
 */
export async function getSnapshotAtTime(
  repoPath: string,
  filePath: string,
  timestamp: number
): Promise<SnapshotResult> {
  return invoke<SnapshotResult>("get_snapshot_at_time", { repoPath, filePath, timestamp });
}

/**
 * 获取项目列表
 *
 * @returns 项目列表，按最后活动时间降序排列
 */
export async function listProjects(): Promise<Project[]> {
  return invoke<Project[]>("list_projects");
}

/**
 * 会话摘要信息（匹配 Rust SessionSummary）
 * Story 2.18: Task 7
 */
export interface SessionSummary {
  /** 会话 ID */
  id: string;
  /** 会话来源 */
  source: "claude" | "gemini" | "cursor" | "unknown";
  /** 创建时间 (ISO 8601 字符串) */
  created_at: string;
  /** 更新时间 (ISO 8601 字符串) */
  updated_at: string;
  /** 消息数量 */
  message_count: number;
  /** 会话标题（来自 metadata，可选） */
  title?: string;
}

/**
 * 获取项目的所有会话
 * Story 2.18: Task 7
 *
 * @param projectId - 项目 ID
 * @returns 会话摘要列表，按更新时间降序排列
 */
export async function getProjectSessions(projectId: string): Promise<SessionSummary[]> {
  return invoke<SessionSummary[]>("get_project_sessions", { projectId });
}

// =============================================================================
// Story 2.19: Project Management IPC
// =============================================================================

/**
 * 更新的会话信息（消息数变化）
 */
export interface UpdatedSession {
  /** 会话 ID */
  session_id: string;
  /** 旧消息数 */
  old_message_count: number;
  /** 新消息数 */
  new_message_count: number;
}

/**
 * 同步结果
 */
export interface SyncResult {
  /** 新发现的会话 */
  new_sessions: SessionSummary[];
  /** 有新消息的会话 */
  updated_sessions: UpdatedSession[];
  /** 无变化的会话数 */
  unchanged_count: number;
}

/**
 * 同步项目：检测新会话和消息更新
 * Story 2.19: Task 9.1
 *
 * @param projectId - 项目 ID
 * @param force - 可选，强制重新解析所有会话（用于修复解析 bug 后恢复数据）
 * @returns 同步结果
 */
export async function syncProject(projectId: string, force?: boolean): Promise<SyncResult> {
  return invoke<SyncResult>("sync_project", { projectId, force });
}

/**
 * 移除项目（软删除）
 * Story 2.19: Task 9.2
 *
 * @param projectId - 项目 ID
 */
export async function removeProject(projectId: string): Promise<void> {
  return invoke<void>("remove_project", { projectId });
}

/**
 * 重命名项目
 * Story 2.19: Task 9.3
 *
 * @param projectId - 项目 ID
 * @param newName - 新的项目名称
 */
export async function renameProject(projectId: string, newName: string): Promise<void> {
  return invoke<void>("rename_project", { projectId, newName });
}

// =============================================================================
// Story 2.20: Import Wizard Enhancement IPC
// =============================================================================

/**
 * 获取所有已导入项目的 cwd 路径列表
 * Story 2.20: Task 2
 *
 * 用于导入向导识别已导入的项目
 *
 * @returns 已导入项目的 cwd 路径列表
 */
export async function getImportedProjectPaths(): Promise<string[]> {
  return invoke<string[]>("get_imported_project_paths");
}
