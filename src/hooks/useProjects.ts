/**
 * useProjects Hook - 获取项目列表
 * Story 2.8: Task 4
 * Story 9.4: 使用 IPC 适配器支持 E2E 测试环境
 * Story 1.12: View-based Project Aggregation
 *
 * 调用 Tauri IPC 获取项目列表，管理加载和错误状态
 */

import { useState, useEffect, useCallback } from "react";
// Story 9.4: 使用 IPC 适配器支持 E2E 测试环境
import { invoke } from "@/lib/ipc-adapter";
import type { Project, ProjectPath, SessionBinding, LogicalProjectStats } from "@/types/project";
import type { Session } from "@/types/project";

/**
 * useProjects 返回值类型
 */
export interface UseProjectsResult {
  /** 项目列表 (按最后活动时间降序排列) */
  projects: Project[];
  /** 是否正在加载 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 重新获取项目列表 */
  refetch: () => void;
}

/**
 * useProjects Hook
 * 获取项目列表，支持加载状态、错误处理和重新获取
 */
export function useProjects(): UseProjectsResult {
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  /**
   * 获取项目列表
   */
  const fetchProjects = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const data = await invoke<Project[]>("list_projects");

      // Rust 已按 last_activity DESC 排序，直接使用
      setProjects(data);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "获取项目列表失败";
      setError(errorMessage);
      setProjects([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 组件挂载时自动获取
  useEffect(() => {
    fetchProjects();
  }, [fetchProjects]);

  return {
    projects,
    isLoading,
    error,
    refetch: fetchProjects,
  };
}

// ============================================================================
// Story 1.12: View-based Project Aggregation API Functions
// ============================================================================

/**
 * 获取项目的所有路径
 * @param projectId 项目 ID
 */
export async function getProjectPaths(projectId: string): Promise<ProjectPath[]> {
  return invoke<ProjectPath[]>("get_project_paths", { projectId });
}

/**
 * 添加项目路径
 * @param projectId 项目 ID
 * @param path 路径
 * @param isPrimary 是否设为主路径
 */
export async function addProjectPath(
  projectId: string,
  path: string,
  isPrimary?: boolean
): Promise<ProjectPath> {
  return invoke<ProjectPath>("add_project_path", { projectId, path, isPrimary });
}

/**
 * 移除项目路径
 * @param pathId 路径记录 ID
 */
export async function removeProjectPath(pathId: string): Promise<void> {
  return invoke<void>("remove_project_path", { pathId });
}

/**
 * 设置项目主路径
 * @param projectId 项目 ID
 * @param path 路径
 */
export async function setProjectPrimaryPath(
  projectId: string,
  path: string
): Promise<ProjectPath> {
  return invoke<ProjectPath>("set_project_primary_path", { projectId, path });
}

/**
 * 将会话手动绑定到项目
 * @param sessionId 会话 ID
 * @param projectId 项目 ID
 */
export async function bindSessionToProject(
  sessionId: string,
  projectId: string
): Promise<SessionBinding> {
  return invoke<SessionBinding>("bind_session_to_project", { sessionId, projectId });
}

/**
 * 解除会话的手动绑定
 * @param sessionId 会话 ID
 */
export async function unbindSession(sessionId: string): Promise<void> {
  return invoke<void>("unbind_session", { sessionId });
}

/**
 * 获取未分类的会话列表
 * 这些会话没有手动绑定，也没有匹配的项目路径
 */
export async function getUnassignedSessions(): Promise<Session[]> {
  return invoke<Session[]>("get_unassigned_sessions");
}

/**
 * useUnassignedSessions Hook
 * 获取未分类会话列表，支持加载状态和重新获取
 */
export function useUnassignedSessions() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchSessions = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const data = await getUnassignedSessions();
      setSessions(data);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "获取未分类会话失败";
      setError(errorMessage);
      setSessions([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  return {
    sessions,
    isLoading,
    error,
    refetch: fetchSessions,
  };
}

/**
 * useProjectPaths Hook
 * 获取项目的所有关联路径
 */
export function useProjectPaths(projectId: string | null) {
  const [paths, setPaths] = useState<ProjectPath[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPaths = useCallback(async () => {
    if (!projectId) {
      setPaths([]);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const data = await getProjectPaths(projectId);
      setPaths(data);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "获取项目路径失败";
      setError(errorMessage);
      setPaths([]);
    } finally {
      setIsLoading(false);
    }
  }, [projectId]);

  useEffect(() => {
    fetchPaths();
  }, [fetchPaths]);

  return {
    paths,
    isLoading,
    error,
    refetch: fetchPaths,
  };
}

// ============================================================================
// Story 1.12: Physical Path Aggregation API Functions
// ============================================================================

/**
 * 获取逻辑项目统计 (按物理路径聚合)
 * 返回所有唯一物理路径的聚合统计信息
 */
export async function getLogicalProjectStats(): Promise<LogicalProjectStats[]> {
  return invoke<LogicalProjectStats[]>("get_logical_project_stats");
}

/**
 * 获取物理路径下的所有会话
 * @param physicalPath 物理路径
 */
export async function getSessionsByPhysicalPath(
  physicalPath: string
): Promise<Session[]> {
  return invoke<Session[]>("get_sessions_by_physical_path", { physicalPath });
}

/**
 * 获取共享物理路径的所有项目
 * @param physicalPath 物理路径
 */
export async function getProjectsByPhysicalPath(
  physicalPath: string
): Promise<Project[]> {
  return invoke<Project[]>("get_projects_by_physical_path", { physicalPath });
}

// ============================================================================
// Story 1.13: Logical Project Rename API Functions
// ============================================================================

/**
 * 重命名逻辑项目 (Story 1.13)
 *
 * 为逻辑项目设置自定义显示名称。自定义名称存储在 logical_project_names 表中，
 * 优先于从路径提取的默认名称。
 *
 * @param physicalPath 逻辑项目的物理路径
 * @param newName 新的显示名称
 * @throws 如果名称为空或操作失败
 */
export async function renameLogicalProject(
  physicalPath: string,
  newName: string
): Promise<void> {
  return invoke<void>("rename_logical_project", { physicalPath, newName });
}

/**
 * 重置逻辑项目名称为默认值 (Story 1.13)
 *
 * 删除逻辑项目的自定义名称，恢复从物理路径提取的默认名称。
 *
 * @param physicalPath 逻辑项目的物理路径
 * @throws 如果没有自定义名称或操作失败
 */
export async function resetLogicalProjectName(
  physicalPath: string
): Promise<void> {
  return invoke<void>("reset_logical_project_name", { physicalPath });
}

/**
 * useLogicalProjectStats Hook
 * 获取按物理路径聚合的逻辑项目统计
 */
export function useLogicalProjectStats() {
  const [stats, setStats] = useState<LogicalProjectStats[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchStats = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const data = await getLogicalProjectStats();
      setStats(data);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "获取逻辑项目统计失败";
      setError(errorMessage);
      setStats([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStats();
  }, [fetchStats]);

  return {
    stats,
    isLoading,
    error,
    refetch: fetchStats,
  };
}

/**
 * useSessionsByPhysicalPath Hook
 * 获取物理路径下的所有会话
 */
export function useSessionsByPhysicalPath(physicalPath: string | null) {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSessions = useCallback(async () => {
    if (!physicalPath) {
      setSessions([]);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const data = await getSessionsByPhysicalPath(physicalPath);
      setSessions(data);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "获取会话列表失败";
      setError(errorMessage);
      setSessions([]);
    } finally {
      setIsLoading(false);
    }
  }, [physicalPath]);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  return {
    sessions,
    isLoading,
    error,
    refetch: fetchSessions,
  };
}

/**
 * useProjectsByPhysicalPath Hook
 * 获取共享物理路径的所有项目
 */
export function useProjectsByPhysicalPath(physicalPath: string | null) {
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchProjects = useCallback(async () => {
    if (!physicalPath) {
      setProjects([]);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const data = await getProjectsByPhysicalPath(physicalPath);
      setProjects(data);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "获取项目列表失败";
      setError(errorMessage);
      setProjects([]);
    } finally {
      setIsLoading(false);
    }
  }, [physicalPath]);

  useEffect(() => {
    fetchProjects();
  }, [fetchProjects]);

  return {
    projects,
    isLoading,
    error,
    refetch: fetchProjects,
  };
}
