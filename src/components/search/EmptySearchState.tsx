/**
 * EmptySearchState - 搜索空状态组件
 * Story 2.10: Task 7.1
 * Story 2.26: 国际化支持
 * Story 2.33: AC7 - 显示友好的空状态提示，含筛选条件
 *
 * 无匹配结果时显示友好提示
 */

import { SearchX } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import type { SearchFilters } from "@/stores/useSearchStore";

/**
 * EmptySearchState Props
 */
export interface EmptySearchStateProps {
    /** 搜索查询 */
    query: string;
    /** 当前筛选器状态 (Story 2.33) */
    filters?: SearchFilters;
    /** 自定义类名 */
    className?: string;
}

/**
 * 检查是否有任何筛选器处于非默认状态
 */
function hasActiveFilters(filters?: SearchFilters): boolean {
    if (!filters) return false;
    return (
        filters.contentType !== "all" ||
        filters.projectId !== null ||
        filters.timePreset !== "all"
    );
}

/**
 * EmptySearchState 组件
 */
export function EmptySearchState({
    query,
    filters,
    className,
}: EmptySearchStateProps) {
    const { t } = useTranslation();
    const hasFilters = hasActiveFilters(filters);

    return (
        <div
            className={cn(
                "flex flex-col items-center justify-center py-12 px-4 text-center",
                className
            )}
        >
            <SearchX className="w-12 h-12 text-muted-foreground/50 mb-4" />
            <p className="text-sm text-muted-foreground mb-2">
                {t("search.noResultsFor", { query })}
            </p>

            {/* AC7: 显示筛选条件提示 */}
            {hasFilters ? (
                <p className="text-xs text-muted-foreground/70">
                    {t("search.tryRelaxFilters")}
                </p>
            ) : (
                <p className="text-xs text-muted-foreground/70">
                    {t("search.tryDifferentKeywords")}
                </p>
            )}

            {/* 显示当前活动的筛选条件 */}
            {hasFilters && filters && (
                <div className="mt-3 flex flex-wrap gap-2 justify-center">
                    {filters.contentType !== "all" && (
                        <span className="text-xs px-2 py-0.5 bg-muted rounded">
                            {t(`search.filters.${filters.contentType}`)}
                        </span>
                    )}
                    {filters.projectId && (
                        <span className="text-xs px-2 py-0.5 bg-muted rounded">
                            {t("search.filters.specificProject")}
                        </span>
                    )}
                    {filters.timePreset !== "all" && (
                        <span className="text-xs px-2 py-0.5 bg-muted rounded">
                            {t(`search.filters.${filters.timePreset === "today" ? "today" : filters.timePreset === "week" ? "thisWeek" : "thisMonth"}`)}
                        </span>
                    )}
                </div>
            )}
        </div>
    );
}

export default EmptySearchState;
