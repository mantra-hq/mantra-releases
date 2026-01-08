/**
 * useMessageFilterStore Tests - 消息过滤状态管理测试
 * Story 2.16: Task 1.6
 */

import { describe, it, expect, beforeEach } from "vitest";
import { act } from "@testing-library/react";
import {
    useMessageFilterStore,
    MESSAGE_TYPES,
    messageMatchesType,
    messageMatchesSearch,
    getToolUseIds,
    hasToolBlocks,
} from "./useMessageFilterStore";
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
const textBlock: ContentBlock = {
    type: "text",
    content: "Hello, this is a test message",
};

const thinkingBlock: ContentBlock = {
    type: "thinking",
    content: "Let me think about this...",
};

const toolUseBlock: ContentBlock = {
    type: "tool_use",
    content: "",
    toolName: "view_file",
    toolInput: { path: "/test/file.ts" },
    toolUseId: "tool-123",
};

const toolResultBlock: ContentBlock = {
    type: "tool_result",
    content: "file contents here",
    toolUseId: "tool-123",
};

const runCommandBlock: ContentBlock = {
    type: "tool_use",
    content: "",
    toolName: "run_command",
    toolInput: { command: "npm test" },
    toolUseId: "tool-456",
};

const shellBlock: ContentBlock = {
    type: "tool_use",
    content: "",
    toolName: "shell",  // Codex CLI
    toolInput: { command: ["ls", "-la"] },
    toolUseId: "tool-457",
};

const searchBlock: ContentBlock = {
    type: "tool_use",
    content: "",
    toolName: "grep_search",
    toolInput: { pattern: "test" },
    toolUseId: "tool-789",
};

describe("useMessageFilterStore", () => {
    beforeEach(() => {
        // Reset store before each test
        act(() => {
            useMessageFilterStore.setState({
                selectedTypes: new Set<string>(),
                searchQuery: "",
                isSearchFocused: false,
            });
        });
    });

    describe("toggleType", () => {
        it("should add type when not selected", () => {
            act(() => {
                useMessageFilterStore.getState().toggleType("tool");
            });

            const state = useMessageFilterStore.getState();
            expect(state.selectedTypes.has("tool")).toBe(true);
            expect(state.selectedTypes.size).toBe(1);
        });

        it("should remove type when already selected", () => {
            act(() => {
                useMessageFilterStore.getState().toggleType("tool");
                useMessageFilterStore.getState().toggleType("tool");
            });

            const state = useMessageFilterStore.getState();
            expect(state.selectedTypes.has("tool")).toBe(false);
            expect(state.selectedTypes.size).toBe(0);
        });

        it("should support multiple selected types", () => {
            act(() => {
                useMessageFilterStore.getState().toggleType("tool");
                useMessageFilterStore.getState().toggleType("file");
                useMessageFilterStore.getState().toggleType("terminal");
            });

            const state = useMessageFilterStore.getState();
            expect(state.selectedTypes.size).toBe(3);
            expect(state.selectedTypes.has("tool")).toBe(true);
            expect(state.selectedTypes.has("file")).toBe(true);
            expect(state.selectedTypes.has("terminal")).toBe(true);
        });
    });

    describe("setSearchQuery", () => {
        it("should update search query", () => {
            act(() => {
                useMessageFilterStore.getState().setSearchQuery("test query");
            });

            expect(useMessageFilterStore.getState().searchQuery).toBe("test query");
        });

        it("should handle empty query", () => {
            act(() => {
                useMessageFilterStore.getState().setSearchQuery("test");
                useMessageFilterStore.getState().setSearchQuery("");
            });

            expect(useMessageFilterStore.getState().searchQuery).toBe("");
        });
    });

    describe("clearFilters", () => {
        it("should reset all filters", () => {
            act(() => {
                useMessageFilterStore.getState().toggleType("tool");
                useMessageFilterStore.getState().toggleType("file");
                useMessageFilterStore.getState().setSearchQuery("test");
                useMessageFilterStore.getState().clearFilters();
            });

            const state = useMessageFilterStore.getState();
            expect(state.selectedTypes.size).toBe(0);
            expect(state.searchQuery).toBe("");
        });
    });

    describe("setSearchFocused", () => {
        it("should update focus state", () => {
            act(() => {
                useMessageFilterStore.getState().setSearchFocused(true);
            });
            expect(useMessageFilterStore.getState().isSearchFocused).toBe(true);

            act(() => {
                useMessageFilterStore.getState().setSearchFocused(false);
            });
            expect(useMessageFilterStore.getState().isSearchFocused).toBe(false);
        });
    });
});

describe("MESSAGE_TYPES configuration", () => {
    it("should have correct number of types", () => {
        expect(MESSAGE_TYPES).toHaveLength(6);
    });

    it("should have unique ids", () => {
        const ids = MESSAGE_TYPES.map((t) => t.id);
        expect(new Set(ids).size).toBe(ids.length);
    });

    it("conversation type should match text blocks", () => {
        const config = MESSAGE_TYPES.find((t) => t.id === "conversation");
        expect(config?.match(textBlock)).toBe(true);
        expect(config?.match(toolUseBlock)).toBe(false);
    });

    it("tool type should match tool_use and tool_result blocks", () => {
        const config = MESSAGE_TYPES.find((t) => t.id === "tool");
        expect(config?.match(toolUseBlock)).toBe(true);
        expect(config?.match(toolResultBlock)).toBe(true);
        expect(config?.match(textBlock)).toBe(false);
    });

    it("file type should match file-related tool_use blocks", () => {
        const config = MESSAGE_TYPES.find((t) => t.id === "file");
        expect(config?.match(toolUseBlock)).toBe(true); // view_file
        expect(config?.match(runCommandBlock)).toBe(false);
    });

    it("terminal type should match command blocks", () => {
        const config = MESSAGE_TYPES.find((t) => t.id === "terminal");
        expect(config?.match(runCommandBlock)).toBe(true);
        expect(config?.match(shellBlock)).toBe(true);  // Codex CLI shell
        expect(config?.match(toolUseBlock)).toBe(false);
    });

    it("thinking type should match thinking blocks", () => {
        const config = MESSAGE_TYPES.find((t) => t.id === "thinking");
        expect(config?.match(thinkingBlock)).toBe(true);
        expect(config?.match(textBlock)).toBe(false);
    });

    it("search type should match search/grep blocks", () => {
        const config = MESSAGE_TYPES.find((t) => t.id === "search");
        expect(config?.match(searchBlock)).toBe(true);
        expect(config?.match(toolUseBlock)).toBe(false);
    });
});

describe("messageMatchesType", () => {
    it("should return true when message contains matching block", () => {
        const message = createMessage("1", "assistant", [textBlock, toolUseBlock]);
        expect(messageMatchesType(message, "conversation")).toBe(true);
        expect(messageMatchesType(message, "tool")).toBe(true);
    });

    it("should return false when message has no matching block", () => {
        const message = createMessage("1", "user", [textBlock]);
        expect(messageMatchesType(message, "tool")).toBe(false);
        expect(messageMatchesType(message, "thinking")).toBe(false);
    });

    it("should return false for unknown type", () => {
        const message = createMessage("1", "assistant", [textBlock]);
        expect(messageMatchesType(message, "unknown-type")).toBe(false);
    });
});

describe("messageMatchesSearch", () => {
    it("should return true for empty query", () => {
        const message = createMessage("1", "user", [textBlock]);
        expect(messageMatchesSearch(message, "")).toBe(true);
        expect(messageMatchesSearch(message, "   ")).toBe(true);
    });

    it("should match text content case-insensitively", () => {
        const message = createMessage("1", "user", [textBlock]);
        expect(messageMatchesSearch(message, "test")).toBe(true);
        expect(messageMatchesSearch(message, "TEST")).toBe(true);
        expect(messageMatchesSearch(message, "hello")).toBe(true);
    });

    it("should match tool names", () => {
        const message = createMessage("1", "assistant", [toolUseBlock]);
        expect(messageMatchesSearch(message, "view_file")).toBe(true);
        expect(messageMatchesSearch(message, "file")).toBe(true);
    });

    it("should return false when no match", () => {
        const message = createMessage("1", "user", [textBlock]);
        expect(messageMatchesSearch(message, "nonexistent")).toBe(false);
    });
});

describe("getToolUseIds", () => {
    it("should extract toolUseIds from message", () => {
        const message = createMessage("1", "assistant", [
            toolUseBlock,
            toolResultBlock,
        ]);
        const ids = getToolUseIds(message);
        expect(ids.size).toBe(1);
        expect(ids.has("tool-123")).toBe(true);
    });

    it("should return empty set when no tool blocks", () => {
        const message = createMessage("1", "user", [textBlock]);
        const ids = getToolUseIds(message);
        expect(ids.size).toBe(0);
    });

    it("should collect multiple unique ids", () => {
        const message = createMessage("1", "assistant", [
            toolUseBlock,
            runCommandBlock,
            searchBlock,
        ]);
        const ids = getToolUseIds(message);
        expect(ids.size).toBe(3);
        expect(ids.has("tool-123")).toBe(true);
        expect(ids.has("tool-456")).toBe(true);
        expect(ids.has("tool-789")).toBe(true);
    });
});

describe("hasToolBlocks", () => {
    it("should return true for messages with tool_use", () => {
        const message = createMessage("1", "assistant", [textBlock, toolUseBlock]);
        expect(hasToolBlocks(message)).toBe(true);
    });

    it("should return true for messages with tool_result", () => {
        const message = createMessage("1", "assistant", [toolResultBlock]);
        expect(hasToolBlocks(message)).toBe(true);
    });

    it("should return false for messages without tool blocks", () => {
        const message = createMessage("1", "user", [textBlock]);
        expect(hasToolBlocks(message)).toBe(false);
    });
});
