/**
 * useTimeMachine - Git Time Machine 集成 Hook
 * Story 2.7: Task 2 - AC #3, #4
 * Story 2.12: Task 4 - AC #5 (File Not Found Handling)
 *
 * 功能:
 * - 封装 Tauri IPC 调用获取历史快照
 * - LRU 缓存优化 (最近 50 个快照)
 * - 加载状态和错误处理
 * - 响应时间目标 <200ms
 * - 文件不存在时保持上一个有效状态 (Story 2.12)
 */

import { useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTimeTravelStore } from "@/stores/useTimeTravelStore";

/**
 * 快照结果接口 (与 Rust 后端对齐)
 */
export interface SnapshotResult {
    /** 文件内容 */
    content: string;
    /** Commit Hash */
    commit_hash: string;
    /** Commit 消息 */
    commit_message: string;
    /** Commit 时间戳 (Unix seconds) */
    commit_timestamp: number;
}

/**
 * LRU 缓存实现
 */
class LRUCache<K, V> {
    private maxSize: number;
    private cache: Map<K, V>;

    constructor(maxSize: number) {
        this.maxSize = maxSize;
        this.cache = new Map();
    }

    get(key: K): V | undefined {
        if (!this.cache.has(key)) return undefined;
        // 移动到末尾 (最近使用)
        const value = this.cache.get(key)!;
        this.cache.delete(key);
        this.cache.set(key, value);
        return value;
    }

    set(key: K, value: V): void {
        if (this.cache.has(key)) {
            this.cache.delete(key);
        } else if (this.cache.size >= this.maxSize) {
            // 删除最旧的条目 (第一个)
            const firstKey = this.cache.keys().next().value;
            if (firstKey !== undefined) {
                this.cache.delete(firstKey);
            }
        }
        this.cache.set(key, value);
    }

    has(key: K): boolean {
        return this.cache.has(key);
    }

    clear(): void {
        this.cache.clear();
    }
}

// 全局缓存实例 (最多 50 个快照)
const snapshotCache = new LRUCache<string, SnapshotResult>(50);

/**
 * 错误消息映射
 */
const ERROR_MESSAGES: Record<string, string> = {
    file_not_found: "文件在该时间点不存在",
    repo_not_found: "未找到 Git 仓库",
    commit_not_found: "该时间点没有可用的提交",
    git_error: "Git 操作失败",
    default: "获取历史快照失败",
};

/**
 * 解析错误消息
 */
function getErrorMessage(error: unknown): string {
    if (error instanceof Error) {
        // 检查是否匹配已知错误类型
        for (const [key, message] of Object.entries(ERROR_MESSAGES)) {
            if (error.message.toLowerCase().includes(key)) {
                return message;
            }
        }
        return error.message;
    }
    return ERROR_MESSAGES.default;
}

/**
 * Story 2.12 AC #5: 检测是否是文件不存在错误
 */
function isFileNotFoundError(error: unknown): boolean {
    if (error instanceof Error) {
        const msg = error.message.toLowerCase();
        return (
            msg.includes("file_not_found") ||
            msg.includes("filenotfound") ||
            msg.includes("not found") ||
            msg.includes("does not exist") ||
            msg.includes("no such file")
        );
    }
    if (typeof error === "string") {
        const msg = error.toLowerCase();
        return (
            msg.includes("file_not_found") ||
            msg.includes("not found") ||
            msg.includes("does not exist")
        );
    }
    return false;
}

/**
 * 生成缓存键
 */
function getCacheKey(repoPath: string, filePath: string, timestamp: number): string {
    return `${repoPath}:${filePath}:${timestamp}`;
}

/**
 * useTimeMachine Hook
 *
 * @param repoPath - Git 仓库路径 (可选，无仓库时返回错误提示)
 * @returns Hook 方法
 */
export function useTimeMachine(repoPath: string | null) {
    const {
        setCode,
        setCommitInfo,
        setLoading,
        setError,
        setFileNotFound,
        clearFileNotFound,
    } = useTimeTravelStore();

    // 用于追踪最新请求，避免竞态条件
    const requestIdRef = useRef(0);

    /**
     * 获取指定时间点的文件快照
     *
     * @param filePath - 文件相对路径
     * @param timestamp - Unix 时间戳 (毫秒)
     * @returns 快照结果或 undefined (出错时)
     */
    const fetchSnapshot = useCallback(
        async (
            filePath: string,
            timestamp: number
        ): Promise<SnapshotResult | undefined> => {
            if (!repoPath) {
                setError("未关联 Git 仓库");
                return undefined;
            }

            // 增加请求 ID
            const currentRequestId = ++requestIdRef.current;

            // 转换为秒 (Rust 后端期望秒级时间戳)
            const timestampSeconds = Math.floor(timestamp / 1000);
            const cacheKey = getCacheKey(repoPath, filePath, timestampSeconds);

            // 检查缓存
            if (snapshotCache.has(cacheKey)) {
                const cached = snapshotCache.get(cacheKey)!;

                // 检查是否仍是最新请求
                if (currentRequestId !== requestIdRef.current) {
                    return undefined;
                }

                // Story 2.12: 清除文件不存在状态
                clearFileNotFound();

                setCode(cached.content, filePath);
                setCommitInfo({
                    hash: cached.commit_hash,
                    message: cached.commit_message,
                    timestamp: cached.commit_timestamp * 1000, // 转回毫秒
                });
                return cached;
            }

            // 开始加载
            setLoading(true);
            setError(null);
            // Story 2.12: 清除之前的文件不存在状态
            clearFileNotFound();

            try {
                const startTime = performance.now();

                const result = await invoke<SnapshotResult>("get_snapshot_at_time", {
                    repoPath: repoPath,
                    filePath: filePath,
                    timestamp: timestampSeconds,
                });

                const elapsed = performance.now() - startTime;
                if (elapsed > 200) {
                    console.warn(
                        `[useTimeMachine] 快照获取耗时 ${elapsed.toFixed(0)}ms，超过 200ms 目标`
                    );
                }

                // 检查是否仍是最新请求 (避免更新过时数据)
                if (currentRequestId !== requestIdRef.current) {
                    return undefined;
                }

                // 缓存结果
                snapshotCache.set(cacheKey, result);

                // 更新状态
                setCode(result.content, filePath);
                setCommitInfo({
                    hash: result.commit_hash,
                    message: result.commit_message,
                    timestamp: result.commit_timestamp * 1000,
                });

                return result;
            } catch (err) {
                // 检查是否仍是最新请求
                if (currentRequestId !== requestIdRef.current) {
                    return undefined;
                }

                const errorMessage = getErrorMessage(err);

                // Story 2.12 AC #5: 检测文件不存在错误
                const isFileNotFound = isFileNotFoundError(err);

                if (isFileNotFound) {
                    // 设置文件不存在状态，但不设置通用错误
                    setFileNotFound(filePath, timestampSeconds);
                    console.log(
                        `[useTimeMachine] 文件不存在: ${filePath} @ ${new Date(timestamp).toISOString()}`
                    );
                } else {
                    // 其他错误设置通用错误
                    setError(errorMessage);
                    console.error("[useTimeMachine] 获取快照失败:", err);
                }

                return undefined;
            } finally {
                // 检查是否仍是最新请求
                if (currentRequestId === requestIdRef.current) {
                    setLoading(false);
                }
            }
        },
        [repoPath, setCode, setCommitInfo, setLoading, setError, setFileNotFound, clearFileNotFound]
    );

    /**
     * 清除缓存
     */
    const clearCache = useCallback(() => {
        snapshotCache.clear();
    }, []);

    /**
     * 预取快照 (用于性能优化)
     * 不更新状态，仅填充缓存
     */
    const prefetchSnapshot = useCallback(
        async (filePath: string, timestamp: number): Promise<void> => {
            if (!repoPath) return;

            const timestampSeconds = Math.floor(timestamp / 1000);
            const cacheKey = getCacheKey(repoPath, filePath, timestampSeconds);

            // 已缓存则跳过
            if (snapshotCache.has(cacheKey)) return;

            try {
                const result = await invoke<SnapshotResult>("get_snapshot_at_time", {
                    repoPath: repoPath,
                    filePath: filePath,
                    timestamp: timestampSeconds,
                });
                snapshotCache.set(cacheKey, result);
            } catch {
                // 预取失败静默处理
            }
        },
        [repoPath]
    );

    return {
        fetchSnapshot,
        clearCache,
        prefetchSnapshot,
    };
}

export default useTimeMachine;
