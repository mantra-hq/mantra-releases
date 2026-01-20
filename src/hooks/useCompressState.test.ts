/**
 * useCompressState Hook Tests
 * Story 10.3: Task 7.1
 *
 * 测试压缩状态管理、操作应用、预览计算
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import * as React from "react";
import {
  useCompressState,
  CompressStateProvider,
  type CompressOperation,
} from "./useCompressState";
import type { NarrativeMessage } from "@/types/message";

// Mock token-counter
vi.mock("@/lib/token-counter", () => ({
  estimateTokenCount: vi.fn((text: string) => Math.ceil(text.length / 4)),
}));

// 创建测试消息
function createTestMessage(
  id: string,
  role: "user" | "assistant",
  content: string
): NarrativeMessage {
  return {
    id,
    role,
    timestamp: new Date().toISOString(),
    content: [{ type: "text", content }],
  };
}

// Wrapper 组件
const wrapper = ({ children }: { children: React.ReactNode }) =>
  React.createElement(CompressStateProvider, {}, children);

describe("useCompressState", () => {
  describe("Context 错误处理", () => {
    it("在 Provider 外部使用时应抛出错误", () => {
      // 捕获控制台错误以保持测试输出干净
      const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});

      expect(() => {
        renderHook(() => useCompressState());
      }).toThrow("useCompressState must be used within a CompressStateProvider");

      consoleError.mockRestore();
    });
  });

  describe("初始状态", () => {
    it("应初始化空的操作映射表", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      expect(result.current.operations.size).toBe(0);
      expect(result.current.insertions.size).toBe(0);
    });

    it("初始变更统计应为零", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      const stats = result.current.getChangeStats();
      expect(stats.deleted).toBe(0);
      expect(stats.modified).toBe(0);
      expect(stats.inserted).toBe(0);
    });
  });

  describe("setOperation", () => {
    it("应正确设置删除操作", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const message = createTestMessage("msg-1", "user", "Hello");

      act(() => {
        result.current.setOperation("msg-1", {
          type: "delete",
          originalMessage: message,
        });
      });

      expect(result.current.operations.has("msg-1")).toBe(true);
      expect(result.current.operations.get("msg-1")?.type).toBe("delete");
    });

    it("应正确设置修改操作", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      act(() => {
        result.current.setOperation("msg-2", {
          type: "modify",
          modifiedContent: "Updated content",
        });
      });

      expect(result.current.operations.get("msg-2")?.type).toBe("modify");
      expect(result.current.operations.get("msg-2")?.modifiedContent).toBe(
        "Updated content"
      );
    });

    it("应覆盖现有操作", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      act(() => {
        result.current.setOperation("msg-1", { type: "delete" });
      });

      act(() => {
        result.current.setOperation("msg-1", {
          type: "modify",
          modifiedContent: "New",
        });
      });

      expect(result.current.operations.get("msg-1")?.type).toBe("modify");
    });
  });

  describe("removeOperation", () => {
    it("应移除操作恢复保留状态", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      act(() => {
        result.current.setOperation("msg-1", { type: "delete" });
      });

      act(() => {
        result.current.removeOperation("msg-1");
      });

      expect(result.current.operations.has("msg-1")).toBe(false);
    });

    it("移除不存在的操作不应报错", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      expect(() => {
        act(() => {
          result.current.removeOperation("non-existent");
        });
      }).not.toThrow();
    });
  });

  describe("resetAll", () => {
    it("应清除所有操作和插入", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const message = createTestMessage("ins-1", "user", "Inserted");

      act(() => {
        result.current.setOperation("msg-1", { type: "delete" });
        result.current.setOperation("msg-2", { type: "modify", modifiedContent: "X" });
        result.current.addInsertion(0, message);
      });

      act(() => {
        result.current.resetAll();
      });

      expect(result.current.operations.size).toBe(0);
      expect(result.current.insertions.size).toBe(0);
    });
  });

  describe("getPreviewMessages (AC #1)", () => {
    it("无操作时应返回所有消息为 keep 状态", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const messages = [
        createTestMessage("msg-1", "user", "Hello"),
        createTestMessage("msg-2", "assistant", "Hi"),
      ];

      const preview = result.current.getPreviewMessages(messages);

      expect(preview).toHaveLength(2);
      expect(preview[0].operation).toBe("keep");
      expect(preview[1].operation).toBe("keep");
    });

    it("应正确处理删除操作并计算 token", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const messages = [createTestMessage("msg-1", "user", "Hello World Test")];

      act(() => {
        result.current.setOperation("msg-1", {
          type: "delete",
          originalMessage: messages[0],
        });
      });

      const preview = result.current.getPreviewMessages(messages);

      expect(preview).toHaveLength(1);
      expect(preview[0].operation).toBe("delete");
      expect(preview[0].originalTokens).toBeDefined();
      expect(preview[0].originalTokens).toBeGreaterThan(0);
    });

    it("应正确处理修改操作并计算 token 差", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const messages = [
        createTestMessage("msg-1", "user", "This is a long message content"),
      ];

      act(() => {
        result.current.setOperation("msg-1", {
          type: "modify",
          modifiedContent: "Short",
        });
      });

      const preview = result.current.getPreviewMessages(messages);

      expect(preview).toHaveLength(1);
      expect(preview[0].operation).toBe("modify");
      expect(preview[0].message.content[0].content).toBe("Short");
      expect(preview[0].tokenDelta).toBeDefined();
      expect(preview[0].tokenDelta).toBeLessThan(0); // 内容变短了
    });
  });

  describe("插入操作", () => {
    it("addInsertion 应添加插入操作", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const insertedMessage = createTestMessage("new-1", "user", "New message");

      act(() => {
        result.current.addInsertion(0, insertedMessage);
      });

      expect(result.current.insertions.size).toBe(1);
      expect(result.current.insertions.get(0)?.type).toBe("insert");
      expect(result.current.insertions.get(0)?.insertedMessage).toBe(insertedMessage);
    });

    it("removeInsertion 应移除插入操作", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const insertedMessage = createTestMessage("new-1", "user", "New message");

      act(() => {
        result.current.addInsertion(0, insertedMessage);
      });

      act(() => {
        result.current.removeInsertion(0);
      });

      expect(result.current.insertions.size).toBe(0);
    });

    it("getPreviewMessages 应在正确位置包含插入的消息", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const messages = [
        createTestMessage("msg-1", "user", "First"),
        createTestMessage("msg-2", "assistant", "Second"),
      ];
      const insertedMessage = createTestMessage("new-1", "user", "Inserted");

      act(() => {
        result.current.addInsertion(0, insertedMessage); // 在 msg-1 之后插入
      });

      const preview = result.current.getPreviewMessages(messages);

      expect(preview).toHaveLength(3);
      expect(preview[0].id).toBe("msg-1");
      expect(preview[1].id).toBe("insert-0");
      expect(preview[1].operation).toBe("insert");
      expect(preview[2].id).toBe("msg-2");
    });
  });

  describe("getChangeStats", () => {
    it("应正确统计各类操作数量", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const insertedMessage = createTestMessage("new-1", "user", "New");

      act(() => {
        result.current.setOperation("msg-1", { type: "delete" });
        result.current.setOperation("msg-2", { type: "delete" });
        result.current.setOperation("msg-3", { type: "modify", modifiedContent: "X" });
        result.current.addInsertion(0, insertedMessage);
      });

      const stats = result.current.getChangeStats();

      expect(stats.deleted).toBe(2);
      expect(stats.modified).toBe(1);
      expect(stats.inserted).toBe(1);
    });

    it("keep 操作不计入统计", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      act(() => {
        result.current.setOperation("msg-1", { type: "keep" });
      });

      const stats = result.current.getChangeStats();

      expect(stats.deleted).toBe(0);
      expect(stats.modified).toBe(0);
      expect(stats.inserted).toBe(0);
    });
  });

  describe("响应时间 (AC #1: < 100ms)", () => {
    it("getPreviewMessages 应在 100ms 内完成", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      // 创建 100+ 消息模拟大量数据
      const messages = Array.from({ length: 150 }, (_, i) =>
        createTestMessage(`msg-${i}`, i % 2 === 0 ? "user" : "assistant", `Message ${i}`)
      );

      // 添加一些操作
      act(() => {
        for (let i = 0; i < 50; i++) {
          if (i % 3 === 0) {
            result.current.setOperation(`msg-${i}`, { type: "delete" });
          } else if (i % 3 === 1) {
            result.current.setOperation(`msg-${i}`, {
              type: "modify",
              modifiedContent: "Modified",
            });
          }
        }
      });

      const start = performance.now();
      const preview = result.current.getPreviewMessages(messages);
      const duration = performance.now() - start;

      expect(preview.length).toBeGreaterThan(0);
      expect(duration).toBeLessThan(100); // 应在 100ms 内完成
    });
  });
});
