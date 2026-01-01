/**
 * useToolPairing 测试
 * Story 2.15: Task 5.5
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useToolPairing, ToolCallMessage, ToolOutputMessage } from "./useToolPairing";

describe("useToolPairing", () => {
    const mockCalls: ToolCallMessage[] = [
        {
            id: "msg-1",
            type: "tool_call",
            toolUseId: "tool-use-1",
            toolName: "read_file",
            toolInput: { path: "/test.ts" },
        },
        {
            id: "msg-2",
            type: "tool_call",
            toolUseId: "tool-use-2",
            toolName: "run_command",
            toolInput: { command: "npm test" },
        },
    ];

    const mockOutputs: ToolOutputMessage[] = [
        {
            id: "msg-3",
            type: "tool_output",
            toolUseId: "tool-use-1",
            content: "file content here",
        },
    ];

    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it("应该构建配对 Map", () => {
        const { result } = renderHook(() => useToolPairing(mockCalls, mockOutputs));

        expect(result.current.pairs.size).toBe(2);
        expect(result.current.pairs.has("tool-use-1")).toBe(true);
        expect(result.current.pairs.has("tool-use-2")).toBe(true);
    });

    it("应该正确关联 call 和 output", () => {
        const { result } = renderHook(() => useToolPairing(mockCalls, mockOutputs));

        const pair1 = result.current.getPair("tool-use-1");
        expect(pair1?.call.toolName).toBe("read_file");
        expect(pair1?.output?.content).toBe("file content here");

        const pair2 = result.current.getPair("tool-use-2");
        expect(pair2?.call.toolName).toBe("run_command");
        expect(pair2?.output).toBeUndefined();
    });

    it("hasOutput 应该正确检测配对输出", () => {
        const { result } = renderHook(() => useToolPairing(mockCalls, mockOutputs));

        expect(result.current.hasOutput("tool-use-1")).toBe(true);
        expect(result.current.hasOutput("tool-use-2")).toBe(false);
        expect(result.current.hasOutput("non-existent")).toBe(false);
    });

    it("应该管理高亮状态", () => {
        const { result } = renderHook(() => useToolPairing(mockCalls, mockOutputs));

        expect(result.current.highlightedId).toBeNull();

        act(() => {
            result.current.setHighlightedId("tool-use-1");
        });

        expect(result.current.highlightedId).toBe("tool-use-1");

        act(() => {
            result.current.setHighlightedId(null);
        });

        expect(result.current.highlightedId).toBeNull();
    });

    it("scrollTo 应该设置临时高亮", () => {
        // Mock scrollIntoView
        const mockScrollIntoView = vi.fn();
        const mockElement = { scrollIntoView: mockScrollIntoView };
        vi.spyOn(document, "querySelector").mockReturnValue(mockElement as unknown as Element);

        const { result } = renderHook(() => useToolPairing(mockCalls, mockOutputs));

        act(() => {
            result.current.scrollTo("tool-use-1", "call");
        });

        expect(mockScrollIntoView).toHaveBeenCalledWith({
            behavior: "smooth",
            block: "center",
        });
        expect(result.current.highlightedId).toBe("tool-use-1");

        // 2秒后高亮应该清除
        act(() => {
            vi.advanceTimersByTime(2000);
        });

        expect(result.current.highlightedId).toBeNull();
    });

    it("getPair 应该返回正确的配对或 undefined", () => {
        const { result } = renderHook(() => useToolPairing(mockCalls, mockOutputs));

        expect(result.current.getPair("tool-use-1")).toBeDefined();
        expect(result.current.getPair("non-existent")).toBeUndefined();
    });

    it("空输入应该返回空 Map", () => {
        const { result } = renderHook(() => useToolPairing([], []));

        expect(result.current.pairs.size).toBe(0);
    });
});
