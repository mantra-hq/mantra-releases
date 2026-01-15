/**
 * FilterBar - 搜索筛选器栏
 * Story 2.33: Task 3.1
 *
 * 容器组件，整合内容类型、项目和时间范围筛选器
 * 布局：[内容类型 Tab] [项目下拉] [时间下拉]
 */

import { ContentTypeFilter } from "./ContentTypeFilter";
import { ProjectFilter } from "./ProjectFilter";
import { TimeRangeFilter } from "./TimeRangeFilter";
import type { SearchFilters } from "@/stores/useSearchStore";

export interface FilterBarProps {
    /** 当前筛选器状态 */
    filters: SearchFilters;
    /** 筛选器变化回调 */
    onFiltersChange: (filters: Partial<SearchFilters>) => void;
}

/**
 * 筛选器栏组件
 */
export function FilterBar({ filters, onFiltersChange }: FilterBarProps) {
    return (
        <div className="flex flex-wrap items-center justify-between gap-2 sm:gap-3 px-3 sm:px-4 py-2 border-b border-border bg-muted/20">
            {/* Left: Content Type Tabs */}
            <ContentTypeFilter
                value={filters.contentType}
                onChange={(contentType) => onFiltersChange({ contentType })}
            />

            {/* Right: Dropdowns */}
            <div className="flex items-center gap-2">
                <TimeRangeFilter
                    value={filters.timePreset}
                    onChange={(timePreset) => onFiltersChange({ timePreset })}
                />
                <ProjectFilter
                    value={filters.projectId}
                    onChange={(projectId) => onFiltersChange({ projectId })}
                />
            </div>
        </div>
    );
}

export default FilterBar;
