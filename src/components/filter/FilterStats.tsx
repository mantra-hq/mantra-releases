/**
 * FilterStats - 过滤统计信息组件
 * Story 2.16: Task 4
 * Story 2.26: 国际化支持
 *
 * 显示过滤后的消息数量统计
 * AC: #4
 */


import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

export interface FilterStatsProps {
    /** 过滤后的消息数量 */
    filteredCount: number;
    /** 总消息数量 */
    totalCount: number;
    /** 自定义 className */
    className?: string;
}

/**
 * FilterStats 组件
 * 显示 '匹配: {n}/{m} 条' 格式的统计信息
 */
export function FilterStats({
    filteredCount,
    totalCount,
    className,
}: FilterStatsProps) {
    const { t } = useTranslation();
    // 只有当过滤后数量与总数不同时才显示
    const isFiltered = filteredCount !== totalCount;

    if (!isFiltered) {
        return null;
    }

    return (
        <span
            className={cn(
                "text-xs text-muted-foreground whitespace-nowrap",
                className
            )}
            role="status"
            aria-live="polite"
        >
            {t("filter.matchCount", { current: filteredCount, total: totalCount })}
        </span>
    );
}

export default FilterStats;
