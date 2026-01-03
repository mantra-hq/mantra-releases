/**
 * useProjectDrawer Hook
 * Story 2.18: Task 6
 *
 * 封装项目抽屉的状态管理：
 * - 抽屉开关状态
 * - 项目展开/折叠状态
 * - 搜索过滤逻辑
 */

import * as React from "react";
import { useProjects } from "./useProjects";
import { getProjectSessions, type SessionSummary } from "@/lib/project-ipc";

/**
 * useProjectDrawer 返回值类型
 */
export interface UseProjectDrawerResult {
  /** 抽屉是否打开 */
  isOpen: boolean;
  /** 设置抽屉打开状态 */
  setIsOpen: (open: boolean) => void;
  /** 打开抽屉 */
  openDrawer: () => void;
  /** 关闭抽屉 */
  closeDrawer: () => void;
  /** 切换抽屉状态 */
  toggleDrawer: () => void;

  /** 项目列表 */
  projects: ReturnType<typeof useProjects>["projects"];
  /** 是否正在加载 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 重新获取项目列表 */
  refetchProjects: () => void;

  /** 获取项目会话列表 */
  getProjectSessions: (projectId: string) => Promise<SessionSummary[]>;
}

/**
 * useProjectDrawer Hook
 * 管理项目抽屉的完整状态
 */
export function useProjectDrawer(): UseProjectDrawerResult {
  // 抽屉开关状态
  const [isOpen, setIsOpen] = React.useState(false);

  // 项目列表
  const { projects, isLoading, error, refetch } = useProjects();

  // 打开抽屉
  const openDrawer = React.useCallback(() => {
    setIsOpen(true);
  }, []);

  // 关闭抽屉
  const closeDrawer = React.useCallback(() => {
    setIsOpen(false);
  }, []);

  // 切换抽屉
  const toggleDrawer = React.useCallback(() => {
    setIsOpen((prev) => !prev);
  }, []);

  return {
    isOpen,
    setIsOpen,
    openDrawer,
    closeDrawer,
    toggleDrawer,

    projects,
    isLoading,
    error,
    refetchProjects: refetch,

    getProjectSessions,
  };
}
