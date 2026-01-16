/**
 * ToolDistributionChart Component - 工具分布饼图
 * Story 2.34: Task 7.3
 *
 * 显示 AI 工具来源分布（Claude Code / Gemini CLI / Cursor / Codex）
 */

import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from "recharts";
import { cn } from "@/lib/utils";

/**
 * 工具来源颜色配置
 */
const TOOL_COLORS: Record<string, string> = {
  claude: "#8b5cf6", // 紫色
  gemini: "#10b981", // 绿色
  cursor: "#3b82f6", // 蓝色
  codex: "#f59e0b", // 橙色
};

/**
 * ToolDistributionChart Props
 */
export interface ToolDistributionChartProps {
  /** 工具分布数据 (tool_type -> count) */
  data: Record<string, number>;
  /** 自定义 className */
  className?: string;
}

/**
 * 自定义 Tooltip 组件
 */
interface CustomTooltipProps {
  active?: boolean;
  payload?: Array<{
    name: string;
    value: number;
    payload: {
      name: string;
      value: number;
      percentage: number;
    };
  }>;
}

function CustomTooltip({ active, payload }: CustomTooltipProps) {
  const { t } = useTranslation();

  if (active && payload && payload.length > 0) {
    const data = payload[0].payload;
    const toolKey = data.name as "claude" | "gemini" | "cursor" | "codex";

    return (
      <div className="bg-popover border border-border rounded-md px-3 py-2 shadow-md">
        <p className="text-sm font-medium">
          {t(`analytics.source.${toolKey}`, data.name)}
        </p>
        <p className="text-xs text-muted-foreground">
          {data.value} {t("analytics.units.sessions")} ({data.percentage.toFixed(1)}%)
        </p>
      </div>
    );
  }
  return null;
}

/**
 * ToolDistributionChart 组件
 *
 * 饼图展示不同 AI 工具的使用分布
 */
export function ToolDistributionChart({
  data,
  className,
}: ToolDistributionChartProps) {
  const { t } = useTranslation();

  // 转换数据格式并计算百分比
  const chartData = useMemo(() => {
    const total = Object.values(data).reduce((sum, val) => sum + val, 0);
    if (total === 0) return [];

    return Object.entries(data)
      .filter(([, value]) => value > 0)
      .map(([name, value]) => ({
        name,
        value,
        percentage: (value / total) * 100,
      }));
  }, [data]);

  const total = useMemo(
    () => Object.values(data).reduce((sum, val) => sum + val, 0),
    [data]
  );

  if (chartData.length === 0) {
    return (
      <div
        className={cn(
          "flex items-center justify-center h-[200px] text-muted-foreground text-sm",
          className
        )}
        data-testid="tool-distribution-empty"
      >
        {t("common.noData")}
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col gap-4", className)} data-testid="tool-distribution-chart">
      {/* 图表标题 */}
      <h3 className="text-sm font-medium text-foreground">
        {t("analytics.charts.toolDistribution")}
      </h3>

      <div className="flex items-center gap-6">
        {/* 饼图 */}
        <div className="h-[160px] w-[160px] flex-shrink-0">
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Pie
                data={chartData}
                cx="50%"
                cy="50%"
                innerRadius={40}
                outerRadius={70}
                paddingAngle={2}
                dataKey="value"
              >
                {chartData.map((entry) => (
                  <Cell
                    key={entry.name}
                    fill={TOOL_COLORS[entry.name] || "#64748b"}
                    stroke="transparent"
                  />
                ))}
              </Pie>
              <Tooltip content={<CustomTooltip />} />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* 图例 */}
        <div className="flex flex-col gap-2">
          {chartData.map((entry) => {
            const toolKey = entry.name as "claude" | "gemini" | "cursor" | "codex";
            return (
              <div
                key={entry.name}
                className="flex items-center gap-2 text-sm"
              >
                <span
                  className="w-3 h-3 rounded-sm flex-shrink-0"
                  style={{
                    backgroundColor: TOOL_COLORS[entry.name] || "#64748b",
                  }}
                />
                <span className="text-foreground">
                  {t(`analytics.source.${toolKey}`, entry.name)}
                </span>
                <span className="text-muted-foreground">
                  {entry.percentage.toFixed(0)}%
                </span>
              </div>
            );
          })}
          <div className="text-xs text-muted-foreground mt-1">
            {t("common.loading") === "加载中"
              ? `共 ${total} 个会话`
              : `Total ${total} sessions`}
          </div>
        </div>
      </div>
    </div>
  );
}
