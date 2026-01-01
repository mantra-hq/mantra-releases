/**
 * StatusBar - 底部状态栏组件
 * Story 2.14: Task 5 - AC #9, #12
 *
 * 功能:
 * - 底部状态栏布局
 * - 左侧: 分支选择器 + 同步状态
 * - 右侧: 光标位置 (Ln X, Col Y)
 */

import * as React from "react";
import { cn } from "@/lib/utils";

export interface CursorPosition {
    /** 行号 (1-indexed) */
    line: number;
    /** 列号 (1-indexed) */
    column: number;
}

export interface StatusBarProps {
    /** 光标位置 */
    cursorPosition?: CursorPosition;
    /** 左侧内容插槽 (分支选择器、同步状态等) */
    leftContent?: React.ReactNode;
    /** 自定义类名 */
    className?: string;
}

/**
 * 底部状态栏组件
 */
export function StatusBar({
    cursorPosition,
    leftContent,
    className,
}: StatusBarProps) {
    return (
        <div
            data-testid="status-bar"
            className={cn(
                "flex items-center justify-between px-3 py-1",
                "h-6 text-xs text-muted-foreground",
                "border-t border-border bg-muted/30",
                className
            )}
        >
            {/* 左侧内容区 */}
            <div className="flex items-center gap-3" data-testid="status-bar-left">
                {leftContent}
            </div>

            {/* 右侧光标位置 */}
            <div className="flex items-center gap-2" data-testid="status-bar-right">
                {cursorPosition && (
                    <span data-testid="cursor-position">
                        Ln {cursorPosition.line}, Col {cursorPosition.column}
                    </span>
                )}
            </div>
        </div>
    );
}

export default StatusBar;
