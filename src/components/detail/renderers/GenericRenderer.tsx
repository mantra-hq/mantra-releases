/**
 * GenericRenderer - 通用 JSON 渲染器
 * Story 2.15: Task 6.5
 *
 * 兜底渲染器，显示格式化 JSON
 */

import * as React from "react";
import { cn } from "@/lib/utils";

export interface GenericRendererProps {
    /** 内容 */
    content: string;
    /** 是否错误 */
    isError?: boolean;
    /** 是否自适应高度填满父容器 */
    autoHeight?: boolean;
    /** 自定义 className */
    className?: string;
}

/** 尝试格式化 JSON */
function formatContent(content: string): string {
    try {
        const parsed = JSON.parse(content);
        return JSON.stringify(parsed, null, 2);
    } catch {
        return content;
    }
}

/**
 * GenericRenderer 组件
 *
 * 通用渲染器，用于未匹配到专属渲染器的工具输出
 */
export function GenericRenderer({
    content,
    isError = false,
    autoHeight = true,
    className,
}: GenericRendererProps) {
    const formatted = React.useMemo(() => formatContent(content), [content]);

    return (
        <pre
            data-testid="generic-renderer"
            className={cn(
                "font-mono text-xs whitespace-pre-wrap break-all p-3",
                "bg-muted/30 rounded-md",
                autoHeight ? "h-full min-h-0 overflow-auto" : "max-h-96 overflow-auto",
                isError ? "text-destructive" : "text-foreground",
                className
            )}
        >
            {formatted}
        </pre>
    );
}

export default GenericRenderer;
