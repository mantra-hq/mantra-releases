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
 * 根据位置获取 Tooltip 对齐方式
 * 边界智能定位：避免 Tooltip 超出屏幕
 */
function getAlignmentClasses(position?: number): string {
    if (position === undefined) {
        // 默认居中
        return "left-1/2 -translate-x-1/2";
    }

    if (position < 15) {
        // 左边缘：左对齐
        return "left-0";
    }

    if (position > 85) {
        // 右边缘：右对齐
        return "right-0";
    }

    // 中间：居中
    return "left-1/2 -translate-x-1/2";
}

/**
 * TimeTooltip 组件
 * 显示时间戳信息的浮动提示
 * Story 2.32: AC2 - 详细信息显示 + 边界智能定位
 */
export const TimeTooltip = React.memo(function TimeTooltip({
    timestamp,
    visible,
    label,
    commitHash,
    style,
    position,
}: TimeTooltipProps) {
    const { i18n } = useTranslation();

    if (!visible) return null;

    // 短格式 commit hash (前 7 位)
    const shortHash = commitHash ? commitHash.slice(0, 7) : null;
    // 是否是 Git 提交
    const isGitCommit = !!shortHash;
    // 完整时间格式
    const fullTime = formatTimestamp(timestamp, i18n.language);
    // 根据位置获取对齐方式
    const alignmentClasses = getAlignmentClasses(position);

    return (
        <div
            className={cn(
                "absolute bottom-full",
                alignmentClasses,
                "mb-2 px-3 py-2",
                "bg-zinc-900 text-zinc-100",
                "rounded-lg shadow-lg",
                "pointer-events-none",
                "z-50",
                "text-xs",
                // 动画
                "animate-in fade-in-0 zoom-in-95 duration-75"
            )}
            style={style}
        >
            {isGitCommit ? (
                // Git 提交: 多行显示 - hash + 消息 + 时间
                <div className="flex flex-col gap-1">
                    <div className="flex items-center gap-2 whitespace-nowrap">
                        <span className="text-emerald-400 font-mono font-medium">{shortHash}</span>
                        <span className="text-zinc-500">{fullTime}</span>
                    </div>
                    {label && (
                        <div className="text-zinc-300 truncate max-w-[200px]" title={label}>
                            {label}
                        </div>
                    )}
                </div>
            ) : (
                // 普通事件: 类型 + 详细时间 (单行不折行)
                <div className="flex items-center gap-2 whitespace-nowrap">
                    <span className="text-zinc-300 font-medium">{label}</span>
                    <span className="text-zinc-500">{fullTime}</span>
                </div>
            )}
        </div>
    );
});

TimeTooltip.displayName = "TimeTooltip";

export default TimeTooltip;
