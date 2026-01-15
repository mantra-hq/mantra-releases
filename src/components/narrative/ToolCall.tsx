/**
 * ToolCall - 工具调用组件
 * Story 2.4: Task 2
 * Story 2.26: 国际化支持
 *
 * 显示 AI 的工具调用请求，包含工具名称和输入参数
 * AC: #3, #5, #6, #7
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronRight, Wrench, Code2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { isFileTool, isFileEditTool, getToolPath, getToolContent } from "@/lib/tool-utils";
import type { StandardTool } from "@/types/message";
import { useEditorStore } from "@/stores/useEditorStore";
import { useDetailPanelStore } from "@/stores/useDetailPanelStore";
import { FileEditDiff } from "./FileEditDiff";

export interface ToolCallProps {
  /** 工具名称 */
  toolName: string;
  /** 工具输入参数 */
  toolInput?: Record<string, unknown>;
  /** 标准化工具 (Story 8.12) */
  standardTool?: StandardTool;
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
  standardTool,
  defaultOpen = false,
  className,
}: ToolCallProps) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = React.useState(defaultOpen);
  const formattedInput = formatJson(toolInput);
  const hasInput = toolInput && Object.keys(toolInput).length > 0;

  // 使用 EditorStore 的 openTab 和 DetailPanelStore 的 setActiveRightTab
  const openTab = useEditorStore((state) => state.openTab);
  const setActiveRightTab = useDetailPanelStore((state) => state.setActiveRightTab);

  // Story 8.12: 使用 standardTool 判断工具类型和提取内容
  const isFileOperation = isFileTool(standardTool);
  const filePath = getToolPath(standardTool);
  // file_write: 使用 content; file_edit: 使用 new_string (从 standardTool 获取)
  const codeContent = getToolContent(standardTool)
    ?? (isFileEditTool(standardTool) && standardTool?.type === "file_edit" ? standardTool.new_string : null);

  // 处理"查看代码"按钮点击 - 使用 openTab 打开文件
  const handleViewCode = React.useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation(); // 阻止触发折叠
      if (filePath && codeContent) {
        // 对于 file_edit 工具，同时传入 old_string 用于 diff 视图
        if (isFileEditTool(standardTool) && standardTool.type === "file_edit") {
          openTab(filePath, {
            preview: true,
            content: codeContent,
            previousContent: standardTool.old_string,
          });
        } else {
          openTab(filePath, {
            preview: true,
            content: codeContent,
          });
        }
        setActiveRightTab("code");
      }
    },
    [filePath, codeContent, standardTool, openTab, setActiveRightTab]
  );

  // 只有当有代码内容时才显示查看按钮 (Write/Edit 工具)
  // Read 工具的代码在 tool_result 中，用户需要点击结果的查看按钮
  const showViewCodeButton = isFileOperation && filePath && codeContent;

  // 阻止事件冒泡，避免触发父组件的消息选中逻辑
  const handleRootClick = React.useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
  }, []);

  return (
    <Collapsible.Root
      open={isOpen}
      onOpenChange={setIsOpen}
      className={cn(
        // 容器样式
        "bg-muted/50 rounded-lg my-2 overflow-hidden",
        className
      )}
      onClick={handleRootClick}
    >
      {/* 头部区域：使用 flex 容器包裹 Trigger 和独立按钮，避免 button 嵌套 */}
      <div className="flex items-center">
        <Collapsible.Trigger
          className={cn(
            // 头部样式
            "flex items-center gap-2 flex-1",
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
                "h-3.5 w-3.5 shrink-0 text-muted-foreground ml-auto",
                "transition-transform duration-150 ease-out",
                isOpen && "rotate-90"
              )}
            />
          )}
        </Collapsible.Trigger>

        {/* 查看代码按钮 - 独立于 Trigger 避免嵌套 button */}
        {showViewCodeButton && (
          <button
            type="button"
            onClick={handleViewCode}
            className={cn(
              "p-1 mr-2 rounded cursor-pointer",
              "text-primary hover:bg-primary/10",
              "transition-colors duration-150",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            )}
            title={t("message.viewInCodePanel")}
          >
            <Code2 className="h-4 w-4" />
          </button>
        )}
      </div>

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
            {/* Story 8.11 Task 9: FileEdit 类型显示 diff 视图 */}
            {isFileEditTool(standardTool) ? (
              <FileEditDiff
                filePath={standardTool.path}
                oldString={standardTool.old_string}
                newString={standardTool.new_string}
              />
            ) : (
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
            )}
          </div>
        </Collapsible.Content>
      )}
    </Collapsible.Root>
  );
}

export default ToolCall;












