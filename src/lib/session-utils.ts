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
 * 将后端内容块转换为前端格式
 */
function convertContentBlock(block: MantraContentBlock): ContentBlock {
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
        case "tool_use":
            return {
                type: "tool_use",
                content: "",
                toolName: block.name,
                toolInput: block.input,
            };
        case "tool_result":
            return {
                type: "tool_result",
                content: block.content,
                isError: block.is_error ?? false,
            };
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
    return session.messages.map((msg, index) => ({
        id: `${session.id}-msg-${index}`,
        role: msg.role,
        timestamp: msg.timestamp ?? session.created_at,
        content: msg.content_blocks.map(convertContentBlock),
    }));
}
