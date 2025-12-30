/**
 * useTimeTravelStore - 时间旅行状态管理
 * Story 2.7: Task 1 - AC #1, #2, #7
 *
 * 管理时间旅行核心状态:
 * - 当前时间戳
 * - 激活消息索引
 * - 加载状态
 * - 代码内容 (用于 Diff)
 * - Commit 信息
 */

import { create } from "zustand";

/**
 * Commit 信息
 */
export interface CommitInfo {
    /** Commit Hash (短格式) */
    hash: string;
    /** Commit 消息 */
    message: string;
    /** Commit 时间戳 */
    timestamp: number;
}

/**
 * 时间旅行状态接口
 */
export interface TimeTravelState {
    /** 当前时间旅行的时间戳 (Unix ms) */
    currentTimestamp: number | null;
    /** 当前激活的消息索引 */
    activeMessageIndex: number | null;
    /** 当前激活的消息 ID */
    activeMessageId: string | null;
    /** 是否正在加载快照 */
    isLoading: boolean;
    /** 上一个代码内容 (用于 Diff) */
    previousCode: string | null;
    /** 当前代码内容 */
    currentCode: string | null;
    /** 当前查看的文件路径 */
    currentFilePath: string | null;
    /** 当前 Commit 信息 */
    commitInfo: CommitInfo | null;
    /** 是否处于历史模式 */
    isHistoricalMode: boolean;
    /** 错误信息 */
    error: string | null;

    // ======== Actions ========

    /** 设置当前时间 (用于 TimberLine 拖动) */
    setCurrentTime: (timestamp: number) => void;

    /** 跳转到指定消息 (用于 NarrativeStream 点击) */
    jumpToMessage: (
        index: number,
        messageId: string,
        timestamp: number
    ) => void;

    /** 设置代码内容 */
    setCode: (code: string, filePath: string) => void;

    /** 设置 Commit 信息 */
    setCommitInfo: (info: CommitInfo | null) => void;

    /** 设置加载状态 */
    setLoading: (loading: boolean) => void;

    /** 设置错误信息 */
    setError: (error: string | null) => void;

    /** 进入历史模式 */
    enterHistoricalMode: () => void;

    /** 返回当前状态 */
    returnToCurrent: () => void;

    /** 重置所有状态 */
    reset: () => void;
}

/**
 * 初始状态
 */
const initialState = {
    currentTimestamp: null,
    activeMessageIndex: null,
    activeMessageId: null,
    isLoading: false,
    previousCode: null,
    currentCode: null,
    currentFilePath: null,
    commitInfo: null,
    isHistoricalMode: false,
    error: null,
};

/**
 * 时间旅行状态 Store
 */
export const useTimeTravelStore = create<TimeTravelState>((set) => ({
    ...initialState,

    setCurrentTime: (timestamp) =>
        set({
            currentTimestamp: timestamp,
            isHistoricalMode: true,
        }),

    jumpToMessage: (index, messageId, timestamp) =>
        set({
            activeMessageIndex: index,
            activeMessageId: messageId,
            currentTimestamp: timestamp,
            isHistoricalMode: true,
        }),

    setCode: (code, filePath) =>
        set((state) => ({
            previousCode: state.currentCode,
            currentCode: code,
            currentFilePath: filePath,
        })),

    setCommitInfo: (info) =>
        set({
            commitInfo: info,
        }),

    setLoading: (loading) =>
        set({
            isLoading: loading,
        }),

    setError: (error) =>
        set({
            error,
        }),

    enterHistoricalMode: () =>
        set({
            isHistoricalMode: true,
        }),

    returnToCurrent: () =>
        set({
            isHistoricalMode: false,
            currentTimestamp: null,
            activeMessageIndex: null,
            activeMessageId: null,
            previousCode: null,
            commitInfo: null,
            error: null,
        }),

    reset: () => set(initialState),
}));

export default useTimeTravelStore;
