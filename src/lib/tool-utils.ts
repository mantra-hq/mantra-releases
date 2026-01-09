/**
 * Tool Utils - 工具类型判断工具函数
 * Story 8.12: Task 1
 *
 * 基于 StandardToolType 的统一工具类型判断模块
 * 替代散落在各组件中的 toolName.toLowerCase().includes() 逻辑
 */

import type { StandardTool } from "@/types/message";

/**
 * 标准工具类型常量
 */
export const StandardToolTypes = {
  FILE_READ: "file_read",
  FILE_WRITE: "file_write",
  FILE_EDIT: "file_edit",
  SHELL_EXEC: "shell_exec",
  FILE_SEARCH: "file_search",
  CONTENT_SEARCH: "content_search",
  OTHER: "other",
} as const;

export type StandardToolType = (typeof StandardToolTypes)[keyof typeof StandardToolTypes];

/**
 * 检查是否为文件读取工具
 * @param standardTool 标准化工具对象
 */
export function isFileReadTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.FILE_READ;
}

/**
 * 检查是否为文件写入工具
 * @param standardTool 标准化工具对象
 */
export function isFileWriteTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.FILE_WRITE;
}

/**
 * 检查是否为文件编辑工具
 * @param standardTool 标准化工具对象
 */
export function isFileEditTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.FILE_EDIT;
}

/**
 * 检查是否为文件类工具 (读取/写入/编辑)
 * @param standardTool 标准化工具对象
 */
export function isFileTool(standardTool: StandardTool | undefined): boolean {
  if (!standardTool) return false;
  return (
    standardTool.type === StandardToolTypes.FILE_READ ||
    standardTool.type === StandardToolTypes.FILE_WRITE ||
    standardTool.type === StandardToolTypes.FILE_EDIT
  );
}

/**
 * 检查是否为 Shell 命令执行工具
 * @param standardTool 标准化工具对象
 */
export function isTerminalTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.SHELL_EXEC;
}

/**
 * 检查是否为文件搜索工具
 * @param standardTool 标准化工具对象
 */
export function isFileSearchTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.FILE_SEARCH;
}

/**
 * 检查是否为内容搜索工具
 * @param standardTool 标准化工具对象
 */
export function isContentSearchTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.CONTENT_SEARCH;
}

/**
 * 检查是否为搜索类工具 (文件搜索/内容搜索)
 * @param standardTool 标准化工具对象
 */
export function isSearchTool(standardTool: StandardTool | undefined): boolean {
  if (!standardTool) return false;
  return (
    standardTool.type === StandardToolTypes.FILE_SEARCH ||
    standardTool.type === StandardToolTypes.CONTENT_SEARCH
  );
}

/**
 * 检查是否为其他类型工具
 * @param standardTool 标准化工具对象
 */
export function isOtherTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.OTHER;
}

/**
 * 获取工具关联的文件路径
 * @param standardTool 标准化工具对象
 * @returns 文件路径，如果工具没有关联路径则返回 undefined
 */
export function getToolPath(standardTool: StandardTool | undefined): string | undefined {
  if (!standardTool) return undefined;

  switch (standardTool.type) {
    case StandardToolTypes.FILE_READ:
    case StandardToolTypes.FILE_WRITE:
    case StandardToolTypes.FILE_EDIT:
      return standardTool.path;
    case StandardToolTypes.FILE_SEARCH:
    case StandardToolTypes.CONTENT_SEARCH:
      return standardTool.path;
    default:
      return undefined;
  }
}

/**
 * 获取 Shell 命令
 * @param standardTool 标准化工具对象
 * @returns 命令字符串，如果不是 shell_exec 则返回 undefined
 */
export function getToolCommand(standardTool: StandardTool | undefined): string | undefined {
  if (!standardTool || standardTool.type !== StandardToolTypes.SHELL_EXEC) {
    return undefined;
  }
  return standardTool.command;
}

/**
 * 获取文件内容 (file_write 工具)
 * @param standardTool 标准化工具对象
 * @returns 文件内容，如果不是 file_write 则返回 undefined
 */
export function getToolContent(standardTool: StandardTool | undefined): string | undefined {
  if (!standardTool || standardTool.type !== StandardToolTypes.FILE_WRITE) {
    return undefined;
  }
  return standardTool.content;
}

/**
 * 获取搜索模式
 * @param standardTool 标准化工具对象
 * @returns 搜索模式，如果不是搜索工具则返回 undefined
 */
export function getToolPattern(standardTool: StandardTool | undefined): string | undefined {
  if (!standardTool) return undefined;

  if (
    standardTool.type === StandardToolTypes.FILE_SEARCH ||
    standardTool.type === StandardToolTypes.CONTENT_SEARCH
  ) {
    return standardTool.pattern;
  }
  return undefined;
}

/**
 * 获取 Other 类型工具的原始名称
 * @param standardTool 标准化工具对象
 * @returns 原始工具名称，如果不是 other 类型则返回 undefined
 */
export function getOtherToolName(standardTool: StandardTool | undefined): string | undefined {
  if (!standardTool || standardTool.type !== StandardToolTypes.OTHER) {
    return undefined;
  }
  return standardTool.name;
}

// === Story 8.12 Task 10: 从 file-path-extractor 迁移的函数 ===

import type { NarrativeMessage, ContentBlock } from "@/types/message";

/** 文件路径提取结果 */
export interface FilePathResult {
  /** 提取到的文件路径 */
  path: string;
  /** 路径来源 */
  source: "tool_use" | "tool_result" | "history";
  /** 置信度 */
  confidence: "high" | "medium" | "low";
}

/**
 * 将绝对路径转换为相对于仓库的路径
 * Story 8.12: 从 file-path-extractor 迁移
 * 
 * @param absolutePath 绝对路径
 * @param repoPath 仓库根路径
 * @returns 相对路径
 */
export function toRelativePath(absolutePath: string, repoPath: string): string {
  // 规范化路径分隔符
  const normalizedAbsolute = absolutePath.replace(/\\/g, "/");
  const normalizedRepo = repoPath.replace(/\\/g, "/").replace(/\/$/, "");

  // 如果已经是相对路径，直接返回
  if (
    !normalizedAbsolute.startsWith("/") &&
    !normalizedAbsolute.match(/^[A-Za-z]:/)
  ) {
    return normalizedAbsolute;
  }

  // 检查是否以仓库路径开头
  if (normalizedAbsolute.startsWith(normalizedRepo)) {
    return normalizedAbsolute.slice(normalizedRepo.length).replace(/^\//, "");
  }

  // 无法转换，返回原路径（尝试作为相对路径使用）
  return normalizedAbsolute.replace(/^\//, "");
}

/**
 * 验证是否为有效的文件路径
 */
function isValidFilePath(path: string): boolean {
  if (!path || path.length < 2 || path.length > 500) {
    return false;
  }
  // 必须包含扩展名
  if (!path.includes(".")) {
    return false;
  }
  // 排除 URL
  if (path.startsWith("http://") || path.startsWith("https://")) {
    return false;
  }
  // 排除无效字符 (Windows 不允许的字符)
  if (/[<>:|?*]/.test(path)) {
    return false;
  }
  return true;
}

/**
 * 从 tool_use 块提取文件路径
 * Story 8.12: 使用 standardTool.path
 */
function extractFilePathFromToolUse(block: ContentBlock): string | null {
  if (block.type !== "tool_use") {
    return null;
  }
  const path = getToolPath(block.standardTool);
  if (path && isValidFilePath(path)) {
    return path;
  }
  return null;
}

/**
 * 从 tool_result 块提取 associatedFilePath
 */
function extractFilePathFromToolResult(block: ContentBlock): string | null {
  if (block.type !== "tool_result") {
    return null;
  }
  if (block.associatedFilePath && isValidFilePath(block.associatedFilePath)) {
    return block.associatedFilePath;
  }
  return null;
}

/**
 * 按优先级从单条消息中提取文件路径
 * Story 8.12: 简化版本，只使用 standardTool.path 和 associatedFilePath
 * 
 * @param message 单条消息
 * @returns 文件路径结果，未找到返回 null
 */
export function extractFilePathWithPriority(message: NarrativeMessage): FilePathResult | null {
  // 1. tool_use 中的 standardTool.path (最高优先级)
  for (const block of message.content) {
    const path = extractFilePathFromToolUse(block);
    if (path) {
      return { path, source: "tool_use", confidence: "high" };
    }
  }

  // 2. tool_result 中的 associatedFilePath
  for (const block of message.content) {
    const path = extractFilePathFromToolResult(block);
    if (path) {
      return { path, source: "tool_result", confidence: "high" };
    }
  }

  return null;
}

/**
 * 从消息列表中查找最近的文件路径
 * Story 8.12: 从 file-path-extractor 迁移并简化
 * 
 * 搜索策略：从当前消息向前搜索，找到第一个包含文件路径的消息
 * 
 * @param messages 消息列表
 * @param fromIndex 起始索引（包含）
 * @returns 文件路径结果，未找到返回 null
 */
export function findRecentFilePathEnhanced(
  messages: NarrativeMessage[],
  fromIndex: number
): FilePathResult | null {
  // 从当前消息向前搜索
  for (let i = fromIndex; i >= 0; i--) {
    const result = extractFilePathWithPriority(messages[i]);
    if (result) {
      // 如果不是当前消息，标记为 history 来源
      return {
        ...result,
        source: i === fromIndex ? result.source : "history",
      };
    }
  }
  return null;
}
