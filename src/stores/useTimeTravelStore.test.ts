/**
 * useTimeTravelStore - 时间旅行状态管理测试
 * Story 2.7: Task 1 验证
 * Story 2.12: Task 4 - File Not Found State 验证
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useTimeTravelStore } from "./useTimeTravelStore";

describe("useTimeTravelStore", () => {
    // 每个测试前重置 store
    beforeEach(() => {
        useTimeTravelStore.getState().reset();
    });

    describe("初始状态", () => {
        it("应该有正确的初始值", () => {
            const state = useTimeTravelStore.getState();

            expect(state.currentTimestamp).toBeNull();
            expect(state.activeMessageIndex).toBeNull();
            expect(state.activeMessageId).toBeNull();
            expect(state.isLoading).toBe(false);
            expect(state.previousCode).toBeNull();
            expect(state.currentCode).toBeNull();
            expect(state.currentFilePath).toBeNull();
            expect(state.commitInfo).toBeNull();
            expect(state.isHistoricalMode).toBe(false);
            expect(state.error).toBeNull();
            // Story 2.12: 文件不存在状态
            expect(state.fileNotFound).toBe(false);
            expect(state.notFoundPath).toBeNull();
            expect(state.notFoundTimestamp).toBeNull();
        });
    });

    describe("setCurrentTime", () => {
        it("应该更新当前时间戳并进入历史模式", () => {
            const timestamp = 1735500000000;

            useTimeTravelStore.getState().setCurrentTime(timestamp);

            const state = useTimeTravelStore.getState();
            expect(state.currentTimestamp).toBe(timestamp);
            expect(state.isHistoricalMode).toBe(true);
        });
    });

    describe("jumpToMessage", () => {
        it("应该更新消息索引、ID、时间戳并进入历史模式", () => {
            const index = 5;
            const messageId = "msg-123";
            const timestamp = 1735500000000;

            useTimeTravelStore.getState().jumpToMessage(index, messageId, timestamp);

            const state = useTimeTravelStore.getState();
            expect(state.activeMessageIndex).toBe(index);
            expect(state.activeMessageId).toBe(messageId);
            expect(state.currentTimestamp).toBe(timestamp);
            expect(state.isHistoricalMode).toBe(true);
        });
    });

    describe("setCode", () => {
        it("应该更新代码并保留前一个代码", () => {
            const code1 = "console.log('hello');";
            const code2 = "console.log('world');";
            const filePath = "src/index.ts";

            // 第一次设置
            useTimeTravelStore.getState().setCode(code1, filePath);

            let state = useTimeTravelStore.getState();
            expect(state.currentCode).toBe(code1);
            expect(state.previousCode).toBeNull();
            expect(state.currentFilePath).toBe(filePath);

            // 第二次设置
            useTimeTravelStore.getState().setCode(code2, filePath);

            state = useTimeTravelStore.getState();
            expect(state.currentCode).toBe(code2);
            expect(state.previousCode).toBe(code1);
        });
    });

    describe("setCommitInfo", () => {
        it("应该更新 Commit 信息", () => {
            const commitInfo = {
                hash: "abc1234",
                message: "feat: add login",
                timestamp: 1735500000000,
            };

            useTimeTravelStore.getState().setCommitInfo(commitInfo);

            const state = useTimeTravelStore.getState();
            expect(state.commitInfo).toEqual(commitInfo);
        });

        it("应该允许清除 Commit 信息", () => {
            const commitInfo = {
                hash: "abc1234",
                message: "feat: add login",
                timestamp: 1735500000000,
            };

            useTimeTravelStore.getState().setCommitInfo(commitInfo);
            useTimeTravelStore.getState().setCommitInfo(null);

            const state = useTimeTravelStore.getState();
            expect(state.commitInfo).toBeNull();
        });
    });

    describe("setLoading", () => {
        it("应该更新加载状态", () => {
            useTimeTravelStore.getState().setLoading(true);
            expect(useTimeTravelStore.getState().isLoading).toBe(true);

            useTimeTravelStore.getState().setLoading(false);
            expect(useTimeTravelStore.getState().isLoading).toBe(false);
        });
    });

    describe("setError", () => {
        it("应该更新错误信息", () => {
            const error = "获取快照失败";

            useTimeTravelStore.getState().setError(error);

            expect(useTimeTravelStore.getState().error).toBe(error);
        });

        it("应该允许清除错误", () => {
            useTimeTravelStore.getState().setError("error");
            useTimeTravelStore.getState().setError(null);

            expect(useTimeTravelStore.getState().error).toBeNull();
        });
    });

    describe("returnToCurrent", () => {
        it("应该重置历史模式相关状态但保留代码", () => {
            // 设置一些状态
            useTimeTravelStore.getState().jumpToMessage(5, "msg-123", 1735500000000);
            useTimeTravelStore.getState().setCode("code", "file.ts");
            useTimeTravelStore
                .getState()
                .setCommitInfo({ hash: "abc", message: "msg", timestamp: 123 });
            useTimeTravelStore.getState().setError("error");

            // 返回当前
            useTimeTravelStore.getState().returnToCurrent();

            const state = useTimeTravelStore.getState();
            expect(state.isHistoricalMode).toBe(false);
            expect(state.currentTimestamp).toBeNull();
            expect(state.activeMessageIndex).toBeNull();
            expect(state.activeMessageId).toBeNull();
            expect(state.previousCode).toBeNull();
            expect(state.commitInfo).toBeNull();
            expect(state.error).toBeNull();
            // 当前代码和文件路径保留
            expect(state.currentCode).toBe("code");
            expect(state.currentFilePath).toBe("file.ts");
        });
    });

    describe("reset", () => {
        it("应该重置所有状态", () => {
            // 设置一些状态
            useTimeTravelStore.getState().jumpToMessage(5, "msg-123", 1735500000000);
            useTimeTravelStore.getState().setCode("code", "file.ts");
            useTimeTravelStore.getState().setLoading(true);
            useTimeTravelStore.getState().setError("error");

            // 重置
            useTimeTravelStore.getState().reset();

            const state = useTimeTravelStore.getState();
            expect(state.currentTimestamp).toBeNull();
            expect(state.activeMessageIndex).toBeNull();
            expect(state.activeMessageId).toBeNull();
            expect(state.isLoading).toBe(false);
            expect(state.previousCode).toBeNull();
            expect(state.currentCode).toBeNull();
            expect(state.currentFilePath).toBeNull();
            expect(state.commitInfo).toBeNull();
            expect(state.isHistoricalMode).toBe(false);
            expect(state.error).toBeNull();
        });
    });

    // Story 2.12: 文件不存在状态测试
    describe("setFileNotFound (Story 2.12)", () => {
        it("应该设置文件不存在状态", () => {
            const path = "src/nonexistent.ts";
            const timestamp = 1735500000;

            useTimeTravelStore.getState().setFileNotFound(path, timestamp);

            const state = useTimeTravelStore.getState();
            expect(state.fileNotFound).toBe(true);
            expect(state.notFoundPath).toBe(path);
            expect(state.notFoundTimestamp).toBe(timestamp);
            // 应该清除其他错误
            expect(state.error).toBeNull();
        });

        it("设置文件不存在状态时应该清除之前的错误", () => {
            useTimeTravelStore.getState().setError("之前的错误");
            useTimeTravelStore.getState().setFileNotFound("file.ts", 123);

            const state = useTimeTravelStore.getState();
            expect(state.fileNotFound).toBe(true);
            expect(state.error).toBeNull();
        });
    });

    describe("clearFileNotFound (Story 2.12)", () => {
        it("应该清除文件不存在状态", () => {
            // 先设置状态
            useTimeTravelStore.getState().setFileNotFound("file.ts", 123);

            // 然后清除
            useTimeTravelStore.getState().clearFileNotFound();

            const state = useTimeTravelStore.getState();
            expect(state.fileNotFound).toBe(false);
            expect(state.notFoundPath).toBeNull();
            expect(state.notFoundTimestamp).toBeNull();
        });
    });

    describe("returnToCurrent 应该清除文件不存在状态 (Story 2.12)", () => {
        it("returnToCurrent 应该同时清除文件不存在状态", () => {
            // 设置文件不存在状态
            useTimeTravelStore.getState().setFileNotFound("file.ts", 123);
            useTimeTravelStore.getState().jumpToMessage(1, "msg-1", 1735500000000);

            // 返回当前
            useTimeTravelStore.getState().returnToCurrent();

            const state = useTimeTravelStore.getState();
            expect(state.fileNotFound).toBe(false);
            expect(state.notFoundPath).toBeNull();
            expect(state.notFoundTimestamp).toBeNull();
        });
    });
});
