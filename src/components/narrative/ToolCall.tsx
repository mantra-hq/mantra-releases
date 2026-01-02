/**
 * ToolCall - 工具调用组件
 * Story 2.4: Task 2
 *
 * 显示 AI 的工具调用请求，包含工具名称和输入参数
 * AC: #3, #5, #6, #7
 */

import * as React from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronRight, Wrench, Code2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { useTimeTravelStore } from "@/stores/useTimeTravelStore";

/** 文件操作相关的工具名称 */
const FILE_TOOLS = ["Read", "Write", "Edit", "Glob", "Grep"];

/** 检查是否是文件操作工具 */
function isFileTool(toolName: string): boolean {
  return FILE_TOOLS.some(t => toolName.toLowerCase().includes(t.toLowerCase()));
}

/** 从工具输入中提取文件路径 */
function extractFilePath(input?: Record<string, unknown>): string | null {
  if (!input) return null;
  // 常见的文件路径字段名
  const pathKeys = ["file_path", "filePath", "path", "file"];
  for (const key of pathKeys) {
    if (typeof input[key] === "string") {
      return input[key] as string;
    }
  }
  return null;
}

/** 从工具输入中提取代码内容 */
function extractCodeContent(toolName: string, input?: Record<string, unknown>): string | null {
  if (!input) return null;

  // Write 工具：content 字段包含代码
  if (toolName.toLowerCase().includes("write") && typeof input.content === "string") {
    return input.content as string;
  }

  // Edit 工具：显示 new_string 作为代码内容
  if (toolName.toLowerCase().includes("edit") && typeof input.new_string === "string") {
    return input.new_string as string;
  }

  return null;
}

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

  // 从 store 获取 setCode 方法
  const setCode = useTimeTravelStore((state) => state.setCode);

  // 检查是否是文件操作工具
  const isFileOperation = isFileTool(toolName);
  const filePath = extractFilePath(toolInput);
  const codeContent = extractCodeContent(toolName, toolInput);

  // 处理"查看代码"按钮点击
  const handleViewCode = React.useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation(); // 阻止触发折叠
      if (filePath && codeContent) {
        setCode(codeContent, filePath);
      }
      // 对于 Read 工具，代码在 tool_result 中，用户需要点击结果查看
      // 不再显示占位文本
    },
    [filePath, codeContent, setCode]
  );

  // 只有当有代码内容时才显示查看按钮 (Write/Edit 工具)
  // Read 工具的代码在 tool_result 中，用户需要点击结果的查看按钮
  const showViewCodeButton = isFileOperation && filePath && codeContent;

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

        {/* 查看代码按钮 (仅 Write/Edit 工具且有代码内容时显示) */}
        {showViewCodeButton && (
          <button
            type="button"
            onClick={handleViewCode}
            className={cn(
              "ml-auto mr-2 p-1 rounded",
              "text-primary hover:bg-primary/10",
              "transition-colors duration-150",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            )}
            title="在代码面板中查看"
          >
            <Code2 className="h-4 w-4" />
          </button>
        )}

        {/* 展开箭头 */}
        {hasInput && (
          <ChevronRight
            className={cn(
              "h-3.5 w-3.5 shrink-0 text-muted-foreground",
              !showViewCodeButton && "ml-auto",
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









