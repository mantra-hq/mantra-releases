/**
 * FloatingCollapseBar - 浮动折叠栏组件
 * Story 2.15: Task 7
 *
 * 滚动离开顶部时显示浮动操作栏
 * AC: #11, #12, #13
 */

import { ChevronUp, ArrowUp } from "lucide-react";
import { cn } from "@/lib/utils";

export interface FloatingCollapseBarProps {
    /** 是否可见 */
    visible: boolean;
    /** 折叠回调 */
    onCollapse: () => void;
    /** 回到顶部回调 */
    onScrollToTop?: () => void;
    /** 自定义 className */
    className?: string;
}

/**
 * FloatingCollapseBar 组件
 *
 * 长内容展开后，当用户滚动离开顶部时显示浮动操作栏：
 * - 回到顶部按钮
 * - 收起按钮
 */
export function FloatingCollapseBar({
    visible,
    onCollapse,
    onScrollToTop,
    className,
}: FloatingCollapseBarProps) {
    if (!visible) return null;

    return (
        <div
            data-testid="floating-collapse-bar"
            className={cn(
                "fixed bottom-20 left-1/2 -translate-x-1/2",
                "flex items-center gap-2",
                "px-4 py-2 rounded-lg",
                "bg-card/95 backdrop-blur-sm",
                "border border-border shadow-lg",
                "z-50",
                "animate-in slide-in-from-bottom-4 duration-200",
                className
            )}
        >
            {/* 回到顶部 */}
            {onScrollToTop && (
                <button
                    type="button"
                    onClick={onScrollToTop}
                    className={cn(
                        "flex items-center gap-1.5 px-3 py-1.5 rounded",
                        "text-sm text-muted-foreground",
                        "hover:text-foreground hover:bg-muted/50",
                        "transition-colors"
                    )}
                >
                    <ArrowUp className="h-4 w-4" />
                    回到顶部
                </button>
            )}

            {/* 分隔线 */}
            {onScrollToTop && (
                <div className="w-px h-5 bg-border" />
            )}

            {/* 收起 */}
            <button
                type="button"
                onClick={onCollapse}
                className={cn(
                    "flex items-center gap-1.5 px-3 py-1.5 rounded",
                    "text-sm",
                    "bg-primary text-primary-foreground",
                    "hover:bg-primary/90",
                    "transition-colors"
                )}
            >
                <ChevronUp className="h-4 w-4" />
                收起
            </button>
        </div>
    );
}

export default FloatingCollapseBar;
