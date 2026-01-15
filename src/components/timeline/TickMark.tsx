/**
 * TickMark - 时间轴刻度标记组件
 * Story 2.6: AC #2
 * Story 2.26: 国际化支持
 *
 * 在时间轴上标记关键节点:
 * - user-message: 蓝色圆点
 * - git-commit: 绿色方块
 * - ai-response: 浅色小圆点
 */

import React from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";
import type { TickMarkProps, TimelineEvent } from "@/types/timeline";

import { TimeTooltip } from "./TimeTooltip";

/**
 * 获取 TickMark 的样式类
 */
function getTickMarkStyles(type: TimelineEvent["type"]): string {
    const baseStyles = "absolute top-1/2 -translate-x-1/2 -translate-y-1/2 z-10";

    switch (type) {
        case "user-message":
            return cn(
                baseStyles,
                "w-2 h-2 rounded-full",
                "bg-blue-500 dark:bg-blue-400",
                "hover:scale-150 hover:ring-2 hover:ring-blue-500/30",
                "transition-transform duration-100"
            );
        case "git-commit":
            return cn(
                baseStyles,
                "w-2 h-2 rounded-sm",
                "bg-emerald-500 dark:bg-emerald-400",
                "hover:scale-150 hover:ring-2 hover:ring-emerald-500/30",
                "transition-transform duration-100"
            );
        case "ai-response":
            return cn(
                baseStyles,
                "w-1 h-1 rounded-full",
                "bg-muted-foreground/30",
                "hover:scale-200 hover:bg-muted-foreground/50",
                "transition-all duration-100"
            );
        default:
            return baseStyles;
    }
}

/**
 * TickMark 组件
 * 在时间轴上显示事件标记
 */
function TickMarkComponent({
    event,
    position,
    isActive,
    onClick,
    onHover,
}: TickMarkProps) {
    const { t } = useTranslation();
    const [showTooltip, setShowTooltip] = React.useState(false);

    // 根据事件类型获取翻译后的标签
    const getEventLabel = React.useCallback((type: TimelineEvent["type"]): string => {
        switch (type) {
            case "user-message":
                return t("timeline.userMessage");
            case "ai-response":
                return t("timeline.aiResponse");
            case "git-commit":
                return t("timeline.gitCommit");
            default:
                return type;
        }
    }, [t]);

    const eventLabel = event.label || getEventLabel(event.type);

    const handleMouseEnter = React.useCallback(() => {
        setShowTooltip(true);
        onHover?.(event);
    }, [event, onHover]);

    const handleMouseLeave = React.useCallback(() => {
        setShowTooltip(false);
        onHover?.(null);
    }, [onHover]);

    const handleClick = React.useCallback(() => {
        onClick?.(event);
    }, [event, onClick]);

    return (
        <div
            className="absolute top-0 bottom-0"
            style={{ left: `${position}%` }}
        >
            {/* Tick Mark */}
            <button
                type="button"
                className={cn(
                    getTickMarkStyles(event.type),
                    isActive && "ring-2 ring-primary scale-150",
                    onClick && "cursor-pointer"
                )}
                onClick={handleClick}
                onMouseEnter={handleMouseEnter}
                onMouseLeave={handleMouseLeave}
                aria-label={eventLabel}
            />

            {/* Tooltip - Story 2.32: 传递 commitHash 和 position 实现智能定位 */}
            <TimeTooltip
                timestamp={event.timestamp}
                visible={showTooltip}
                label={eventLabel}
                commitHash={event.commitHash}
                position={position}
            />
        </div>
    );
}

/**
 * 自定义 memo 比较函数 - 仅比较影响渲染的 props
 * 忽略回调函数的引用变化以优化性能
 */
function areTickMarkPropsEqual(prevProps: TickMarkProps, nextProps: TickMarkProps): boolean {
    return (
        prevProps.event.timestamp === nextProps.event.timestamp &&
        prevProps.event.type === nextProps.event.type &&
        prevProps.position === nextProps.position &&
        prevProps.isActive === nextProps.isActive
    );
}

export const TickMark = React.memo(TickMarkComponent, areTickMarkPropsEqual);

TickMark.displayName = "TickMark";

export default TickMark;
