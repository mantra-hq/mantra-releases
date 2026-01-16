/**
 * ToolCallDetailsList Component - 工具调用详情列表
 * Story 2.34: Task 8.4
 *
 * 显示会话中工具调用的详细列表
 */

import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { format } from "date-fns";
import { zhCN, enUS } from "date-fns/locale";
import { CheckCircle2, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ToolCallDetail } from "@/types/analytics";
import { ScrollArea } from "@/components/ui/scroll-area";

/**
 * ToolCallDetailsList Props
 */
export interface ToolCallDetailsListProps {
  /** 工具调用详情列表 */
  data: ToolCallDetail[];
  /** 最大显示条目数 */
  maxItems?: number;
  /** 自定义 className */
  className?: string;
}

/**
 * ToolCallDetailsList 组件
 *
 * 表格形式展示工具调用详情
 */
export function ToolCallDetailsList({
  data,
  maxItems = 100,
  className,
}: ToolCallDetailsListProps) {
  const { t, i18n } = useTranslation();

  // 格式化数据
  const listData = useMemo(() => {
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
        data-testid="tool-details-empty"
      >
        {t("common.noData")}
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col gap-4", className)} data-testid="tool-call-details-list">
      {/* 标题 */}
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-foreground">
          {t("analytics.charts.callDetails")}
        </h3>
        <span className="text-xs text-muted-foreground">
          {data.length > maxItems
            ? `${maxItems}/${data.length}`
            : data.length}{" "}
          {t("analytics.units.calls")}
        </span>
      </div>

      {/* 列表 */}
      <ScrollArea className="h-[280px]">
        <div className="space-y-1">
          {listData.map((item, index) => (
            <div
              key={`${item.timestamp}-${index}`}
              className={cn(
                "flex items-center gap-3 px-3 py-2 rounded-md",
                "hover:bg-muted/50 transition-colors",
                item.is_error && "bg-destructive/5"
              )}
            >
              {/* 状态图标 */}
              {item.is_error ? (
                <XCircle className="w-4 h-4 text-destructive flex-shrink-0" />
              ) : (
                <CheckCircle2 className="w-4 h-4 text-emerald-500 flex-shrink-0" />
              )}

              {/* 工具名称 */}
              <span
                className={cn(
                  "text-sm font-medium min-w-[100px]",
                  item.is_error ? "text-destructive" : "text-foreground"
                )}
              >
                {item.displayName}
              </span>

              {/* 描述 */}
              <span className="text-xs text-muted-foreground flex-1 truncate">
                {item.description || "-"}
              </span>

              {/* 时间 */}
              <span className="text-xs text-muted-foreground flex-shrink-0">
                {item.timeLabel}
              </span>
            </div>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}
