/**
 * Session Utilities - 会话数据转换工具
 *
 * 将后端 MantraSession 格式转换为前端 NarrativeMessage 格式
 */

import type { NarrativeMessage, ContentBlock } from "@/types/message";

/**
 * 后端 MantraSession 类型 (来自 Rust)
 */
export interface MantraSession {
    id: string;
    source: "claude" | "gemini" | "cursor" | "unknown";
    cwd: string;
    created_at: string;
    updated_at: string;
    messages: MantraMessage[];
    metadata?: {
        model?: string;
        total_tokens?: number;
        title?: string;
        original_path?: string;
    };
}

/**
 * 后端消息类型
 */
export interface MantraMessage {
    role: "user" | "assistant";
    content_blocks: MantraContentBlock[];
    timestamp?: string;
}

/**
 * 后端内容块类型
 */
export type MantraContentBlock =
    | { type: "text"; text: string }
    | { type: "thinking"; thinking: string }
    | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
    | { type: "tool_result"; tool_use_id: string; content: string; is_error?: boolean };

/**
 * tool_use 信息缓存，用于关联 tool_result
 */
interface ToolUseInfo {
    toolName: string;
    filePath?: string;
}

/** 文件操作相关的工具名称 */
const FILE_TOOLS = ["Read", "Write", "Edit", "Glob", "Grep"];

/** 检查是否是文件操作工具 */
function isFileTool(toolName: string): boolean {
    return FILE_TOOLS.some(t => toolName.toLowerCase().includes(t.toLowerCase()));
}

/** 从工具输入中提取文件路径 */
function extractFilePath(input?: Record<string, unknown>): string | undefined {
    if (!input) return undefined;
    const pathKeys = ["file_path", "filePath", "path", "file"];
    for (const key of pathKeys) {
        if (typeof input[key] === "string") {
            return input[key] as string;
        }
    }
    return undefined;
}

/**
 * 将后端内容块转换为前端格式
 * @param block - 后端内容块
 * @param toolUseMap - tool_use ID 到信息的映射
 */
function convertContentBlock(
    block: MantraContentBlock,
    toolUseMap: Map<string, ToolUseInfo>
): ContentBlock {
    switch (block.type) {
        case "text":
            return {
                type: "text",
                content: block.text,
            };
        case "thinking":
            return {
                type: "thinking",
                content: block.thinking,
            };
        case "tool_use": {
            // 提取文件路径（如果是文件操作工具）
            const filePath = isFileTool(block.name) ? extractFilePath(block.input) : undefined;

            // 缓存到映射中，供后续 tool_result 使用
            toolUseMap.set(block.id, {
                toolName: block.name,
                filePath,
            });

            return {
                type: "tool_use",
                content: "",
                toolName: block.name,
                toolInput: block.input,
                toolUseId: block.id,
            };
        }
        case "tool_result": {
            // 从映射中获取关联的 tool_use 信息
            const toolUseInfo = toolUseMap.get(block.tool_use_id);

            return {
                type: "tool_result",
                content: block.content,
                isError: block.is_error ?? false,
                associatedFilePath: toolUseInfo?.filePath,
                associatedToolName: toolUseInfo?.toolName,
            };
        }
        default:
            // Fallback for unknown types
            return {
                type: "text",
                content: JSON.stringify(block),
            };
    }
}

/**
 * 将 MantraSession 转换为 NarrativeMessage 数组
 *
 * @param session - 后端会话数据
 * @returns 前端消息数组
 */
export function convertSessionToMessages(session: MantraSession): NarrativeMessage[] {
    // 创建 tool_use ID -> 信息的映射，跨消息追踪
    const toolUseMap = new Map<string, ToolUseInfo>();

    return session.messages.map((msg, index) => ({
        id: `${session.id}-msg-${index}`,
        role: msg.role,
        timestamp: msg.timestamp ?? session.created_at,
        content: msg.content_blocks.map(block => convertContentBlock(block, toolUseMap)),
    }));
}
