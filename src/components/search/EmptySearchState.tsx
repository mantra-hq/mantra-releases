/**
 * EmptySearchState - 搜索空状态组件
 * Story 2.10: Task 7.1
 *
 * 无匹配结果时显示友好提示
 */

import { SearchX } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * EmptySearchState Props
 */
export interface EmptySearchStateProps {
    /** 搜索查询 */
    query: string;
    /** 自定义类名 */
    className?: string;
}

/**
 * EmptySearchState 组件
 */
export function EmptySearchState({ query, className }: EmptySearchStateProps) {
    return (
        <div
            className={cn(
                "flex flex-col items-center justify-center py-12 px-4 text-center",
                className
            )}
        >
            <SearchX className="w-12 h-12 text-muted-foreground/50 mb-4" />
            <p className="text-sm text-muted-foreground mb-2">
                未找到包含 "<span className="text-foreground font-medium">{query}</span>" 的结果
            </p>
            <p className="text-xs text-muted-foreground/70">
                尝试使用不同的关键词或检查拼写
            </p>
        </div>
    );
}

export default EmptySearchState;
