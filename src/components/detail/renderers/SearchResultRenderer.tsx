/**
 * SearchResultRenderer - 搜索结果渲染器
 * Story 2.15: Task 6.4
 *
 * 显示搜索结果列表
 */

import * as React from "react";
import { FileText, Hash } from "lucide-react";
import { cn } from "@/lib/utils";

export interface SearchMatch {
    /** 文件路径 */
    file: string;
    /** 行号 */
    line: number;
    /** 匹配内容 */
    content: string;
}

export interface SearchResultRendererProps {
    /** 搜索结果 JSON 字符串 */
    content: string;
    /** 点击结果回调 */
    onResultClick?: (file: string, line: number) => void;
    /** 自定义 className */
    className?: string;
}

/** 解析搜索结果 */
function parseSearchResults(content: string): SearchMatch[] {
    try {
        const parsed = JSON.parse(content);
        if (Array.isArray(parsed)) {
            return parsed.filter(
                (item): item is SearchMatch =>
                    typeof item === "object" &&
                    typeof item.file === "string" &&
                    typeof item.line === "number"
            );
        }
        // 尝试从行文本解析
        return [];
    } catch {
        // 如果不是 JSON，尝试按行解析
        const lines = content.split("\n").filter(Boolean);
        const matches: SearchMatch[] = [];

        for (const line of lines) {
            // 常见格式: file:line:content
            const match = line.match(/^(.+?):(\d+):(.*)$/);
            if (match) {
                matches.push({
                    file: match[1],
                    line: parseInt(match[2], 10),
                    content: match[3],
                });
            }
        }

        return matches;
    }
}

/**
 * SearchResultRenderer 组件
 *
 * 用于渲染搜索类输出：
 * - grep_search
 * - find_by_name
 */
export function SearchResultRenderer({
    content,
    onResultClick,
    className,
}: SearchResultRendererProps) {
    const results = React.useMemo(() => parseSearchResults(content), [content]);

    if (results.length === 0) {
        // 如果无法解析为结构化结果，显示原始内容
        return (
            <pre
                data-testid="search-result-renderer"
                className={cn(
                    "font-mono text-xs whitespace-pre-wrap text-muted-foreground p-3",
                    className
                )}
            >
                {content}
            </pre>
        );
    }

    return (
        <div
            data-testid="search-result-renderer"
            className={cn("space-y-1", className)}
        >
            <div className="text-xs text-muted-foreground mb-2">
                共 {results.length} 个结果
            </div>

            <div className="max-h-80 overflow-auto space-y-1">
                {results.map((result, idx) => (
                    <button
                        key={`${result.file}-${result.line}-${idx}`}
                        type="button"
                        onClick={() => onResultClick?.(result.file, result.line)}
                        className={cn(
                            "w-full text-left p-2 rounded",
                            "bg-muted/30 hover:bg-muted/50",
                            "transition-colors"
                        )}
                    >
                        <div className="flex items-center gap-2 text-xs">
                            <FileText className="h-3 w-3 shrink-0 text-muted-foreground" />
                            <span className="truncate text-foreground">
                                {result.file.split("/").pop()}
                            </span>
                            <span className="flex items-center gap-0.5 text-muted-foreground">
                                <Hash className="h-3 w-3" />
                                {result.line}
                            </span>
                        </div>
                        {result.content && (
                            <div className="mt-1 font-mono text-xs text-muted-foreground truncate pl-5">
                                {result.content.trim()}
                            </div>
                        )}
                    </button>
                ))}
            </div>
        </div>
    );
}

export default SearchResultRenderer;
