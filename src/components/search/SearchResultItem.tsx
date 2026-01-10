/**
 * SearchResultItem - 搜索结果项组件
 * Story 2.10: Task 3.2
 * Story 2.26: 国际化支持
 *
 * 显示单个搜索结果项，包含项目名、会话名和匹配片段
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { cn, formatSessionName } from "@/lib/utils";
import type { SearchResult } from "@/stores/useSearchStore";

/**
 * SearchResultItem Props
 */
export interface SearchResultItemProps {
    /** 搜索结果 */
    result: SearchResult;
    /** 是否选中 */
    isSelected: boolean;
    /** 点击回调 */
    onClick: () => void;
    /** 鼠标进入回调 (用于更新 hover 状态) */
    onMouseEnter?: () => void;
}

/**
 * 高亮文本的渲染函数
 * 根据 highlightRanges 将匹配的部分高亮显示
 */
function renderHighlightedText(
    text: string,
    ranges: Array<[number, number]>
): React.ReactNode {
    if (!ranges || ranges.length === 0) {
        return text;
    }

    // 排序 ranges
    const sortedRanges = [...ranges].sort((a, b) => a[0] - b[0]);
    const parts: React.ReactNode[] = [];
    let lastIndex = 0;

    sortedRanges.forEach(([start, end], i) => {
        // 添加高亮前的文本
        if (start > lastIndex) {
            parts.push(
                <span key={`text-${i}`}>{text.slice(lastIndex, start)}</span>
            );
        }
        // 添加高亮文本
        parts.push(
            <span
                key={`highlight-${i}`}
                className="bg-primary/20 text-primary rounded px-0.5"
            >
                {text.slice(start, end)}
            </span>
        );
        lastIndex = end;
    });

    // 添加最后一段文本
    if (lastIndex < text.length) {
        parts.push(<span key="text-end">{text.slice(lastIndex)}</span>);
    }

    return parts;
}

/**
 * 格式化时间戳
 */
function formatTimestamp(
    timestamp: number,
    locale: string,
    t: (key: string, options?: Record<string, unknown>) => string
): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
        return date.toLocaleTimeString(locale, {
            hour: "2-digit",
            minute: "2-digit",
        });
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
 * SearchResultItem 组件
 */
export function SearchResultItem({
    result,
    isSelected,
    onClick,
    onMouseEnter,
}: SearchResultItemProps) {
    const { t, i18n } = useTranslation();

    // 格式化会话名称
    const displaySessionName = formatSessionName(result.sessionId, result.sessionName);

    return (
        <div
            data-testid="search-result-item"
            data-title={result.sessionName}
            data-project-name={result.projectName}
            data-session-name={result.sessionName}
            role="option"
            aria-selected={isSelected}
            onClick={onClick}
            onMouseEnter={onMouseEnter}
            className={cn(
                "flex flex-col gap-1 px-4 py-3 cursor-pointer transition-colors duration-150",
                isSelected
                    ? "bg-primary/10"
                    : "hover:bg-accent"
            )}
        >
            {/* Header: Project / Session */}
            <div className="flex items-center gap-2 text-sm">
                <span className="text-primary font-medium truncate max-w-[200px]">
                    {result.projectName}
                </span>
                <span className="text-muted-foreground">/</span>
                <span className="text-foreground truncate flex-1" title={displaySessionName}>
                    {displaySessionName}
                </span>
                <span className="text-xs text-muted-foreground shrink-0">
                    {formatTimestamp(result.timestamp, i18n.language, t)}
                </span>
            </div>

            {/* Snippet */}
            <div className="text-sm text-muted-foreground line-clamp-2 leading-relaxed">
                {renderHighlightedText(result.snippet, result.highlightRanges)}
            </div>
        </div>
    );
}

export default SearchResultItem;
