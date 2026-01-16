/**
 * ToolCallTimeline Component - 工具调用时间线
 * Story 2.34: Task 8.3
 *
 * 显示会话中工具调用的时间线
 */

import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { format } from "date-fns";
import { zhCN, enUS } from "date-fns/locale";
import { CheckCircle, XCircle, Circle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ToolCallDetail } from "@/types/analytics";
import { ScrollArea } from "@/components/ui/scroll-area";

/**
 * ToolCallTimeline Props
 */
export interface ToolCallTimelineProps {
  /** 工具调用详情列表 */
  data: ToolCallDetail[];
  /** 最大显示条目数 */
  maxItems?: number;
  /** 自定义 className */
  className?: string;
}

/**
 * ToolCallTimeline 组件
 *
 * 垂直时间线展示工具调用历史
 */
export function ToolCallTimeline({
  data,
  maxItems = 50,
  className,
}: ToolCallTimelineProps) {
  const { t, i18n } = useTranslation();

  // 格式化并限制数量
  const timelineData = useMemo(() => {
    const dateLocale = i18n.language === "zh-CN" ? zhCN : enUS;
    return data.slice(0, maxItems).map((item) => ({
      ...item,
      timeLabel: format(new Date(item.timestamp * 1000), "HH:mm:ss", {
        locale: dateLocale,
      }),
      displayName: t(`analytics.tools.${item.tool_type}`, item.tool_type),
    }));
  }, [data, maxItems, i18n.language, t]);

  if (data.length === 0) {
    return (
      <div
        className={cn(
          "flex items-center justify-center h-[200px] text-muted-foreground text-sm",
          className
        )}
        data-testid="tool-timeline-empty"
      >
        {t("common.noData")}
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col gap-4", className)} data-testid="tool-call-timeline">
      {/* 标题 */}
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-foreground">
          {t("analytics.charts.callTimeline")}
        </h3>
        <span className="text-xs text-muted-foreground">
          {data.length > maxItems
            ? `${maxItems}/${data.length}`
            : data.length}{" "}
          {t("analytics.units.calls")}
        </span>
      </div>

      {/* 时间线 */}
      <ScrollArea className="h-[280px]">
        <div className="relative pl-6">
          {/* 垂直线 */}
          <div className="absolute left-2 top-0 bottom-0 w-px bg-border" />

          {/* 时间线项 */}
          <div className="space-y-3">
            {timelineData.map((item, index) => (
              <div
                key={`${item.timestamp}-${index}`}
                className="relative flex items-start gap-3"
              >
                {/* 节点图标 */}
                <div className="absolute left-[-20px] flex items-center justify-center w-4 h-4 bg-background">
                  {item.is_error ? (
                    <XCircle className="w-4 h-4 text-destructive" />
                  ) : (
                    <Circle className="w-3 h-3 text-primary fill-primary" />
                  )}
                </div>

                {/* 内容 */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span
                      className={cn(
                        "text-sm font-medium",
                        item.is_error ? "text-destructive" : "text-foreground"
                      )}
                    >
                      {item.displayName}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {item.timeLabel}
                    </span>
                  </div>
                  {item.description && (
                    <p className="text-xs text-muted-foreground truncate mt-0.5">
                      {item.description}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
