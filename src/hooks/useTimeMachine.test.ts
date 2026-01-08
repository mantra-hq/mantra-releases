/**
 * useTimeMachine - Git Time Machine Hook 测试
 * Story 2.7: Code Review - 补充测试覆盖
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTimeMachine } from "./useTimeMachine";

// Mock Tauri IPC
vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn(),
}));

// Mock useTimeTravelStore
const mockSetCode = vi.fn();
const mockSetCommitInfo = vi.fn();
const mockSetLoading = vi.fn();
const mockSetError = vi.fn();
// Story 2.12: 文件不存在状态 mock
const mockSetFileNotFound = vi.fn();
const mockClearFileNotFound = vi.fn();
// Story 2.30: 快照来源 mock
const mockSetSnapshotSource = vi.fn();

// Store mock state (used by selectors)
const mockStoreState = {
    setCode: mockSetCode,
    setCommitInfo: mockSetCommitInfo,
    setLoading: mockSetLoading,
    setError: mockSetError,
    setFileNotFound: mockSetFileNotFound,
    clearFileNotFound: mockClearFileNotFound,
    setSnapshotSource: mockSetSnapshotSource,
};

// Mock useTimeTravelStore with selector support
vi.mock("@/stores/useTimeTravelStore", () => ({
    useTimeTravelStore: (selector?: (state: typeof mockStoreState) => unknown) => {
        if (selector) {
            return selector(mockStoreState);
        }
        return mockStoreState;
    },
}));

import { invoke } from "@tauri-apps/api/core";

describe("useTimeMachine", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // 重置模块以清除缓存
        vi.resetModules();
    });

    afterEach(() => {
        vi.clearAllMocks();
    });

    // 每个测试使用不同的文件路径来避免缓存冲突
    let testFileCounter = 0;
    const getUniqueFilePath = () => `src/test-${++testFileCounter}-${Date.now()}.ts`;

    describe("初始化", () => {
        it("应该返回 fetchSnapshot, clearCache, prefetchSnapshot 方法", () => {
            const { result } = renderHook(() => useTimeMachine("/repo/path"));

            expect(result.current.fetchSnapshot).toBeDefined();
            expect(result.current.clearCache).toBeDefined();
            expect(result.current.prefetchSnapshot).toBeDefined();
        });
    });

    describe("fetchSnapshot", () => {
        it("无 repoPath 时应该设置错误", async () => {
            const { result } = renderHook(() => useTimeMachine(null));

            await act(async () => {
                const snapshot = await result.current.fetchSnapshot(
                    "src/index.ts",
                    1735500000000
                );
                expect(snapshot).toBeUndefined();
            });

            expect(mockSetError).toHaveBeenCalledWith("未关联 Git 仓库");
        });

        it("成功获取快照时应该更新状态", async () => {
            const mockResult = {
                content: "const a = 1;",
                commit_hash: "abc1234",
                commit_message: "feat: add feature",
                commit_timestamp: 1735500000,
                source: "git" as const,
            };

            vi.mocked(invoke).mockResolvedValueOnce(mockResult);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));

            await act(async () => {
                const snapshot = await result.current.fetchSnapshot(
                    "src/index.ts",
                    1735500000000
                );
                expect(snapshot).toEqual(mockResult);
            });

            expect(mockSetLoading).toHaveBeenCalledWith(true);
            expect(mockSetCode).toHaveBeenCalledWith("const a = 1;", "src/index.ts");
            expect(mockSetCommitInfo).toHaveBeenCalledWith({
                hash: "abc1234",
                message: "feat: add feature",
                timestamp: 1735500000000,
            });
            expect(mockSetLoading).toHaveBeenCalledWith(false);
        });

        it("IPC 失败时应该设置错误", async () => {
            vi.mocked(invoke).mockRejectedValueOnce(new Error("repo_not_found"));

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                const snapshot = await result.current.fetchSnapshot(
                    uniqueFile,
                    1735500000000
                );
                expect(snapshot).toBeUndefined();
            });

            expect(mockSetError).toHaveBeenCalledWith("未找到 Git 仓库");
        });

        it("文件不存在时应该调用 setFileNotFound (Story 2.12)", async () => {
            vi.mocked(invoke).mockRejectedValueOnce(new Error("file_not_found"));

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, 1735500000000);
            });

            // Story 2.12: 文件不存在时应该调用 setFileNotFound 而非 setError(errorMessage)
            expect(mockSetFileNotFound).toHaveBeenCalledWith(uniqueFile, 1735500000);
            // setError(null) 会在加载开始时被调用，但不应该被调用为错误消息
            expect(mockSetError).not.toHaveBeenCalledWith(expect.stringContaining("文件"));
        });

        it("commit 不存在时应该返回正确错误消息", async () => {
            vi.mocked(invoke).mockRejectedValueOnce(new Error("commit_not_found"));

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, 1735500000000);
            });

            expect(mockSetError).toHaveBeenCalledWith("该时间点没有可用的提交");
        });

        it("未知错误应该返回默认错误消息", async () => {
            vi.mocked(invoke).mockRejectedValueOnce(new Error("unknown_error"));

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, 1735500000000);
            });

            expect(mockSetError).toHaveBeenCalledWith("unknown_error");
        });

        it("Tauri 对象格式错误（FILE_NOT_FOUND 错误码）应该正确识别文件不存在", async () => {
            // 模拟 Tauri 返回的结构化错误 (新格式: 精确错误码)
            const tauriError = {
                code: "FILE_NOT_FOUND",
                message: "在 Commit de46d01 中找不到文件: src/test.tsx",
            };
            vi.mocked(invoke).mockRejectedValueOnce(tauriError);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, 1735500000000);
            });

            // 应该识别为文件不存在错误，调用 setFileNotFound
            expect(mockSetFileNotFound).toHaveBeenCalledWith(uniqueFile, 1735500000);
            // 不应该设置通用错误消息
            expect(mockSetError).not.toHaveBeenCalledWith(expect.stringContaining("Git"));
        });

        it("Tauri 对象格式错误（GIT_ERROR 通用错误码）应该正常处理", async () => {
            const tauriError = {
                code: "GIT_ERROR",
                message: "Git 操作失败: invalid commit",
            };
            vi.mocked(invoke).mockRejectedValueOnce(tauriError);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, 1735500000000);
            });

            // 应该设置 Git 错误消息 (使用错误码映射)
            expect(mockSetError).toHaveBeenCalledWith("Git 操作失败");
            expect(mockSetFileNotFound).not.toHaveBeenCalled();
        });

        it("Tauri 对象格式错误（COMMIT_NOT_FOUND 错误码）应该正常处理", async () => {
            const tauriError = {
                code: "COMMIT_NOT_FOUND",
                message: "找不到 Commit: 在 2020-01-01 之前没有找到任何 Commit",
            };
            vi.mocked(invoke).mockRejectedValueOnce(tauriError);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, 1735500000000);
            });

            // 应该设置 commit 不存在错误消息 (使用错误码映射)
            expect(mockSetError).toHaveBeenCalledWith("该时间点没有可用的提交");
            expect(mockSetFileNotFound).not.toHaveBeenCalled();
        });

        it("应该正确转换时间戳从毫秒到秒", async () => {
            const mockResult = {
                content: "code",
                commit_hash: "abc",
                commit_message: "msg",
                commit_timestamp: 1735500000,
                source: "git" as const,
            };

            vi.mocked(invoke).mockResolvedValueOnce(mockResult);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, 1735500000000);
            });

            expect(invoke).toHaveBeenCalledWith("get_snapshot_with_fallback", {
                repoPath: "/repo/path",
                filePath: uniqueFile,
                timestamp: 1735500000, // 毫秒转秒
            });
        });
    });

    describe("缓存", () => {
        it("缓存命中时应该直接返回缓存结果", async () => {
            const mockResult = {
                content: "cached code",
                commit_hash: "abc1234",
                commit_message: "cached",
                commit_timestamp: 1735500000,
                source: "git" as const,
            };

            vi.mocked(invoke).mockResolvedValueOnce(mockResult);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();
            const uniqueTimestamp = Date.now();

            // 第一次请求
            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, uniqueTimestamp);
            });

            // 清除 mock 调用记录
            vi.mocked(invoke).mockClear();
            mockSetLoading.mockClear();

            // 第二次请求 (应该命中缓存)
            await act(async () => {
                const snapshot = await result.current.fetchSnapshot(
                    uniqueFile,
                    uniqueTimestamp
                );
                expect(snapshot).toEqual(mockResult);
            });

            // 不应该调用 invoke (缓存命中)
            expect(invoke).not.toHaveBeenCalled();
            // 不应该设置 loading (缓存命中)
            expect(mockSetLoading).not.toHaveBeenCalledWith(true);
        });

        it("clearCache 应该清除缓存", async () => {
            const mockResult = {
                content: "code",
                commit_hash: "abc",
                commit_message: "msg",
                commit_timestamp: 1735500000,
                source: "git" as const,
            };

            vi.mocked(invoke).mockResolvedValue(mockResult);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();
            const uniqueTimestamp = Date.now();

            // 第一次请求
            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, uniqueTimestamp);
            });

            // 清除缓存
            act(() => {
                result.current.clearCache();
            });

            vi.mocked(invoke).mockClear();

            // 第二次请求 (缓存已清除，应该调用 invoke)
            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, uniqueTimestamp);
            });

            expect(invoke).toHaveBeenCalled();
        });
    });

    describe("prefetchSnapshot", () => {
        it("无 repoPath 时应该静默返回", async () => {
            const { result } = renderHook(() => useTimeMachine(null));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.prefetchSnapshot(uniqueFile, 1735500000000);
            });

            expect(invoke).not.toHaveBeenCalled();
        });

        it("应该预取并缓存快照", async () => {
            const mockResult = {
                content: "prefetched",
                commit_hash: "abc",
                commit_message: "msg",
                commit_timestamp: 1735500000,
                source: "git" as const,
            };

            vi.mocked(invoke).mockResolvedValueOnce(mockResult);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            // 预取
            await act(async () => {
                await result.current.prefetchSnapshot(uniqueFile, 1735500000000);
            });

            expect(invoke).toHaveBeenCalled();
            // 预取不应该更新状态
            expect(mockSetCode).not.toHaveBeenCalled();
            expect(mockSetLoading).not.toHaveBeenCalled();
        });

        it("预取失败应该静默处理", async () => {
            vi.mocked(invoke).mockRejectedValueOnce(new Error("prefetch error"));

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            // 不应该抛出错误
            await act(async () => {
                await result.current.prefetchSnapshot(uniqueFile, 1735500000000);
            });

            expect(mockSetError).not.toHaveBeenCalled();
        });
    });
});
