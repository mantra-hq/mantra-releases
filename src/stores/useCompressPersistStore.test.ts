/**
 * useCompressPersistStore 测试
 * Story 10.9: Task 9.1
 *
 * 测试压缩状态持久化 Store 的存储/加载/清除功能
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { act } from "@testing-library/react";
import { useCompressPersistStore } from "./useCompressPersistStore";
import type { StateSnapshot } from "@/hooks/useCompressState";

// Mock sessionStorage
const mockSessionStorage = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
    get length() {
      return Object.keys(store).length;
    },
    key: vi.fn((index: number) => Object.keys(store)[index] || null),
  };
})();

Object.defineProperty(window, "sessionStorage", {
  value: mockSessionStorage,
  writable: true,
});

describe("useCompressPersistStore", () => {
  beforeEach(() => {
    // 重置 store 状态
    act(() => {
      useCompressPersistStore.getState().clearState();
    });
    mockSessionStorage.clear();
    vi.clearAllMocks();
  });

  describe("saveState", () => {
    it("应该正确保存状态快照", () => {
      const snapshot: StateSnapshot = {
        operations: new Map([
          ["msg-1", { type: "delete", originalMessage: undefined }],
          ["msg-2", { type: "modify", modifiedContent: "modified content" }],
        ]),
        insertions: new Map([
          [0, { type: "insert", insertAfterIndex: 0 }],
        ]),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot);
      });

      const state = useCompressPersistStore.getState();
      expect(state.sessionId).toBe("session-123");
      expect(state.snapshot).not.toBeNull();
      expect(state.snapshot?.operations).toHaveLength(2);
      expect(state.snapshot?.insertions).toHaveLength(1);
    });

    it("应该将 Map 序列化为数组", () => {
      const snapshot: StateSnapshot = {
        operations: new Map([
          ["msg-1", { type: "delete" }],
        ]),
        insertions: new Map(),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot);
      });

      const state = useCompressPersistStore.getState();
      // 验证序列化后是数组格式
      expect(Array.isArray(state.snapshot?.operations)).toBe(true);
      expect(Array.isArray(state.snapshot?.insertions)).toBe(true);
    });
  });

  describe("loadState", () => {
    it("应该正确加载并反序列化状态", () => {
      const snapshot: StateSnapshot = {
        operations: new Map([
          ["msg-1", { type: "delete" }],
          ["msg-2", { type: "modify", modifiedContent: "new content" }],
        ]),
        insertions: new Map([
          [1, { type: "insert", insertAfterIndex: 1 }],
        ]),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot);
      });

      const loaded = useCompressPersistStore.getState().loadState("session-123");

      expect(loaded).not.toBeNull();
      expect(loaded?.operations instanceof Map).toBe(true);
      expect(loaded?.insertions instanceof Map).toBe(true);
      expect(loaded?.operations.size).toBe(2);
      expect(loaded?.insertions.size).toBe(1);
      expect(loaded?.operations.get("msg-1")?.type).toBe("delete");
      expect(loaded?.operations.get("msg-2")?.modifiedContent).toBe("new content");
    });

    it("对不存在的 sessionId 应该返回 null", () => {
      const snapshot: StateSnapshot = {
        operations: new Map([["msg-1", { type: "delete" }]]),
        insertions: new Map(),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot);
      });

      const loaded = useCompressPersistStore.getState().loadState("session-456");
      expect(loaded).toBeNull();
    });

    it("对空 store 应该返回 null", () => {
      const loaded = useCompressPersistStore.getState().loadState("session-123");
      expect(loaded).toBeNull();
    });
  });

  describe("clearState", () => {
    it("应该清除存储的状态", () => {
      const snapshot: StateSnapshot = {
        operations: new Map([["msg-1", { type: "delete" }]]),
        insertions: new Map(),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot);
      });

      // 验证已保存
      expect(useCompressPersistStore.getState().sessionId).toBe("session-123");

      act(() => {
        useCompressPersistStore.getState().clearState();
      });

      const state = useCompressPersistStore.getState();
      expect(state.sessionId).toBeNull();
      expect(state.snapshot).toBeNull();
    });
  });

  describe("hasState", () => {
    it("有保存状态时应该返回 true", () => {
      const snapshot: StateSnapshot = {
        operations: new Map([["msg-1", { type: "delete" }]]),
        insertions: new Map(),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot);
      });

      expect(useCompressPersistStore.getState().hasState("session-123")).toBe(true);
    });

    it("sessionId 不匹配时应该返回 false", () => {
      const snapshot: StateSnapshot = {
        operations: new Map([["msg-1", { type: "delete" }]]),
        insertions: new Map(),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot);
      });

      expect(useCompressPersistStore.getState().hasState("session-456")).toBe(false);
    });

    it("没有保存状态时应该返回 false", () => {
      expect(useCompressPersistStore.getState().hasState("session-123")).toBe(false);
    });
  });

  describe("状态覆盖", () => {
    it("保存新状态应该覆盖旧状态", () => {
      const snapshot1: StateSnapshot = {
        operations: new Map([["msg-1", { type: "delete" }]]),
        insertions: new Map(),
      };

      const snapshot2: StateSnapshot = {
        operations: new Map([["msg-2", { type: "modify", modifiedContent: "new" }]]),
        insertions: new Map([[0, { type: "insert", insertAfterIndex: 0 }]]),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot1);
      });

      act(() => {
        useCompressPersistStore.getState().saveState("session-456", snapshot2);
      });

      const state = useCompressPersistStore.getState();
      expect(state.sessionId).toBe("session-456");

      const loaded = useCompressPersistStore.getState().loadState("session-456");
      expect(loaded?.operations.size).toBe(1);
      expect(loaded?.operations.has("msg-2")).toBe(true);
      expect(loaded?.insertions.size).toBe(1);
    });
  });

  describe("空状态处理", () => {
    it("应该正确处理空的 operations 和 insertions", () => {
      const snapshot: StateSnapshot = {
        operations: new Map(),
        insertions: new Map(),
      };

      act(() => {
        useCompressPersistStore.getState().saveState("session-123", snapshot);
      });

      const loaded = useCompressPersistStore.getState().loadState("session-123");
      expect(loaded).not.toBeNull();
      expect(loaded?.operations.size).toBe(0);
      expect(loaded?.insertions.size).toBe(0);
    });
  });
});
