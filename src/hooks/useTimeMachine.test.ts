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

vi.mock("@/stores/useTimeTravelStore", () => ({
    useTimeTravelStore: () => ({
        setCode: mockSetCode,
        setCommitInfo: mockSetCommitInfo,
        setLoading: mockSetLoading,
        setError: mockSetError,
        // Story 2.12: 文件不存在状态
        setFileNotFound: mockSetFileNotFound,
        clearFileNotFound: mockClearFileNotFound,
    }),
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

        it("应该正确转换时间戳从毫秒到秒", async () => {
            const mockResult = {
                content: "code",
                commit_hash: "abc",
                commit_message: "msg",
                commit_timestamp: 1735500000,
            };

            vi.mocked(invoke).mockResolvedValueOnce(mockResult);

            const { result } = renderHook(() => useTimeMachine("/repo/path"));
            const uniqueFile = getUniqueFilePath();

            await act(async () => {
                await result.current.fetchSnapshot(uniqueFile, 1735500000000);
            });

            expect(invoke).toHaveBeenCalledWith("get_snapshot_at_time", {
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
