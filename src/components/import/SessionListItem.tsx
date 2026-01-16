/**
 * SessionListItem Component - 会话列表项
 * Story 2.9 UX Redesign
 * Story 2.26: 国际化支持
 *
 * 单个会话的列表项，显示文件名、大小、时间
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { FileJson } from "lucide-react";
import { Checkbox } from "@/components/ui";
import { cn } from "@/lib/utils";
import type { DiscoveredFile } from "@/components/import";

/** SessionListItem Props */
export interface SessionListItemProps {
    /** 会话文件信息 */
    session: DiscoveredFile;
    /** 是否选中 */
    selected: boolean;
    /** 切换选中状态 */
    onToggle: () => void;
    /** 是否禁用 (Story 2.20: 已导入项目的会话) */
    disabled?: boolean;
    /** 是否已导入 (Story 2.20 改进: 显示导入状态) */
    isImported?: boolean;
}

/**
 * 格式化文件大小
 */
function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * 格式化相对时间
 */
function formatRelativeTime(
    timestamp: number,
    locale: string,
    t: (key: string, options?: Record<string, unknown>) => string
): string {
    const now = Date.now();
    const diff = now - timestamp;

    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return t("time.justNow");
    if (minutes < 60) return t("time.minutesAgo", { count: minutes });
    if (hours < 24) return t("time.hoursAgo", { count: hours });
    if (days < 7) return t("time.daysAgo", { count: days });
    return new Date(timestamp).toLocaleDateString(locale);
}

/**
 * SessionListItem 组件
 */
export function SessionListItem({
    session,
    selected,
    onToggle,
    disabled = false,
    isImported = false,
}: SessionListItemProps) {
    const { t, i18n } = useTranslation();

    // 处理复选框点击，阻止事件冒泡
    const handleCheckboxClick = (e: React.MouseEvent) => {
        e.stopPropagation();
    };

    // 处理行点击
    const handleRowClick = () => {
        if (!disabled) {
            onToggle();
        }
    };

    return (
        <div
            data-testid={`session-item-${session.path}`}
            onClick={handleRowClick}
            className={cn(
                "flex items-center gap-3 px-3 py-2 pl-10 border-b border-border/30 last:border-b-0 transition-colors",
                disabled
                    ? "cursor-not-allowed opacity-50"
                    : "cursor-pointer hover:bg-muted/30"
            )}
        >
            {/* 复选框 */}
            <div onClick={handleCheckboxClick}>
                <Checkbox
                    data-testid={`session-checkbox-${session.path}`}
                    checked={selected}
                    disabled={disabled}
                    onCheckedChange={disabled ? undefined : onToggle}
                    aria-label={`选择会话 ${session.name}`}
                    className={disabled ? "cursor-not-allowed" : "cursor-pointer"}
                />
            </div>

            {/* 文件图标 */}
            <FileJson className="w-4 h-4 text-muted-foreground shrink-0" />

            {/* 文件信息 */}
            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                    <div className="text-sm text-foreground truncate" title={session.name}>{session.name}</div>
                    {isImported && (
                        <span className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded shrink-0">
                            {t("import.imported")}
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-2 mt-0.5">
                    {/* 仅当 size > 0 时显示文件大小（Cursor 会话没有大小信息） */}
                    {session.size > 0 && (
                        <>
                            <span className="text-xs text-muted-foreground">
                                {formatFileSize(session.size)}
                            </span>
                            <span className="text-xs text-muted-foreground">·</span>
                        </>
                    )}
                    <span className="text-xs text-muted-foreground">
                        {formatRelativeTime(session.modifiedAt, i18n.language, t)}
                    </span>
                </div>
            </div>
        </div>
    );
}
