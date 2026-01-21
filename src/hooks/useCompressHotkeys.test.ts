/**
 * useCompressHotkeys Hook 单元测试
 * Story 10.10: Task 9.2
 */

import { renderHook } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { useCompressHotkeys } from "./useCompressHotkeys";
import type { UseMessageFocusReturn } from "./useMessageFocus";
import type { NarrativeMessage } from "@/types/message";

// Mock usePlatform hook
vi.mock("./usePlatform", () => ({
  usePlatform: () => "windows", // 使用有效的 Platform 类型值
}));

// 创建测试消息
function createTestMessage(id: string): NarrativeMessage {
  return {
    id,
    role: "user",
    content: [{ type: "text", text: `Message ${id}` }],
    timestamp: Date.now(),
  };
}

// 创建模拟焦点管理
function createMockFocus(focusedIndex: number = -1): UseMessageFocusReturn {
  return {
    focusedIndex,
    setFocusedIndex: vi.fn(),
    focusNext: vi.fn(),
    focusPrevious: vi.fn(),
    clearFocus: vi.fn(),
    hasFocus: focusedIndex >= 0,
  };
}

describe("useCompressHotkeys", () => {
  let originalAddEventListener: typeof window.addEventListener;
  let originalRemoveEventListener: typeof window.removeEventListener;
  let keydownHandler: ((e: KeyboardEvent) => void) | null = null;

  beforeEach(() => {
    keydownHandler = null;
    originalAddEventListener = window.addEventListener;
    originalRemoveEventListener = window.removeEventListener;

    window.addEventListener = vi.fn((event, handler) => {
      if (event === "keydown") {
        keydownHandler = handler as (e: KeyboardEvent) => void;
      }
    });

    window.removeEventListener = vi.fn();
  });

  afterEach(() => {
    window.addEventListener = originalAddEventListener;
    window.removeEventListener = originalRemoveEventListener;
  });

  // 模拟按键事件
  function simulateKeyDown(key: string, options: Partial<KeyboardEvent> = {}) {
    if (!keydownHandler) return;

    const event = {
      key,
      preventDefault: vi.fn(),
      target: document.body,
      ctrlKey: false,
      metaKey: false,
      shiftKey: false,
      ...options,
    } as unknown as KeyboardEvent;

    keydownHandler(event);
    return event;
  }

  describe("初始化", () => {
    it("enabled=true 时应注册事件监听器", () => {
      const focus = createMockFocus();
      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages: [],
        })
      );

      expect(window.addEventListener).toHaveBeenCalledWith(
        "keydown",
        expect.any(Function)
      );
    });

    it("enabled=false 时不应注册事件监听器", () => {
      const focus = createMockFocus();
      renderHook(() =>
        useCompressHotkeys({
          enabled: false,
          focus,
          messages: [],
        })
      );

      expect(window.addEventListener).not.toHaveBeenCalled();
    });
  });

  describe("导航快捷键", () => {
    it("ArrowDown 应调用 focusNext", () => {
      const focus = createMockFocus();
      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages: [createTestMessage("1")],
        })
      );

      const event = simulateKeyDown("ArrowDown");

      expect(focus.focusNext).toHaveBeenCalled();
      expect(event?.preventDefault).toHaveBeenCalled();
    });

    it("ArrowUp 应调用 focusPrevious", () => {
      const focus = createMockFocus();
      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages: [createTestMessage("1")],
        })
      );

      const event = simulateKeyDown("ArrowUp");

      expect(focus.focusPrevious).toHaveBeenCalled();
      expect(event?.preventDefault).toHaveBeenCalled();
    });
  });

  describe("消息操作快捷键", () => {
    it("K 键应触发 onKeep (有焦点时)", () => {
      const focus = createMockFocus(0);
      const onKeep = vi.fn();
      const messages = [createTestMessage("msg-1")];

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages,
          onKeep,
        })
      );

      simulateKeyDown("k");

      expect(onKeep).toHaveBeenCalledWith("msg-1");
    });

    it("D 键应触发 onDelete (有焦点时)", () => {
      const focus = createMockFocus(0);
      const onDelete = vi.fn();
      const messages = [createTestMessage("msg-1")];

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages,
          onDelete,
        })
      );

      simulateKeyDown("d");

      expect(onDelete).toHaveBeenCalledWith("msg-1");
    });

    it("E 键应触发 onEdit (有焦点时)", () => {
      const focus = createMockFocus(0);
      const onEdit = vi.fn();
      const messages = [createTestMessage("msg-1")];

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages,
          onEdit,
        })
      );

      simulateKeyDown("e");

      expect(onEdit).toHaveBeenCalledWith("msg-1");
    });

    it("I 键应触发 onInsert (有焦点时)", () => {
      const focus = createMockFocus(2);
      const onInsert = vi.fn();
      const messages = [
        createTestMessage("1"),
        createTestMessage("2"),
        createTestMessage("3"),
      ];

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages,
          onInsert,
        })
      );

      simulateKeyDown("i");

      expect(onInsert).toHaveBeenCalledWith(2);
    });

    it("无焦点时不应触发消息操作", () => {
      const focus = createMockFocus(-1); // 无焦点
      const onKeep = vi.fn();
      const messages = [createTestMessage("msg-1")];

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages,
          onKeep,
        })
      );

      simulateKeyDown("k");

      expect(onKeep).not.toHaveBeenCalled();
    });
  });

  describe("全局快捷键", () => {
    it("Ctrl+S 应触发 onOpenExport", () => {
      const focus = createMockFocus();
      const onOpenExport = vi.fn();

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages: [],
          onOpenExport,
        })
      );

      const event = simulateKeyDown("s", { ctrlKey: true });

      expect(onOpenExport).toHaveBeenCalled();
      expect(event?.preventDefault).toHaveBeenCalled();
    });

    it("? 键应触发 onToggleHelp", () => {
      const focus = createMockFocus();
      const onToggleHelp = vi.fn();

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages: [],
          onToggleHelp,
        })
      );

      simulateKeyDown("?");

      expect(onToggleHelp).toHaveBeenCalled();
    });
  });

  describe("输入框排除", () => {
    it("在 input 元素中不应触发快捷键", () => {
      const focus = createMockFocus(0);
      const onKeep = vi.fn();
      const messages = [createTestMessage("msg-1")];

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages,
          onKeep,
        })
      );

      // 模拟在 input 中按键
      const input = document.createElement("input");
      simulateKeyDown("k", { target: input } as unknown as Partial<KeyboardEvent>);

      expect(onKeep).not.toHaveBeenCalled();
    });

    it("在 textarea 元素中不应触发快捷键", () => {
      const focus = createMockFocus(0);
      const onDelete = vi.fn();
      const messages = [createTestMessage("msg-1")];

      renderHook(() =>
        useCompressHotkeys({
          enabled: true,
          focus,
          messages,
          onDelete,
        })
      );

      const textarea = document.createElement("textarea");
      simulateKeyDown("d", { target: textarea } as unknown as Partial<KeyboardEvent>);

      expect(onDelete).not.toHaveBeenCalled();
    });
  });
});
