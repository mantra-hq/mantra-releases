/**
 * CodeBlockWithCopy - 带复制功能的代码块组件
 * Story 2.22: Task 4
 *
 * 在代码块右上角显示复制按钮，支持语法高亮
 */

import { cn } from "@/lib/utils";
import { CopyButton } from "./CopyButton";

export interface CodeBlockWithCopyProps {
  /** 代码内容 */
  code: string;
  /** 编程语言 */
  language?: string;
  /** 自定义 className */
  className?: string;
}

/**
 * CodeBlockWithCopy 组件
 *
 * 功能:
 * - 代码块右上角显示复制按钮 (AC2)
 * - 支持语言标识显示 (Task 4.5)
 * - 仅复制代码内容（不含语言标识）(AC2)
 * - 半透明背景和 backdrop-filter (Task 4.4)
 */
export function CodeBlockWithCopy({
  code,
  language,
  className,
}: CodeBlockWithCopyProps) {
  return (
    <div className={cn("group/code relative", className)}>
      {/* 语言标识和复制按钮容器 - 悬浮时显示 */}
      <div
        className={cn(
          "absolute right-2 top-2 z-10",
          "flex items-center gap-2",
          // 半透明背景 (Task 4.4)
          "rounded-md bg-muted/80 backdrop-blur-sm",
          "px-2 py-1",
          // 悬浮时显示
          "opacity-0 transition-opacity duration-150 group-hover/code:opacity-100"
        )}
      >
        {/* 语言标识 (Task 4.5) */}
        {language && (
          <span className="text-xs text-muted-foreground">{language}</span>
        )}
        {/* 复制按钮 (AC2) */}
        <CopyButton
          content={code}
          size="sm"
          ariaLabel="复制代码"
          tooltip="复制代码"
        />
      </div>

      {/* 代码块 */}
      <pre
        className={cn(
          "overflow-x-auto rounded-lg bg-muted p-4",
          "text-sm leading-relaxed",
          // 确保浅色/深色模式下文字对比度足够
          "text-foreground"
        )}
      >
        <code className={language ? `language-${language}` : undefined}>
          {code}
        </code>
      </pre>
    </div>
  );
}

export default CodeBlockWithCopy;
