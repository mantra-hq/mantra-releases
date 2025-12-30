/**
 * useProjects Hook - 获取项目列表
 * Story 2.8: Task 4
 *
 * 调用 Tauri IPC 获取项目列表，管理加载和错误状态
 */

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Project } from "@/types/project";

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
  refetch: () => Promise<void>;
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
      const data = await invoke<Project[]>("get_projects");
      
      // 按最后活动时间降序排列
      const sorted = [...data].sort((a, b) => b.lastActivity - a.lastActivity);
      setProjects(sorted);
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

