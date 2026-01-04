/**
 * MessageFilterBar - 消息过滤栏主组件
 * Story 2.16: Task 5, 8
 * Story 2.26: 国际化支持
 *
 * 组合 TypeChips, FilterSearchInput, FilterStats 和清除过滤按钮
 * 支持键盘快捷键 Cmd/Ctrl+F 和 Escape
 * AC: #10, #12, #13
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { RotateCcw } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useMessageFilterStore } from "@/stores/useMessageFilterStore";
import { TypeChips } from "./TypeChips";
import { FilterSearchInput } from "./FilterSearchInput";
import { FilterStats } from "./FilterStats";

export interface MessageFilterBarProps {
    /** 过滤后的消息数量 */
    filteredCount: number;
    /** 总消息数量 */
    totalCount: number;
    /** 自定义 className */
    className?: string;
}

/**
 * MessageFilterBar 组件
 * 消息过滤栏，包含类型 Chips、搜索框、统计和清除按钮
 */
export const MessageFilterBar = React.forwardRef<
    HTMLInputElement,
    MessageFilterBarProps
>(({ filteredCount, totalCount, className }, ref) => {
    const { t } = useTranslation();
    const { selectedTypes, searchQuery, clearFilters, setSearchQuery } = useMessageFilterStore();
    const searchInputRef = React.useRef<HTMLInputElement>(null);

    // 是否有活动的过滤条件
    const hasActiveFilters = selectedTypes.size > 0 || searchQuery.length > 0;

    // 键盘快捷键处理 (AC: #12, #13)
    React.useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            // Cmd/Ctrl + F: 聚焦搜索框
            if ((event.metaKey || event.ctrlKey) && event.key === "f") {
                event.preventDefault();
                searchInputRef.current?.focus();
            }

            // Escape: 清除搜索关键词
            if (event.key === "Escape") {
                // 只在搜索框聚焦或有搜索内容时触发
                if (
                    document.activeElement === searchInputRef.current ||
                    searchQuery.length > 0
                ) {
                    event.preventDefault();
                    setSearchQuery("");
                    searchInputRef.current?.blur();
                }
            }
        };

        document.addEventListener("keydown", handleKeyDown);
        return () => document.removeEventListener("keydown", handleKeyDown);
    }, [searchQuery, setSearchQuery]);

    // 合并外部 ref 和内部 ref
    React.useImperativeHandle(ref, () => searchInputRef.current!, []);

    return (
        <div
            className={cn(
                "flex items-center gap-2 p-2 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60",
                className
            )}
        >
            {/* 类型 Chips */}
            <TypeChips className="flex-1 min-w-0" />

            {/* 搜索框 - 紧凑宽度 */}
            <FilterSearchInput ref={searchInputRef} className="w-48 shrink-0" />

            {/* 统计信息 */}
            <FilterStats filteredCount={filteredCount} totalCount={totalCount} />

            {/* 清除过滤按钮 */}
            {hasActiveFilters && (
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={clearFilters}
                    className="h-7 px-2 text-xs text-muted-foreground shrink-0"
                >
                    <RotateCcw className="size-3 mr-1" />
                    <span>{t("filter.clearFilter")}</span>
                </Button>
            )}
        </div>
    );
});

MessageFilterBar.displayName = "MessageFilterBar";

export default MessageFilterBar;
