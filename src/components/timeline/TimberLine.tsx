/**
 * TimberLine - 时间轴控制器主组件
 * Story 2.6: AC #1, #3, #4, #5, #6, #7
 *
 * 功能:
 * - 水平时间轴覆盖整个会话时间范围
 * - 可拖拽滑块跳转到任意时间点
 * - Tick Marks 标记关键节点
 * - 键盘导航支持
 * - 主题兼容
 */

import React from "react";

import { cn } from "@/lib/utils";
import type {
    TimberLineProps,
    TimelineEvent,
} from "@/types/timeline";
import {
    timeToPosition,
    positionToTime,
    findNearestEvent,
} from "@/types/timeline";

import { TickMark } from "./TickMark";
import { TimeTooltip } from "./TimeTooltip";

/**
 * TimberLine 时间轴控制器
 */
export const TimberLine = React.memo(function TimberLine({
    startTime,
    endTime,
    currentTime,
    events,
    onSeek,
    onHover,
    className,
    disabled = false,
}: TimberLineProps) {
    // Refs
    const trackRef = React.useRef<HTMLDivElement>(null);

    // State
    const [isDragging, setIsDragging] = React.useState(false);
    const [hoverPosition, setHoverPosition] = React.useState<number | null>(null);
    const [showKnobTooltip, setShowKnobTooltip] = React.useState(false);

    // 计算当前位置百分比
    const currentPosition = React.useMemo(
        () => timeToPosition(currentTime, startTime, endTime),
        [currentTime, startTime, endTime]
    );

    // 计算悬停时间戳
    const hoverTimestamp = React.useMemo(() => {
        if (hoverPosition === null) return null;
        return positionToTime(hoverPosition, startTime, endTime);
    }, [hoverPosition, startTime, endTime]);

    // 缓存 Tick 位置
    const tickPositions = React.useMemo(() => {
        return events.map((event) => ({
            event,
            position: timeToPosition(event.timestamp, startTime, endTime),
        }));
    }, [events, startTime, endTime]);

    // ============================================
    // 拖拽处理
    // ============================================

    const updatePositionFromEvent = React.useCallback(
        (clientX: number) => {
            if (!trackRef.current || disabled) return;
            const rect = trackRef.current.getBoundingClientRect();
            const position = ((clientX - rect.left) / rect.width) * 100;
            const clampedPosition = Math.max(0, Math.min(100, position));
            const timestamp = positionToTime(clampedPosition, startTime, endTime);
            onSeek(timestamp);
        },
        [startTime, endTime, onSeek, disabled]
    );

    const handleMouseDown = React.useCallback(
        (e: React.MouseEvent) => {
            if (disabled) return;
            e.preventDefault();
            setIsDragging(true);
            setShowKnobTooltip(true);
            updatePositionFromEvent(e.clientX);
        },
        [disabled, updatePositionFromEvent]
    );

    const rafRef = React.useRef<number | null>(null);

    const handleMouseMove = React.useCallback(
        (e: MouseEvent) => {
            if (!isDragging) return;
            // 使用 requestAnimationFrame 节流优化性能
            if (rafRef.current !== null) {
                cancelAnimationFrame(rafRef.current);
            }
            rafRef.current = requestAnimationFrame(() => {
                updatePositionFromEvent(e.clientX);
                rafRef.current = null;
            });
        },
        [isDragging, updatePositionFromEvent]
    );

    const handleMouseUp = React.useCallback(() => {
        setIsDragging(false);
        setShowKnobTooltip(false);
    }, []);

    // ============================================
    // 触摸事件处理 (M3 修复)
    // ============================================

    const handleTouchStart = React.useCallback(
        (e: React.TouchEvent) => {
            if (disabled) return;
            e.preventDefault();
            setIsDragging(true);
            setShowKnobTooltip(true);
            if (e.touches.length > 0) {
                updatePositionFromEvent(e.touches[0].clientX);
            }
        },
        [disabled, updatePositionFromEvent]
    );

    const handleTouchMove = React.useCallback(
        (e: TouchEvent) => {
            if (!isDragging) return;
            if (e.touches.length > 0) {
                if (rafRef.current !== null) {
                    cancelAnimationFrame(rafRef.current);
                }
                rafRef.current = requestAnimationFrame(() => {
                    updatePositionFromEvent(e.touches[0].clientX);
                    rafRef.current = null;
                });
            }
        },
        [isDragging, updatePositionFromEvent]
    );

    const handleTouchEnd = React.useCallback(() => {
        setIsDragging(false);
        setShowKnobTooltip(false);
    }, []);

    // 全局鼠标和触摸事件监听
    React.useEffect(() => {
        if (isDragging) {
            window.addEventListener("mousemove", handleMouseMove);
            window.addEventListener("mouseup", handleMouseUp);
            window.addEventListener("touchmove", handleTouchMove, { passive: false });
            window.addEventListener("touchend", handleTouchEnd);
            return () => {
                window.removeEventListener("mousemove", handleMouseMove);
                window.removeEventListener("mouseup", handleMouseUp);
                window.removeEventListener("touchmove", handleTouchMove);
                window.removeEventListener("touchend", handleTouchEnd);
                // 清理 pending RAF
                if (rafRef.current !== null) {
                    cancelAnimationFrame(rafRef.current);
                    rafRef.current = null;
                }
            };
        }
    }, [isDragging, handleMouseMove, handleMouseUp, handleTouchMove, handleTouchEnd]);

    // ============================================
    // 悬停处理
    // ============================================

    const handleTrackMouseMove = React.useCallback(
        (e: React.MouseEvent) => {
            if (!trackRef.current || isDragging) return;
            const rect = trackRef.current.getBoundingClientRect();
            const position = ((e.clientX - rect.left) / rect.width) * 100;
            setHoverPosition(Math.max(0, Math.min(100, position)));
        },
        [isDragging]
    );

    const handleTrackMouseLeave = React.useCallback(() => {
        setHoverPosition(null);
        onHover?.(null);
    }, [onHover]);

    // ============================================
    // 键盘导航
    // ============================================

    const handleKeyDown = React.useCallback(
        (e: React.KeyboardEvent) => {
            if (disabled) return;

            const step = (endTime - startTime) * 0.01; // 1% 步进

            switch (e.key) {
                case "ArrowLeft": {
                    e.preventDefault();
                    // 尝试跳转到前一个事件，否则移动 1%
                    const prevEvent = findNearestEvent(events, currentTime, "prev");
                    if (prevEvent) {
                        onSeek(prevEvent.timestamp);
                    } else {
                        onSeek(Math.max(startTime, currentTime - step));
                    }
                    break;
                }
                case "ArrowRight": {
                    e.preventDefault();
                    // 尝试跳转到下一个事件，否则移动 1%
                    const nextEvent = findNearestEvent(events, currentTime, "next");
                    if (nextEvent) {
                        onSeek(nextEvent.timestamp);
                    } else {
                        onSeek(Math.min(endTime, currentTime + step));
                    }
                    break;
                }
                case "Home":
                    e.preventDefault();
                    onSeek(startTime);
                    break;
                case "End":
                    e.preventDefault();
                    onSeek(endTime);
                    break;
            }
        },
        [startTime, endTime, currentTime, events, onSeek, disabled]
    );

    // ============================================
    // Tick 点击处理
    // ============================================

    const handleTickClick = React.useCallback(
        (event: TimelineEvent) => {
            if (disabled) return;
            onSeek(event.timestamp);
        },
        [onSeek, disabled]
    );

    const handleTickHover = React.useCallback(
        (event: TimelineEvent | null) => {
            onHover?.(event?.timestamp ?? null);
        },
        [onHover]
    );

    return (
        <div
            className={cn(
                "relative w-full h-12 px-4 py-3",
                "bg-muted/50 border-t border-border",
                "select-none shrink-0",
                // Focus 样式 - 键盘导航时显示
                "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-inset",
                disabled && "opacity-50 cursor-not-allowed",
                className
            )}
            role="slider"
            aria-label="会话时间轴"
            aria-valuemin={startTime}
            aria-valuemax={endTime}
            aria-valuenow={currentTime}
            tabIndex={disabled ? -1 : 0}
            onKeyDown={handleKeyDown}
        >
            {/* 轨道 */}
            <div
                ref={trackRef}
                className={cn(
                    "relative h-1 w-full rounded-full",
                    "bg-muted",
                    !disabled && "cursor-pointer"
                )}
                onMouseDown={handleMouseDown}
                onTouchStart={handleTouchStart}
                onMouseMove={handleTrackMouseMove}
                onMouseLeave={handleTrackMouseLeave}
            >
                {/* 进度条 */}
                <div
                    className="absolute left-0 top-0 h-full rounded-full bg-primary pointer-events-none"
                    style={{ width: `${currentPosition}%` }}
                />

                {/* Tick Marks */}
                {tickPositions.map(({ event, position }, index) => (
                    <TickMark
                        key={`${event.type}-${event.timestamp}-${index}`}
                        event={event}
                        position={position}
                        isActive={event.timestamp === currentTime}
                        onClick={handleTickClick}
                        onHover={handleTickHover}
                    />
                ))}

                {/* 滑块 (Knob) */}
                <div
                    className={cn(
                        "absolute top-1/2 -translate-y-1/2 -translate-x-1/2",
                        "w-3.5 h-3.5 rounded-full",
                        "bg-background border-2 border-primary",
                        "shadow-md",
                        "transition-transform duration-100",
                        isDragging && "scale-125",
                        !disabled && "cursor-grab active:cursor-grabbing",
                        // Focus 样式
                        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
                    )}
                    style={{ left: `${currentPosition}%` }}
                    onMouseEnter={() => setShowKnobTooltip(true)}
                    onMouseLeave={() => !isDragging && setShowKnobTooltip(false)}
                >
                    {/* Knob Tooltip */}
                    <TimeTooltip
                        timestamp={currentTime}
                        visible={showKnobTooltip || isDragging}
                    />
                </div>

                {/* 悬停指示器 */}
                {hoverPosition !== null && !isDragging && hoverTimestamp !== null && (
                    <div
                        className="absolute top-0 bottom-0 w-px bg-muted-foreground/50 pointer-events-none"
                        style={{ left: `${hoverPosition}%` }}
                    >
                        <TimeTooltip
                            timestamp={hoverTimestamp}
                            visible
                        />
                    </div>
                )}
            </div>

            {/* 垂直播放头线 (延伸到内容区域) */}
            <div
                className="absolute top-0 w-px h-full bg-primary/30 pointer-events-none"
                style={{ left: `calc(${currentPosition}% + 1rem)` }}
            />
        </div>
    );
});

TimberLine.displayName = "TimberLine";

export default TimberLine;
