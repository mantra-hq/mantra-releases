/**
 * Breadcrumbs - 面包屑导航组件
 * Story 2.13: Task 3 - AC #6, #7, #20
 *
 * 功能:
 * - 显示文件路径分段 (src > components > editor > CodeSnapshotView.tsx)
 * - 点击路径段弹出下拉菜单导航
 * - 历史模式时间戳指示器
 */

import * as React from "react";
import { ChevronRight, Clock } from "lucide-react";
import { cn } from "@/lib/utils";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";

/** 同级文件/目录信息 */
export interface SiblingItem {
    /** 文件/目录名 */
    name: string;
    /** 完整路径 */
    path: string;
    /** 是否为目录 */
    isDirectory: boolean;
}

/** 历史信息 (UX 优化: 合并显示) */
export interface HistoryInfo {
    /** 时间戳 (Unix ms) */
    timestamp: number;
    /** Commit Hash */
    commitHash?: string;
    /** Commit 消息 */
    commitMessage?: string;
}

export interface BreadcrumbsProps {
    /** 文件路径 */
    filePath: string;
    /** 同级文件列表 (用于导航下拉) */
    siblings?: SiblingItem[];
    /** 历史模式时间戳 (Unix ms) - 向后兼容 */
    timestamp?: number;
    /** 历史信息 (UX 优化: 合并显示 commit 信息) */
    historyInfo?: HistoryInfo;
    /** 是否隐藏文件名 (已在标签页显示时设为 true) */
    hideFileName?: boolean;
    /** 点击路径段回调 */
    onNavigate?: (path: string) => void;
    /** 自定义类名 */
    className?: string;
}

/**
 * 面包屑导航组件
 */
export function Breadcrumbs({
    filePath,
    siblings = [],
    timestamp,
    historyInfo,
    hideFileName = false,
    onNavigate,
    className,
}: BreadcrumbsProps) {
    // 历史信息优先使用 historyInfo，否则回退到 timestamp
    const effectiveHistoryInfo = historyInfo || (timestamp ? { timestamp } : undefined);
    const segments = React.useMemo(() => {
        if (!filePath) return [];
        const allSegments = filePath.split("/").filter(Boolean);
        // UX 优化: 隐藏最后一段（文件名），因为已在标签页显示
        return hideFileName && allSegments.length > 1
            ? allSegments.slice(0, -1)
            : allSegments;
    }, [filePath, hideFileName]);

    // 预计算每个路径段的同级项 (优化渲染性能和 UX)
    const segmentSiblings = React.useMemo(() => {
        return segments.map((_, index) => {
            const parentPath = segments.slice(0, index).join("/");
            return siblings.filter((s) => {
                const itemParent = s.path.split("/").slice(0, -1).join("/");
                return itemParent === parentPath;
            });
        });
    }, [segments, siblings]);

    const handleSegmentClick = React.useCallback(
        (index: number) => {
            if (!onNavigate) return;
            const path = segments.slice(0, index + 1).join("/");
            onNavigate(path);
        },
        [onNavigate, segments]
    );

    if (segments.length === 0) return null;

    return (
        <div
            className={cn(
                "flex items-center gap-1 px-3 py-1 text-sm text-muted-foreground",
                "border-b border-border bg-muted/20",
                className
            )}
        >
            {segments.map((segment, index) => {
                const hasSiblings = segmentSiblings[index].length > 0;
                const isLast = index === segments.length - 1;

                return (
                    <React.Fragment key={index}>
                        {index > 0 && (
                            <ChevronRight
                                className="h-3 w-3 text-muted-foreground/50"
                                data-testid="breadcrumb-separator"
                            />
                        )}
                        {/* 有同级项时显示下拉菜单，否则只显示文本 */}
                        {hasSiblings ? (
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <button
                                        className={cn(
                                            "hover:text-foreground hover:underline transition-colors",
                                            isLast && "text-foreground font-medium"
                                        )}
                                    >
                                        {segment}
                                    </button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align="start" className="max-h-[300px] overflow-y-auto">
                                    {segmentSiblings[index].map((sibling) => (
                                        <DropdownMenuItem
                                            key={sibling.path}
                                            onClick={() => onNavigate?.(sibling.path)}
                                            className={cn(
                                                sibling.path === filePath && "bg-accent"
                                            )}
                                        >
                                            {sibling.isDirectory ? "📁" : "📄"} {sibling.name}
                                        </DropdownMenuItem>
                                    ))}
                                </DropdownMenuContent>
                            </DropdownMenu>
                        ) : (
                            <button
                                onClick={() => handleSegmentClick(index)}
                                className={cn(
                                    "hover:text-foreground transition-colors",
                                    isLast && "text-foreground font-medium cursor-default"
                                )}
                            >
                                {segment}
                            </button>
                        )}
                    </React.Fragment>
                );
            })}

            {/* 历史模式时间戳指示器 (UX 优化: 合并显示 commit 信息) */}
            {effectiveHistoryInfo && (
                <div className="ml-auto flex items-center gap-1.5 text-xs text-amber-500">
                    <Clock className="h-3 w-3" />
                    <span className="flex items-center gap-1">
                        {effectiveHistoryInfo.commitHash && (
                            <span className="font-mono opacity-80">
                                {effectiveHistoryInfo.commitHash.slice(0, 7)}
                            </span>
                        )}
                        {effectiveHistoryInfo.commitHash && <span>·</span>}
                        {formatDistanceToNow(new Date(effectiveHistoryInfo.timestamp), {
                            addSuffix: true,
                            locale: zhCN,
                        })}
                    </span>
                </div>
            )}
        </div>
    );
}

export default Breadcrumbs;

