/**
 * Session Utilities - 会话数据转换工具
 * Story 8.12: Task 5 - 使用 standardTool 替代手动路径提取
 *
 * 将后端 MantraSession 格式转换为前端 NarrativeMessage 格式
 */

import type { NarrativeMessage, ContentBlock, StandardTool, ToolResultData } from "@/types/message";
import { isFileTool, getToolPath } from "@/lib/tool-utils";

/**
 * 将后端 StandardTool (snake_case) 转换为前端格式 (camelCase)
 * 后端字段: old_string, new_string
 * 前端字段: oldString, newString
 */
function convertStandardTool(backendTool: Record<string, unknown> | undefined): StandardTool | undefined {
    if (!backendTool) return undefined;

    const tool = { ...backendTool } as Record<string, unknown>;

    // 转换 file_edit 类型的字段
    if (tool.type === "file_edit") {
        if ("old_string" in tool) {
            tool.oldString = tool.old_string;
            delete tool.old_string;
        }
        if ("new_string" in tool) {
            tool.newString = tool.new_string;
            delete tool.new_string;
        }
    }

    return tool as StandardTool;
}

/**
 * 后端 MantraSession 类型 (来自 Rust)
 */
export interface MantraSession {
    id: string;
    source: "claude" | "gemini" | "cursor" | "codex" | "unknown";
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
 * 后端内容块类型 (Story 8.12: 包含 standardTool)
 */
export type MantraContentBlock =
    | { type: "text"; text: string }
    | { type: "thinking"; thinking: string; subject?: string; timestamp?: string }
    | {
        type: "tool_use";
        id: string;
        name: string;
        input: Record<string, unknown>;
        standard_tool?: StandardTool;
        display_name?: string;
        description?: string;
    }
    | {
        type: "tool_result";
        tool_use_id: string;
        content: string;
        is_error?: boolean;
        structured_result?: ToolResultData;
        display_content?: string;
        render_as_markdown?: boolean;
        user_decision?: string;
    }
    | { type: "code_suggestion"; file_path?: string; code?: string; language?: string };

/**
 * tool_use 信息缓存，用于关联 tool_result
 */
interface ToolUseInfo {
    toolName: string;
    filePath?: string;
    standardTool?: StandardTool;
}

/**
 * 将后端内容块转换为前端格式
 * Story 8.12: 使用 standardTool 替代手动路径提取
 *
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
                // Story 8.12: 传递新字段
                subject: block.subject,
                thinkingTimestamp: block.timestamp,
            };
        case "tool_use": {
            // Story 8.12: 使用 standardTool 获取文件路径
            // 转换 snake_case 字段名为 camelCase
            const standardTool = convertStandardTool(block.standard_tool as Record<string, unknown> | undefined);
            const filePath = isFileTool(standardTool) ? getToolPath(standardTool) : undefined;

            // 缓存到映射中，供后续 tool_result 使用
            toolUseMap.set(block.id, {
                toolName: block.name,
                filePath,
                standardTool,
            });

            return {
                type: "tool_use",
                content: "",
                toolName: block.name,
                toolInput: block.input,
                toolUseId: block.id,
                // Story 8.12: 传递 standardTool
                standardTool,
                displayName: block.display_name,
                description: block.description,
            };
        }
        case "tool_result": {
            // 从映射中获取关联的 tool_use 信息
            const toolUseInfo = toolUseMap.get(block.tool_use_id);

            return {
                type: "tool_result",
                content: block.content,
                isError: block.is_error ?? false,
                toolUseId: block.tool_use_id,
                associatedFilePath: toolUseInfo?.filePath,
                associatedToolName: toolUseInfo?.toolName,
                // Story 8.12: 传递 structuredResult 和其他新字段
                structuredResult: block.structured_result,
                displayContent: block.display_content,
                renderAsMarkdown: block.render_as_markdown,
                userDecision: block.user_decision,
            };
        }
        case "code_suggestion":
            return {
                type: "code_suggestion",
                content: block.code || "",
                filePath: block.file_path,
                code: block.code,
                language: block.language,
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
    // 创建 tool_use ID -> 信息的映射，跨消息追踪
    const toolUseMap = new Map<string, ToolUseInfo>();

    return session.messages.map((msg, index) => ({
        id: `${session.id}-msg-${index}`,
        role: msg.role,
        timestamp: msg.timestamp ?? session.created_at,
        content: msg.content_blocks.map(block => convertContentBlock(block, toolUseMap)),
    }));
}
