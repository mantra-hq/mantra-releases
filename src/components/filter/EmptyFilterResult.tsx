/**
 * EmptyFilterResult - 空过滤结果组件
 * Story 2.16: Task 7
 *
 * 当过滤结果为空时显示的友好提示
 * AC: #11
 */


import { SearchX } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useMessageFilterStore } from "@/stores/useMessageFilterStore";

export interface EmptyFilterResultProps {
    /** 自定义 className */
    className?: string;
}

/**
 * EmptyFilterResult 组件
 * 显示无匹配结果的友好提示和清除过滤按钮
 */
export function EmptyFilterResult({ className }: EmptyFilterResultProps) {
    const { clearFilters } = useMessageFilterStore();

    return (
        <div
            className={cn(
                "flex flex-col items-center justify-center gap-4 p-8 text-center",
                className
            )}
        >
            <div className="rounded-full bg-muted p-4">
                <SearchX className="size-8 text-muted-foreground" />
            </div>
            <div className="space-y-2">
                <h3 className="text-lg font-semibold text-foreground">
                    没有找到匹配的消息
                </h3>
                <p className="text-sm text-muted-foreground max-w-xs">
                    尝试调整过滤条件或清除搜索关键词
                </p>
            </div>
            <Button variant="outline" size="sm" onClick={clearFilters}>
                清除过滤条件
            </Button>
        </div>
    );
}

export default EmptyFilterResult;
