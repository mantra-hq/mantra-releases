/**
 * useTimeTravelStore - 时间旅行状态管理
 * Story 2.7: Task 1 - AC #1, #2, #7
 * Story 2.12: Task 4 - AC #5 (File Not Found State)
 *
 * 管理时间旅行核心状态:
 * - 当前时间戳
 * - 激活消息索引
 * - 加载状态
 * - 代码内容 (用于 Diff)
 * - Commit 信息
 * - 文件不存在状态 (Story 2.12)
 */

import { create } from "zustand";
import type { SnapshotSource } from "@/hooks/useTimeMachine";

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

    // Story 2.30: 快照来源
    /** 当前快照来源 */
    snapshotSource: SnapshotSource | null;

    // Story 2.12 AC #5: 文件不存在状态
    /** 文件是否不存在 */
    fileNotFound: boolean;
    /** 不存在的文件路径 */
    notFoundPath: string | null;
    /** 文件不存在时的时间戳 (Unix 秒) */
    notFoundTimestamp: number | null;

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

    /** Story 2.12: 设置文件不存在状态 */
    setFileNotFound: (path: string, timestamp: number) => void;

    /** Story 2.12: 清除文件不存在状态 */
    clearFileNotFound: () => void;

    /** Story 2.30: 设置快照来源 */
    setSnapshotSource: (source: SnapshotSource | null) => void;

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
    // Story 2.12 AC #5: 文件不存在状态
    fileNotFound: false,
    notFoundPath: null,
    notFoundTimestamp: null,
    // Story 2.30: 快照来源
    snapshotSource: null,
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
            // 只有当文件路径相同时才保留 previousCode 用于 diff
            // 切换到不同文件时清除 previousCode，避免跨文件 diff
            previousCode: state.currentFilePath === filePath ? state.currentCode : null,
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
            // Story 2.12: 清除文件不存在状态
            fileNotFound: false,
            notFoundPath: null,
            notFoundTimestamp: null,
        }),

    // Story 2.12 AC #5: 设置文件不存在状态
    setFileNotFound: (path, timestamp) =>
        set({
            fileNotFound: true,
            notFoundPath: path,
            notFoundTimestamp: timestamp,
            error: null, // 清除其他错误
        }),

    // Story 2.12 AC #5: 清除文件不存在状态
    clearFileNotFound: () =>
        set({
            fileNotFound: false,
            notFoundPath: null,
            notFoundTimestamp: null,
        }),

    // Story 2.30: 设置快照来源
    setSnapshotSource: (source) =>
        set({
            snapshotSource: source,
        }),

    reset: () => set(initialState),
}));

export default useTimeTravelStore;
