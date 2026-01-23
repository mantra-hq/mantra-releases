/**
 * useProjectDrawer Hook
 * Story 2.18: Task 6
 * Story 2.21: Task 3 (支持 defaultOpen)
 * Story 1.12: Phase 5 - 完全切换到逻辑项目视图
 *
 * 封装项目抽屉的状态管理：
 * - 抽屉开关状态
 * - 逻辑项目列表（按物理路径聚合）
 * - 搜索过滤逻辑
 */

import * as React from "react";
import { useLogicalProjectStats, getSessionsByPhysicalPath } from "./useProjects";
import type { LogicalProjectStats } from "@/types/project";
import type { SessionSummary } from "@/lib/project-ipc";

/**
 * useProjectDrawer 选项
 * Story 2.21: Task 3.1
 */
export interface UseProjectDrawerOptions {
  /** 是否默认打开 */
  defaultOpen?: boolean;
}

/**
 * useProjectDrawer 返回值类型
 * Story 1.12: 改用逻辑项目视图
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

  /** 逻辑项目列表 (按物理路径聚合) - Story 1.12 */
  logicalProjects: LogicalProjectStats[];
  /** 是否正在加载 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 重新获取项目列表 */
  refetchProjects: () => void;

  /** 获取逻辑项目的会话列表 (按物理路径) - Story 1.12 */
  getLogicalProjectSessions: (physicalPath: string) => Promise<SessionSummary[]>;
}

/**
 * useProjectDrawer Hook
 * 管理项目抽屉的完整状态
 * Story 2.21: Task 3.1 - 支持 defaultOpen 参数
 * Story 1.12: Phase 5 - 使用逻辑项目视图
 */
export function useProjectDrawer(options?: UseProjectDrawerOptions): UseProjectDrawerResult {
  // 抽屉开关状态 - 支持默认打开 (Story 2.21 AC #2)
  const [isOpen, setIsOpen] = React.useState(options?.defaultOpen ?? false);

  // Story 1.12: 使用逻辑项目统计（替代存储层项目列表）
  const { stats: logicalProjects, isLoading, error, refetch } = useLogicalProjectStats();

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

  // Story 1.12: 获取逻辑项目的会话列表
  // 将 Session[] 转换为 SessionSummary[] 以保持接口兼容
  const getLogicalProjectSessions = React.useCallback(
    async (physicalPath: string): Promise<SessionSummary[]> => {
      const sessions = await getSessionsByPhysicalPath(physicalPath);
      // Session 和 SessionSummary 结构基本一致，直接返回
      return sessions.map((s) => ({
        id: s.id,
        source: s.source,
        created_at: s.created_at,
        updated_at: s.updated_at,
        message_count: s.message_count,
        is_empty: s.is_empty ?? false,
        title: s.title,
        original_cwd: s.original_cwd,
      }));
    },
    []
  );

  return {
    isOpen,
    setIsOpen,
    openDrawer,
    closeDrawer,
    toggleDrawer,

    logicalProjects,
    isLoading,
    error,
    refetchProjects: refetch,

    getLogicalProjectSessions,
  };
}
