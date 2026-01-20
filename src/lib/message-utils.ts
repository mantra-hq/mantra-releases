/**
 * Message Utilities - 消息处理工具函数
 * 
 * 提供消息内容处理的共享工具函数
 * 从 Story 10.4 代码审查中抽取，消除重复代码
 */

import type { ContentBlock } from "@/types/message";

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
