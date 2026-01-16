/**
 * ToolTypesChart Component - 工具类型分布图
 * Story 2.34: Task 7.4
 *
 * 显示工具调用类型分布（Read / Edit / Bash / Grep 等）
 * 使用水平条形图展示
 */

import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
  Cell,
} from "recharts";
import { cn } from "@/lib/utils";

/**
 * 工具类型颜色 - 使用渐变色系
 */
const TOOL_TYPE_COLOR = "#3b82f6"; // Blue-500

/**
 * ToolTypesChart Props
 */
export interface ToolTypesChartProps {
  /** 工具类型分布数据 (tool_type -> count) */
  data: Record<string, number>;
  /** 最大显示条目数 */
  maxItems?: number;
  /** 自定义 className */
  className?: string;
}

/**
 * 自定义 Tooltip
 */
interface CustomTooltipProps {
  active?: boolean;
  payload?: Array<{
    payload: {
      name: string;
      displayName: string;
      value: number;
      percentage: number;
    };
  }>;
}

function CustomTooltip({ active, payload }: CustomTooltipProps) {
  const { t } = useTranslation();

  if (active && payload && payload.length > 0) {
    const data = payload[0].payload;
    return (
      <div className="bg-popover border border-border rounded-md px-3 py-2 shadow-md">
        <p className="text-sm font-medium">{data.displayName}</p>
        <p className="text-xs text-muted-foreground">
          {data.value} {t("analytics.units.calls")} ({data.percentage.toFixed(1)}%)
        </p>
      </div>
    );
  }
  return null;
}

/**
 * ToolTypesChart 组件
 *
 * 水平条形图展示工具类型调用排行
 */
export function ToolTypesChart({
  data,
  maxItems = 8,
  className,
}: ToolTypesChartProps) {
  const { t } = useTranslation();

  // 转换并排序数据，取前 N 项
  const chartData = useMemo(() => {
    const total = Object.values(data).reduce((sum, val) => sum + val, 0);
    if (total === 0) return [];

    return Object.entries(data)
      .filter(([, value]) => value > 0)
      .sort((a, b) => b[1] - a[1])
      .slice(0, maxItems)
      .map(([name, value]) => ({
        name,
        displayName: t(`analytics.tools.${name}`, name),
        value,
        percentage: (value / total) * 100,
      }));
  }, [data, maxItems, t]);

  if (chartData.length === 0) {
    return (
      <div
        className={cn(
          "flex items-center justify-center h-[200px] text-muted-foreground text-sm",
          className
        )}
        data-testid="tool-types-empty"
      >
        {t("common.noData")}
      </div>
    );
  }

  // 计算图表高度（每项 36px）
  const chartHeight = Math.max(chartData.length * 36, 120);

  return (
    <div className={cn("flex flex-col gap-4", className)} data-testid="tool-types-chart">
      {/* 图表标题 */}
      <h3 className="text-sm font-medium text-foreground">
        {t("analytics.charts.callRanking")}
      </h3>

      {/* 条形图 */}
      <div style={{ height: chartHeight }}>
        <ResponsiveContainer width="100%" height="100%">
          <BarChart
            data={chartData}
            layout="vertical"
            margin={{ top: 0, right: 0, bottom: 0, left: 0 }}
          >
            <XAxis type="number" hide />
            <YAxis
              type="category"
              dataKey="displayName"
              width={100}
              tick={{ fontSize: 12, fill: "var(--muted-foreground)" }}
              tickLine={false}
              axisLine={false}
            />
            <Tooltip content={<CustomTooltip />} cursor={{ fill: "rgba(128, 128, 128, 0.15)" }} />
            <Bar
              dataKey="value"
              radius={[0, 4, 4, 0]}
              maxBarSize={24}
            >
              {chartData.map((entry, index) => (
                <Cell
                  key={entry.name}
                  fill={TOOL_TYPE_COLOR}
                  fillOpacity={1 - index * 0.08}
                />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
