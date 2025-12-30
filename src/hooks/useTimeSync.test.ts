/**
 * useTimeSync - 时间旅行同步 Hook 测试
 * Story 2.7: Code Review - 补充测试覆盖
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTimeSync } from "./useTimeSync";
import type { NarrativeMessage } from "@/types/message";

// Mock useTimeTravelStore
const mockSetCurrentTime = vi.fn();
const mockJumpToMessage = vi.fn();
const mockReturnToCurrent = vi.fn();
const mockReset = vi.fn();

vi.mock("@/stores/useTimeTravelStore", () => ({
    useTimeTravelStore: () => ({
        currentTimestamp: null,
        activeMessageId: null,
        activeMessageIndex: null,
        isLoading: false,
        error: null,
        currentCode: null,
        previousCode: null,
        currentFilePath: null,
        isHistoricalMode: false,
        commitInfo: null,
        setCurrentTime: mockSetCurrentTime,
        jumpToMessage: mockJumpToMessage,
        returnToCurrent: mockReturnToCurrent,
        reset: mockReset,
    }),
}));

// Mock useTimeMachine
const mockFetchSnapshot = vi.fn();

vi.mock("@/hooks/useTimeMachine", () => ({
    useTimeMachine: () => ({
        fetchSnapshot: mockFetchSnapshot,
    }),
}));

/**
 * 创建测试消息
 */
function createTestMessage(
    id: string,
    timestamp: string,
    role: "user" | "assistant" = "user"
): NarrativeMessage {
    return {
        id,
        role,
        timestamp,
        content: [{ type: "text", content: `Message ${id}` }],
    };
}

describe("useTimeSync", () => {
    const testMessages: NarrativeMessage[] = [
        createTestMessage("msg-1", "2025-12-30T10:00:00.000Z"),
        createTestMessage("msg-2", "2025-12-30T10:05:00.000Z", "assistant"),
        createTestMessage("msg-3", "2025-12-30T10:10:00.000Z"),
        createTestMessage("msg-4", "2025-12-30T10:15:00.000Z", "assistant"),
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe("初始化", () => {
        it("应该返回所有必要的状态和方法", () => {
            const { result } = renderHook(() =>
                useTimeSync({ messages: testMessages })
            );

            // 状态
            expect(result.current.currentTimestamp).toBeNull();
            expect(result.current.activeMessageId).toBeNull();
            expect(result.current.isLoading).toBe(false);
            expect(result.current.isHistoricalMode).toBe(false);

            // 时间轴数据
            expect(result.current.sessionStartTime).toBeDefined();
            expect(result.current.sessionEndTime).toBeDefined();
            expect(result.current.playbackTime).toBeDefined();

            // 方法
            expect(result.current.handleMessageClick).toBeDefined();
            expect(result.current.handleTimelineSeek).toBeDefined();
            expect(result.current.handleReturnToCurrent).toBeDefined();
            expect(result.current.reset).toBeDefined();
        });

        it("应该正确计算会话时间范围", () => {
            const { result } = renderHook(() =>
                useTimeSync({ messages: testMessages })
            );

            const msg1Time = Date.parse("2025-12-30T10:00:00.000Z");
            const msg4Time = Date.parse("2025-12-30T10:15:00.000Z");

            expect(result.current.sessionStartTime).toBe(msg1Time);
            expect(result.current.sessionEndTime).toBe(msg4Time);
        });

        it("空消息列表应该返回当前时间", () => {
            const now = Date.now();
            vi.setSystemTime(now);

            const { result } = renderHook(() => useTimeSync({ messages: [] }));

            expect(result.current.sessionStartTime).toBe(now);
            expect(result.current.sessionEndTime).toBe(now);
        });
    });

    describe("handleMessageClick (AC #1)", () => {
        it("点击消息应该调用 jumpToMessage", () => {
            const { result } = renderHook(() =>
                useTimeSync({ messages: testMessages })
            );

            act(() => {
                result.current.handleMessageClick("msg-2", testMessages[1]);
            });

            expect(mockJumpToMessage).toHaveBeenCalledWith(
                1, // index
                "msg-2",
                Date.parse("2025-12-30T10:05:00.000Z")
            );
        });

        it("点击消息应该调用外部回调", () => {
            const onMessageSelect = vi.fn();
            const onTimelineSeek = vi.fn();

            const { result } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    onMessageSelect,
                    onTimelineSeek,
                })
            );

            act(() => {
                result.current.handleMessageClick("msg-2", testMessages[1]);
            });

            expect(onMessageSelect).toHaveBeenCalledWith("msg-2", testMessages[1]);
            expect(onTimelineSeek).toHaveBeenCalledWith(
                Date.parse("2025-12-30T10:05:00.000Z")
            );
        });

        it("有 repoPath 和 filePath 时应该获取快照", () => {
            const { result } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    repoPath: "/repo/path",
                    currentFilePath: "src/index.ts",
                })
            );

            act(() => {
                result.current.handleMessageClick("msg-2", testMessages[1]);
            });

            expect(mockFetchSnapshot).toHaveBeenCalledWith(
                "src/index.ts",
                Date.parse("2025-12-30T10:05:00.000Z")
            );
        });

        it("无 repoPath 时不应该获取快照", () => {
            const { result } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    currentFilePath: "src/index.ts",
                })
            );

            act(() => {
                result.current.handleMessageClick("msg-2", testMessages[1]);
            });

            expect(mockFetchSnapshot).not.toHaveBeenCalled();
        });
    });

    describe("handleTimelineSeek (AC #2, #7)", () => {
        it("应该立即更新当前时间", () => {
            const { result } = renderHook(() =>
                useTimeSync({ messages: testMessages })
            );

            const seekTime = Date.parse("2025-12-30T10:07:00.000Z");

            act(() => {
                result.current.handleTimelineSeek(seekTime);
            });

            expect(mockSetCurrentTime).toHaveBeenCalledWith(seekTime);
        });

        it("应该防抖后跳转到最近消息", () => {
            const scrollToMessage = vi.fn();

            const { result } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    scrollToMessage,
                    seekDebounceMs: 150,
                })
            );

            const seekTime = Date.parse("2025-12-30T10:07:00.000Z");

            act(() => {
                result.current.handleTimelineSeek(seekTime);
            });

            // 防抖期间不应该调用
            expect(mockJumpToMessage).not.toHaveBeenCalled();
            expect(scrollToMessage).not.toHaveBeenCalled();

            // 快进 150ms
            act(() => {
                vi.advanceTimersByTime(150);
            });

            // 防抖后应该调用
            expect(mockJumpToMessage).toHaveBeenCalled();
            // 最近的消息是 msg-2 (10:05)
            expect(scrollToMessage).toHaveBeenCalledWith("msg-2");
        });

        it("快速连续 seek 应该只触发最后一次", () => {
            const scrollToMessage = vi.fn();

            const { result } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    scrollToMessage,
                    seekDebounceMs: 150,
                })
            );

            // 快速连续 seek
            act(() => {
                result.current.handleTimelineSeek(
                    Date.parse("2025-12-30T10:02:00.000Z")
                );
            });

            act(() => {
                vi.advanceTimersByTime(50);
                result.current.handleTimelineSeek(
                    Date.parse("2025-12-30T10:08:00.000Z")
                );
            });

            act(() => {
                vi.advanceTimersByTime(50);
                result.current.handleTimelineSeek(
                    Date.parse("2025-12-30T10:12:00.000Z")
                );
            });

            // 快进到防抖结束
            act(() => {
                vi.advanceTimersByTime(150);
            });

            // 只应该调用一次 (最后一次 seek)
            expect(scrollToMessage).toHaveBeenCalledTimes(1);
            // 最近的消息是 msg-3 (10:10)
            expect(scrollToMessage).toHaveBeenCalledWith("msg-3");
        });

        it("有 repoPath 时应该获取快照", () => {
            const { result } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    repoPath: "/repo/path",
                    currentFilePath: "src/index.ts",
                    seekDebounceMs: 150,
                })
            );

            const seekTime = Date.parse("2025-12-30T10:07:00.000Z");

            act(() => {
                result.current.handleTimelineSeek(seekTime);
            });

            act(() => {
                vi.advanceTimersByTime(150);
            });

            expect(mockFetchSnapshot).toHaveBeenCalledWith("src/index.ts", seekTime);
        });
    });

    describe("handleReturnToCurrent (AC #6)", () => {
        it("应该调用 returnToCurrent", () => {
            const { result } = renderHook(() =>
                useTimeSync({ messages: testMessages })
            );

            act(() => {
                result.current.handleReturnToCurrent();
            });

            expect(mockReturnToCurrent).toHaveBeenCalled();
        });

        it("应该清除待处理的防抖定时器", () => {
            const scrollToMessage = vi.fn();

            const { result } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    scrollToMessage,
                    seekDebounceMs: 150,
                })
            );

            // 开始 seek
            act(() => {
                result.current.handleTimelineSeek(
                    Date.parse("2025-12-30T10:07:00.000Z")
                );
            });

            // 立即返回当前
            act(() => {
                result.current.handleReturnToCurrent();
            });

            // 快进超过防抖时间
            act(() => {
                vi.advanceTimersByTime(200);
            });

            // 不应该调用 scroll (定时器已清除)
            expect(scrollToMessage).not.toHaveBeenCalled();
        });
    });

    describe("findNearestMessage 算法", () => {
        it("应该找到时间上最近的消息", () => {
            const scrollToMessage = vi.fn();

            const { result } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    scrollToMessage,
                    seekDebounceMs: 0,
                })
            );

            // 测试各种时间点
            const testCases = [
                { time: "2025-12-30T10:02:00.000Z", expected: "msg-1" }, // 更接近 msg-1
                { time: "2025-12-30T10:03:00.000Z", expected: "msg-2" }, // 更接近 msg-2
                { time: "2025-12-30T10:07:00.000Z", expected: "msg-2" }, // 更接近 msg-2
                { time: "2025-12-30T10:08:00.000Z", expected: "msg-3" }, // 更接近 msg-3
                { time: "2025-12-30T10:14:00.000Z", expected: "msg-4" }, // 更接近 msg-4
            ];

            for (const { time, expected } of testCases) {
                scrollToMessage.mockClear();

                act(() => {
                    result.current.handleTimelineSeek(Date.parse(time));
                });

                act(() => {
                    vi.advanceTimersByTime(1);
                });

                expect(scrollToMessage).toHaveBeenCalledWith(expected);
            }
        });
    });

    describe("清理", () => {
        it("卸载时应该清除定时器", () => {
            const { result, unmount } = renderHook(() =>
                useTimeSync({
                    messages: testMessages,
                    seekDebounceMs: 150,
                })
            );

            // 开始 seek
            act(() => {
                result.current.handleTimelineSeek(
                    Date.parse("2025-12-30T10:07:00.000Z")
                );
            });

            // 卸载
            unmount();

            // 不应该有错误
            expect(() => {
                vi.advanceTimersByTime(200);
            }).not.toThrow();
        });
    });
});
