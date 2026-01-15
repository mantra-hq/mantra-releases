/**
 * ToolOutput - 工具输出组件
 * Story 2.4: Task 3
 * Story 2.26: 国际化支持
 * Story 8.12: Task 7 - 删除 stripLineNumbers (移到 Parser 层)
 *
 * 显示工具执行结果，支持成功/错误两种状态
 * AC: #4, #5, #6, #7
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronRight, Check, X, Code2, FileText, Edit3, Terminal, CheckCircle2, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { useEditorStore } from "@/stores/useEditorStore";
import { useDetailPanelStore } from "@/stores/useDetailPanelStore";
import type { ToolResultData } from "@/types/message";

export interface ToolOutputProps {
  /** 输出内容 (原始 content) */
  content: string;
  /** 是否为错误结果 */
  isError?: boolean;
  /** 默认是否展开 */
  defaultOpen?: boolean;
  /** 关联的文件路径 (用于代码面板显示) */
  filePath?: string;
  /** 关联的工具名称 (用于推断语言类型) */
  toolName?: string;
  /** 工具调用 ID (用于配对高亮和跳转) */
  toolUseId?: string;
  /** 是否高亮 (配对悬停时) */
  isHighlighted?: boolean;
  /** 悬停回调 */
  onHover?: (toolUseId: string | null) => void;
  /** 自定义 className */
  className?: string;
  // === Story 8.11: 新增字段 ===
  /** 结构化结果 (AC: #4) */
  structuredResult?: ToolResultData;
  /** 用户决策 (AC: #5) */
  userDecision?: string;
  // === Story 8.19: 显示内容 ===
  /** 显示内容 (优先于 content，用于提取 JSON 中的实际内容) */
  displayContent?: string;
}

/**
 * Story 8.12: 从 structuredResult 获取文件路径用于代码显示
 * 添加防御性检查：file_path 可能为 undefined（数据不完整）
 */
function getFilePathFromResult(result?: ToolResultData, fallbackPath?: string): string {
  if (result) {
    switch (result.type) {
      case "file_read":
      case "file_write":
      case "file_edit":
        // 防御性检查：file_path 可能为 undefined
        if (result.file_path) {
          return result.file_path;
        }
        break;
    }
  }
  return fallbackPath || "tool-output.txt";
}

/**
 * Story 8.11: 渲染 structuredResult 摘要 (AC #4)
 *
 * - FileRead: 显示 "读取 {path} L{start}-L{end} ({numLines}/{totalLines} 行)"
 * - FileWrite: 显示 "写入 {path}"
 * - FileEdit: 显示 "编辑 {path}"
 * - ShellExec: 显示退出码徽章 (绿色/红色)
 *
 * @param result 结构化工具结果
 * @param t i18n 翻译函数
 * @returns 渲染的摘要节点，未知类型返回 null
 */
function renderStructuredResultSummary(
  result: ToolResultData,
  t: (key: string, options?: Record<string, unknown>) => string
): React.ReactNode {
  switch (result.type) {
    case "file_read": {
      // 防御性检查：file_path 可能为 undefined（数据不完整）
      if (!result.file_path) return null;
      const fileName = result.file_path.split("/").pop() || result.file_path;
      // Fix: endLine = startLine + numLines - 1 (e.g., L10 + 5 lines = L10-L14)
      const lineRange = result.start_line !== undefined && result.num_lines !== undefined
        ? `L${result.start_line}-L${result.start_line + result.num_lines - 1}`
        : "";
      const lineInfo = result.num_lines !== undefined && result.total_lines !== undefined
        ? t("message.lines", { count: `${result.num_lines}/${result.total_lines}` })
        : "";
      return (
        <>
          <FileText className="h-3.5 w-3.5 shrink-0" />
          <span>{t("message.readFile", { fileName })} {lineRange} {lineInfo}</span>
        </>
      );
    }
    case "file_write": {
      // 防御性检查：file_path 可能为 undefined（数据不完整）
      if (!result.file_path) return null;
      const fileName = result.file_path.split("/").pop() || result.file_path;
      return (
        <>
          <FileText className="h-3.5 w-3.5 shrink-0" />
          <span>{t("message.writeFile", { fileName })}</span>
        </>
      );
    }
    case "file_edit": {
      // 防御性检查：file_path 可能为 undefined（数据不完整）
      if (!result.file_path) return null;
      const fileName = result.file_path.split("/").pop() || result.file_path;
      return (
        <>
          <Edit3 className="h-3.5 w-3.5 shrink-0" />
          <span>{t("message.editFile", { fileName })}</span>
        </>
      );
    }
    case "shell_exec": {
      const isSuccess = result.exit_code === 0;
      return (
        <>
          <Terminal className="h-3.5 w-3.5 shrink-0" />
          <span>{t("message.shellExec")}</span>
          <span className={cn(
            "ml-1 px-1.5 py-0.5 rounded text-[10px] font-mono",
            isSuccess ? "bg-success/20 text-success" : "bg-destructive/20 text-destructive"
          )}>
            exit {result.exit_code ?? "?"}
          </span>
        </>
      );
    }
    default:
      // 未知类型返回 null，由调用方处理回退逻辑
      return null;
  }
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
  filePath,
  toolName: _toolName, // Story 8.12: 保留接口兼容性，但不再使用此字段
  toolUseId,
  isHighlighted = false,
  onHover,
  className,
  structuredResult,
  userDecision,
  displayContent,
}: ToolOutputProps) {
  // Story 8.19: 优先使用 displayContent，否则使用 content
  // 注意：使用 !== undefined 而不是 ||，避免空字符串被视为 falsy
  const effectiveContent = displayContent !== undefined && displayContent !== null 
    ? displayContent 
    : content;
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = React.useState(defaultOpen);

  // 使用 EditorStore 的 openTab 和 DetailPanelStore 的 setActiveRightTab
  const openTab = useEditorStore((state) => state.openTab);
  const setActiveRightTab = useDetailPanelStore((state) => state.setActiveRightTab);

  // 悬停处理
  const handleMouseEnter = React.useCallback(() => {
    if (toolUseId) onHover?.(toolUseId);
  }, [onHover, toolUseId]);

  const handleMouseLeave = React.useCallback(() => {
    onHover?.(null);
  }, [onHover]);

  // 截断长内容的预览
  const previewLength = 100;
  const isLongContent = effectiveContent.length > previewLength;
  const previewContent = isLongContent
    ? effectiveContent.slice(0, previewLength) + "..."
    : effectiveContent;

  // Story 8.19: 通过 structuredResult 类型判断是否是文件操作
  const isFileOperation = structuredResult?.type === "file_read" ||
    structuredResult?.type === "file_write" ||
    structuredResult?.type === "file_edit";

  // 处理"查看代码"按钮点击
  const handleViewCode = React.useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      const path = getFilePathFromResult(structuredResult, filePath);
      // Story 8.19: 使用 effectiveContent (优先 displayContent)
      openTab(path, {
        preview: true,
        content: effectiveContent,
      });
      setActiveRightTab("code");
    },
    [effectiveContent, filePath, structuredResult, openTab, setActiveRightTab]
  );

  // Story 8.19: 简化判断 - 通过 structuredResult 或 filePath 判断
  const showViewCodeButton = !isError && (isFileOperation || filePath);

  // 阻止事件冒泡，避免触发父组件的消息选中逻辑
  const handleRootClick = React.useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
  }, []);

  return (
    <Collapsible.Root
      open={isOpen}
      onOpenChange={setIsOpen}
      data-tool-output-id={toolUseId}
      className={cn(
        // 容器样式
        "rounded-lg my-2 overflow-hidden",
        // 状态变体
        isError
          ? "border-l-[3px] border-l-destructive bg-destructive/5"
          : "border-l-[3px] border-l-success bg-success/5",
        // 高亮状态 (配对悬停)
        isHighlighted && "ring-2 ring-primary/50",
        className
      )}
      aria-label={isError ? t("message.toolExecuteFailed") : t("message.toolExecuteSuccess")}
      onClick={handleRootClick}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* 头部区域：使用 flex 容器包裹 Trigger 和独立按钮 */}
      <div className="flex items-center">
        {/* Story 8.11: userDecision 徽章 (AC #5) */}
        {userDecision && (
          <div className={cn(
            "ml-2 flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium shrink-0",
            userDecision === "approved"
              ? "bg-success/10 text-success"
              : "bg-destructive/10 text-destructive"
          )}>
            {userDecision === "approved" ? (
              <>
                <CheckCircle2 className="h-3 w-3" />
                <span>{t("message.approved")}</span>
              </>
            ) : (
              <>
                <XCircle className="h-3 w-3" />
                <span>{t("message.rejected")}</span>
              </>
            )}
          </div>
        )}
        <Collapsible.Trigger
          className={cn(
            // 头部样式
            "flex items-center gap-2 flex-1",
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

          {/* Story 8.11: structuredResult 摘要 (AC #4) */}
          {/* Story 8.12 fix: 当 renderStructuredResultSummary 返回 null 时回退到 previewContent */}
          {(() => {
            const summary = structuredResult ? renderStructuredResultSummary(structuredResult, t) : null;
            if (summary) {
              return (
                <span className={cn("flex items-center gap-1.5 flex-1 truncate font-mono text-xs", "text-muted-foreground")}>
                  {summary}
                </span>
              );
            }
            return (
              <span className={cn("flex-1 truncate font-mono text-xs", "text-muted-foreground")}>
                {isOpen ? (isError ? t("message.errorDetails") : t("message.executeResult")) : previewContent}
              </span>
            );
          })()}

          {/* 展开箭头 */}
          <ChevronRight
            className={cn(
              "h-3.5 w-3.5 shrink-0 text-muted-foreground",
              "transition-transform duration-150 ease-out",
              isOpen && "rotate-90"
            )}
          />
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
            {/* Story 8.19: 使用 effectiveContent */}
            {effectiveContent}
          </pre>
        </div>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}

export default ToolOutput;












