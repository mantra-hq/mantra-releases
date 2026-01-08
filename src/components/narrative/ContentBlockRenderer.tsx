/**
 * ContentBlockRenderer - 内容块渲染器
 * Story 2.4: Task 4
 * Story 2.15: Task 8.1 (添加 ToolCallCard + TodoWriteCard 支持)
 *
 * 根据 ContentBlock.type 分发渲染对应组件
 * AC: #1, #2, #3, #4
 */

import { cn } from "@/lib/utils";
import type { ContentBlock } from "@/types/message";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Components } from "react-markdown";
import { ChainOfThought } from "./ChainOfThought";
import { ToolCall } from "./ToolCall";
import { ToolCallCard, type ToolCallStatus } from "./ToolCallCard";
import { TodoWriteCard } from "./TodoWriteCard";
import { ToolOutput } from "./ToolOutput";
import { useDetailPanelStore } from "@/stores/useDetailPanelStore";
import { useToolPairingContext } from "@/contexts/ToolPairingContext";
import { useEditorStore } from "@/stores/useEditorStore";
import { CodeBlockWithCopy } from "@/components/common/CodeBlockWithCopy";
import { ChevronRight, FileText } from "lucide-react";

/** 长 markdown 折叠阈值 (行数) */
const LONG_MARKDOWN_THRESHOLD = 15;
/** 折叠时显示的预览行数 */
const PREVIEW_LINES = 5;

export interface ContentBlockRendererProps {
  /** 内容块数据 */
  block: ContentBlock;
  /** 是否使用新版 ToolCallCard (Story 2.15) */
  useNewToolCard?: boolean;
  /** 自定义 className */
  className?: string;
}

/** 终端类工具列表 */
const TERMINAL_TOOLS = [
  "bash",
  "Bash",
  "shell",  // Codex CLI
  "Shell",
  "run_command",
  "execute_command",
  "send_command_input",
];

/** 检查是否为终端类工具 */
function isTerminalTool(toolName: string): boolean {
  return TERMINAL_TOOLS.some(t =>
    toolName.toLowerCase().includes(t.toLowerCase())
  );
}

/** 文件类工具列表 */
const FILE_TOOLS = [
  "Read",
  "read_file",
  "view_file",
  "Write",
  "write_to_file",
  "Edit",
  "replace_file_content",
  "multi_replace_file_content",
];

/** 检查是否为文件类工具 */
function isFileTool(toolName: string): boolean {
  return FILE_TOOLS.some(t =>
    toolName.toLowerCase().includes(t.toLowerCase())
  );
}

/** 从 toolInput 提取文件路径 */
function extractFilePath(toolInput?: Record<string, unknown>): string | null {
  if (!toolInput) return null;

  const pathKeys = [
    "file_path",
    "filePath",
    "path",
    "AbsolutePath",
    "TargetFile",
    "filename",
  ];

  for (const key of pathKeys) {
    const value = toolInput[key];
    if (typeof value === "string" && value.length > 0) {
      return value;
    }
  }
  return null;
}

/** 从 toolInput 提取命令 (支持字符串或数组格式) */
function extractCommand(toolInput?: Record<string, unknown>): string | undefined {
  if (!toolInput) return undefined;

  const cmd = toolInput.command;
  if (typeof cmd === "string") {
    return cmd;
  }
  if (Array.isArray(cmd)) {
    // Codex CLI: command is an array of strings
    return cmd.join(" ");
  }
  return undefined;
}

/**
 * ContentBlockRenderer 组件
 *
 * 渲染策略:
 * - text: 直接渲染文本 (支持 Markdown 格式的换行)
 * - thinking: 使用 ChainOfThought 组件
 * - tool_use: 根据工具类型使用不同组件
 *   - TodoWrite: 使用 TodoWriteCard
 *   - 其他: 使用 ToolCallCard
 * - tool_result: 使用 ToolOutput 组件
 */
export function ContentBlockRenderer({
  block,
  useNewToolCard = false,
  className,
}: ContentBlockRendererProps) {
  // 使用独立的选择器获取 action 函数，确保引用稳定
  const openToolDetail = useDetailPanelStore((state) => state.openToolDetail);
  const openTerminalDetail = useDetailPanelStore((state) => state.openTerminalDetail);
  const setHighlightedToolId = useDetailPanelStore((state) => state.setHighlightedToolId);
  const highlightedToolId = useDetailPanelStore((state) => state.highlightedToolId);
  const setActiveRightTab = useDetailPanelStore((state) => state.setActiveRightTab);

  // 文件类工具 - 打开文件到右侧代码面板
  const openTab = useEditorStore((state) => state.openTab);

  // Story 2.15: 获取配对信息
  const pairingContext = useToolPairingContext();

  switch (block.type) {
    case "text":
      // 检测是否为长 markdown
      const lines = block.content.split('\n');
      const isLongMarkdown = lines.length > LONG_MARKDOWN_THRESHOLD;

      // 长 markdown 显示折叠卡片
      if (isLongMarkdown) {
        const previewContent = lines.slice(0, PREVIEW_LINES).join('\n');
        const lineCount = lines.length;

        // 点击打开右侧预览
        const handleOpenPreview = () => {
          // 生成临时文件名（使用时间戳避免冲突）
          const tempPath = `markdown-preview-${Date.now()}.md`;
          openTab(tempPath, {
            preview: true,
            content: block.content,
          });
          setActiveRightTab('code');
        };

        return (
          <div
            className={cn(
              "group rounded-lg border border-border bg-muted/30 p-3",
              "hover:bg-muted/50 hover:border-primary/30 transition-colors cursor-pointer",
              className
            )}
            onClick={handleOpenPreview}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => e.key === 'Enter' && handleOpenPreview()}
          >
            {/* 预览内容 */}
            <div className="prose prose-sm dark:prose-invert max-w-none line-clamp-3 text-muted-foreground">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {previewContent}
              </ReactMarkdown>
            </div>

            {/* 展开提示 */}
            <div className="flex items-center gap-2 mt-2 pt-2 border-t border-border/50 text-xs text-muted-foreground">
              <FileText className="h-3.5 w-3.5" />
              <span>{lineCount} 行</span>
              <span className="flex-1" />
              <span className="group-hover:text-primary transition-colors flex items-center gap-1">
                点击展开查看
                <ChevronRight className="h-3.5 w-3.5" />
              </span>
            </div>
          </div>
        );
      }

      // Story 2.22: 自定义代码块组件，添加复制功能 (AC2)
      const markdownComponents: Components = {
        code({ className, children, ...props }) {
          const match = /language-(\w+)/.exec(className || "");
          const language = match ? match[1] : undefined;
          const codeString = String(children).replace(/\n$/, "");

          // 代码块检测逻辑:
          // 1. 有 language-xxx class → 明确是代码块 (来自 ```lang 语法)
          // 2. 包含换行符 → 多行代码视为代码块
          // 注: ReactMarkdown 对 ``` 代码块总会传递 className
          const isCodeBlock =
            className?.includes("language-") || codeString.includes("\n");

          if (isCodeBlock) {
            // 代码块使用 CodeBlockWithCopy (Task 5.5)
            return <CodeBlockWithCopy code={codeString} language={language} />;
          }

          // 内联代码保持原样渲染 (Task 5.4)
          return (
            <code className={className} {...props}>
              {children}
            </code>
          );
        },
        // 禁用默认的 pre 包装，因为 CodeBlockWithCopy 自带
        pre({ children }) {
          return <>{children}</>;
        },
      };

      return (
        <div
          className={cn(
            // Markdown 渲染样式
            "prose prose-sm dark:prose-invert max-w-none",
            // 自定义 prose 样式覆盖
            "prose-p:my-1 prose-p:leading-relaxed",
            "prose-pre:bg-transparent prose-pre:p-0",
            // 内联代码样式 - 确保浅色/深色模式下文字可读
            "prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-sm",
            "prose-code:text-foreground prose-code:font-normal",
            "prose-code:before:content-none prose-code:after:content-none",
            // 引用块样式 - 确保文字颜色足够深
            "prose-blockquote:border-l-primary prose-blockquote:text-foreground/80",
            // 列表样式
            "prose-ul:my-1 prose-ol:my-1",
            "prose-li:my-0",
            // 标题样式
            "prose-headings:mt-2 prose-headings:mb-1",
            // 表格样式 - 确保文字可读
            "prose-th:text-foreground prose-td:text-foreground/90",
            // 强调样式
            "prose-strong:text-foreground prose-em:text-foreground/90",
            className
          )}
        >
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={markdownComponents}
          >
            {block.content}
          </ReactMarkdown>
        </div>
      );

    case "thinking":
      return (
        <ChainOfThought
          content={block.content}
          className={className}
        />
      );

    case "tool_use":
      // TodoWrite 使用专属卡片
      if (block.toolName === "TodoWrite" && block.toolUseId) {
        return (
          <TodoWriteCard
            toolUseId={block.toolUseId}
            toolInput={block.toolInput}
            isHighlighted={highlightedToolId === block.toolUseId}
            onHover={setHighlightedToolId}
            className={className}
          />
        );
      }

      // 使用新版 ToolCallCard 支持详情面板交互
      if (useNewToolCard && block.toolUseId) {
        const toolName = block.toolName || "Unknown Tool";

        // Story 2.15: 从配对信息获取状态
        const pairInfo = pairingContext?.pairs.get(block.toolUseId);
        const hasOutput = Boolean(pairInfo?.outputContent);
        const isError = pairInfo?.isError ?? false;
        const status: ToolCallStatus = hasOutput
          ? (isError ? "error" : "success")
          : "pending";

        return (
          <ToolCallCard
            toolUseId={block.toolUseId}
            toolName={toolName}
            toolInput={block.toolInput}
            status={status}
            isHighlighted={highlightedToolId === block.toolUseId}
            onHover={setHighlightedToolId}
            onJumpToOutput={pairingContext ? () => {
              pairingContext.scrollTo(block.toolUseId!, "output");
            } : undefined}
            onClick={
              isTerminalTool(toolName) ? () => {
                // 终端类工具 - 点击卡片打开终端 Tab
                openTerminalDetail({
                  command: extractCommand(block.toolInput),
                  output: pairInfo?.outputContent ?? "",
                  isError: isError,
                });
              } : isFileTool(toolName) ? () => {
                // 文件类工具 - 点击卡片打开文件到右侧代码面板
                const filePath = extractFilePath(block.toolInput);
                if (filePath) {
                  // 使用 tool_result 的内容作为文件内容（如果有配对输出）
                  const fileContent = pairInfo?.outputContent;
                  openTab(filePath, {
                    preview: true,
                    content: fileContent || undefined,
                  });
                  setActiveRightTab("code");
                }
              } : undefined
            }
            onViewDetail={() => {
              // 所有工具 - 点击详情按钮打开工具详情 Tab
              openToolDetail({
                toolUseId: block.toolUseId!,
                toolName,
                toolInput: block.toolInput,
                toolOutput: pairInfo?.outputContent,
                isError: isError,
              });
            }}
            className={className}
          />
        );
      }
      // 回退到旧版 ToolCall
      return (
        <ToolCall
          toolName={block.toolName || "Unknown Tool"}
          toolInput={block.toolInput}
          className={className}
        />
      );

    case "tool_result":
      return (
        <ToolOutput
          content={block.content}
          isError={block.isError}
          filePath={block.associatedFilePath}
          toolName={block.associatedToolName}
          toolUseId={block.toolUseId}
          isHighlighted={highlightedToolId === block.toolUseId}
          onHover={setHighlightedToolId}
          className={className}
        />
      );

    default:
      // 未知类型，返回 null
      console.warn(`Unknown content block type: ${(block as ContentBlock).type}`);
      return null;
  }
}

export default ContentBlockRenderer;
