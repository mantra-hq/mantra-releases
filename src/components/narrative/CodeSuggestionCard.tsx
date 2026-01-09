/**
 * CodeSuggestionCard - 代码建议卡片组件
 * Story 8.11: Task 7 (AC #8)
 *
 * 显示 Cursor 的代码建议块，包含文件路径和语法高亮代码。
 * 复制功能由内嵌的 CodeBlockWithCopy 组件提供。
 */

import { FileCode2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";
import { CodeBlockWithCopy } from "@/components/common/CodeBlockWithCopy";

export interface CodeSuggestionCardProps {
    /** 文件路径 */
    filePath?: string;
    /** 代码内容 */
    code: string;
    /** 编程语言 */
    language?: string;
    /** 自定义 className */
    className?: string;
}

/**
 * CodeSuggestionCard 组件
 *
 * 显示代码建议卡片：
 * - 顶部显示文件路径（breadcrumb 样式）
 * - 语法高亮代码（由 CodeBlockWithCopy 提供复制功能）
 */
export function CodeSuggestionCard({
    filePath,
    code,
    language,
    className,
}: CodeSuggestionCardProps) {
    const { t } = useTranslation();

    return (
        // 使用 onClickCapture 在捕获阶段阻止冒泡，避免触发消息选中
        // 但不阻止子元素（如复制按钮）的正常交互
        <div
            className={cn(
                "rounded-lg border border-border bg-muted/30 overflow-hidden my-2",
                className
            )}
            onClickCapture={(e) => e.stopPropagation()}
        >
            {/* 头部：文件路径 */}
            <div className="flex items-center gap-2 px-3 py-2 bg-muted/50 border-b border-border">
                <FileCode2 className="h-4 w-4 text-primary shrink-0" />
                {filePath ? (
                    <span className="text-xs font-mono text-muted-foreground truncate flex-1">
                        {filePath}
                    </span>
                ) : (
                    <span className="text-xs text-muted-foreground flex-1">
                        {t("message.codeSuggestion")}
                    </span>
                )}
            </div>

            {/* 代码内容 - CodeBlockWithCopy 已包含复制按钮 */}
            <div className="p-0">
                <CodeBlockWithCopy
                    code={code}
                    language={language}
                />
            </div>
        </div>
    );
}

export default CodeSuggestionCard;
