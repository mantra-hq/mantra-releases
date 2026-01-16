/**
 * SessionStatsView Component - 会话级统计视图
 * Story 2.34: Task 8.1
 *
 * 会话统计仪表盘，显示单个会话的统计信息
 * 包含指标卡片、工具分布、调用时间线和详情列表
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { format } from "date-fns";
import { zhCN, enUS } from "date-fns/locale";
import {
  Clock,
  MessageSquare,
  Wrench,
  AlertCircle,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { getSessionStatsView } from "@/lib/analytics-ipc";
import type { SessionStatsView as SessionStatsViewType } from "@/types/analytics";
import { MetricCard } from "./MetricCard";
import { ToolDistributionChart } from "./ToolDistributionChart";
import { ToolCallTimeline } from "./ToolCallTimeline";
import { ToolCallDetailsList } from "./ToolCallDetailsList";
import { Skeleton } from "@/components/ui/skeleton";

/**
 * SessionStatsView Props
 */
export interface SessionStatsViewProps {
  /** 会话 ID */
  sessionId: string;
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
 * 格式化数字
 */
function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

/**
 * Loading Skeleton
 */
function SessionStatsSkeleton() {
  return (
    <div className="flex flex-col gap-6 p-6" data-testid="session-stats-loading">
      {/* 标题骨架 */}
      <Skeleton className="h-8 w-64" />

      {/* 指标卡片骨架 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <Skeleton key={i} className="h-24 rounded-lg" />
        ))}
      </div>

      {/* 图表骨架 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Skeleton className="h-[280px] rounded-lg" />
        <Skeleton className="h-[280px] rounded-lg" />
      </div>
    </div>
  );
}

/**
 * SessionStatsView 组件
 *
 * 会话统计仪表盘主视图
 */
export function SessionStatsView({
  sessionId,
  className,
}: SessionStatsViewProps) {
  const { t, i18n } = useTranslation();
  const [statsView, setStatsView] = useState<SessionStatsViewType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 加载会话统计数据
  const loadStats = useCallback(async () => {
    if (!sessionId) return;

    setLoading(true);
    setError(null);

    try {
      const data = await getSessionStatsView(sessionId);
      setStatsView(data);
    } catch (err) {
      console.error("Failed to load session stats:", err);
      setError(err instanceof Error ? err.message : "Failed to load stats");
    } finally {
      setLoading(false);
    }
  }, [sessionId]);

  useEffect(() => {
    loadStats();
  }, [loadStats]);

  // 格式化会话日期
  const sessionDateLabel = statsView?.metrics.start_time
    ? format(
        new Date(statsView.metrics.start_time * 1000),
        "yyyy-MM-dd HH:mm",
        { locale: i18n.language === "zh-CN" ? zhCN : enUS }
      )
    : "";

  // Loading 状态
  if (loading) {
    return <SessionStatsSkeleton />;
  }

  // 错误状态
  if (error) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center gap-4 p-6 h-full",
          className
        )}
        data-testid="session-stats-error"
      >
        <AlertCircle className="h-12 w-12 text-destructive/70" />
        <p className="text-sm text-muted-foreground">{error}</p>
      </div>
    );
  }

  // 无数据状态
  if (!statsView) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center gap-4 p-6 h-full",
          className
        )}
        data-testid="session-stats-empty"
      >
        <AlertCircle className="h-12 w-12 text-muted-foreground/70" />
        <p className="text-sm text-muted-foreground">{t("common.noData")}</p>
      </div>
    );
  }

  const { metrics, tool_call_timeline, tool_distribution } = statsView;

  return (
    <div
      className={cn("flex flex-col gap-6 p-6 overflow-auto", className)}
      data-testid="session-stats-view"
    >
      {/* 头部：标题 */}
      <div className="flex items-center justify-between flex-shrink-0">
        <h2 className="text-lg font-semibold text-foreground">
          📊 {t("analytics.sessionStats")} - {sessionDateLabel}
        </h2>
      </div>

      {/* 指标卡片网格 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard
          title={t("analytics.metrics.duration")}
          value={formatDuration(metrics.duration_seconds)}
          icon={Clock}
          data-testid="session-metric-duration"
        />
        <MetricCard
          title={t("analytics.metrics.messages")}
          value={formatNumber(metrics.message_count)}
          icon={MessageSquare}
          data-testid="session-metric-messages"
        />
        <MetricCard
          title={t("analytics.metrics.toolCalls")}
          value={formatNumber(metrics.tool_call_count)}
          icon={Wrench}
          data-testid="session-metric-tool-calls"
        />
        <MetricCard
          title={t("analytics.metrics.errors")}
          value={formatNumber(metrics.tool_error_count)}
          icon={AlertCircle}
          data-testid="session-metric-errors"
        />
      </div>

      {/* 工具调用时间线 */}
      <div className="p-4 rounded-lg bg-muted/30 border border-border/50">
        <ToolCallTimeline data={tool_call_timeline} />
      </div>

      {/* 图表区域 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* 工具分布饼图 */}
        <div className="p-4 rounded-lg bg-muted/30 border border-border/50">
          <ToolDistributionChart data={tool_distribution} />
        </div>

        {/* 调用详情列表 */}
        <div className="p-4 rounded-lg bg-muted/30 border border-border/50">
          <ToolCallDetailsList data={tool_call_timeline} />
        </div>
      </div>
    </div>
  );
}
