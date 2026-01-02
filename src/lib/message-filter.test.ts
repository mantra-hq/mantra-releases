/**
 * message-filter Tests - 消息过滤逻辑测试
 * Story 2.16: Task 6.4
 */

import { describe, it, expect } from "vitest";
import {
    filterWithPairedResults,
    isMessageVisible,
} from "./message-filter";
import type { NarrativeMessage, ContentBlock } from "@/types/message";

// 测试数据工厂函数
function createMessage(
    id: string,
    role: "user" | "assistant",
    blocks: ContentBlock[]
): NarrativeMessage {
    return {
        id,
        role,
        timestamp: new Date().toISOString(),
        content: blocks,
    };
}

// 测试用的内容块
const textBlock = (content: string): ContentBlock => ({
    type: "text",
    content,
});

const toolUseBlock = (toolName: string, toolUseId: string): ContentBlock => ({
    type: "tool_use",
    content: "",
    toolName,
    toolInput: {},
    toolUseId,
});

const toolResultBlock = (toolUseId: string, content: string = "result"): ContentBlock => ({
    type: "tool_result",
    content,
    toolUseId,
});

const thinkingBlock: ContentBlock = {
    type: "thinking",
    content: "Let me think...",
};

describe("filterWithPairedResults", () => {
    describe("basic filtering", () => {
        it("should return empty result for empty messages", () => {
            const result = filterWithPairedResults([], new Set(), "");
            expect(result.messages).toEqual([]);
            expect(result.filteredCount).toBe(0);
            expect(result.totalCount).toBe(0);
        });

        it("should return all messages when no filters active", () => {
            const messages = [
                createMessage("1", "user", [textBlock("hello")]),
                createMessage("2", "assistant", [textBlock("hi")]),
            ];

            const result = filterWithPairedResults(messages, new Set(), "");
            expect(result.messages).toHaveLength(2);
            expect(result.filteredCount).toBe(2);
            expect(result.totalCount).toBe(2);
        });

        it("should filter by type", () => {
            const messages = [
                createMessage("1", "user", [textBlock("hello")]),
                createMessage("2", "assistant", [thinkingBlock]),
                createMessage("3", "assistant", [textBlock("response")]),
            ];

            const result = filterWithPairedResults(messages, new Set(["thinking"]), "");
            expect(result.messages).toHaveLength(1);
            expect(result.messages[0].id).toBe("2");
        });

        it("should filter by search query", () => {
            const messages = [
                createMessage("1", "user", [textBlock("hello world")]),
                createMessage("2", "assistant", [textBlock("goodbye")]),
            ];

            const result = filterWithPairedResults(messages, new Set(), "hello");
            expect(result.messages).toHaveLength(1);
            expect(result.messages[0].id).toBe("1");
        });

        it("should combine type and search filters (AND logic)", () => {
            const messages = [
                createMessage("1", "user", [textBlock("hello")]),
                createMessage("2", "assistant", [textBlock("hello")]),
                createMessage("3", "assistant", [thinkingBlock]),
            ];

            // Only messages with text type AND containing "hello"
            const result = filterWithPairedResults(
                messages,
                new Set(["conversation"]),
                "hello"
            );
            expect(result.messages).toHaveLength(2);
        });
    });

    describe("paired filtering (AC #5, #6)", () => {
        it("should include paired ToolResult when ToolCall matches", () => {
            const messages = [
                createMessage("1", "assistant", [toolUseBlock("view_file", "tool-1")]),
                createMessage("2", "assistant", [toolResultBlock("tool-1", "file contents")]),
            ];

            // Filter for file type (matches view_file)
            const result = filterWithPairedResults(messages, new Set(["file"]), "");

            // Both messages should be included
            expect(result.messages).toHaveLength(2);
            expect(result.messageIds.has("1")).toBe(true);
            expect(result.messageIds.has("2")).toBe(true);
        });

        it("should include paired ToolCall when ToolResult matches search", () => {
            const messages = [
                createMessage("1", "assistant", [toolUseBlock("run_command", "tool-2")]),
                createMessage("2", "assistant", [toolResultBlock("tool-2", "npm test passed")]),
            ];

            // Search for "passed" - only in result, but should include call
            const result = filterWithPairedResults(messages, new Set(), "passed");

            expect(result.messages).toHaveLength(2);
            expect(result.messageIds.has("1")).toBe(true);
            expect(result.messageIds.has("2")).toBe(true);
        });

        it("should handle multiple tool pairs correctly", () => {
            const messages = [
                createMessage("1", "assistant", [toolUseBlock("view_file", "tool-a")]),
                createMessage("2", "assistant", [toolResultBlock("tool-a")]),
                createMessage("3", "assistant", [toolUseBlock("grep_search", "tool-b")]),
                createMessage("4", "assistant", [toolResultBlock("tool-b")]),
                createMessage("5", "user", [textBlock("thanks")]),
            ];

            // Filter for search type
            const result = filterWithPairedResults(messages, new Set(["search"]), "");

            // Should include grep_search and its result, but not view_file pair
            expect(result.messages).toHaveLength(2);
            expect(result.messageIds.has("3")).toBe(true);
            expect(result.messageIds.has("4")).toBe(true);
        });

        it("should maintain original order", () => {
            const messages = [
                createMessage("1", "user", [textBlock("hello")]),
                createMessage("2", "assistant", [toolUseBlock("view_file", "tool-1")]),
                createMessage("3", "assistant", [toolResultBlock("tool-1")]),
                createMessage("4", "user", [textBlock("thanks")]),
            ];

            // Filter for conversation type
            const result = filterWithPairedResults(
                messages,
                new Set(["conversation"]),
                ""
            );

            // Should be in order 1, 4
            expect(result.messages.map((m) => m.id)).toEqual(["1", "4"]);
        });
    });

    describe("OR logic for multiple types (AC #2)", () => {
        it("should use OR logic for multiple selected types", () => {
            const messages = [
                createMessage("1", "user", [textBlock("hello")]),
                createMessage("2", "assistant", [thinkingBlock]),
                createMessage("3", "assistant", [toolUseBlock("run_command", "t1")]),
            ];

            // Select both conversation and thinking types
            const result = filterWithPairedResults(
                messages,
                new Set(["conversation", "thinking"]),
                ""
            );

            // Should include messages 1 and 2
            expect(result.messages).toHaveLength(2);
            expect(result.messageIds.has("1")).toBe(true);
            expect(result.messageIds.has("2")).toBe(true);
        });
    });
});

describe("isMessageVisible", () => {
    it("should return true when no filter is active (empty set)", () => {
        expect(isMessageVisible("any-id", new Set())).toBe(true);
    });

    it("should return true when message is in filtered set", () => {
        expect(isMessageVisible("msg-1", new Set(["msg-1", "msg-2"]))).toBe(true);
    });

    it("should return false when message is not in filtered set", () => {
        expect(isMessageVisible("msg-3", new Set(["msg-1", "msg-2"]))).toBe(false);
    });
});
