/**
 * FilterSearchInput - 过滤搜索输入组件
 * Story 2.16: Task 3
 *
 * 支持 debounce 的搜索输入框
 * AC: #8, #9
 */

import * as React from "react";
import { Search, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { useMessageFilterStore } from "@/stores/useMessageFilterStore";

export interface FilterSearchInputProps {
    /** 自定义 className */
    className?: string;
    /** Debounce 延迟时间 (ms)，默认 300ms */
    debounceMs?: number;
    /** 占位符文本 */
    placeholder?: string;
}

/**
 * FilterSearchInput 组件
 * 带 debounce 的搜索输入框
 */
export const FilterSearchInput = React.forwardRef<
    HTMLInputElement,
    FilterSearchInputProps
>(({ className, debounceMs = 300, placeholder = "搜索消息..." }, ref) => {
    const { searchQuery, setSearchQuery, setSearchFocused } = useMessageFilterStore();
    const [localValue, setLocalValue] = React.useState(searchQuery);
    const inputRef = React.useRef<HTMLInputElement>(null);
    const debounceTimerRef = React.useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

    // 同步外部 store 变化到本地状态
    React.useEffect(() => {
        setLocalValue(searchQuery);
    }, [searchQuery]);

    // Debounce 处理
    const handleChange = React.useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            const value = e.target.value;
            setLocalValue(value);

            // 清除之前的 timer
            if (debounceTimerRef.current) {
                clearTimeout(debounceTimerRef.current);
            }

            // 设置新的 debounce timer
            debounceTimerRef.current = setTimeout(() => {
                setSearchQuery(value);
            }, debounceMs);
        },
        [debounceMs, setSearchQuery]
    );

    // 清除搜索
    const handleClear = React.useCallback(() => {
        setLocalValue("");
        setSearchQuery("");
        inputRef.current?.focus();
    }, [setSearchQuery]);

    // 处理焦点
    const handleFocus = React.useCallback(() => {
        setSearchFocused(true);
    }, [setSearchFocused]);

    const handleBlur = React.useCallback(() => {
        setSearchFocused(false);
    }, [setSearchFocused]);

    // 清理 timer
    React.useEffect(() => {
        return () => {
            if (debounceTimerRef.current) {
                clearTimeout(debounceTimerRef.current);
            }
        };
    }, []);

    // 合并 ref
    React.useImperativeHandle(ref, () => inputRef.current!, []);

    return (
        <div className={cn("relative flex items-center", className)}>
            {/* 搜索图标 */}
            <Search
                className="absolute left-2.5 size-4 text-muted-foreground pointer-events-none"
                aria-hidden="true"
            />

            {/* 输入框 */}
            <input
                ref={inputRef}
                type="text"
                value={localValue}
                onChange={handleChange}
                onFocus={handleFocus}
                onBlur={handleBlur}
                placeholder={placeholder}
                className={cn(
                    "w-full h-8 pl-8 pr-8 rounded-md text-sm",
                    "bg-muted/50 border border-transparent",
                    "placeholder:text-muted-foreground",
                    "focus:outline-none focus:ring-2 focus:ring-ring focus:border-ring",
                    "transition-colors"
                )}
                aria-label="搜索消息"
            />

            {/* 清除按钮 */}
            {localValue && (
                <button
                    type="button"
                    onClick={handleClear}
                    className={cn(
                        "absolute right-2 p-0.5 rounded-sm",
                        "text-muted-foreground hover:text-foreground",
                        "focus:outline-none focus:ring-2 focus:ring-ring"
                    )}
                    aria-label="清除搜索"
                >
                    <X className="size-3.5" />
                </button>
            )}
        </div>
    );
});

FilterSearchInput.displayName = "FilterSearchInput";

export default FilterSearchInput;
