/**
 * useLogStore - 应用运行日志管理
 * Story 2.28: 运行日志复制
 *
 * 管理应用运行日志:
 * - 记录关键操作日志（导入、同步、错误等）
 * - 格式化日志为可读文本
 * - 复制到剪贴板
 */

import { create } from "zustand";

/**
 * 日志级别
 */
export type LogLevel = "info" | "warn" | "error";

/**
 * 日志条目
 */
export interface LogEntry {
    /** 时间戳 */
    timestamp: string;
    /** 日志级别 */
    level: LogLevel;
    /** 操作描述 */
    action: string;
    /** 详细信息 */
    details?: string;
}

/**
 * 日志状态接口
 */
export interface LogState {
    // ======== 状态 ========
    /** 日志条目列表 */
    entries: LogEntry[];

    // ======== Actions ========
    /** 添加日志 */
    addLog: (level: LogLevel, action: string, details?: string) => void;
    /** 添加 info 日志 */
    info: (action: string, details?: string) => void;
    /** 添加 warn 日志 */
    warn: (action: string, details?: string) => void;
    /** 添加 error 日志 */
    error: (action: string, details?: string) => void;
    /** 清空日志 */
    clear: () => void;
    /** 格式化日志为文本 */
    formatLogs: () => string;
    /** 复制日志到剪贴板 */
    copyToClipboard: () => Promise<boolean>;
}

/**
 * 最大日志条目数量
 */
const MAX_LOG_ENTRIES = 500;

/**
 * 格式化时间戳
 */
const formatTimestamp = (): string => {
    return new Date().toISOString();
};

/**
 * 日志状态 Store
 */
export const useLogStore = create<LogState>((set, get) => ({
    entries: [],

    addLog: (level, action, details) =>
        set((state) => {
            const entry: LogEntry = {
                timestamp: formatTimestamp(),
                level,
                action,
                details,
            };
            // 保持最近 MAX_LOG_ENTRIES 条日志
            const newEntries = [...state.entries, entry].slice(-MAX_LOG_ENTRIES);
            return { entries: newEntries };
        }),

    info: (action, details) => get().addLog("info", action, details),

    warn: (action, details) => get().addLog("warn", action, details),

    error: (action, details) => get().addLog("error", action, details),

    clear: () => set({ entries: [] }),

    formatLogs: () => {
        const { entries } = get();
        if (entries.length === 0) {
            return "No logs available.";
        }

        const lines = entries.map((entry) => {
            const levelTag = `[${entry.level.toUpperCase()}]`.padEnd(7);
            const time = entry.timestamp;
            const action = entry.action;
            const details = entry.details ? ` - ${entry.details}` : "";
            return `${time} ${levelTag} ${action}${details}`;
        });

        // 添加头部信息 (包含系统信息用于调试)
        const header = [
            "=".repeat(60),
            "Mantra Application Logs",
            `Generated: ${new Date().toISOString()}`,
            `Platform: ${navigator.platform}`,
            `User Agent: ${navigator.userAgent}`,
            `Total entries: ${entries.length}`,
            "=".repeat(60),
            "",
        ];

        return [...header, ...lines].join("\n");
    },

    copyToClipboard: async () => {
        try {
            const text = get().formatLogs();
            await navigator.clipboard.writeText(text);
            return true;
        } catch {
            return false;
        }
    },
}));

export default useLogStore;
