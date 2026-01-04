/**
 * SearchFilter Component - 搜索过滤
 * Story 2.9 UX Redesign
 * Story 2.26: 国际化支持
 *
 * 搜索输入框，支持实时过滤项目和会话
 */

import { useTranslation } from "react-i18next";
import { Search, X } from "lucide-react";

/** SearchFilter Props */
export interface SearchFilterProps {
    /** 搜索值 */
    value: string;
    /** 值变化回调 */
    onChange: (value: string) => void;
    /** 搜索结果数量 */
    resultCount: number;
    /** 总数量 */
    totalCount: number;
    /** 占位符文本 */
    placeholder?: string;
}

/**
 * SearchFilter 组件
 */
export function SearchFilter({
    value,
    onChange,
    resultCount,
    totalCount,
    placeholder,
}: SearchFilterProps) {
    const { t } = useTranslation();
    const hasQuery = value.trim().length > 0;
    const displayPlaceholder = placeholder || t("import.searchProjectOrSession");

    return (
        <div className="relative">
            {/* 搜索图标 */}
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />

            {/* 输入框 */}
            <input
                type="text"
                role="searchbox"
                value={value}
                onChange={(e) => onChange(e.target.value)}
                placeholder={displayPlaceholder}
                className="w-full h-10 pl-10 pr-20 text-sm bg-muted/50 border border-border rounded-lg placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-colors"
                aria-label={t("import.searchProjectOrSession")}
            />

            {/* 结果计数 / 清除按钮 */}
            <div className="absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-2">
                {hasQuery && (
                    <>
                        <span className="text-xs text-muted-foreground">
                            {resultCount}/{totalCount}
                        </span>
                        <button
                            type="button"
                            onClick={() => onChange("")}
                            className="p-1 rounded hover:bg-muted transition-colors"
                            aria-label={t("common.clearSearch")}
                        >
                            <X className="w-3 h-3 text-muted-foreground" />
                        </button>
                    </>
                )}
            </div>
        </div>
    );
}
