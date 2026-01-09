/**
 * Copy Utils - 复制相关工具函数
 * Story 2.22: Task 2
 * Story 8.12: Task 4 - 使用 standardTool 替代 toolName 字符串匹配
 * Story 8.13: 使用新工具类型判断函数
 *
 * 根据消息类型智能提取主体内容用于复制
 */

import type { NarrativeMessage, ContentBlock } from "@/types/message";
import {
  isTerminalTool,
  isFileReadTool,
  isFileWriteTool,
  isFileEditTool,
  isSearchTool,
  isUnknownTool,
  isWebFetchTool,
  isWebSearchTool,
  isSubTaskTool,
  isTodoManageTool,
  isKnowledgeQueryTool,
  getToolPath,
  getToolCommand,
  getToolPattern,
  getOtherToolName,
} from "@/lib/tool-utils";
import type { StandardTool } from "@/types/message";

/**
 * 从 ToolCall 块提取主体内容（可直接使用的内容）
 * Story 8.12: 使用 standardTool 进行类型判断和内容提取
 * Story 8.13: 使用新工具类型判断函数
 */
function getToolCallContent(block: ContentBlock): string {
  const { standardTool, toolInput: input } = block;

  // Shell 命令工具 - 复制命令
  if (isTerminalTool(standardTool)) {
    const command = getToolCommand(standardTool);
    if (command) return command;
  }

  // 文件读取工具 - 复制文件路径
  if (isFileReadTool(standardTool)) {
    const path = getToolPath(standardTool);
    if (path) return path;
  }

  // 文件写入/编辑工具 - 复制文件路径
  if (isFileWriteTool(standardTool) || isFileEditTool(standardTool)) {
    const path = getToolPath(standardTool);
    if (path) return path;
  }

  // 搜索工具 - 复制搜索模式
  if (isSearchTool(standardTool)) {
    const pattern = getToolPattern(standardTool);
    if (pattern) return pattern;
  }

  // Story 8.13: 新工具类型的复制规则

  // WebFetch - 复制 URL
  if (isWebFetchTool(standardTool)) {
    const st = standardTool as Extract<StandardTool, { type: "web_fetch" }>;
    return st.url || "";
  }

  // WebSearch - 复制查询
  if (isWebSearchTool(standardTool)) {
    const st = standardTool as Extract<StandardTool, { type: "web_search" }>;
    return st.query || "";
  }

  // SubTask - 复制 prompt
  if (isSubTaskTool(standardTool)) {
    const st = standardTool as Extract<StandardTool, { type: "sub_task" }>;
    return st.prompt || "";
  }

  // KnowledgeQuery - 复制 question
  if (isKnowledgeQueryTool(standardTool)) {
    const st = standardTool as Extract<StandardTool, { type: "knowledge_query" }>;
    return st.question || "";
  }

  // TodoManage - 不复制（通常是结构化数据）
  if (isTodoManageTool(standardTool)) {
    return "";
  }

  // Unknown 类型工具 - 通用处理
  // 注意: 以下 toolName 字符串匹配是 Unknown 类型的启发式回退，
  // 不违背 StandardTool 完备化目标（AC3），因为仅对未知工具生效。
  if (isUnknownTool(standardTool)) {
    const toolName = getOtherToolName(standardTool);

    // 从 toolInput 尝试提取内容
    if (input) {
      // 尝试提取 URL 或查询 (Unknown 类型的启发式回退)
      if (toolName?.toLowerCase().includes("web")) {
        const url = input.url;
        const query = input.query;
        if (typeof url === "string") return url;
        if (typeof query === "string") return query;
      }

      // 尝试提取 prompt
      if (typeof input.prompt === "string") return input.prompt;
    }
  }

  // 通用回退：尝试从 toolInput 提取有意义的字段
  if (input) {
    const descKeys = ["description", "content", "message", "text"];
    for (const key of descKeys) {
      const value = input[key];
      if (typeof value === "string" && value.length > 0) {
        return value;
      }
    }
  }

  return "";
}

/**
 * 从 ToolOutput 块提取主体内容
 */
function getToolOutputContent(block: ContentBlock): string {
  // 直接返回输出内容
  return block.content || "";
}

/**
 * 获取消息的可复制主体内容
 *
 * 处理规则：
 * - text: 直接复制文本
 * - thinking: 复制思考内容
 * - tool_use: 根据工具类型提取主体（命令、路径、查询等）
 * - tool_result: 复制输出结果
 * - code_diff: 复制 diff 内容
 * - reference: 复制引用代码
 *
 * @param message - 消息对象
 * @returns 格式化后的主体内容
 */
export function getMessageCopyContent(message: NarrativeMessage): string {
  const parts: string[] = [];

  for (const block of message.content) {
    let content = "";

    switch (block.type) {
      case "text":
        content = block.content?.trim() || "";
        break;

      case "thinking":
        content = block.content?.trim() || "";
        break;

      case "tool_use":
        content = getToolCallContent(block);
        break;

      case "tool_result":
        content = getToolOutputContent(block);
        break;

      case "code_diff":
        content = block.diff || "";
        break;

      case "reference":
        content = block.content?.trim() || "";
        break;

      case "image":
        // 图片不复制
        break;

      default:
        break;
    }

    if (content) {
      parts.push(content);
    }
  }

  // 使用双换行连接各部分
  return parts.join("\n\n");
}

/**
 * 检查消息是否有可复制内容
 */
export function hasCopiableContent(message: NarrativeMessage): boolean {
  return message.content.some(
    (block) =>
      block.type === "text" ||
      block.type === "thinking" ||
      block.type === "tool_use" ||
      block.type === "tool_result" ||
      block.type === "code_diff" ||
      block.type === "reference"
  );
}
