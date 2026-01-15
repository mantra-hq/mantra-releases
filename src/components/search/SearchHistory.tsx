/**
 * SearchHistory - 搜索历史组件
 * Story 2.33: Task 7
 *
 * AC5: 显示最近搜索关键词
 * - 最多显示 10 条历史
 * - 点击历史词自动填充并搜索
 * - 支持删除单条历史
 * - 支持清空全部历史
 */

import { useTranslation } from "react-i18next";
import { History, X, Trash2 } from "lucide-react";
import { cn } from "@/lib/utils";

export interface SearchHistoryProps {
    /** 搜索历史列表 */
    queries: string[];
    /** 当前选中的索引 */
    selectedIndex: number;
    /** 点击历史词回调 */
    onSelect: (query: string) => void;
    /** 删除单条历史回调 */
    onRemove: (query: string) => void;
    /** 清空全部历史回调 */
    onClear: () => void;
    /** hover 事件回调 */
    onHover: (index: number) => void;
}

/**
 * 搜索历史组件
 */
export function SearchHistory({
    queries,
    selectedIndex,
    onSelect,
    onRemove,
    onClear,
    onHover,
}: SearchHistoryProps) {
    const { t } = useTranslation();

    if (queries.length === 0) {
        return null;
    }

    return (
        <div className="py-2">
            {/* Header */}
            <div className="flex items-center justify-between px-4 py-1">
                <span className="text-xs font-medium text-muted-foreground">
                    {t("search.history.title")}
                </span>
                <button
                    type="button"
                    onClick={onClear}
                    className={cn(
                        "flex items-center gap-1 text-xs text-muted-foreground",
                        "hover:text-foreground transition-colors",
                        "focus:outline-none focus:text-foreground"
                    )}
                    title={t("search.history.clearAll")}
                >
                    <Trash2 className="w-3 h-3" />
                    <span>{t("search.history.clearAll")}</span>
                </button>
            </div>

            {/* History List */}
            <ul className="mt-1">
                {queries.map((query, index) => (
                    <li key={query}>
                        <div
                            role="option"
                            aria-selected={index === selectedIndex}
                            onMouseEnter={() => onHover(index)}
                            className={cn(
                                "group flex items-center justify-between px-4 py-2",
                                "hover:bg-accent cursor-pointer transition-colors",
                                index === selectedIndex && "bg-accent"
                            )}
                        >
                            <button
                                type="button"
                                onClick={() => onSelect(query)}
                                className="flex items-center gap-3 flex-1 text-left min-w-0"
                            >
                                <History className="w-4 h-4 text-muted-foreground shrink-0" />
                                <span className="text-sm truncate">{query}</span>
                            </button>

                            {/* Remove Button */}
                            <button
                                type="button"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onRemove(query);
                                }}
                                className={cn(
                                    "p-1 rounded opacity-0 group-hover:opacity-100",
                                    "hover:bg-destructive/10 text-muted-foreground hover:text-destructive",
                                    "transition-all focus:opacity-100 focus:outline-none"
                                )}
                                title={t("search.history.remove")}
                            >
                                <X className="w-3.5 h-3.5" />
                            </button>
                        </div>
                    </li>
                ))}
            </ul>
        </div>
    );
}

export default SearchHistory;
