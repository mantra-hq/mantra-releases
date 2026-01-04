/**
 * Copy Utils 单元测试
 * Story 2.22: Task 2.8
 */

import { describe, it, expect } from "vitest";
import { getMessageCopyContent, hasCopiableContent } from "./copy-utils";
import type { NarrativeMessage } from "@/types/message";

describe("getMessageCopyContent", () => {
  describe("text 类型", () => {
    it("应该直接返回文本内容", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [{ type: "text", content: "Hello World" }],
      };

      expect(getMessageCopyContent(message)).toBe("Hello World");
    });

    it("应该跳过空文本块", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          { type: "text", content: "" },
          { type: "text", content: "   " },
          { type: "text", content: "Valid" },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("Valid");
    });

    it("应该用双换行连接多个文本块", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          { type: "text", content: "First" },
          { type: "text", content: "Second" },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("First\n\nSecond");
    });
  });

  describe("thinking 类型", () => {
    it("应该复制思考内容", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [{ type: "thinking", content: "Let me think..." }],
      };

      expect(getMessageCopyContent(message)).toBe("Let me think...");
    });
  });

  describe("tool_use 类型 - 智能提取主体内容", () => {
    it("Bash 工具应该复制命令", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "Bash",
            toolInput: { command: "npm install", description: "Install deps" },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("npm install");
    });

    it("Read 工具应该复制文件路径", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "Read",
            toolInput: { file_path: "/src/app.ts" },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("/src/app.ts");
    });

    it("Write 工具应该复制文件路径", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "Write",
            toolInput: { file_path: "/src/new-file.ts", content: "..." },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("/src/new-file.ts");
    });

    it("Edit 工具应该复制文件路径", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "Edit",
            toolInput: { file_path: "/src/edit.ts", old_string: "a", new_string: "b" },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("/src/edit.ts");
    });

    it("Grep 工具应该复制搜索模式", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "Grep",
            toolInput: { pattern: "function\\s+\\w+" },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("function\\s+\\w+");
    });

    it("Glob 工具应该复制 glob 模式", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "Glob",
            toolInput: { pattern: "**/*.tsx" },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("**/*.tsx");
    });

    it("WebFetch 工具应该复制 URL", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "WebFetch",
            toolInput: { url: "https://example.com" },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("https://example.com");
    });

    it("WebSearch 工具应该复制查询", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "WebSearch",
            toolInput: { query: "React hooks tutorial" },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("React hooks tutorial");
    });

    it("TodoWrite 工具不应该复制内容", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "TodoWrite",
            toolInput: { todos: [] },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("");
    });

    it("未知工具应该尝试提取 description 字段", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          {
            type: "tool_use",
            content: "",
            toolName: "CustomTool",
            toolInput: { description: "Do something useful" },
          },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("Do something useful");
    });
  });

  describe("tool_result 类型", () => {
    it("应该复制输出内容", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          { type: "tool_result", content: "Command executed successfully" },
        ],
      };

      expect(getMessageCopyContent(message)).toBe("Command executed successfully");
    });

    it("空内容应该返回空字符串", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [{ type: "tool_result", content: "" }],
      };

      expect(getMessageCopyContent(message)).toBe("");
    });
  });

  describe("混合内容", () => {
    it("应该正确组合多种类型的主体内容", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: "2024-01-01T00:00:00Z",
        content: [
          { type: "text", content: "Let me read the file." },
          {
            type: "tool_use",
            content: "",
            toolName: "Read",
            toolInput: { file_path: "/app.ts" },
          },
          { type: "tool_result", content: "const app = 1;", isError: false },
        ],
      };

      const result = getMessageCopyContent(message);
      expect(result).toBe("Let me read the file.\n\n/app.ts\n\nconst app = 1;");
    });
  });

  describe("用户消息", () => {
    it("用户消息应该复制文本内容", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "user",
        timestamp: "2024-01-01T00:00:00Z",
        content: [{ type: "text", content: "Please help me with this code" }],
      };

      expect(getMessageCopyContent(message)).toBe("Please help me with this code");
    });
  });
});

describe("hasCopiableContent", () => {
  it("text 类型应该返回 true", () => {
    const message: NarrativeMessage = {
      id: "1",
      role: "assistant",
      timestamp: "2024-01-01T00:00:00Z",
      content: [{ type: "text", content: "Hello" }],
    };

    expect(hasCopiableContent(message)).toBe(true);
  });

  it("image 类型应该返回 false", () => {
    const message: NarrativeMessage = {
      id: "1",
      role: "assistant",
      timestamp: "2024-01-01T00:00:00Z",
      content: [{ type: "image", content: "", source: "data:image/png" }],
    };

    expect(hasCopiableContent(message)).toBe(false);
  });

  it("空消息应该返回 false", () => {
    const message: NarrativeMessage = {
      id: "1",
      role: "assistant",
      timestamp: "2024-01-01T00:00:00Z",
      content: [],
    };

    expect(hasCopiableContent(message)).toBe(false);
  });
});
