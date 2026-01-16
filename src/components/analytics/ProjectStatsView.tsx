/**
 * ProjectStatsView Component - 项目级统计视图
 * Story 2.34: Task 7.1
 *
 * 项目统计仪表盘，显示项目级别的统计信息
 * 包含指标卡片、工具分布、调用排行和活动趋势
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  CalendarDays,
  Clock,
  MessageSquare,
  Wrench,
  AlertCircle,
  Percent,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { getProjectAnalytics } from "@/lib/analytics-ipc";
import type { ProjectAnalytics, TimeRange } from "@/types/analytics";
import { MetricCard } from "./MetricCard";
import { TimeRangeSelector } from "./TimeRangeSelector";
import { ToolDistributionChart } from "./ToolDistributionChart";
import { ToolTypesChart } from "./ToolTypesChart";
import { ActivityTrendChart } from "./ActivityTrendChart";
import { AnalyticsEmptyState } from "./AnalyticsEmptyState";
import { Skeleton } from "@/components/ui/skeleton";

/**
 * ProjectStatsView Props
 */
export interface ProjectStatsViewProps {
  /** 项目 ID */
  projectId: string;
  /** 项目名称（用于显示） */
  projectName?: string;
  /** 打开导入向导回调 */
  onImport?: () => void;
  /** 自定义 className */
  className?: string;
}

/**
 * 格式化时长（秒 -> 可读格式）
 */
function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.round((seconds % 3600) / 60);
  if (minutes === 0) return `${hours}h`;
  return `${hours}h ${minutes}m`;
}

/**
 * 格式化数字（大数字使用 K/M 后缀）
 */
function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

/**
 * Loading Skeleton
 */
function ProjectStatsSkeleton() {
  return (
    <div className="flex flex-col gap-6 p-6" data-testid="project-stats-loading">
      {/* 指标卡片骨架 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <Skeleton key={i} className="h-24 rounded-lg" />
        ))}
      </div>

      {/* 趋势图骨架 */}
      <Skeleton className="h-[260px] rounded-lg" />

      {/* 图表骨架 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Skeleton className="h-[220px] rounded-lg" />
        <Skeleton className="h-[320px] rounded-lg" />
      </div>
    </div>
  );
}

/**
 * ProjectStatsView 组件
 *
 * 项目统计仪表盘主视图
 */
export function ProjectStatsView({
  projectId,
  projectName,
  onImport,
  className,
}: ProjectStatsViewProps) {
  const { t } = useTranslation();
  const [timeRange, setTimeRange] = useState<TimeRange>("days30");
  const [analytics, setAnalytics] = useState<ProjectAnalytics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 加载统计数据
  const loadAnalytics = useCallback(async () => {
    if (!projectId) return;

    setLoading(true);
    setError(null);

    try {
      const data = await getProjectAnalytics(projectId, timeRange);
      setAnalytics(data);
    } catch (err) {
      console.error("Failed to load project analytics:", err);
      setError(err instanceof Error ? err.message : "Failed to load analytics");
    } finally {
      setLoading(false);
    }
  }, [projectId, timeRange]);

  useEffect(() => {
    loadAnalytics();
  }, [loadAnalytics]);

  // Loading 状态
  if (loading) {
    return <ProjectStatsSkeleton />;
  }

  // 错误状态
  if (error) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center gap-4 p-6 h-full",
          className
        )}
        data-testid="project-stats-error"
      >
        <AlertCircle className="h-12 w-12 text-destructive/70" />
        <p className="text-sm text-muted-foreground">{error}</p>
      </div>
    );
  }

  // 空数据状态
  if (!analytics || analytics.total_sessions === 0) {
    return (
      <AnalyticsEmptyState
        type="no-data"
        onImport={onImport}
        className={className}
      />
    );
  }

  // 时间范围内无数据
  if (
    analytics.activity_trend.length === 0 &&
    analytics.total_sessions > 0 &&
    timeRange !== "all"
  ) {
    return (
      <div className={cn("flex flex-col gap-6 p-6", className)}>
        {/* 头部 */}
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-foreground">
            📊 {projectName || t("analytics.projectStats")}
          </h2>
          <TimeRangeSelector value={timeRange} onChange={setTimeRange} />
        </div>

        <AnalyticsEmptyState
          type="no-data-in-range"
          onChangeTimeRange={() => setTimeRange("all")}
        />
      </div>
    );
  }

  return (
    <div
      className={cn("flex flex-col gap-6 p-6 overflow-auto", className)}
      data-testid="project-stats-view"
    >
      {/* 头部：标题 + 时间范围选择器 */}
      <div className="flex items-center justify-between flex-shrink-0">
        <h2 className="text-lg font-semibold text-foreground">
          📊 {projectName || t("analytics.projectStats")}
        </h2>
        <TimeRangeSelector
          value={timeRange}
          onChange={setTimeRange}
          className="w-[120px]"
        />
      </div>

      {/* 指标卡片网格 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard
          title={t("analytics.metrics.sessions")}
          value={formatNumber(analytics.total_sessions)}
          icon={MessageSquare}
          data-testid="metric-sessions"
        />
        <MetricCard
          title={t("analytics.metrics.duration")}
          value={formatDuration(analytics.total_duration_seconds)}
          icon={Clock}
          data-testid="metric-duration"
        />
        <MetricCard
          title={t("analytics.metrics.activeDays")}
          value={analytics.active_days}
          unit={t("analytics.units.days")}
          icon={CalendarDays}
          data-testid="metric-active-days"
        />
        <MetricCard
          title={t("analytics.metrics.errorRate")}
          value={`${(analytics.tool_error_rate * 100).toFixed(1)}%`}
          icon={Percent}
          data-testid="metric-error-rate"
        />
      </div>

      {/* 更多指标（第二行） */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard
          title={t("analytics.metrics.avgDuration")}
          value={formatDuration(analytics.avg_duration_seconds)}
          icon={Clock}
          data-testid="metric-avg-duration"
        />
        <MetricCard
          title={t("analytics.metrics.toolCalls")}
          value={formatNumber(analytics.total_tool_calls)}
          icon={Wrench}
          data-testid="metric-tool-calls"
        />
        <MetricCard
          title={t("analytics.metrics.messages")}
          value={formatNumber(analytics.total_messages)}
          icon={MessageSquare}
          data-testid="metric-messages"
        />
        <MetricCard
          title={t("analytics.metrics.errors")}
          value={formatNumber(analytics.total_tool_errors)}
          icon={AlertCircle}
          data-testid="metric-errors"
        />
      </div>

      {/* 活动趋势图 */}
      <div className="p-4 rounded-lg bg-muted/30 border border-border/50">
        <ActivityTrendChart data={analytics.activity_trend} />
      </div>

      {/* 图表区域 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* 工具分布饼图 */}
        <div className="p-4 rounded-lg bg-muted/30 border border-border/50">
          <ToolDistributionChart data={analytics.tool_distribution} />
        </div>

        {/* 工具类型排行 */}
        <div className="p-4 rounded-lg bg-muted/30 border border-border/50">
          <ToolTypesChart data={analytics.tool_types_distribution} />
        </div>
      </div>
    </div>
  );
}
