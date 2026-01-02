/**
 * message-filter - 消息过滤逻辑
 * Story 2.16: Task 6
 *
 * 实现配对过滤逻辑，确保 ToolCall/ToolResult 成对出现
 * AC: #5, #6, #7
 */

import type { NarrativeMessage } from "@/types/message";
import {
    MESSAGE_TYPES,
    messageMatchesSearch,
    getToolUseIds,
} from "@/stores/useMessageFilterStore";

/**
 * 过滤结果接口
 */
export interface FilterResult {
    /** 过滤后的消息列表 */
    messages: NarrativeMessage[];
    /** 过滤后的消息 ID 集合 */
    messageIds: Set<string>;
    /** 过滤后消息数量 */
    filteredCount: number;
    /** 总消息数量 */
    totalCount: number;
}

/**
 * 检查消息是否匹配任意选中的类型
 */
function messageMatchesAnyType(
    message: NarrativeMessage,
    selectedTypes: Set<string>
): boolean {
    // 空选择 = 显示全部
    if (selectedTypes.size === 0) return true;

    for (const typeId of selectedTypes) {
        const typeConfig = MESSAGE_TYPES.find((t) => t.id === typeId);
        if (typeConfig && message.content.some((block) => typeConfig.match(block))) {
            return true;
        }
    }
    return false;
}

/**
 * 构建 toolUseId 到消息的映射
 */
function buildToolUseIdMap(
    messages: NarrativeMessage[]
): Map<string, NarrativeMessage[]> {
    const map = new Map<string, NarrativeMessage[]>();

    for (const message of messages) {
        const toolIds = getToolUseIds(message);
        for (const toolId of toolIds) {
            const existing = map.get(toolId) || [];
            existing.push(message);
            map.set(toolId, existing);
        }
    }

    return map;
}

/**
 * 带配对逻辑的消息过滤
 *
 * 规则：
 * 1. 按类型和搜索关键词过滤消息
 * 2. 如果匹配的消息包含 tool_use 或 tool_result，自动包含其配对消息
 * 3. 结果按原始顺序排序
 */
export function filterWithPairedResults(
    messages: NarrativeMessage[],
    selectedTypes: Set<string>,
    searchQuery: string
): FilterResult {
    if (messages.length === 0) {
        return {
            messages: [],
            messageIds: new Set(),
            filteredCount: 0,
            totalCount: 0,
        };
    }

    // 无过滤条件，返回全部
    if (selectedTypes.size === 0 && !searchQuery.trim()) {
        return {
            messages,
            messageIds: new Set(messages.map((m) => m.id)),
            filteredCount: messages.length,
            totalCount: messages.length,
        };
    }

    // 构建 toolUseId -> 消息列表的映射
    const toolUseIdMap = buildToolUseIdMap(messages);

    // 收集匹配的消息 ID
    const matchedIds = new Set<string>();

    // 第一轮：找出直接匹配的消息
    for (const message of messages) {
        const matchesType = messageMatchesAnyType(message, selectedTypes);
        const matchesSearch = messageMatchesSearch(message, searchQuery);

        if (matchesType && matchesSearch) {
            matchedIds.add(message.id);
        }
    }

    // 第二轮：补充配对消息
    const pairedIds = new Set<string>();

    for (const messageId of matchedIds) {
        const message = messages.find((m) => m.id === messageId);
        if (!message) continue;

        // 获取该消息的所有 toolUseId
        const toolIds = getToolUseIds(message);

        // 找到所有配对消息
        for (const toolId of toolIds) {
            const pairedMessages = toolUseIdMap.get(toolId);
            if (pairedMessages) {
                for (const paired of pairedMessages) {
                    pairedIds.add(paired.id);
                }
            }
        }
    }

    // 合并直接匹配和配对消息
    const allMatchedIds = new Set([...matchedIds, ...pairedIds]);

    // 按原始顺序过滤消息
    const filteredMessages = messages.filter((m) => allMatchedIds.has(m.id));

    return {
        messages: filteredMessages,
        messageIds: allMatchedIds,
        filteredCount: filteredMessages.length,
        totalCount: messages.length,
    };
}

/**
 * 计算消息是否被过滤可见
 */
export function isMessageVisible(
    messageId: string,
    filteredIds: Set<string>
): boolean {
    return filteredIds.size === 0 || filteredIds.has(messageId);
}

export default filterWithPairedResults;
