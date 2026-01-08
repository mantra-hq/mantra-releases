/**
 * Session File Extractor - 从会话日志中提取文件内容
 * Story 2.30: AC2 - 会话日志内容回退
 *
 * 当文件在 Git 历史和工作目录中都不存在时，
 * 尝试从会话的 tool_use Write 操作中提取文件内容。
 */

import type { NarrativeMessage } from "@/types/message";

/**
 * 从会话消息中提取文件内容的结果
 */
export interface SessionFileResult {
    /** 文件内容 */
    content: string;
    /** 文件路径 */
    filePath: string;
    /** 发现该内容的消息索引 */
    messageIndex: number;
    /** 消息时间戳 */
    timestamp: number;
}

/**
 * 从消息中提取 Write 工具调用的文件路径和内容
 */
function extractWriteToolContent(
    message: NarrativeMessage,
    targetPath: string
): { content: string; path: string } | null {
    if (message.role !== "assistant") return null;

    for (const block of message.content) {
        // 检查 ToolUse 块
        if (block.type === "tool_use") {
            const toolName = block.toolName?.toLowerCase() ?? "";
            // 匹配 Write 相关的工具名
            if (
                toolName === "write" ||
                toolName === "write_file" ||
                toolName === "writefile" ||
                toolName === "create_file" ||
                toolName === "createfile"
            ) {
                // 尝试从 toolInput 中获取文件路径和内容
                // 注意: ContentBlock 使用 toolInput 而不是 input
                const input = block.toolInput as Record<string, unknown> | undefined;
                if (input) {
                    const filePath =
                        (input.file_path as string) ||
                        (input.filePath as string) ||
                        (input.path as string) ||
                        "";
                    const content =
                        (input.content as string) ||
                        (input.file_content as string) ||
                        "";

                    // 规范化路径进行比较
                    const normalizedTarget = normalizePath(targetPath);
                    const normalizedPath = normalizePath(filePath);

                    if (normalizedPath === normalizedTarget && content) {
                        return { content, path: filePath };
                    }
                }
            }
        }

        // 检查 ToolResult 块（某些格式可能在 result 中包含内容）
        // 注意: ContentBlock 使用 content 字段存储工具结果内容
        // Read 工具的输出可能带有行号前缀，需要去除
        if (block.type === "tool_result" && block.associatedFilePath) {
            const normalizedTarget = normalizePath(targetPath);
            const normalizedPath = normalizePath(block.associatedFilePath);

            if (normalizedPath === normalizedTarget && block.content) {
                // 去除可能存在的行号前缀 (如 "1→", "42|" 等)
                const cleanContent = stripLineNumberPrefix(block.content);
                return { content: cleanContent, path: block.associatedFilePath };
            }
        }
    }

    return null;
}

/**
 * 规范化文件路径（移除前导 ./ 和 /）
 */
function normalizePath(path: string): string {
    return path
        .replace(/^\.\//, "")
        .replace(/^\//, "")
        .toLowerCase();
}

/**
 * 去除行号前缀 (如 "1→", "42|", "  123→" 等格式)
 * 
 * tool_result 中的内容可能带有 Claude 的行号格式:
 * - "1→import ..." 或 "1|import ..."
 * - 行号可能有前导空格用于对齐
 */
function stripLineNumberPrefix(content: string): string {
    const lines = content.split('\n');
    
    // 检测是否大部分行都有行号前缀（避免误处理）
    const lineNumberPattern = /^\s*\d+[→|]/;
    const linesWithPrefix = lines.filter(line => lineNumberPattern.test(line)).length;
    
    // 如果超过 50% 的非空行有行号前缀，才进行处理
    const nonEmptyLines = lines.filter(line => line.trim().length > 0).length;
    if (nonEmptyLines === 0 || linesWithPrefix / nonEmptyLines < 0.5) {
        return content;
    }
    
    return lines
        .map(line => {
            // 匹配 "空格+数字+→或|" 格式，提取后面的内容
            const match = line.match(/^\s*\d+[→|](.*)$/);
            return match ? match[1] : line;
        })
        .join('\n');
}

/**
 * 从会话消息中提取指定文件的内容
 *
 * 搜索策略:
 * 1. 从当前消息向前搜索
 * 2. 找到第一个包含该文件 Write 操作的消息
 * 3. 返回 tool_use.input.content
 *
 * @param messages - 会话消息列表
 * @param targetPath - 目标文件路径
 * @param currentIndex - 当前消息索引（从此处向前搜索）
 * @returns 文件内容结果，未找到返回 null
 */
export function extractFileFromSession(
    messages: NarrativeMessage[],
    targetPath: string,
    currentIndex: number
): SessionFileResult | null {
    if (!targetPath || messages.length === 0) {
        return null;
    }

    // 从当前消息向前搜索（包括当前消息）
    for (let i = currentIndex; i >= 0; i--) {
        const message = messages[i];
        const result = extractWriteToolContent(message, targetPath);

        if (result) {
            return {
                content: result.content,
                filePath: result.path,
                messageIndex: i,
                timestamp: new Date(message.timestamp).getTime(),
            };
        }
    }

    // 如果向前没找到，也搜索当前消息之后的消息
    // （某些情况下文件可能在后续消息中被创建）
    for (let i = currentIndex + 1; i < messages.length; i++) {
        const message = messages[i];
        const result = extractWriteToolContent(message, targetPath);

        if (result) {
            return {
                content: result.content,
                filePath: result.path,
                messageIndex: i,
                timestamp: new Date(message.timestamp).getTime(),
            };
        }
    }

    return null;
}

export default extractFileFromSession;
