/**
 * ActivityTrendChart Component - 活动趋势图
 * Story 2.34: Task 7.5
 *
 * 显示项目活动趋势（会话数、工具调用等随时间变化）
 */

import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
  CartesianGrid,
} from "recharts";
import { format, parseISO } from "date-fns";
import { zhCN, enUS } from "date-fns/locale";
import { cn } from "@/lib/utils";
import type { ActivityDataPoint } from "@/types/analytics";

/**
 * ActivityTrendChart Props
 */
export interface ActivityTrendChartProps {
  /** 活动趋势数据 */
  data: ActivityDataPoint[];
  /** 自定义 className */
  className?: string;
}

/**
 * 自定义 Tooltip
 */
interface CustomTooltipProps {
  active?: boolean;
  payload?: Array<{
    payload: ActivityDataPoint;
    dataKey: string;
    value: number;
    color: string;
  }>;
  label?: string;
}

function CustomTooltip({ active, payload }: CustomTooltipProps) {
  const { t, i18n } = useTranslation();

  if (active && payload && payload.length > 0) {
    const data = payload[0].payload;
    const dateLocale = i18n.language === "zh-CN" ? zhCN : enUS;

    return (
      <div className="bg-popover border border-border rounded-md px-3 py-2 shadow-md">
        <p className="text-sm font-medium mb-1">
          {format(parseISO(data.date), "MMM d, yyyy", { locale: dateLocale })}
        </p>
        <div className="space-y-1 text-xs">
          <p className="text-muted-foreground">
            {t("analytics.metrics.sessions")}: {data.session_count}
          </p>
          <p className="text-muted-foreground">
            {t("analytics.metrics.toolCalls")}: {data.tool_call_count}
          </p>
          <p className="text-muted-foreground">
            {t("analytics.metrics.duration")}: {formatDuration(data.duration_seconds)}
          </p>
        </div>
      </div>
    );
  }
  return null;
}

/**
 * 格式化时长（秒 -> 可读格式）
 */
function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
  return `${(seconds / 3600).toFixed(1)}h`;
}

/**
 * ActivityTrendChart 组件
 *
 * 面积图展示项目活动趋势
 */
export function ActivityTrendChart({
  data,
  className,
}: ActivityTrendChartProps) {
  const { t, i18n } = useTranslation();

  // 格式化日期标签
  const formattedData = useMemo(() => {
    const dateLocale = i18n.language === "zh-CN" ? zhCN : enUS;
    return data.map((point) => ({
      ...point,
      dateLabel: format(parseISO(point.date), "M/d", { locale: dateLocale }),
    }));
  }, [data, i18n.language]);

  if (data.length === 0) {
    return (
      <div
        className={cn(
          "flex items-center justify-center h-[200px] text-muted-foreground text-sm",
          className
        )}
        data-testid="activity-trend-empty"
      >
        {t("common.noData")}
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col gap-4", className)} data-testid="activity-trend-chart">
      {/* 图表标题 */}
      <h3 className="text-sm font-medium text-foreground">
        {t("analytics.charts.activityTrend")}
      </h3>

      {/* 面积图 */}
      <div className="h-[200px]">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
            data={formattedData}
            margin={{ top: 10, right: 10, bottom: 0, left: 0 }}
          >
            <defs>
              <linearGradient id="sessionGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
              </linearGradient>
              <linearGradient id="toolCallGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid
              strokeDasharray="3 3"
              stroke="var(--border)"
              strokeOpacity={0.5}
              vertical={false}
            />
            <XAxis
              dataKey="dateLabel"
              tick={{ fontSize: 11, fill: "var(--muted-foreground)" }}
              tickLine={false}
              axisLine={false}
              interval="preserveStartEnd"
            />
            <YAxis
              tick={{ fontSize: 11, fill: "var(--muted-foreground)" }}
              tickLine={false}
              axisLine={false}
              width={30}
            />
            <Tooltip content={<CustomTooltip />} />
            <Area
              type="monotone"
              dataKey="session_count"
              stroke="#3b82f6"
              strokeWidth={2}
              fill="url(#sessionGradient)"
              name={t("analytics.metrics.sessions")}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* 图例 */}
      <div className="flex items-center gap-4 text-xs">
        <div className="flex items-center gap-1.5">
          <span className="w-3 h-0.5 bg-blue-500 rounded" />
          <span className="text-muted-foreground">{t("analytics.metrics.sessions")}</span>
        </div>
      </div>
    </div>
  );
}
