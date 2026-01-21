/**
 * Message Utilities - 消息处理工具函数
 * 
 * 提供消息内容处理的共享工具函数
 * 从 Story 10.4 代码审查中抽取，消除重复代码
 * Story 10.6/10-2 Fix: 扩展支持所有内容类型
 */

import type { ContentBlock, ContentBlockType } from "@/types/message";

/**
 * 获取消息的纯文本内容
 * 从 ContentBlock 数组中提取所有 text 类型块的内容并合并
 * 
 * @param content - 消息内容块数组
 * @returns 合并后的纯文本内容
 */
export function getMessageTextContent(content: ContentBlock[]): string {
  return content
    .filter((block) => block.type === "text")
    .map((block) => block.content)
    .join("\n");
}

/**
 * 获取消息的完整显示内容
 * Story 10.6/10-2 Fix: 从所有内容块类型中提取可显示的文本
 * 
 * 支持的内容类型:
 * - text: 纯文本内容
 * - thinking: 思考过程内容
 * - tool_use: 工具调用 (显示工具名 + 参数摘要)
 * - tool_result: 工具结果内容
 * - code_diff: 代码差异
 * - code_suggestion: 代码建议
 * - reference: 代码引用
 * - image: 图片 (返回占位符)
 * 
 * @param content - 消息内容块数组
 * @returns 合并后的显示内容
 */
export function getMessageDisplayContent(content: ContentBlock[]): string {
  const parts: string[] = [];

  for (const block of content) {
    const extracted = extractBlockContent(block);
    if (extracted) {
      parts.push(extracted);
    }
  }

  return parts.join("\n");
}

/**
 * 从单个内容块提取文本内容
 * @param block - 内容块
 * @returns 提取的文本内容
 */
function extractBlockContent(block: ContentBlock): string {
  switch (block.type) {
    case "text":
      return block.content || "";

    case "thinking":
      // 思考过程内容
      return block.content || "";

    case "tool_use":
      // 工具调用: 显示工具名 + 参数摘要
      return formatToolUseContent(block);

    case "tool_result":
      // 工具结果: 优先显示 displayContent，否则显示 content
      return block.displayContent || block.content || "";

    case "code_diff":
      // 代码差异
      return block.diff || block.content || "";

    case "code_suggestion":
      // 代码建议
      return block.code || block.content || "";

    case "reference":
      // 代码引用: 显示文件路径和行号
      return formatReferenceContent(block);

    case "image":
      // 图片: 返回占位符
      return "[Image]";

    default:
      // 未知类型: 尝试返回 content 字段
      return block.content || "";
  }
}

/**
 * 格式化工具调用内容
 */
function formatToolUseContent(block: ContentBlock): string {
  const parts: string[] = [];

  // 工具名称
  const toolName = block.toolName || block.displayName || "Unknown Tool";
  parts.push(`[${toolName}]`);

  // 工具参数摘要 (最多显示前 200 字符)
  if (block.toolInput) {
    const inputStr = JSON.stringify(block.toolInput);
    const summary = inputStr.length > 200 
      ? inputStr.slice(0, 200) + "..." 
      : inputStr;
    parts.push(summary);
  } else if (block.content) {
    parts.push(block.content);
  }

  return parts.join(" ");
}

/**
 * 格式化代码引用内容
 */
function formatReferenceContent(block: ContentBlock): string {
  const parts: string[] = [];

  if (block.filePath) {
    let ref = block.filePath;
    if (block.startLine !== undefined) {
      ref += `:${block.startLine}`;
      if (block.endLine !== undefined && block.endLine !== block.startLine) {
        ref += `-${block.endLine}`;
      }
    }
    if (block.symbol) {
      ref += ` (${block.symbol})`;
    }
    parts.push(`[Ref: ${ref}]`);
  }

  if (block.content) {
    parts.push(block.content);
  }

  return parts.join("\n");
}

/**
 * 检测消息的主要内容类型
 * Story 10.6/10-2 Fix: 用于确定消息在压缩模式中的显示类型
 * 
 * @param content - 消息内容块数组
 * @returns 主要内容类型
 */
export function detectPrimaryContentType(content: ContentBlock[]): ContentBlockType | null {
  if (!content || content.length === 0) return null;

  // 优先级: tool_use > tool_result > thinking > code_diff > code_suggestion > text
  const priorities: ContentBlockType[] = [
    "tool_use",
    "tool_result", 
    "thinking",
    "code_diff",
    "code_suggestion",
    "reference",
    "image",
    "text",
  ];

  for (const type of priorities) {
    if (content.some((block) => block.type === type)) {
      return type;
    }
  }

  return content[0]?.type || null;
}

/**
 * 检测消息是否包含指定类型的内容块
 */
export function hasContentType(content: ContentBlock[], type: ContentBlockType): boolean {
  return content.some((block) => block.type === type);
}
