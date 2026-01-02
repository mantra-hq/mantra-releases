/**
 * useMessageFilterStore - 消息过滤状态管理
 * Story 2.16: Task 1
 *
 * 管理消息类型过滤和搜索的状态:
 * - selectedTypes: 选中的过滤类型集合
 * - searchQuery: 搜索关键词
 * - filteredMessageIds: 过滤后的消息 ID 集合
 */

import { create } from "zustand";
import type { ContentBlock, NarrativeMessage } from "@/types/message";

/**
 * 消息类型配置
 */
export interface MessageTypeConfig {
    /** 类型标识 */
    id: string;
    /** 显示标签 */
    label: string;
    /** 图标 */
    icon: string;
    /** 匹配函数 - 检查内容块是否匹配该类型 */
    match: (block: ContentBlock) => boolean;
}

/**
 * 预定义的消息类型配置
 */
export const MESSAGE_TYPES: MessageTypeConfig[] = [
    {
        id: "conversation",
        label: "对话",
        icon: "💬",
        match: (b) => b.type === "text",
    },
    {
        id: "tool",
        label: "工具",
        icon: "🔧",
        match: (b) => b.type === "tool_use" || b.type === "tool_result",
    },
    {
        id: "file",
        label: "文件",
        icon: "📄",
        match: (b) =>
            b.type === "tool_use" &&
            (b.toolName?.includes("file") ?? false),
    },
    {
        id: "terminal",
        label: "命令",
        icon: "$",
        match: (b) =>
            b.type === "tool_use" &&
            ["run_command", "bash", "command"].some((name) =>
                b.toolName?.toLowerCase().includes(name)
            ),
    },
    {
        id: "thinking",
        label: "思考",
        icon: "💭",
        match: (b) => b.type === "thinking",
    },
    {
        id: "search",
        label: "搜索",
        icon: "🔍",
        match: (b) =>
            b.type === "tool_use" &&
            ["search", "grep", "find"].some((name) =>
                b.toolName?.toLowerCase().includes(name)
            ),
    },
];

/**
 * 消息过滤状态接口
 */
export interface MessageFilterState {
    // ======== 状态 ========
    /** 选中的类型 ID 集合，空集 = 显示全部 */
    selectedTypes: Set<string>;
    /** 搜索关键词 */
    searchQuery: string;
    /** 搜索框聚焦状态 */
    isSearchFocused: boolean;

    // ======== Actions ========
    /** 切换类型选中状态 */
    toggleType: (typeId: string) => void;
    /** 设置搜索关键词 */
    setSearchQuery: (query: string) => void;
    /** 重置所有过滤条件 */
    clearFilters: () => void;
    /** 设置搜索框聚焦状态 */
    setSearchFocused: (focused: boolean) => void;
}

/**
 * 消息过滤状态 Store
 */
export const useMessageFilterStore = create<MessageFilterState>()((set) => ({
    // 初始状态
    selectedTypes: new Set<string>(),
    searchQuery: "",
    isSearchFocused: false,

    // Actions
    toggleType: (typeId) =>
        set((state) => {
            const newTypes = new Set(state.selectedTypes);
            if (newTypes.has(typeId)) {
                newTypes.delete(typeId);
            } else {
                newTypes.add(typeId);
            }
            return { selectedTypes: newTypes };
        }),

    setSearchQuery: (query) =>
        set({
            searchQuery: query,
        }),

    clearFilters: () =>
        set({
            selectedTypes: new Set<string>(),
            searchQuery: "",
        }),

    setSearchFocused: (focused) =>
        set({
            isSearchFocused: focused,
        }),
}));

/**
 * 检查消息是否匹配某个类型
 */
export function messageMatchesType(
    message: NarrativeMessage,
    typeId: string
): boolean {
    const typeConfig = MESSAGE_TYPES.find((t) => t.id === typeId);
    if (!typeConfig) return false;
    return message.content.some((block) => typeConfig.match(block));
}

/**
 * 检查消息是否包含搜索关键词
 */
export function messageMatchesSearch(
    message: NarrativeMessage,
    query: string
): boolean {
    if (!query.trim()) return true;
    const lowerQuery = query.toLowerCase();
    return message.content.some((block) => {
        // 检查文本内容
        if (block.content?.toLowerCase().includes(lowerQuery)) return true;
        // 检查工具名称
        if (block.toolName?.toLowerCase().includes(lowerQuery)) return true;
        return false;
    });
}

/**
 * 获取消息的 toolUseId（用于配对匹配）
 */
export function getToolUseIds(message: NarrativeMessage): Set<string> {
    const ids = new Set<string>();
    for (const block of message.content) {
        if (block.toolUseId) {
            ids.add(block.toolUseId);
        }
    }
    return ids;
}

/**
 * 检查消息是否包含 tool_use 或 tool_result
 */
export function hasToolBlocks(message: NarrativeMessage): boolean {
    return message.content.some(
        (block) => block.type === "tool_use" || block.type === "tool_result"
    );
}

export default useMessageFilterStore;
