/**
 * ToolOutput - 工具输出组件
 * Story 2.4: Task 3
 *
 * 显示工具执行结果，支持成功/错误两种状态
 * AC: #4, #5, #6, #7
 */

import * as React from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronRight, Check, X } from "lucide-react";
import { cn } from "@/lib/utils";

export interface ToolOutputProps {
  /** 输出内容 */
  content: string;
  /** 是否为错误结果 */
  isError?: boolean;
  /** 默认是否展开 */
  defaultOpen?: boolean;
  /** 自定义 className */
  className?: string;
}

/**
 * ToolOutput 组件
 *
 * 视觉规范:
 * - 成功状态: ✓ 图标 + 绿色边框
 * - 错误状态: ✗ 图标 + 红色边框 + 红色背景
 * - 内容: 等宽字体，可折叠
 * - 动画: 150ms ease-out
 */
export function ToolOutput({
  content,
  isError = false,
  defaultOpen = false,
  className,
}: ToolOutputProps) {
  const [isOpen, setIsOpen] = React.useState(defaultOpen);
  
  // 截断长内容的预览
  const previewLength = 100;
  const isLongContent = content.length > previewLength;
  const previewContent = isLongContent
    ? content.slice(0, previewLength) + "..."
    : content;

  return (
    <Collapsible.Root
      open={isOpen}
      onOpenChange={setIsOpen}
      className={cn(
        // 容器样式
        "rounded-lg my-2 overflow-hidden",
        // 状态变体
        isError
          ? "border-l-[3px] border-l-destructive bg-destructive/5"
          : "border-l-[3px] border-l-success bg-success/5",
        className
      )}
      aria-label={isError ? "工具执行失败" : "工具执行成功"}
    >
      <Collapsible.Trigger
        className={cn(
          // 头部样式
          "flex items-center gap-2 w-full",
          "px-3 py-2",
          "cursor-pointer select-none",
          "text-[13px]",
          // Hover 效果
          "hover:bg-muted/30",
          // Focus 状态
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
          "focus-visible:ring-inset"
        )}
        aria-expanded={isOpen}
      >
        {/* 状态图标 */}
        {isError ? (
          <X className="h-4 w-4 shrink-0 text-destructive" />
        ) : (
          <Check className="h-4 w-4 shrink-0 text-success" />
        )}

        {/* 预览内容 */}
        <span
          className={cn(
            "flex-1 truncate font-mono text-xs",
            "text-muted-foreground"
          )}
        >
          {isOpen ? (isError ? "错误详情" : "执行结果") : previewContent}
        </span>

        {/* 展开箭头 */}
        <ChevronRight
          className={cn(
            "h-3.5 w-3.5 shrink-0 text-muted-foreground",
            "transition-transform duration-150 ease-out",
            isOpen && "rotate-90"
          )}
        />
      </Collapsible.Trigger>

      <Collapsible.Content
        className={cn(
          // 内容样式
          "overflow-hidden",
          // 动画 (150ms ease-out)
          "data-[state=open]:animate-collapsible-down",
          "data-[state=closed]:animate-collapsible-up"
        )}
      >
        <div
          className={cn(
            // 内容容器
            "px-3 py-3",
            "border-t",
            isError ? "border-destructive/20" : "border-success/20"
          )}
        >
          <pre
            className={cn(
              // 输出内容样式
              "font-mono text-xs",
              "whitespace-pre-wrap break-all",
              isError ? "text-destructive" : "text-muted-foreground"
            )}
          >
            {content}
          </pre>
        </div>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}

export default ToolOutput;


