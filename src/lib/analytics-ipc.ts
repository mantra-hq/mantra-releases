/**
 * analytics-ipc - Tauri IPC 统计分析功能封装
 * Story 2.34: 项目分析统计功能
 *
 * 提供统计分析相关的 Tauri IPC 调用封装：
 * - getProjectAnalytics - 获取项目级统计数据
 * - getSessionMetrics - 获取会话级指标
 * - getSessionStatsView - 获取会话详细统计视图
 */

import { invoke } from "@/lib/ipc-adapter";
import type {
  ProjectAnalytics,
  SessionMetrics,
  SessionStatsView,
  TimeRange,
} from "@/types/analytics";

/**
 * 获取项目级统计数据
 *
 * 聚合项目下所有会话的统计指标，支持时间范围筛选
 *
 * @param projectId - 项目 ID
 * @param timeRange - 时间范围 ("days7" | "days30" | "all")
 * @returns 项目统计数据
 */
export async function getProjectAnalytics(
  projectId: string,
  timeRange: TimeRange = "days30"
): Promise<ProjectAnalytics> {
  return invoke<ProjectAnalytics>("get_project_analytics", {
    projectId,
    timeRange,
  });
}

/**
 * 获取会话级指标
 *
 * 返回单个会话的统计指标
 *
 * @param sessionId - 会话 ID
 * @returns 会话指标
 */
export async function getSessionMetrics(
  sessionId: string
): Promise<SessionMetrics> {
  return invoke<SessionMetrics>("get_session_metrics", { sessionId });
}

/**
 * 获取会话详细统计视图
 *
 * 包含工具调用时间线和分布，用于会话级统计页面显示
 *
 * @param sessionId - 会话 ID
 * @returns 会话统计视图
 */
export async function getSessionStatsView(
  sessionId: string
): Promise<SessionStatsView> {
  return invoke<SessionStatsView>("get_session_stats_view", { sessionId });
}
