/**
 * file-path-extractor.ts - 文件路径提取工具模块
 * Story 2.12: Task 1
 *
 * 从消息内容中提取文件路径，支持多种来源：
 * - tool_use 中的 file_path 参数
 * - tool_result 中的 associatedFilePath
 * - 代码块标注 (```ts:path)
 * - 文件路径注释 (// filepath:)
 * - 文本中的明确文件引用
 */

import type { ContentBlock, NarrativeMessage } from "@/types/message";

/** 文件路径提取结果 */
export interface FilePathResult {
  /** 提取到的文件路径 */
  path: string;
  /** 路径来源 */
  source:
    | "tool_use"
    | "tool_result"
    | "code_block"
    | "comment"
    | "text_match"
    | "history";
  /** 置信度 */
  confidence: "high" | "medium" | "low";
}

/**
 * 验证是否为有效的文件路径
 */
export function isValidFilePath(path: string): boolean {
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
 * 排除常见词（避免误匹配）
 */
function isCommonWord(path: string): boolean {
  const commonWords = ["e.g", "i.e", "etc.", "vs.", "v.s.", "1.0", "2.0", "3.0"];
  return commonWords.includes(path.toLowerCase());
}

/**
 * 从 tool_use 提取文件路径
 * AC #1: 支持 Read/Write/Edit 等工具的 file_path 参数
 */
export function extractFilePathFromToolUse(block: ContentBlock): string | null {
  if (block.type !== "tool_use" || !block.toolInput) {
    return null;
  }

  const input = block.toolInput as Record<string, unknown>;

  // 支持的工具和参数名映射
  const toolPathMapping: Record<string, string[]> = {
    Read: ["file_path", "filePath", "path"],
    Write: ["file_path", "filePath", "path"],
    Edit: ["file_path", "filePath", "path"],
    Glob: ["path", "pattern"],
    Grep: ["path"],
  };

  // 根据工具名查找参数
  const toolName = block.toolName || "";
  const pathKeys =
    toolPathMapping[toolName] || ["file_path", "filePath", "path", "filename"];

  for (const key of pathKeys) {
    const value = input[key];
    if (typeof value === "string" && value && isValidFilePath(value)) {
      return value;
    }
  }

  return null;
}

/**
 * 从 tool_result 提取 associatedFilePath
 * AC #7: tool_result 中的 associatedFilePath 优先级次于 tool_use
 */
export function extractFilePathFromToolResult(
  block: ContentBlock
): string | null {
  if (block.type !== "tool_result") {
    return null;
  }

  // associatedFilePath 是在前端转换时添加的
  if (block.associatedFilePath && isValidFilePath(block.associatedFilePath)) {
    return block.associatedFilePath;
  }

  return null;
}

// 预编译正则表达式 (性能优化)
const CODE_BLOCK_REGEX = /```\w+:([^\s`]+)/g;

/**
 * 解析代码块标注 (```ts:path 格式)
 * AC #2: 支持 ```language:path 格式
 */
export function parseCodeBlockAnnotation(text: string): string[] {
  const paths: string[] = [];

  // 匹配 ```language:path 格式
  // 例如: ```typescript:src/components/Button.tsx
  let match;
  // 重置 lastIndex
  CODE_BLOCK_REGEX.lastIndex = 0;

  while ((match = CODE_BLOCK_REGEX.exec(text)) !== null) {
    const path = match[1].trim();
    if (isValidFilePath(path)) {
      paths.push(path);
    }
  }

  return paths;
}

// 文件路径注释正则表达式
const FILEPATH_COMMENT_PATTERNS = [
  /\/\/\s*filepath:\s*([^\s\n]+)/gi, // // filepath: path
  /\/\/\s*file:\s*([^\s\n]+)/gi, // // file: path
  /\/\*\s*filepath:\s*([^\s*]+)/gi, // /* filepath: path
  /#\s*filepath:\s*([^\s\n]+)/gi, // # filepath: path (Python/Shell)
  /<!--\s*filepath:\s*([^\s-]+)/gi, // <!-- filepath: path (HTML/Markdown)
];

/**
 * 解析文件路径注释
 * AC #2: 支持 // filepath: 等注释格式
 */
export function parseFilePathComment(text: string): string[] {
  const paths: string[] = [];

  for (const pattern of FILEPATH_COMMENT_PATTERNS) {
    // 重置 lastIndex
    pattern.lastIndex = 0;
    let match;
    while ((match = pattern.exec(text)) !== null) {
      const path = match[1].trim();
      if (isValidFilePath(path)) {
        paths.push(path);
      }
    }
  }

  return paths;
}

// 文件路径匹配正则表达式
const FILE_PATH_PATTERNS = [
  // 相对路径: ./path, ../path, src/path (带扩展名)
  /(?:^|\s|['"`])(\.{0,2}\/[\w\-./]+\.\w{1,10})(?:\s|['"`]|$|:)/gm,
  // 绝对路径: /path/to/file (带扩展名)
  /(?:^|\s|['"`])(\/[\w\-./]+\.\w{1,10})(?:\s|['"`]|$|:)/gm,
  // 带引号的路径
  /['"`]([\w\-./]+\.\w{1,10})['"`]/g,
  // 反引号包裹的文件名
  /`([\w\-./]+\.\w{1,10})`/g,
];

/**
 * 从文本中提取明确的文件引用
 * AC #7: 文本中的明确文件引用（正则匹配）
 */
export function extractFilePathFromText(text: string): string[] {
  const paths: string[] = [];

  for (const pattern of FILE_PATH_PATTERNS) {
    // 重置 lastIndex
    pattern.lastIndex = 0;
    let match;
    while ((match = pattern.exec(text)) !== null) {
      const path = match[1].trim();
      if (isValidFilePath(path) && !isCommonWord(path)) {
        paths.push(path);
      }
    }
  }

  // 去重
  return [...new Set(paths)];
}

/**
 * 按优先级提取文件路径
 * AC #7: 按以下优先级提取：
 * 1. tool_use 中的 file_path 参数 (高)
 * 2. tool_result 中的 associatedFilePath (高)
 * 3. 代码块标注 (```ts:path) (中)
 * 4. 文件路径注释 (// filepath:) (中)
 * 5. 文本中的明确文件引用 (低)
 */
export function extractFilePathWithPriority(
  message: NarrativeMessage
): FilePathResult | null {
  // 1. tool_use 中的 file_path (最高优先级)
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

  // 3. 代码块标注
  for (const block of message.content) {
    if (block.type === "text" && block.content) {
      const paths = parseCodeBlockAnnotation(block.content);
      if (paths.length > 0) {
        return { path: paths[0], source: "code_block", confidence: "medium" };
      }
    }
  }

  // 4. 文件路径注释
  for (const block of message.content) {
    if (block.type === "text" && block.content) {
      const paths = parseFilePathComment(block.content);
      if (paths.length > 0) {
        return { path: paths[0], source: "comment", confidence: "medium" };
      }
    }
  }

  // 5. 文本中的文件引用
  for (const block of message.content) {
    if (block.type === "text" && block.content) {
      const paths = extractFilePathFromText(block.content);
      if (paths.length > 0) {
        return { path: paths[0], source: "text_match", confidence: "low" };
      }
    }
  }

  return null;
}

/**
 * 将绝对路径转换为相对于仓库的路径
 * AC #6: 自动转换绝对路径为相对路径
 */
export function toRelativePath(
  absolutePath: string,
  repoPath: string
): string {
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
 * 从消息列表中查找最近的文件路径
 * AC #3: 向前搜索逻辑
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
