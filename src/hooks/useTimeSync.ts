/**
 * useTimeSync - 时间旅行同步 Hook
 * Story 2.7: Task 5, 6, 7 - AC #1, #2, #7
 *
 * 协调消息点击、时间轴拖动和代码快照更新之间的同步
 * 提供双向联动的统一接口
 */

import { useCallback, useMemo, useRef, useEffect } from "react";
import { useTimeTravelStore } from "@/stores/useTimeTravelStore";
import { useTimeMachine } from "@/hooks/useTimeMachine";
import type { NarrativeMessage } from "@/types/message";

/**
 * 消息时间信息
 */
interface MessageTimeInfo {
    /** 消息索引 */
    index: number;
    /** 消息 ID */
    id: string;
    /** 时间戳 (Unix ms) */
    timestamp: number;
}

/**
 * useTimeSync Hook 参数
 */
export interface UseTimeSyncOptions {
    /** 消息列表 */
    messages: NarrativeMessage[];
    /** Git 仓库路径 (可选，无仓库则不获取快照) */
    repoPath?: string | null;
    /** 当前查看的文件路径 */
    currentFilePath?: string | null;
    /** 消息选中回调 (外部传入) */
    onMessageSelect?: (messageId: string, message: NarrativeMessage) => void;
    /** 时间轴位置变化回调 (外部传入) */
    onTimelineSeek?: (timestamp: number) => void;
    /** 滚动到消息回调 */
    scrollToMessage?: (messageId: string) => void;
    /** Seek 防抖延迟 (ms) */
    seekDebounceMs?: number;
}

/**
 * useTimeSync Hook 返回值
 */
export interface UseTimeSyncResult {
    // === Store 状态 ===
    /** 当前时间戳 */
    currentTimestamp: number | null;
    /** 当前激活的消息 ID */
    activeMessageId: string | null;
    /** 当前激活的消息索引 */
    activeMessageIndex: number | null;
    /** 是否正在加载 */
    isLoading: boolean;
    /** 错误信息 */
    error: string | null;
    /** 当前代码内容 */
    currentCode: string | null;
    /** 前一个代码内容 (用于 Diff) */
    previousCode: string | null;
    /** 当前文件路径 */
    currentFilePath: string | null;
    /** 是否处于历史模式 */
    isHistoricalMode: boolean;
    /** Commit 信息 */
    commitInfo: {
        hash: string;
        message: string;
        timestamp: number;
    } | null;

    // === 时间轴数据 ===
    /** 会话开始时间 (Unix ms) */
    sessionStartTime: number;
    /** 会话结束时间 (Unix ms) */
    sessionEndTime: number;
    /** 当前播放位置 (Unix ms) */
    playbackTime: number;

    // === 事件处理器 ===
    /** 消息点击处理 (用于 NarrativeStream) */
    handleMessageClick: (messageId: string, message: NarrativeMessage) => void;
    /** 时间轴 Seek 处理 (用于 TimberLine) */
    handleTimelineSeek: (timestamp: number) => void;
    /** 返回当前状态 */
    handleReturnToCurrent: () => void;
    /** 重置状态 */
    reset: () => void;
}

/**
 * 解析消息时间戳为 Unix 毫秒
 */
function parseMessageTimestamp(timestamp: string): number {
    try {
        const parsed = Date.parse(timestamp);
        return isNaN(parsed) ? Date.now() : parsed;
    } catch {
        return Date.now();
    }
}

/**
 * 根据时间戳找到最近的消息
 */
function findNearestMessage(
    timestamp: number,
    messagesTimeInfo: MessageTimeInfo[]
): MessageTimeInfo | null {
    if (messagesTimeInfo.length === 0) return null;

    let closest = messagesTimeInfo[0];
    let minDiff = Math.abs(timestamp - closest.timestamp);

    for (const info of messagesTimeInfo) {
        const diff = Math.abs(timestamp - info.timestamp);
        if (diff < minDiff) {
            minDiff = diff;
            closest = info;
        }
    }

    return closest;
}

/**
 * useTimeSync - 时间旅行同步 Hook
 */
export function useTimeSync(options: UseTimeSyncOptions): UseTimeSyncResult {
    const {
        messages,
        repoPath = null,
        currentFilePath: inputFilePath = null,
        onMessageSelect,
        onTimelineSeek,
        scrollToMessage,
        seekDebounceMs = 150,
    } = options;

    // Store 状态
    const store = useTimeTravelStore();
    const {
        currentTimestamp,
        activeMessageId,
        activeMessageIndex,
        isLoading,
        error,
        currentCode,
        previousCode,
        currentFilePath,
        isHistoricalMode,
        commitInfo,
        setCurrentTime,
        jumpToMessage,
        returnToCurrent,
        reset,
    } = store;

    // Git Time Machine
    const { fetchSnapshot } = useTimeMachine(repoPath);

    // 防抖定时器
    const seekDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    // 构建消息时间信息列表
    const messagesTimeInfo = useMemo<MessageTimeInfo[]>(() => {
        return messages.map((msg, index) => ({
            index,
            id: msg.id,
            timestamp: parseMessageTimestamp(msg.timestamp),
        }));
    }, [messages]);

    // 计算会话时间范围
    const sessionTimeRange = useMemo(() => {
        if (messagesTimeInfo.length === 0) {
            const now = Date.now();
            return { start: now, end: now };
        }

        const timestamps = messagesTimeInfo.map((m) => m.timestamp);
        return {
            start: Math.min(...timestamps),
            end: Math.max(...timestamps),
        };
    }, [messagesTimeInfo]);

    // 当前播放位置 (历史模式时使用 currentTimestamp，否则使用 sessionEndTime)
    const playbackTime = currentTimestamp ?? sessionTimeRange.end;

    /**
     * 处理消息点击 (Story 2.7 AC #1)
     */
    const handleMessageClick = useCallback(
        (messageId: string, message: NarrativeMessage) => {
            const timestamp = parseMessageTimestamp(message.timestamp);
            const index = messagesTimeInfo.findIndex((m) => m.id === messageId);

            // 更新 Store
            jumpToMessage(index, messageId, timestamp);

            // 通知外部
            onMessageSelect?.(messageId, message);
            onTimelineSeek?.(timestamp);

            // 获取代码快照
            if (repoPath && inputFilePath) {
                fetchSnapshot(inputFilePath, timestamp);
            }
        },
        [
            messagesTimeInfo,
            jumpToMessage,
            onMessageSelect,
            onTimelineSeek,
            repoPath,
            inputFilePath,
            fetchSnapshot,
        ]
    );

    /**
     * 处理时间轴 Seek (Story 2.7 AC #2, #7)
     */
    const handleTimelineSeek = useCallback(
        (timestamp: number) => {
            // 更新当前时间
            setCurrentTime(timestamp);

            // 防抖处理
            if (seekDebounceRef.current) {
                clearTimeout(seekDebounceRef.current);
            }

            seekDebounceRef.current = setTimeout(() => {
                // 找到最近的消息
                const nearest = findNearestMessage(timestamp, messagesTimeInfo);

                if (nearest) {
                    // 更新 Store
                    jumpToMessage(nearest.index, nearest.id, timestamp);

                    // 滚动到消息
                    scrollToMessage?.(nearest.id);

                    // 通知外部
                    const message = messages[nearest.index];
                    if (message) {
                        onMessageSelect?.(nearest.id, message);
                    }
                }

                // 获取代码快照
                if (repoPath && inputFilePath) {
                    fetchSnapshot(inputFilePath, timestamp);
                }

                // 通知外部
                onTimelineSeek?.(timestamp);
            }, seekDebounceMs);
        },
        [
            setCurrentTime,
            seekDebounceMs,
            messagesTimeInfo,
            jumpToMessage,
            scrollToMessage,
            messages,
            onMessageSelect,
            repoPath,
            inputFilePath,
            fetchSnapshot,
            onTimelineSeek,
        ]
    );

    /**
     * 返回当前状态 (Story 2.7 AC #6)
     */
    const handleReturnToCurrent = useCallback(() => {
        // 清除防抖定时器
        if (seekDebounceRef.current) {
            clearTimeout(seekDebounceRef.current);
            seekDebounceRef.current = null;
        }

        // 重置 Store
        returnToCurrent();
    }, [returnToCurrent]);

    // 清理定时器
    useEffect(() => {
        return () => {
            if (seekDebounceRef.current) {
                clearTimeout(seekDebounceRef.current);
            }
        };
    }, []);

    return {
        // Store 状态
        currentTimestamp,
        activeMessageId,
        activeMessageIndex,
        isLoading,
        error,
        currentCode,
        previousCode,
        currentFilePath,
        isHistoricalMode,
        commitInfo,

        // 时间轴数据
        sessionStartTime: sessionTimeRange.start,
        sessionEndTime: sessionTimeRange.end,
        playbackTime,

        // 事件处理器
        handleMessageClick,
        handleTimelineSeek,
        handleReturnToCurrent,
        reset,
    };
}

export default useTimeSync;
