/**
 * RecentSessions - 最近会话组件
 * Story 2.10: Task 7.2
 * Story 2.26: 国际化支持
 *
 * 输入为空时显示最近访问的会话列表
 */

import { Clock } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { RecentSession } from "@/stores/useSearchStore";
import { cn, formatSessionName } from "@/lib/utils";

/**
 * RecentSessions Props
 */
export interface RecentSessionsProps {
    /** 最近会话列表 */
    sessions: RecentSession[];
    /** 当前选中的索引 */
    selectedIndex: number;
    /** 选择会话回调 */
    onSelect: (session: RecentSession) => void;
    /** hover 时更新选中索引 */
    onHover?: (index: number) => void;
    /** 自定义类名 */
    className?: string;
}

/**
 * 格式化访问时间
 */
function formatAccessTime(
    timestamp: number,
    locale: string,
    t: (key: string, options?: Record<string, unknown>) => string
): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMinutes < 1) {
        return t("time.justNow");
    } else if (diffMinutes < 60) {
        return t("time.minutesAgo", { count: diffMinutes });
    } else if (diffHours < 24) {
        return t("time.hoursAgo", { count: diffHours });
    } else if (diffDays === 1) {
        return t("time.yesterday");
    } else if (diffDays < 7) {
        return t("time.daysAgo", { count: diffDays });
    } else {
        return date.toLocaleDateString(locale, {
            month: "short",
            day: "numeric",
        });
    }
}

/**
 * RecentSessions 组件
 */
export function RecentSessions({
    sessions,
    selectedIndex,
    onSelect,
    onHover,
    className,
}: RecentSessionsProps) {
    const { t, i18n } = useTranslation();

    if (sessions.length === 0) {
        return (
            <div className={cn("py-12 px-4 text-center", className)}>
                <Clock className="w-10 h-10 text-muted-foreground/50 mx-auto mb-3" />
                <p className="text-sm text-muted-foreground">
                    {t("search.noRecentSessions")}
                </p>
                <p className="text-xs text-muted-foreground/70 mt-1">
                    {t("search.recentSessionsHint")}
                </p>
            </div>
        );
    }

    return (
        <div className={className}>
            {/* 标题 */}
            <div className="px-4 py-2 text-xs font-medium text-muted-foreground bg-muted/30">
                {t("search.recentAccess")}
            </div>

            {/* 会话列表 */}
            <div role="listbox" aria-label={t("search.recentSessions")}>
                {sessions.map((session, index) => {
                    // 格式化会话名称
                    const displaySessionName = formatSessionName(session.sessionId, session.sessionName);

                    return (
                        <div
                            key={session.sessionId}
                            role="option"
                            aria-selected={index === selectedIndex}
                            onClick={() => onSelect(session)}
                            onMouseEnter={() => onHover?.(index)}
                            className={cn(
                                "flex items-center gap-3 px-4 py-3 cursor-pointer transition-colors duration-150",
                                index === selectedIndex
                                    ? "bg-primary/10"
                                    : "hover:bg-accent"
                            )}
                        >
                            <Clock className="w-4 h-4 text-muted-foreground shrink-0" />
                            <div className="flex-1 min-w-0">
                                <div className="flex items-center gap-2 text-sm">
                                    <span className="text-primary font-medium truncate max-w-[180px]">
                                        {session.projectName}
                                    </span>
                                    <span className="text-muted-foreground">/</span>
                                    <span className="text-foreground truncate flex-1" title={displaySessionName}>
                                        {displaySessionName}
                                    </span>
                                </div>
                            </div>
                            <span className="text-xs text-muted-foreground shrink-0">
                                {formatAccessTime(session.accessedAt, i18n.language, t)}
                            </span>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

export default RecentSessions;
