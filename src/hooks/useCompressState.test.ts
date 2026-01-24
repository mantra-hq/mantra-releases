/**
 * useCompressState Hook Tests
 * Story 10.3: Task 7.1
 * Story 10.8: Task 8.1 - 添加 undo/redo 测试用例
 *
 * 测试压缩状态管理、操作应用、预览计算、撤销/重做
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

    // [Fix #1] 测试 index=-1 的插入 (列表开头)
    it("getPreviewMessages 应在列表开头包含 index=-1 的插入消息", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const messages = [
        createTestMessage("msg-1", "user", "First"),
        createTestMessage("msg-2", "assistant", "Second"),
      ];
      const insertedMessage = createTestMessage("new-1", "user", "Inserted at start");

      act(() => {
        result.current.addInsertion(-1, insertedMessage); // 在第一条消息之前插入
      });

      const preview = result.current.getPreviewMessages(messages);

      expect(preview).toHaveLength(3);
      expect(preview[0].id).toBe("insert--1"); // 插入的消息应该在最前面
      expect(preview[0].operation).toBe("insert");
      expect(preview[0].message.content[0].content).toBe("Inserted at start");
      expect(preview[1].id).toBe("msg-1");
      expect(preview[2].id).toBe("msg-2");
    });

    it("getPreviewMessages 应同时处理 index=-1 和其他位置的插入", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const messages = [
        createTestMessage("msg-1", "user", "First"),
        createTestMessage("msg-2", "assistant", "Second"),
      ];
      const insertedAtStart = createTestMessage("new-start", "user", "At start");
      const insertedAfterFirst = createTestMessage("new-middle", "assistant", "After first");

      act(() => {
        result.current.addInsertion(-1, insertedAtStart); // 在列表开头
        result.current.addInsertion(0, insertedAfterFirst); // 在 msg-1 之后
      });

      const preview = result.current.getPreviewMessages(messages);

      expect(preview).toHaveLength(4);
      expect(preview[0].id).toBe("insert--1");
      expect(preview[1].id).toBe("msg-1");
      expect(preview[2].id).toBe("insert-0");
      expect(preview[3].id).toBe("msg-2");
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

  // Story 10.4: Task 4 新增方法测试
  describe("getOperationForMessage (Story 10.4)", () => {
    it("应返回指定消息的操作", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });
      const message = createTestMessage("msg-1", "user", "Hello");

      act(() => {
        result.current.setOperation("msg-1", {
          type: "delete",
          originalMessage: message,
        });
      });

      const operation = result.current.getOperationForMessage("msg-1");
      expect(operation).toBeDefined();
      expect(operation?.type).toBe("delete");
    });

    it("不存在的操作应返回 undefined", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      const operation = result.current.getOperationForMessage("non-existent");
      expect(operation).toBeUndefined();
    });
  });

  describe("getOperationType (Story 10.4)", () => {
    it("有操作时应返回操作类型", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      act(() => {
        result.current.setOperation("msg-1", { type: "delete" });
        result.current.setOperation("msg-2", { type: "modify", modifiedContent: "X" });
      });

      expect(result.current.getOperationType("msg-1")).toBe("delete");
      expect(result.current.getOperationType("msg-2")).toBe("modify");
    });

    it("无操作时应返回 keep", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      expect(result.current.getOperationType("msg-1")).toBe("keep");
    });

    it("移除操作后应返回 keep", () => {
      const { result } = renderHook(() => useCompressState(), { wrapper });

      act(() => {
        result.current.setOperation("msg-1", { type: "delete" });
      });

      act(() => {
        result.current.removeOperation("msg-1");
      });

      expect(result.current.getOperationType("msg-1")).toBe("keep");
    });
  });

  // Story 10.8: 撤销/重做功能测试
  describe("undo/redo (Story 10.8)", () => {
    describe("初始状态", () => {
      it("初始时 canUndo 应为 false", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });
        expect(result.current.canUndo).toBe(false);
      });

      it("初始时 canRedo 应为 false", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });
        expect(result.current.canRedo).toBe(false);
      });

      it("初始时 hasAnyChanges 应为 false", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });
        expect(result.current.hasAnyChanges).toBe(false);
      });
    });

    describe("undo 功能 (AC2)", () => {
      it("执行操作后 canUndo 应为 true", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        expect(result.current.canUndo).toBe(true);
      });

      it("undo 应恢复到上一个状态", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        expect(result.current.operations.has("msg-1")).toBe(true);

        act(() => {
          result.current.undo();
        });

        expect(result.current.operations.has("msg-1")).toBe(false);
      });

      it("undo 后 canRedo 应为 true", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        act(() => {
          result.current.undo();
        });

        expect(result.current.canRedo).toBe(true);
      });

      it("多次操作后可以多次 undo", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        act(() => {
          result.current.setOperation("msg-2", { type: "delete" });
        });

        act(() => {
          result.current.setOperation("msg-3", { type: "delete" });
        });

        expect(result.current.operations.size).toBe(3);

        act(() => {
          result.current.undo();
        });
        expect(result.current.operations.size).toBe(2);

        act(() => {
          result.current.undo();
        });
        expect(result.current.operations.size).toBe(1);

        act(() => {
          result.current.undo();
        });
        expect(result.current.operations.size).toBe(0);
      });

      it("空栈时 undo 不应报错", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        expect(() => {
          act(() => {
            result.current.undo();
          });
        }).not.toThrow();
      });
    });

    describe("redo 功能 (AC3)", () => {
      it("redo 应恢复被撤销的操作", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        act(() => {
          result.current.undo();
        });

        expect(result.current.operations.has("msg-1")).toBe(false);

        act(() => {
          result.current.redo();
        });

        expect(result.current.operations.has("msg-1")).toBe(true);
        expect(result.current.operations.get("msg-1")?.type).toBe("delete");
      });

      it("redo 后 canUndo 应为 true", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        act(() => {
          result.current.undo();
        });

        act(() => {
          result.current.redo();
        });

        expect(result.current.canUndo).toBe(true);
      });

      it("空栈时 redo 不应报错", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        expect(() => {
          act(() => {
            result.current.redo();
          });
        }).not.toThrow();
      });
    });

    describe("重做栈清空 (AC7)", () => {
      it("新操作后 redo 栈应被清空", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        act(() => {
          result.current.undo();
        });

        expect(result.current.canRedo).toBe(true);

        // 执行新操作
        act(() => {
          result.current.setOperation("msg-2", { type: "delete" });
        });

        expect(result.current.canRedo).toBe(false);
      });
    });

    describe("历史栈限制 (AC6)", () => {
      it("历史栈不应超过 50 条", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        // 执行 60 次操作
        for (let i = 0; i < 60; i++) {
          act(() => {
            result.current.setOperation(`msg-${i}`, { type: "delete" });
          });
        }

        // 撤销 50 次应该成功
        for (let i = 0; i < 50; i++) {
          act(() => {
            result.current.undo();
          });
        }

        // 第 51 次撤销后 canUndo 应为 false
        expect(result.current.canUndo).toBe(false);
      });
    });

    describe("resetAll 清空历史栈 (AC4)", () => {
      it("resetAll 应清空 undo 栈", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        expect(result.current.canUndo).toBe(true);

        act(() => {
          result.current.resetAll();
        });

        expect(result.current.canUndo).toBe(false);
      });

      it("resetAll 应清空 redo 栈", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        act(() => {
          result.current.undo();
        });

        expect(result.current.canRedo).toBe(true);

        act(() => {
          result.current.resetAll();
        });

        expect(result.current.canRedo).toBe(false);
      });
    });

    describe("hasAnyChanges 状态 (AC5)", () => {
      it("有操作时 hasAnyChanges 应为 true", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        expect(result.current.hasAnyChanges).toBe(true);
      });

      it("有插入时 hasAnyChanges 应为 true", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });
        const insertedMessage = createTestMessage("new-1", "user", "New");

        act(() => {
          result.current.addInsertion(0, insertedMessage);
        });

        expect(result.current.hasAnyChanges).toBe(true);
      });

      it("撤销所有操作后 hasAnyChanges 应为 false", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });

        act(() => {
          result.current.setOperation("msg-1", { type: "delete" });
        });

        act(() => {
          result.current.undo();
        });

        expect(result.current.hasAnyChanges).toBe(false);
      });
    });

    describe("插入操作的 undo/redo", () => {
      it("addInsertion 应可以被撤销", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });
        const insertedMessage = createTestMessage("new-1", "user", "New");

        act(() => {
          result.current.addInsertion(0, insertedMessage);
        });

        expect(result.current.insertions.size).toBe(1);

        act(() => {
          result.current.undo();
        });

        expect(result.current.insertions.size).toBe(0);
      });

      it("removeInsertion 应可以被撤销", () => {
        const { result } = renderHook(() => useCompressState(), { wrapper });
        const insertedMessage = createTestMessage("new-1", "user", "New");

        act(() => {
          result.current.addInsertion(0, insertedMessage);
        });

        act(() => {
          result.current.removeInsertion(0);
        });

        expect(result.current.insertions.size).toBe(0);

        act(() => {
          result.current.undo();
        });

        expect(result.current.insertions.size).toBe(1);
      });
    });
  });
});
