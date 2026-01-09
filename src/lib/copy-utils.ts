/**
 * Copy Utils - 复制相关工具函数
 * Story 2.22: Task 2
 * Story 8.12: Task 4 - 使用 standardTool 替代 toolName 字符串匹配
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
  isOtherTool,
  getToolPath,
  getToolCommand,
  getToolPattern,
  getOtherToolName,
} from "@/lib/tool-utils";

/**
 * 从 ToolCall 块提取主体内容（可直接使用的内容）
 * Story 8.12: 使用 standardTool 进行类型判断和内容提取
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

  // Other 类型工具 - 特殊处理
  if (isOtherTool(standardTool)) {
    const toolName = getOtherToolName(standardTool);

    // TodoWrite - 不复制（通常是结构化数据）
    if (toolName === "TodoWrite") {
      return "";
    }

    // 从 toolInput 尝试提取内容
    if (input) {
      // WebFetch/WebSearch - 复制 URL 或查询
      if (toolName?.toLowerCase().includes("web")) {
        const url = input.url;
        const query = input.query;
        if (typeof url === "string") return url;
        if (typeof query === "string") return query;
      }

      // Task 工具 - 复制 prompt
      if (toolName?.toLowerCase() === "task") {
        const prompt = input.prompt;
        if (typeof prompt === "string") return prompt;
      }
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
