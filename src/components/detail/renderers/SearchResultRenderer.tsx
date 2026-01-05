/**
 * SearchResultRenderer - 搜索结果渲染器
 * Story 2.15: Task 6.4
 * Story 2.26: 国际化支持
 *
 * 显示搜索结果列表
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
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
    /** 是否自适应高度填满父容器 */
    autoHeight?: boolean;
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
            // 格式 1: file:line:content (grep 格式)
            const grepMatch = line.match(/^(.+?):(\d+):(.*)$/);
            if (grepMatch) {
                matches.push({
                    file: grepMatch[1],
                    line: parseInt(grepMatch[2], 10),
                    content: grepMatch[3],
                });
                continue;
            }

            // 格式 2: 纯文件路径 (Glob 格式)
            // 检查是否看起来像文件路径 (以 / 开头或包含 /)
            const trimmedLine = line.trim();
            if (trimmedLine && (trimmedLine.startsWith("/") || trimmedLine.includes("/"))) {
                matches.push({
                    file: trimmedLine,
                    line: 0, // Glob 没有行号
                    content: "", // Glob 没有内容预览
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
    autoHeight = true,
    className,
}: SearchResultRendererProps) {
    const { t } = useTranslation();
    const results = React.useMemo(() => parseSearchResults(content), [content]);

    if (results.length === 0) {
        // 如果无法解析为结构化结果，显示原始内容
        return (
            <pre
                data-testid="search-result-renderer"
                className={cn(
                    "font-mono text-xs whitespace-pre-wrap text-muted-foreground p-3",
                    autoHeight && "h-full min-h-0 overflow-auto",
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
            className={cn(
                "space-y-1",
                autoHeight && "h-full min-h-0 overflow-auto",
                className
            )}
        >
            <div className="text-xs text-muted-foreground mb-2">
                {results[0]?.line === 0
                    ? t("search.totalFiles", { count: results.length })
                    : t("search.totalResults", { count: results.length })}
            </div>

            <div className="space-y-1">
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
                            <span className="truncate text-foreground" title={result.file}>
                                {result.line === 0
                                    ? result.file // Glob: 显示完整路径
                                    : result.file.split("/").pop() // Grep: 只显示文件名
                                }
                            </span>
                            {result.line > 0 && (
                                <span className="flex items-center gap-0.5 text-muted-foreground shrink-0">
                                    <Hash className="h-3 w-3" />
                                    {result.line}
                                </span>
                            )}
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
