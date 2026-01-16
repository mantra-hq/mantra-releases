/**
 * Analytics 类型定义
 * Story 2.34: 项目分析统计功能
 *
 * 与 Rust 后端类型保持一致 (snake_case 命名)
 */

/**
 * 时间范围筛选
 * - days7: 最近 7 天
 * - days30: 最近 30 天（默认）
 * - all: 全部时间
 */
export type TimeRange = "days7" | "days30" | "all";

/**
 * 会话级指标
 *
 * 在会话导入时预计算并存储，用于快速检索
 */
export interface SessionMetrics {
  /** 会话 ID */
  session_id: string;

  /** 工具/来源类型 (claude, gemini, cursor, codex) */
  tool_type: string;

  /** 会话开始时间 (Unix 时间戳秒) */
  start_time: number;

  /** 会话结束时间 (Unix 时间戳秒) */
  end_time: number;

  /** 会话时长（秒） */
  duration_seconds: number;

  /** 消息总数 (用户 + 助手) */
  message_count: number;

  /** 工具调用总数 */
  tool_call_count: number;

  /** 工具调用错误数 */
  tool_error_count: number;

  /** 使用的工具类型列表 (如 ["file_read", "shell_exec", "content_search"]) */
  tool_types_used: string[];

  /** 工具类型分布 (tool_type -> count) */
  tool_type_counts: Record<string, number>;
}

/**
 * 活动趋势数据点
 *
 * 用于趋势图表显示
 */
export interface ActivityDataPoint {
  /** 日期字符串 (YYYY-MM-DD) */
  date: string;

  /** 当日会话数 */
  session_count: number;

  /** 当日工具调用数 */
  tool_call_count: number;

  /** 当日总时长（秒） */
  duration_seconds: number;
}

/**
 * 项目级分析统计
 *
 * 按需从会话指标聚合计算，用于项目统计仪表盘显示
 */
export interface ProjectAnalytics {
  /** 项目 ID */
  project_id: string;

  /** 计算使用的时间范围 */
  time_range: TimeRange;

  // === 会话统计 ===

  /** 会话总数 */
  total_sessions: number;

  /** 总时长（秒） */
  total_duration_seconds: number;

  /** 平均会话时长（秒） */
  avg_duration_seconds: number;

  /** 活跃天数 */
  active_days: number;

  /** 工具/来源分布 (claude/gemini/cursor/codex -> count) */
  tool_distribution: Record<string, number>;

  // === 效率指标 ===

  /** 总工具调用数 */
  total_tool_calls: number;

  /** 总工具错误数 */
  total_tool_errors: number;

  /** 工具错误率 (0.0 - 1.0) */
  tool_error_rate: number;

  /** 工具类型分布 (file_read/shell_exec/etc -> count) */
  tool_types_distribution: Record<string, number>;

  // === 活动趋势 ===

  /** 每日活动数据 */
  activity_trend: ActivityDataPoint[];

  // === 消息统计 ===

  /** 消息总数 */
  total_messages: number;
}

/**
 * 工具调用详情
 *
 * 用于会话级统计视图的时间线显示
 */
export interface ToolCallDetail {
  /** 工具类型 (如 "file_read", "shell_exec") */
  tool_type: string;

  /** 调用时间 (Unix 时间戳秒) */
  timestamp: number;

  /** 是否为错误 */
  is_error: boolean;

  /** 简要描述或路径 */
  description?: string;
}

/**
 * 会话统计视图
 *
 * 包含工具调用时间线和详细分布，用于会话级统计显示
 */
export interface SessionStatsView {
  /** 基础会话指标 */
  metrics: SessionMetrics;

  /** 工具调用时间线（按时间顺序） */
  tool_call_timeline: ToolCallDetail[];

  /** 工具分布（饼图用） */
  tool_distribution: Record<string, number>;
}
