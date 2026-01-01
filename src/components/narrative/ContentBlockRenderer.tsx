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
import { ChainOfThought } from "./ChainOfThought";
import { ToolCall } from "./ToolCall";
import { ToolCallCard } from "./ToolCallCard";
import { TodoWriteCard } from "./TodoWriteCard";
import { ToolOutput } from "./ToolOutput";
import { useDetailPanelStore } from "@/stores/useDetailPanelStore";

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
  "run_command",
  "execute_command",
  "send_command_input",
];

/** 文件类工具列表 */
const FILE_TOOLS = [
  "read_file",
  "Read",
  "view_file",
  "write_to_file",
  "Write",
  "Edit",
  "replace_file_content",
  "multi_replace_file_content",
];

/** 检查是否为终端类工具 */
function isTerminalTool(toolName: string): boolean {
  return TERMINAL_TOOLS.some(t =>
    toolName.toLowerCase().includes(t.toLowerCase())
  );
}

/** 检查是否为文件类工具 */
function isFileTool(toolName: string): boolean {
  return FILE_TOOLS.some(t =>
    toolName.toLowerCase().includes(t.toLowerCase())
  );
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

  switch (block.type) {
    case "text":
      return (
        <div
          className={cn(
            // Markdown 渲染样式
            "prose prose-sm dark:prose-invert max-w-none",
            // 自定义 prose 样式覆盖
            "prose-p:my-1 prose-p:leading-relaxed",
            "prose-pre:bg-muted prose-pre:text-foreground",
            "prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-sm",
            "prose-code:before:content-none prose-code:after:content-none",
            "prose-ul:my-1 prose-ol:my-1",
            "prose-li:my-0",
            "prose-headings:mt-2 prose-headings:mb-1",
            className
          )}
        >
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
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

        return (
          <ToolCallCard
            toolUseId={block.toolUseId}
            toolName={toolName}
            toolInput={block.toolInput}
            isHighlighted={highlightedToolId === block.toolUseId}
            onHover={setHighlightedToolId}
            onViewDetail={(toolUseId) => {
              // 根据工具类型决定打开哪个面板
              if (isTerminalTool(toolName)) {
                // 终端类工具 - 打开终端 tab
                // 注意: 这里暂时传入空输出，实际输出需要从 tool_result 获取
                openTerminalDetail({
                  command: block.toolInput?.command as string | undefined,
                  output: "", // 将由配对的 tool_result 填充
                  isError: false,
                });
              } else if (isFileTool(toolName)) {
                // 文件类工具 - 切换到代码 tab
                // TODO: 触发 TimeTravelStore 显示对应文件
                openToolDetail({
                  toolUseId,
                  toolName,
                  toolInput: block.toolInput,
                });
              } else {
                // 其他工具 - 通用详情
                openToolDetail({
                  toolUseId,
                  toolName,
                  toolInput: block.toolInput,
                });
              }
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
