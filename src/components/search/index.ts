/**
 * Search Components Index - 搜索组件导出
 * Story 2.8: Task 6
 * Story 2.10: Task 1-7 (Global Search)
 * Story 2.33: Task 3, 6, 7 (Filter Components, Grouped Results, Search History)
 */

export { ProjectSearch, type ProjectSearchProps } from "./ProjectSearch";
export { GlobalSearch } from "./GlobalSearch";
export { SearchResultList, type SearchResultListProps } from "./SearchResultList";
export { SearchResultItem, type SearchResultItemProps } from "./SearchResultItem";
export { EmptySearchState, type EmptySearchStateProps } from "./EmptySearchState";
export { RecentSessions, type RecentSessionsProps } from "./RecentSessions";

// Story 2.33: New Components
export { FilterBar, type FilterBarProps } from "./FilterBar";
export { ContentTypeFilter, type ContentTypeFilterProps } from "./ContentTypeFilter";
export { ProjectFilter, type ProjectFilterProps } from "./ProjectFilter";
export { TimeRangeFilter, type TimeRangeFilterProps } from "./TimeRangeFilter";
export { GroupedSearchResultList, type GroupedSearchResultListProps } from "./GroupedSearchResultList";
export { SearchHistory, type SearchHistoryProps } from "./SearchHistory";
