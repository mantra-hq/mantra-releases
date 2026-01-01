/**
 * FileRenderer - 文件类输出渲染器
 * Story 2.15: Task 6.3
 *
 * 复用 Monaco 渲染文件代码
 */

import Editor from "@monaco-editor/react";
import { cn } from "@/lib/utils";

export interface FileRendererProps {
    /** 文件内容 */
    content: string;
    /** 文件路径 (用于推断语言) */
    filePath?: string;
    /** 自定义 className */
    className?: string;
}

/** 从文件路径推断语言 */
function inferLanguage(filePath?: string): string {
    if (!filePath) return "plaintext";

    const ext = filePath.split(".").pop()?.toLowerCase();
    const langMap: Record<string, string> = {
        ts: "typescript",
        tsx: "typescript",
        js: "javascript",
        jsx: "javascript",
        py: "python",
        rs: "rust",
        go: "go",
        json: "json",
        yaml: "yaml",
        yml: "yaml",
        md: "markdown",
        css: "css",
        scss: "scss",
        html: "html",
        xml: "xml",
        sql: "sql",
        sh: "shell",
        bash: "shell",
    };

    return langMap[ext || ""] || "plaintext";
}

/**
 * FileRenderer 组件
 *
 * 用于渲染文件类输出：
 * - read_file
 * - write_to_file
 */
export function FileRenderer({
    content,
    filePath,
    className,
}: FileRendererProps) {
    const language = inferLanguage(filePath);
    const lineCount = content.split("\n").length;
    const height = Math.min(Math.max(lineCount * 20, 100), 400);

    return (
        <div
            data-testid="file-renderer"
            className={cn(
                "rounded-md overflow-hidden border border-border",
                className
            )}
            style={{ height: `${height}px` }}
        >
            <Editor
                language={language}
                value={content}
                theme="vs-dark"
                options={{
                    readOnly: true,
                    minimap: { enabled: false },
                    scrollBeyondLastLine: false,
                    fontSize: 13,
                    lineNumbers: "on",
                    folding: true,
                    wordWrap: "on",
                    automaticLayout: true,
                }}
            />
        </div>
    );
}

export default FileRenderer;
