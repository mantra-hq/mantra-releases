/**
 * Compress Exporter Tests
 * Story 10.7: Task 7.1
 *
 * 测试压缩会话导出工具的功能
 * - JSONL 格式化
 * - Markdown 格式化
 * - 文件名生成
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  exportToJsonl,
  exportToMarkdown,
  getExportContent,
  formatExportFilename,
} from "./compress-exporter";
import type { PreviewMessage } from "@/hooks/useCompressState";
import type { TokenStats } from "@/components/compress/TokenStatistics";
import type { NarrativeMessage } from "@/types/message";

// 创建测试用的消息数据
function createTestMessage(
  id: string,
  role: "user" | "assistant",
  content: string
): NarrativeMessage {
  return {
    id,
    role,
    content: [{ type: "text", content }],
    timestamp: new Date().toISOString(),
  };
}

function createPreviewMessage(
  id: string,
  role: "user" | "assistant",
  content: string,
  operation: "keep" | "delete" | "modify" | "insert" = "keep"
): PreviewMessage {
  return {
    id,
    operation,
    message: createTestMessage(id, role, content),
  };
}

// 创建测试用的 TokenStats
function createTokenStats(
  originalTotal: number,
  compressedTotal: number
): TokenStats {
  const savedTokens = originalTotal - compressedTotal;
  const savedPercentage = originalTotal > 0 ? (savedTokens / originalTotal) * 100 : 0;
  return {
    originalTotal,
    compressedTotal,
    savedTokens,
    savedPercentage,
    changeStats: {
      deleted: 1,
      modified: 1,
      inserted: 1,
    },
  };
}

describe("exportToJsonl", () => {
  describe("基本格式", () => {
    it("应生成每行一个 JSON 对象的格式", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Hello"),
        createPreviewMessage("2", "assistant", "Hi there"),
      ];

      const result = exportToJsonl(messages);
      const lines = result.split("\n");

      expect(lines).toHaveLength(2);
      expect(JSON.parse(lines[0])).toEqual({ role: "user", content: "Hello" });
      expect(JSON.parse(lines[1])).toEqual({ role: "assistant", content: "Hi there" });
    });

    it("应保留 role 和 content 字段", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Test message"),
      ];

      const result = exportToJsonl(messages);
      const parsed = JSON.parse(result);

      expect(parsed).toHaveProperty("role", "user");
      expect(parsed).toHaveProperty("content", "Test message");
      expect(Object.keys(parsed)).toHaveLength(2);
    });
  });

  describe("消息过滤", () => {
    it("应过滤掉删除的消息", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Keep this"),
        createPreviewMessage("2", "assistant", "Delete this", "delete"),
        createPreviewMessage("3", "user", "Keep this too"),
      ];

      const result = exportToJsonl(messages);
      const lines = result.split("\n");

      expect(lines).toHaveLength(2);
      expect(JSON.parse(lines[0]).content).toBe("Keep this");
      expect(JSON.parse(lines[1]).content).toBe("Keep this too");
    });

    it("应包含修改后的消息", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Modified content", "modify"),
      ];

      const result = exportToJsonl(messages);
      const parsed = JSON.parse(result);

      expect(parsed.content).toBe("Modified content");
    });

    it("应包含插入的消息", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("insert-0", "user", "Inserted message", "insert"),
      ];

      const result = exportToJsonl(messages);
      const parsed = JSON.parse(result);

      expect(parsed.content).toBe("Inserted message");
    });
  });

  describe("边界情况", () => {
    it("空消息列表应返回空字符串", () => {
      const result = exportToJsonl([]);
      expect(result).toBe("");
    });

    it("所有消息都被删除时应返回空字符串", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Deleted", "delete"),
        createPreviewMessage("2", "assistant", "Also deleted", "delete"),
      ];

      const result = exportToJsonl(messages);
      expect(result).toBe("");
    });

    it("应正确处理包含特殊字符的内容", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", 'Hello "world"\nNew line'),
      ];

      const result = exportToJsonl(messages);
      const parsed = JSON.parse(result);

      expect(parsed.content).toBe('Hello "world"\nNew line');
    });
  });
});

describe("exportToMarkdown", () => {
  describe("标题和元信息", () => {
    it("应包含会话名称作为标题", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Hello"),
      ];
      const stats = createTokenStats(100, 80);

      const result = exportToMarkdown(messages, stats, "My Session");

      expect(result).toContain("# My Session");
    });

    it("无会话名称时应使用默认标题", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Hello"),
      ];
      const stats = createTokenStats(100, 80);

      const result = exportToMarkdown(messages, stats);

      expect(result).toContain("# Compressed Session");
    });

    it("应包含 Token 统计元信息", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Hello"),
      ];
      const stats = createTokenStats(12500, 9400);

      const result = exportToMarkdown(messages, stats);

      // formatTokenCount 对于 10000+ 返回整数 k，对于 1000-9999 返回一位小数 k
      expect(result).toContain("Original:");
      expect(result).toContain("tokens");
      expect(result).toContain("Compressed:");
      expect(result).toContain("saved 25%");
    });

    it("应包含导出时间戳", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Hello"),
      ];
      const stats = createTokenStats(100, 80);

      const result = exportToMarkdown(messages, stats);

      expect(result).toMatch(/Exported at: \d{4}-\d{2}-\d{2}T/);
    });
  });

  describe("消息格式", () => {
    it("应使用 ## User / ## Assistant 作为角色标题", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "User message"),
        createPreviewMessage("2", "assistant", "Assistant message"),
      ];
      const stats = createTokenStats(100, 100);

      const result = exportToMarkdown(messages, stats);

      expect(result).toContain("## User");
      expect(result).toContain("## Assistant");
    });

    it("应使用 --- 分隔消息", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "First"),
        createPreviewMessage("2", "assistant", "Second"),
      ];
      const stats = createTokenStats(100, 100);

      const result = exportToMarkdown(messages, stats);
      const separatorCount = (result.match(/^---$/gm) || []).length;

      // 标题后一个 + 每条消息后一个
      expect(separatorCount).toBeGreaterThanOrEqual(3);
    });

    it("应过滤掉删除的消息", () => {
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "user", "Keep"),
        createPreviewMessage("2", "assistant", "Delete", "delete"),
      ];
      const stats = createTokenStats(100, 50);

      const result = exportToMarkdown(messages, stats);

      expect(result).toContain("Keep");
      expect(result).not.toContain("Delete");
    });
  });

  describe("代码块处理", () => {
    it("应保持代码块格式", () => {
      const codeContent = "```typescript\nconst x = 1;\n```";
      const messages: PreviewMessage[] = [
        createPreviewMessage("1", "assistant", codeContent),
      ];
      const stats = createTokenStats(100, 100);

      const result = exportToMarkdown(messages, stats);

      expect(result).toContain("```typescript");
      expect(result).toContain("const x = 1;");
    });
  });
});

describe("getExportContent", () => {
  it("应返回 Markdown 格式的纯文本", () => {
    const messages: PreviewMessage[] = [
      createPreviewMessage("1", "user", "Hello"),
      createPreviewMessage("2", "assistant", "Hi"),
    ];

    const result = getExportContent(messages);

    expect(result).toContain("## User");
    expect(result).toContain("Hello");
    expect(result).toContain("## Assistant");
    expect(result).toContain("Hi");
  });

  it("应过滤掉删除的消息", () => {
    const messages: PreviewMessage[] = [
      createPreviewMessage("1", "user", "Keep"),
      createPreviewMessage("2", "assistant", "Delete", "delete"),
    ];

    const result = getExportContent(messages);

    expect(result).toContain("Keep");
    expect(result).not.toContain("Delete");
  });

  it("空消息列表应返回空字符串", () => {
    const result = getExportContent([]);
    expect(result).toBe("");
  });

  it("应去除首尾空白", () => {
    const messages: PreviewMessage[] = [
      createPreviewMessage("1", "user", "Hello"),
    ];

    const result = getExportContent(messages);

    expect(result).not.toMatch(/^\s/);
    expect(result).not.toMatch(/\s$/);
  });
});

describe("formatExportFilename", () => {
  // Mock Date for consistent timestamps
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-01-21T10:30:45.123Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("文件名格式", () => {
    it("应生成正确的 JSONL 文件名格式", () => {
      const result = formatExportFilename("my-session", "jsonl");

      expect(result).toBe("my-session-compressed-2026-01-21T10-30-45.jsonl");
    });

    it("应生成正确的 Markdown 文件名格式", () => {
      const result = formatExportFilename("my-session", "md");

      expect(result).toBe("my-session-compressed-2026-01-21T10-30-45.md");
    });
  });

  describe("会话名称处理", () => {
    it("无会话名称时应使用默认名称", () => {
      const result = formatExportFilename(undefined, "jsonl");

      expect(result).toContain("session-compressed");
    });

    it("应移除不安全字符", () => {
      const result = formatExportFilename('test<>:"/\\|?*name', "jsonl");

      expect(result).not.toContain("<");
      expect(result).not.toContain(">");
      expect(result).not.toContain(":");
      expect(result).not.toContain('"');
      expect(result).not.toContain("/");
      expect(result).not.toContain("\\");
      expect(result).not.toContain("|");
      expect(result).not.toContain("?");
      expect(result).not.toContain("*");
    });

    it("应将空格替换为连字符", () => {
      const result = formatExportFilename("my test session", "jsonl");

      expect(result).toContain("my-test-session");
    });

    it("应截断过长的会话名称", () => {
      const longName = "a".repeat(100);
      const result = formatExportFilename(longName, "jsonl");

      // 名称部分不超过 50 字符
      const namePart = result.split("-compressed")[0];
      expect(namePart.length).toBeLessThanOrEqual(50);
    });
  });

  describe("时间戳格式", () => {
    it("应包含 ISO 格式的时间戳", () => {
      const result = formatExportFilename("test", "jsonl");

      expect(result).toMatch(/\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}/);
    });

    it("时间戳中的冒号和点应被替换为连字符", () => {
      const result = formatExportFilename("test", "jsonl");

      // 时间戳部分不应包含冒号或点
      const timestampPart = result.match(/\d{4}-\d{2}-\d{2}T[\d-]+/)?.[0] || "";
      expect(timestampPart).not.toContain(":");
      expect(timestampPart).not.toContain(".");
    });
  });
});
