/**
 * useSearchStore - 全局搜索状态管理
 * Story 2.10: Task 5
 *
 * 管理全局搜索的所有状态:
 * - Modal 开关状态
 * - 搜索查询和结果
 * - 键盘导航选择
 * - 最近访问的会话
 * - 加载状态
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";

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
}

/** 保留的最近会话数量 */
const MAX_RECENT_SESSIONS = 10;

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

            // Actions
            open: () =>
                set({
                    isOpen: true,
                    query: "",
                    results: [],
                    selectedIndex: 0,
                    isLoading: false,
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
                }),
        }),
        {
            name: "mantra-search",
            // 只持久化 recentSessions
            partialize: (state) => ({
                recentSessions: state.recentSessions,
            }),
        }
    )
);

export default useSearchStore;
