/**
 * useSearchStore - 全局搜索状态管理
 * Story 2.10: Task 5
 * Story 2.33: Task 4 (添加 filters 和 recentQueries)
 *
 * 管理全局搜索的所有状态:
 * - Modal 开关状态
 * - 搜索查询和结果
 * - 键盘导航选择
 * - 最近访问的会话
 * - 加载状态
 * - 搜索筛选器 (Story 2.33)
 * - 搜索历史 (Story 2.33)
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";

// ============================================================================
// Story 2.33: Search Filters Types
// ============================================================================

/**
 * 内容类型筛选
 * AC1: 全部 | 代码 | 对话
 */
export type ContentType = "all" | "code" | "conversation";

/**
 * 时间范围预设
 * AC3: 全部时间 | 今天 | 本周 | 本月
 */
export type TimePreset = "all" | "today" | "week" | "month";

/**
 * 搜索筛选器
 * AC1-AC3: 内容类型 + 项目 + 时间范围
 */
export interface SearchFilters {
    /** 内容类型 (全部/代码/对话) */
    contentType: ContentType;
    /** 项目 ID (null = 全部项目) */
    projectId: string | null;
    /** 时间范围预设 */
    timePreset: TimePreset;
}

/**
 * 默认筛选器
 */
export const DEFAULT_FILTERS: SearchFilters = {
    contentType: "all",
    projectId: null,
    timePreset: "all",
};

/**
 * 搜索结果项
 */
export interface SearchResult {
    /** 唯一标识 (session_id-message_id) */
    id: string;
    /** 项目 ID */
    projectId: string;
    /** 项目名称 */
    projectName: string;
    /** 会话 ID */
    sessionId: string;
    /** 会话名称 */
    sessionName: string;
    /** 消息 ID */
    messageId: string;
    /** 匹配的文本片段 */
    snippet: string;
    /** 高亮范围 [start, end] */
    highlightRanges: Array<[number, number]>;
    /** 时间戳 */
    timestamp: number;
    /** 内容类型 (Story 2.33) */
    contentType?: ContentType;
}

/**
 * 最近访问的会话
 */
export interface RecentSession {
    /** 项目 ID */
    projectId: string;
    /** 项目名称 */
    projectName: string;
    /** 会话 ID */
    sessionId: string;
    /** 会话名称 */
    sessionName: string;
    /** 访问时间 */
    accessedAt: number;
}

/**
 * 搜索状态接口
 */
export interface SearchState {
    // ======== 状态 ========
    /** 搜索框是否打开 */
    isOpen: boolean;
    /** 搜索查询 */
    query: string;
    /** 搜索结果 */
    results: SearchResult[];
    /** 是否正在加载 */
    isLoading: boolean;
    /** 当前选中的结果索引 */
    selectedIndex: number;
    /** 最近访问的会话 */
    recentSessions: RecentSession[];

    // ======== Story 2.33: 新增状态 ========
    /** 搜索筛选器 */
    filters: SearchFilters;
    /** 最近搜索关键词 (AC5) */
    recentQueries: string[];

    // ======== Actions ========
    /** 打开搜索框 */
    open: () => void;
    /** 关闭搜索框 */
    close: () => void;
    /** 设置搜索查询 */
    setQuery: (query: string) => void;
    /** 设置搜索结果 */
    setResults: (results: SearchResult[]) => void;
    /** 设置加载状态 */
    setLoading: (loading: boolean) => void;
    /** 选择下一个结果 */
    selectNext: () => void;
    /** 选择上一个结果 */
    selectPrev: () => void;
    /** 确认选择，返回当前选中的结果 */
    confirm: () => SearchResult | null;
    /** 添加最近访问的会话 */
    addRecentSession: (session: RecentSession) => void;
    /** 重置搜索状态 (不重置 recentSessions) */
    reset: () => void;

    // ======== Story 2.33: 新增 Actions ========
    /** 设置筛选器 (部分更新) */
    setFilters: (filters: Partial<SearchFilters>) => void;
    /** 重置筛选器到默认状态 (AC6) */
    resetFilters: () => void;
    /** 添加搜索历史 (AC5) */
    addRecentQuery: (query: string) => void;
    /** 删除单条搜索历史 (AC5) */
    removeRecentQuery: (query: string) => void;
    /** 清空全部搜索历史 (AC5) */
    clearRecentQueries: () => void;
}

/** 保留的最近会话数量 */
const MAX_RECENT_SESSIONS = 10;
/** 保留的最近搜索关键词数量 (AC5) */
const MAX_RECENT_QUERIES = 10;

/**
 * 搜索状态 Store
 */
export const useSearchStore = create<SearchState>()(
    persist(
        (set, get) => ({
            // 初始状态
            isOpen: false,
            query: "",
            results: [],
            isLoading: false,
            selectedIndex: 0,
            recentSessions: [],

            // Story 2.33: 新增状态
            filters: { ...DEFAULT_FILTERS },
            recentQueries: [],

            // Actions
            open: () =>
                set({
                    isOpen: true,
                    query: "",
                    results: [],
                    selectedIndex: 0,
                    isLoading: false,
                    // AC6: 每次打开搜索框，筛选器归位
                    filters: { ...DEFAULT_FILTERS },
                }),

            close: () =>
                set({
                    isOpen: false,
                }),

            setQuery: (query) =>
                set({
                    query,
                    selectedIndex: 0,
                }),

            setResults: (results) =>
                set({
                    results,
                    isLoading: false,
                    selectedIndex: 0,
                }),

            setLoading: (isLoading) =>
                set({
                    isLoading,
                }),

            selectNext: () =>
                set((state) => ({
                    selectedIndex: Math.min(
                        state.selectedIndex + 1,
                        state.results.length - 1
                    ),
                })),

            selectPrev: () =>
                set((state) => ({
                    selectedIndex: Math.max(state.selectedIndex - 1, 0),
                })),

            confirm: () => {
                const { results, selectedIndex } = get();
                return results[selectedIndex] || null;
            },

            addRecentSession: (session) =>
                set((state) => {
                    // 过滤掉相同的会话
                    const filtered = state.recentSessions.filter(
                        (s) => s.sessionId !== session.sessionId
                    );
                    return {
                        recentSessions: [session, ...filtered].slice(0, MAX_RECENT_SESSIONS),
                    };
                }),

            reset: () =>
                set({
                    query: "",
                    results: [],
                    isLoading: false,
                    selectedIndex: 0,
                    filters: { ...DEFAULT_FILTERS },
                }),

            // Story 2.33: 新增 Actions

            setFilters: (filters) =>
                set((state) => ({
                    filters: { ...state.filters, ...filters },
                    // 筛选器变化时重置选中索引
                    selectedIndex: 0,
                })),

            resetFilters: () =>
                set({
                    filters: { ...DEFAULT_FILTERS },
                    selectedIndex: 0,
                }),

            addRecentQuery: (query) =>
                set((state) => {
                    const trimmed = query.trim();
                    if (!trimmed) return state;

                    // 过滤掉相同的查询词
                    const filtered = state.recentQueries.filter(
                        (q) => q.toLowerCase() !== trimmed.toLowerCase()
                    );
                    return {
                        recentQueries: [trimmed, ...filtered].slice(0, MAX_RECENT_QUERIES),
                    };
                }),

            removeRecentQuery: (query) =>
                set((state) => ({
                    recentQueries: state.recentQueries.filter(
                        (q) => q.toLowerCase() !== query.toLowerCase()
                    ),
                })),

            clearRecentQueries: () =>
                set({
                    recentQueries: [],
                }),
        }),
        {
            name: "mantra-search",
            // 持久化 recentSessions 和 recentQueries (AC5)
            partialize: (state) => ({
                recentSessions: state.recentSessions,
                recentQueries: state.recentQueries,
            }),
        }
    )
);

export default useSearchStore;
