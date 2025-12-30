/**
 * ToolCall - 工具调用组件
 * Story 2.4: Task 2
 *
 * 显示 AI 的工具调用请求，包含工具名称和输入参数
 * AC: #3, #5, #6, #7
 */

import * as React from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronRight, Wrench } from "lucide-react";
import { cn } from "@/lib/utils";

export interface ToolCallProps {
  /** 工具名称 */
  toolName: string;
  /** 工具输入参数 */
  toolInput?: Record<string, unknown>;
  /** 默认是否展开 */
  defaultOpen?: boolean;
  /** 自定义 className */
  className?: string;
}

/**
 * 格式化 JSON 显示
 */
function formatJson(obj: Record<string, unknown> | undefined): string {
  if (!obj || Object.keys(obj).length === 0) {
    return "{}";
  }
  return JSON.stringify(obj, null, 2);
}

/**
 * ToolCall 组件
 *
 * 视觉规范:
 * - 容器: muted 背景，8px 圆角
 * - 头部: 🔧 图标 + 工具名称 + 展开箭头
 * - 内容: 等宽字体，JSON 格式化显示
 * - 动画: 150ms ease-out
 */
export function ToolCall({
  toolName,
  toolInput,
  defaultOpen = false,
  className,
}: ToolCallProps) {
  const [isOpen, setIsOpen] = React.useState(defaultOpen);
  const formattedInput = formatJson(toolInput);
  const hasInput = toolInput && Object.keys(toolInput).length > 0;

  return (
    <Collapsible.Root
      open={isOpen}
      onOpenChange={setIsOpen}
      className={cn(
        // 容器样式
        "bg-muted/50 rounded-lg my-2 overflow-hidden",
        className
      )}
    >
      <Collapsible.Trigger
        className={cn(
          // 头部样式
          "flex items-center gap-2 w-full",
          "px-3 py-2",
          "cursor-pointer select-none",
          "text-[13px] font-medium",
          // Hover 效果
          "hover:bg-muted/70",
          // Focus 状态
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
          "focus-visible:ring-inset"
        )}
        aria-expanded={isOpen}
      >
        {/* 工具图标 */}
        <Wrench className="h-4 w-4 shrink-0 text-muted-foreground" />

        {/* 工具名称 */}
        <span className="truncate">{toolName}</span>

        {/* 展开箭头 */}
        {hasInput && (
          <ChevronRight
            className={cn(
              "h-3.5 w-3.5 shrink-0 ml-auto text-muted-foreground",
              "transition-transform duration-150 ease-out",
              isOpen && "rotate-90"
            )}
          />
        )}
      </Collapsible.Trigger>

      {hasInput && (
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
              "border-t border-border",
              "bg-background"
            )}
          >
            <pre
              className={cn(
                // JSON 显示样式
                "font-mono text-xs",
                "whitespace-pre-wrap break-all",
                "text-muted-foreground"
              )}
            >
              {formattedInput}
            </pre>
          </div>
        </Collapsible.Content>
      )}
    </Collapsible.Root>
  );
}

export default ToolCall;


