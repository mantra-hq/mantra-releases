/**
 * Tool Utils - 工具类型判断工具函数
 * Story 8.12: Task 1
 * Story 8.13: 扩展新工具类型支持
 *
 * 基于 StandardToolType 的统一工具类型判断模块
 * 替代散落在各组件中的 toolName.toLowerCase().includes() 逻辑
 */

import type { StandardTool, StandardToolFileEdit } from "@/types/message";

/**
 * 标准工具类型常量
 * Story 8.13: 扩展完整应用级概念
 */
export const StandardToolTypes = {
  // 文件操作
  FILE_READ: "file_read",
  FILE_WRITE: "file_write",
  FILE_EDIT: "file_edit",
  // 终端操作
  SHELL_EXEC: "shell_exec",
  // 搜索操作
  FILE_SEARCH: "file_search",
  CONTENT_SEARCH: "content_search",
  // 网络操作 (Story 8.13)
  WEB_FETCH: "web_fetch",
  WEB_SEARCH: "web_search",
  // 知识查询 (Story 8.13)
  KNOWLEDGE_QUERY: "knowledge_query",
  // 代码操作 (Story 8.13)
  CODE_EXEC: "code_exec",
  DIAGNOSTIC: "diagnostic",
  NOTEBOOK_EDIT: "notebook_edit",
  // 任务管理 (Story 8.13)
  TODO_MANAGE: "todo_manage",
  // 代理操作 (Story 8.13)
  SUB_TASK: "sub_task",
  // 用户交互 (Story 8.13)
  USER_PROMPT: "user_prompt",
  // 计划模式 (Story 8.13)
  PLAN_MODE: "plan_mode",
  // 技能调用 (Story 8.13)
  SKILL_INVOKE: "skill_invoke",
  // 未知工具 (Story 8.13: Other → Unknown)
  UNKNOWN: "unknown",
  /** @deprecated 使用 UNKNOWN 代替 */
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
 * Story 8.11: 改为 Type Guard 以支持类型收窄
 * @param standardTool 标准化工具对象
 */
export function isFileEditTool(
  standardTool: StandardTool | undefined
): standardTool is StandardToolFileEdit {
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
 * @deprecated 使用 isUnknownTool 代替
 */
export function isOtherTool(standardTool: StandardTool | undefined): boolean {
  return (
    standardTool?.type === StandardToolTypes.OTHER ||
    standardTool?.type === StandardToolTypes.UNKNOWN
  );
}

/**
 * 检查是否为未知类型工具
 * Story 8.13: 新增，替代 isOtherTool
 * @param standardTool 标准化工具对象
 */
export function isUnknownTool(standardTool: StandardTool | undefined): boolean {
  return (
    standardTool?.type === StandardToolTypes.UNKNOWN ||
    standardTool?.type === StandardToolTypes.OTHER
  );
}

// === Story 8.13: 新增工具类型判断函数 ===

/**
 * 检查是否为网络获取工具
 * @param standardTool 标准化工具对象
 */
export function isWebFetchTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.WEB_FETCH;
}

/**
 * 检查是否为网络搜索工具
 * @param standardTool 标准化工具对象
 */
export function isWebSearchTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.WEB_SEARCH;
}

/**
 * 检查是否为网络类工具 (网络获取/网络搜索)
 * @param standardTool 标准化工具对象
 */
export function isWebTool(standardTool: StandardTool | undefined): boolean {
  if (!standardTool) return false;
  return (
    standardTool.type === StandardToolTypes.WEB_FETCH ||
    standardTool.type === StandardToolTypes.WEB_SEARCH
  );
}

/**
 * 检查是否为知识查询工具
 * @param standardTool 标准化工具对象
 */
export function isKnowledgeQueryTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.KNOWLEDGE_QUERY;
}

/**
 * 检查是否为代码执行工具
 * @param standardTool 标准化工具对象
 */
export function isCodeExecTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.CODE_EXEC;
}

/**
 * 检查是否为诊断工具
 * @param standardTool 标准化工具对象
 */
export function isDiagnosticTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.DIAGNOSTIC;
}

/**
 * 检查是否为笔记本编辑工具
 * @param standardTool 标准化工具对象
 */
export function isNotebookEditTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.NOTEBOOK_EDIT;
}

/**
 * 检查是否为任务管理工具
 * @param standardTool 标准化工具对象
 */
export function isTodoManageTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.TODO_MANAGE;
}

/**
 * 检查是否为子任务/代理工具
 * @param standardTool 标准化工具对象
 */
export function isSubTaskTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.SUB_TASK;
}

/**
 * 检查是否为用户问询工具
 * @param standardTool 标准化工具对象
 */
export function isUserPromptTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.USER_PROMPT;
}

/**
 * 检查是否为计划模式工具
 * @param standardTool 标准化工具对象
 */
export function isPlanModeTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.PLAN_MODE;
}

/**
 * 检查是否为技能调用工具
 * @param standardTool 标准化工具对象
 */
export function isSkillInvokeTool(standardTool: StandardTool | undefined): boolean {
  return standardTool?.type === StandardToolTypes.SKILL_INVOKE;
}

/**
 * 检查是否为代理/自动化类工具 (子任务/计划模式/技能调用)
 * @param standardTool 标准化工具对象
 */
export function isAgentTool(standardTool: StandardTool | undefined): boolean {
  if (!standardTool) return false;
  return (
    standardTool.type === StandardToolTypes.SUB_TASK ||
    standardTool.type === StandardToolTypes.PLAN_MODE ||
    standardTool.type === StandardToolTypes.SKILL_INVOKE
  );
}

/**
 * 检查是否为用户交互类工具 (用户问询/任务管理)
 * @param standardTool 标准化工具对象
 */
export function isInteractiveTool(standardTool: StandardTool | undefined): boolean {
  if (!standardTool) return false;
  return (
    standardTool.type === StandardToolTypes.USER_PROMPT ||
    standardTool.type === StandardToolTypes.TODO_MANAGE
  );
}

// === Story 8.13: Unknown 日志记录 ===

/**
 * 记录 Unknown 工具出现
 * 开发环境下帮助发现需要扩展的新类型
 * @param standardTool Unknown 类型工具对象
 */
export function logUnknownTool(standardTool: StandardTool | undefined): void {
  if (!standardTool || !isUnknownTool(standardTool)) return;

  // 仅在开发环境记录
  if (import.meta.env.DEV) {
    const name =
      standardTool.type === "unknown" || standardTool.type === "other"
        ? (standardTool as { name: string }).name
        : "unknown";
    console.warn(
      `[StandardTool] Unknown tool detected: "${name}". Consider extending StandardTool enum.`,
      standardTool
    );
  }
}

/**
 * 获取 Unknown 工具的名称
 * Story 8.13: 兼容 unknown 和 other 类型
 * @param standardTool 标准化工具对象
 * @returns 工具名称，如果不是 unknown/other 类型则返回 undefined
 */
export function getUnknownToolName(standardTool: StandardTool | undefined): string | undefined {
  if (!standardTool) return undefined;
  if (standardTool.type === "unknown" || standardTool.type === "other") {
    return (standardTool as { name: string }).name;
  }
  return undefined;
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
