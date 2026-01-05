/**
 * SearchResultList - 搜索结果列表组件
 * Story 2.10: Task 3.1
 * Story 2.26: 国际化支持
 *
 * 虚拟化列表展示搜索结果，支持键盘导航
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { useVirtualizer } from "@tanstack/react-virtual";
import { SearchResultItem } from "./SearchResultItem";
import type { SearchResult } from "@/stores/useSearchStore";
import { cn } from "@/lib/utils";

/**
 * SearchResultList Props
 */
export interface SearchResultListProps {
    /** 搜索结果列表 */
    results: SearchResult[];
    /** 当前选中的索引 */
    selectedIndex: number;
    /** 选择结果回调 */
    onSelect: (result: SearchResult) => void;
    /** hover 时更新选中索引 */
    onHover?: (index: number) => void;
    /** 自定义类名 */
    className?: string;
}

/** 每个结果项的估算高度 */
const ITEM_HEIGHT = 72;

/**
 * SearchResultList 组件
 * 使用 @tanstack/react-virtual 实现虚拟化滚动
 */
export function SearchResultList({
    results,
    selectedIndex,
    onSelect,
    onHover,
    className,
}: SearchResultListProps) {
    const { t } = useTranslation();
    // 滚动容器 ref
    const parentRef = React.useRef<HTMLDivElement>(null);

    // 虚拟化
    const virtualizer = useVirtualizer({
        count: results.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => ITEM_HEIGHT,
        overscan: 5,
    });

    // 当 selectedIndex 变化时，滚动到可视区域
    React.useEffect(() => {
        if (selectedIndex >= 0 && selectedIndex < results.length) {
            virtualizer.scrollToIndex(selectedIndex, {
                align: "auto",
                behavior: "smooth",
            });
        }
    }, [selectedIndex, results.length, virtualizer]);

    if (results.length === 0) {
        return null;
    }

    const virtualItems = virtualizer.getVirtualItems();

    return (
        <div
            ref={parentRef}
            role="listbox"
            aria-label={t("search.searchResults")}
            className={cn("overflow-y-auto max-h-[400px]", className)}
        >
            <div
                style={{
                    height: `${virtualizer.getTotalSize()}px`,
                    width: "100%",
                    position: "relative",
                }}
            >
                {virtualItems.map((virtualItem) => {
                    const result = results[virtualItem.index];
                    return (
                        <div
                            key={virtualItem.key}
                            style={{
                                position: "absolute",
                                top: 0,
                                left: 0,
                                width: "100%",
                                height: `${virtualItem.size}px`,
                                transform: `translateY(${virtualItem.start}px)`,
                            }}
                        >
                            <SearchResultItem
                                result={result}
                                isSelected={virtualItem.index === selectedIndex}
                                onClick={() => onSelect(result)}
                                onMouseEnter={() => onHover?.(virtualItem.index)}
                            />
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

export default SearchResultList;
