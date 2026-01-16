/**
 * MetricCard Component - 指标卡片组件
 * Story 2.34: Task 7.2
 *
 * 显示单个统计指标的卡片
 * 支持标题、数值、单位和图标
 */

import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * MetricCard Props
 */
export interface MetricCardProps {
  /** 指标标题 */
  title: string;
  /** 指标数值 */
  value: string | number;
  /** 可选单位 */
  unit?: string;
  /** 可选图标 */
  icon?: LucideIcon;
  /** 可选描述/副标题 */
  description?: string;
  /** 自定义 className */
  className?: string;
  /** 测试 ID */
  "data-testid"?: string;
}

/**
 * MetricCard 组件
 *
 * 统计指标展示卡片，包含图标、数值和单位
 */
export function MetricCard({
  title,
  value,
  unit,
  icon: Icon,
  description,
  className,
  "data-testid": testId,
}: MetricCardProps) {
  return (
    <div
      className={cn(
        "flex flex-col gap-2 p-4 rounded-lg",
        "bg-muted/50 border border-border/50",
        className
      )}
      data-testid={testId}
    >
      {/* 标题行 */}
      <div className="flex items-center justify-between">
        <span className="text-sm text-muted-foreground">{title}</span>
        {Icon && <Icon className="h-4 w-4 text-muted-foreground/70" />}
      </div>

      {/* 数值行 */}
      <div className="flex items-baseline gap-1">
        <span className="text-2xl font-semibold text-foreground">{value}</span>
        {unit && (
          <span className="text-sm text-muted-foreground">{unit}</span>
        )}
      </div>

      {/* 可选描述 */}
      {description && (
        <span className="text-xs text-muted-foreground">{description}</span>
      )}
    </div>
  );
}
