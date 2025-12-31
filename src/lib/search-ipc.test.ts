/**
 * search-ipc Tests - 搜索 IPC 测试
 * Story 2.10: Task 8
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { searchSessions, createDebouncedSearch } from "./search-ipc";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = invoke as ReturnType<typeof vi.fn>;

describe("search-ipc", () => {
    beforeEach(() => {
        mockInvoke.mockClear();
    });

    afterEach(() => {
        vi.clearAllMocks();
    });

    describe("searchSessions", () => {
        it("should return empty array for empty query", async () => {
            const results = await searchSessions("");
            expect(results).toEqual([]);
            expect(mockInvoke).not.toHaveBeenCalled();
        });

        it("should return empty array for whitespace query", async () => {
            const results = await searchSessions("   ");
            expect(results).toEqual([]);
            expect(mockInvoke).not.toHaveBeenCalled();
        });

        it("should call invoke with correct parameters", async () => {
            mockInvoke.mockResolvedValue([]);
            await searchSessions("test query");

            expect(mockInvoke).toHaveBeenCalledWith("search_sessions", {
                query: "test query",
                limit: 50,
            });
        });

        it("should transform results correctly", async () => {
            const mockBackendResults = [
                {
                    session_id: "s1",
                    project_id: "p1",
                    project_name: "Project One",
                    session_name: "Session One",
                    message_id: "m1",
                    content: "Test content",
                    match_positions: [[0, 4] as [number, number]],
                    timestamp: 1234567890,
                },
            ];

            mockInvoke.mockResolvedValue(mockBackendResults);
            const results = await searchSessions("test");

            expect(results).toEqual([
                {
                    id: "s1-m1",
                    projectId: "p1",
                    projectName: "Project One",
                    sessionId: "s1",
                    sessionName: "Session One",
                    messageId: "m1",
                    snippet: "Test content",
                    highlightRanges: [[0, 4]],
                    timestamp: 1234567890,
                },
            ]);
        });

        it("should handle errors gracefully", async () => {
            mockInvoke.mockRejectedValue(new Error("Backend error"));
            const results = await searchSessions("test");

            expect(results).toEqual([]);
        });
    });

    describe("createDebouncedSearch", () => {
        beforeEach(() => {
            vi.useFakeTimers();
        });

        afterEach(() => {
            vi.useRealTimers();
        });

        it("should debounce search calls", async () => {
            const { debouncedSearch } = createDebouncedSearch(300);
            const onResult = vi.fn();
            const onLoading = vi.fn();

            mockInvoke.mockResolvedValue([]);

            debouncedSearch("test", onResult, onLoading);
            debouncedSearch("test2", onResult, onLoading);
            debouncedSearch("test3", onResult, onLoading);

            // Loading should be called for each input
            expect(onLoading).toHaveBeenCalledWith(true);

            // But invoke should not be called yet
            expect(mockInvoke).not.toHaveBeenCalled();

            // Fast forward debounce delay
            await vi.advanceTimersByTimeAsync(300);

            // Only the last query should be invoked
            expect(mockInvoke).toHaveBeenCalledTimes(1);
            expect(mockInvoke).toHaveBeenCalledWith("search_sessions", {
                query: "test3",
                limit: 50,
            });
        });

        it("should return empty results for empty query immediately", async () => {
            const { debouncedSearch } = createDebouncedSearch(300);
            const onResult = vi.fn();
            const onLoading = vi.fn();

            debouncedSearch("", onResult, onLoading);

            expect(onResult).toHaveBeenCalledWith([]);
            expect(onLoading).toHaveBeenCalledWith(false);
            expect(mockInvoke).not.toHaveBeenCalled();
        });

        it("should cancel pending searches", async () => {
            const { debouncedSearch, cancel } = createDebouncedSearch(300);
            const onResult = vi.fn();
            const onLoading = vi.fn();

            mockInvoke.mockResolvedValue([]);

            debouncedSearch("test", onResult, onLoading);
            cancel();

            await vi.advanceTimersByTimeAsync(300);

            expect(mockInvoke).not.toHaveBeenCalled();
        });
    });
});
