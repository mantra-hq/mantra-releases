/**
 * useLogStore - 应用日志状态管理测试
 * Story 2.28: 运行日志复制
 * AC: #1 复制入口, #2 日志内容, #3 复制反馈
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { useLogStore } from "./useLogStore";

describe("useLogStore", () => {
    // 每个测试前重置 store
    beforeEach(() => {
        useLogStore.getState().clear();
    });

    describe("初始状态", () => {
        it("应该有正确的初始值", () => {
            const state = useLogStore.getState();
            expect(state.entries).toEqual([]);
        });
    });

    describe("addLog", () => {
        it("应该添加日志条目", () => {
            useLogStore.getState().addLog("info", "Test action", "Test details");

            const state = useLogStore.getState();
            expect(state.entries).toHaveLength(1);
            expect(state.entries[0].level).toBe("info");
            expect(state.entries[0].action).toBe("Test action");
            expect(state.entries[0].details).toBe("Test details");
            expect(state.entries[0].timestamp).toBeDefined();
        });

        it("应该限制日志条目数量为 500", () => {
            const store = useLogStore.getState();
            // 添加 510 条日志
            for (let i = 0; i < 510; i++) {
                store.addLog("info", `Action ${i}`);
            }

            const state = useLogStore.getState();
            expect(state.entries).toHaveLength(500);
            // 应该保留最新的 500 条，即从 10 开始
            expect(state.entries[0].action).toBe("Action 10");
            expect(state.entries[499].action).toBe("Action 509");
        });
    });

    describe("便捷方法 (info, warn, error)", () => {
        it("info() 应该添加 info 级别日志", () => {
            useLogStore.getState().info("Info action", "Info details");

            const state = useLogStore.getState();
            expect(state.entries[0].level).toBe("info");
            expect(state.entries[0].action).toBe("Info action");
        });

        it("warn() 应该添加 warn 级别日志", () => {
            useLogStore.getState().warn("Warning action");

            const state = useLogStore.getState();
            expect(state.entries[0].level).toBe("warn");
            expect(state.entries[0].action).toBe("Warning action");
        });

        it("error() 应该添加 error 级别日志", () => {
            useLogStore.getState().error("Error action", "Error reason");

            const state = useLogStore.getState();
            expect(state.entries[0].level).toBe("error");
            expect(state.entries[0].action).toBe("Error action");
            expect(state.entries[0].details).toBe("Error reason");
        });
    });

    describe("clear", () => {
        it("应该清空所有日志", () => {
            useLogStore.getState().info("Action 1");
            useLogStore.getState().info("Action 2");
            useLogStore.getState().clear();

            expect(useLogStore.getState().entries).toHaveLength(0);
        });
    });

    describe("formatLogs (AC #2)", () => {
        it("无日志时应返回空消息", () => {
            const result = useLogStore.getState().formatLogs();
            expect(result).toBe("No logs available.");
        });

        it("应该格式化日志为可读文本并包含系统信息", () => {
            useLogStore.getState().info("Test action", "Some details");

            const result = useLogStore.getState().formatLogs();

            // 检查头部信息
            expect(result).toContain("Mantra Application Logs");
            expect(result).toContain("Total entries: 1");
            expect(result).toContain("Platform:");
            expect(result).toContain("User Agent:");

            // 检查日志内容
            expect(result).toContain("[INFO]");
            expect(result).toContain("Test action");
            expect(result).toContain("Some details");
        });

        it("应该正确处理多条日志", () => {
            useLogStore.getState().info("Action 1");
            useLogStore.getState().warn("Action 2");
            useLogStore.getState().error("Action 3");

            const result = useLogStore.getState().formatLogs();

            expect(result).toContain("Total entries: 3");
            expect(result).toContain("[INFO]");
            expect(result).toContain("[WARN]");
            expect(result).toContain("[ERROR]");
            expect(result).toContain("Action 1");
            expect(result).toContain("Action 2");
            expect(result).toContain("Action 3");
        });

        it("应该包含时间戳", () => {
            useLogStore.getState().info("Test action");

            const result = useLogStore.getState().formatLogs();

            // ISO 时间戳格式
            expect(result).toMatch(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
        });
    });

    describe("copyToClipboard (AC #3)", () => {
        it("成功复制时应返回 true", async () => {
            // Mock clipboard API
            const writeTextMock = vi.fn().mockResolvedValue(undefined);
            Object.assign(navigator, {
                clipboard: { writeText: writeTextMock },
            });

            useLogStore.getState().info("Test log");
            const result = await useLogStore.getState().copyToClipboard();

            expect(result).toBe(true);
            expect(writeTextMock).toHaveBeenCalled();
        });

        it("复制失败时应返回 false", async () => {
            // Mock clipboard API 失败
            Object.assign(navigator, {
                clipboard: {
                    writeText: vi.fn().mockRejectedValue(new Error("Clipboard error")),
                },
            });

            const result = await useLogStore.getState().copyToClipboard();
            expect(result).toBe(false);
        });
    });
});
