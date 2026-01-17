/**
 * AnalyticsEmptyState Component - 统计空状态组件
 * Story 2.34: Task 9
 *
 * 显示统计数据为空时的状态：
 * - no-data: 项目没有任何会话
 * - no-data-in-range: 所选时间范围内没有数据
 */

import { useTranslation } from "react-i18next";
import { Calendar, FolderOpen, Rocket } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/**
 * 空状态类型
 */
export type EmptyStateType = "no-data" | "no-data-in-range";

/**
 * AnalyticsEmptyState Props
 */
export interface AnalyticsEmptyStateProps {
  /** 空状态类型 */
  type: EmptyStateType;
  /** 打开导入向导回调 (no-data 时使用) */
  onImport?: () => void;
  /** 切换时间范围回调 (no-data-in-range 时使用) */
  onChangeTimeRange?: () => void;
  /** 自定义 className */
  className?: string;
}

/**
 * AnalyticsEmptyState 组件
 *
 * 根据不同空状态类型显示对应的引导信息
 */
export function AnalyticsEmptyState({
  type,
  onImport,
  onChangeTimeRange,
  className,
}: AnalyticsEmptyStateProps) {
  const { t } = useTranslation();

  // 无数据状态（项目没有任何会话）
  if (type === "no-data") {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center gap-6 p-6 h-full",
          className
        )}
        data-testid="analytics-empty-no-data"
      >
        {/* 图标 */}
        <div
          className={cn(
            "w-24 h-24",
            "flex items-center justify-center",
            "rounded-2xl",
            "bg-muted/50"
          )}
        >
          <FolderOpen className="w-12 h-12 text-muted-foreground/70" />
        </div>

        {/* 主标题 */}
        <div className="text-center">
          <h3 className="text-xl font-semibold text-foreground mb-2">
            {t("analytics.empty.noData")}
          </h3>
          <p className="text-sm text-muted-foreground max-w-md">
            {t("analytics.empty.importHint")}
          </p>
        </div>

        {/* CTA 按钮 */}
        {onImport && (
          <Button onClick={onImport} size="lg" className="gap-2">
            <Rocket className="w-4 h-4" />
            {t("import.importSession")}
          </Button>
        )}
      </div>
    );
  }

  // 时间范围内无数据状态
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-6 p-6 h-[300px]",
        className
      )}
      data-testid="analytics-empty-no-range-data"
    >
      {/* 图标 */}
      <div
        className={cn(
          "w-20 h-20",
          "flex items-center justify-center",
          "rounded-2xl",
          "bg-muted/50"
        )}
      >
        <Calendar className="w-10 h-10 text-muted-foreground/70" />
      </div>

      {/* 提示文本 */}
      <div className="text-center">
        <h3 className="text-lg font-medium text-foreground mb-2">
          {t("analytics.empty.noDataInRange")}
        </h3>
        <p className="text-sm text-muted-foreground max-w-md">
          {t("analytics.empty.tryLongerRange")}
        </p>
      </div>

      {/* 切换时间范围按钮 */}
      {onChangeTimeRange && (
        <Button onClick={onChangeTimeRange} variant="outline" size="default">
          {t("analytics.timeRange.all")}
        </Button>
      )}
    </div>
  );
}
