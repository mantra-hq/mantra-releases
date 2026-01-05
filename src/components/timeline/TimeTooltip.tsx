/**
 * TimeTooltip - 时间戳提示组件
 * Story 2.6: AC #5
 * Story 2.26: 国际化支持
 */

import { format, isToday } from "date-fns";
import { zhCN, enUS } from "date-fns/locale";
import React from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";
import type { TimeTooltipProps } from "@/types/timeline";

/**
 * 格式化时间戳为可读字符串
 * 当天显示 HH:mm:ss，否则显示完整日期时间
 */
function formatTimestamp(timestamp: number, language: string): string {
    const date = new Date(timestamp);
    const locale = language === "zh-CN" ? zhCN : enUS;

    if (isToday(date)) {
        return format(date, "HH:mm:ss", { locale });
    }

    return format(date, "yyyy-MM-dd HH:mm:ss", { locale });
}

/**
 * TimeTooltip 组件
 * 显示时间戳信息的浮动提示
 */
export const TimeTooltip = React.memo(function TimeTooltip({
    timestamp,
    visible,
    label,
    style,
}: TimeTooltipProps) {
    const { i18n } = useTranslation();

    if (!visible) return null;

    const formattedTime = formatTimestamp(timestamp, i18n.language);

    return (
        <div
            className={cn(
                "absolute bottom-full left-1/2 -translate-x-1/2",
                "px-2 py-1 mb-2",
                "bg-popover text-popover-foreground",
                "border border-border rounded",
                "text-xs font-mono whitespace-nowrap",
                "pointer-events-none",
                "shadow-md",
                "z-50",
                // 动画
                "animate-in fade-in-0 zoom-in-95 duration-100"
            )}
            style={style}
        >
            <span>{formattedTime}</span>
            {label && (
                <span className="ml-2 text-muted-foreground">
                    {label}
                </span>
            )}
        </div>
    );
});

TimeTooltip.displayName = "TimeTooltip";

export default TimeTooltip;
