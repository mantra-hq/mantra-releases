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

export interface BreadcrumbsProps {
    /** 文件路径 */
    filePath: string;
    /** 同级文件列表 (用于导航下拉) */
    siblings?: SiblingItem[];
    /** 历史模式时间戳 (Unix ms) */
    timestamp?: number;
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
    onNavigate,
    className,
}: BreadcrumbsProps) {
    const segments = React.useMemo(() => {
        if (!filePath) return [];
        return filePath.split("/").filter(Boolean);
    }, [filePath]);

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

            {/* 历史模式时间戳指示器 (AC #20) */}
            {timestamp && (
                <div className="ml-auto flex items-center gap-1 text-xs text-amber-500">
                    <Clock className="h-3 w-3" />
                    <span>
                        历史 ·{" "}
                        {formatDistanceToNow(new Date(timestamp), {
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

