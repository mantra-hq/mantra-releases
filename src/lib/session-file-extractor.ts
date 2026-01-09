/**
 * Session File Extractor - 从会话日志中提取文件内容
 * Story 2.30: AC2 - 会话日志内容回退
 * Story 8.12: Task 6 - 使用 standardTool 替代工具名和路径字段匹配
 *
 * 当文件在 Git 历史和工作目录中都不存在时，
 * 尝试从会话的 tool_use Write 操作中提取文件内容。
 */

import type { NarrativeMessage } from "@/types/message";
import { isFileWriteTool, isFileEditTool, getToolPath, getToolContent } from "@/lib/tool-utils";

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
 * Story 8.12: 使用 standardTool.type 判断，直接用 standardTool.path
 */
function extractWriteToolContent(
    message: NarrativeMessage,
    targetPath: string
): { content: string; path: string } | null {
    if (message.role !== "assistant") return null;

    for (const block of message.content) {
        // 检查 ToolUse 块
        if (block.type === "tool_use") {
            const { standardTool } = block;

            // Story 8.12: 使用 standardTool.type 判断写入/编辑工具
            if (isFileWriteTool(standardTool) || isFileEditTool(standardTool)) {
                const filePath = getToolPath(standardTool) ?? "";
                let content = "";

                // file_write 有 content 字段
                if (isFileWriteTool(standardTool)) {
                    content = getToolContent(standardTool) ?? "";
                }
                // file_edit 有 newString 字段（部分内容）
                else if (isFileEditTool(standardTool) && standardTool?.type === "file_edit") {
                    content = standardTool.newString ?? "";
                }

                // 规范化路径进行比较
                const normalizedTarget = normalizePath(targetPath);
                const normalizedPath = normalizePath(filePath);

                if (normalizedPath === normalizedTarget && content) {
                    return { content, path: filePath };
                }
            }
        }

        // 检查 ToolResult 块（某些格式可能在 result 中包含内容）
        // 注意: Parser 层已处理行号前缀，这里直接使用 content
        if (block.type === "tool_result" && block.associatedFilePath) {
            const normalizedTarget = normalizePath(targetPath);
            const normalizedPath = normalizePath(block.associatedFilePath);

            if (normalizedPath === normalizedTarget && block.content) {
                return { content: block.content, path: block.associatedFilePath };
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
