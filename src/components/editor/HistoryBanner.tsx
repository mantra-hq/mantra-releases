/**
 * HistoryBanner - 历史状态 Banner
 * Story 2.7: Task 4 - AC #6
 *
 * 功能:
 * - 显示当前查看的历史时间戳
 * - 显示 Commit 信息 (如有)
 * - 提供"返回当前"按钮
 * - 使用警告色强调历史状态
 */

import { cn } from "@/lib/utils";
import { History, GitCommit, ArrowLeft, Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { format } from "date-fns";
import { zhCN } from "date-fns/locale";

export interface HistoryBannerProps {
    /** 当前查看的时间戳 (Unix ms) */
    timestamp: number;
    /** Commit Hash (短格式) */
    commitHash?: string;
    /** Commit 消息 */
    commitMessage?: string;
    /** 返回当前回调 */
    onReturnToCurrent: () => void;
    /** 自定义 className */
    className?: string;
}

/**
 * 格式化时间戳为可读格式
 */
function formatTimestamp(timestamp: number): string {
    try {
        return format(new Date(timestamp), "yyyy-MM-dd HH:mm:ss", {
            locale: zhCN,
        });
    } catch {
        return "未知时间";
    }
}

/**
 * 截断 Commit 消息
 */
function truncateMessage(message: string | undefined, maxLength = 50): string {
    if (!message) return "";
    if (message.length <= maxLength) return message;
    return message.slice(0, maxLength) + "...";
}

/**
 * 历史状态 Banner 组件
 */
export function HistoryBanner({
    timestamp,
    commitHash,
    commitMessage,
    onReturnToCurrent,
    className,
}: HistoryBannerProps) {
    const formattedTime = formatTimestamp(timestamp);
    const shortHash = commitHash?.slice(0, 7);
    const truncatedMessage = truncateMessage(commitMessage);

    return (
        <div
            className={cn(
                // 基础布局
                "flex items-center justify-between",
                "px-3 py-2",
                // 警告色背景
                "bg-blue-500/10 dark:bg-blue-500/15",
                "border-b border-blue-500/30 dark:border-blue-500/40",
                // 动画
                "animate-fade-in",
                className
            )}
            role="banner"
            aria-label="历史模式提示"
        >
            {/* 左侧信息 */}
            <div className="flex items-center gap-3 min-w-0 flex-1">
                {/* 图标 */}
                <div className="flex items-center justify-center size-7 rounded-full bg-blue-500/20 dark:bg-blue-500/30 flex-shrink-0">
                    <History className="size-4 text-blue-600 dark:text-blue-400" />
                </div>

                {/* 文本信息 */}
                <div className="flex flex-col min-w-0">
                    {/* 第一行: 时间信息 */}
                    <div className="flex items-center gap-2 text-sm">
                        <Clock className="size-3.5 text-muted-foreground" />
                        <span className="text-foreground font-medium">查看历史状态:</span>
                        <span className="font-mono text-muted-foreground">
                            {formattedTime}
                        </span>
                    </div>

                    {/* 第二行: Commit 信息 (可选) */}
                    {shortHash && (
                        <div className="flex items-center gap-2 text-xs mt-0.5">
                            <GitCommit className="size-3 text-emerald-500" />
                            <span className="font-mono text-emerald-600 dark:text-emerald-400">
                                {shortHash}
                            </span>
                            {truncatedMessage && (
                                <span className="text-muted-foreground truncate">
                                    - {truncatedMessage}
                                </span>
                            )}
                        </div>
                    )}
                </div>
            </div>

            {/* 右侧: 返回按钮 */}
            <Button
                variant="ghost"
                size="sm"
                onClick={onReturnToCurrent}
                className={cn(
                    "h-8 px-3 flex-shrink-0",
                    "text-blue-600 dark:text-blue-400",
                    "hover:bg-blue-500/20 dark:hover:bg-blue-500/30",
                    "hover:text-blue-700 dark:hover:text-blue-300"
                )}
            >
                <ArrowLeft className="size-4 mr-1" />
                <span>返回当前</span>
            </Button>
        </div>
    );
}

HistoryBanner.displayName = "HistoryBanner";

export default HistoryBanner;
