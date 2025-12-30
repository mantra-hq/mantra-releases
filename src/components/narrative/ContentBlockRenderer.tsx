/**
 * ContentBlockRenderer - 内容块渲染器
 * Story 2.4: Task 4
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
import { ToolOutput } from "./ToolOutput";

export interface ContentBlockRendererProps {
  /** 内容块数据 */
  block: ContentBlock;
  /** 自定义 className */
  className?: string;
}

/**
 * ContentBlockRenderer 组件
 *
 * 渲染策略:
 * - text: 直接渲染文本 (支持 Markdown 格式的换行)
 * - thinking: 使用 ChainOfThought 组件
 * - tool_use: 使用 ToolCall 组件
 * - tool_result: 使用 ToolOutput 组件
 */
export function ContentBlockRenderer({
  block,
  className,
}: ContentBlockRendererProps) {
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

