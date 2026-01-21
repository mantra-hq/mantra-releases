/**
 * useMessageFocus Hook 单元测试
 * Story 10.10: Task 9.1
 */

import { renderHook, act } from "@testing-library/react";
import { useMessageFocus } from "./useMessageFocus";

describe("useMessageFocus", () => {
  describe("初始状态", () => {
    it("初始焦点索引应为 -1", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      expect(result.current.focusedIndex).toBe(-1);
      expect(result.current.hasFocus).toBe(false);
    });
  });

  describe("focusNext", () => {
    it("首次调用应聚焦第一条消息 (索引 0)", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.focusNext();
      });

      expect(result.current.focusedIndex).toBe(0);
      expect(result.current.hasFocus).toBe(true);
    });

    it("应移动到下一条消息", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.setFocusedIndex(2);
      });

      act(() => {
        result.current.focusNext();
      });

      expect(result.current.focusedIndex).toBe(3);
    });

    it("在末尾时不应越界", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.setFocusedIndex(4); // 最后一条
      });

      act(() => {
        result.current.focusNext();
      });

      expect(result.current.focusedIndex).toBe(4); // 保持不变
    });

    it("消息数量为 0 时不应操作", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 0 })
      );

      act(() => {
        result.current.focusNext();
      });

      expect(result.current.focusedIndex).toBe(-1);
    });
  });

  describe("focusPrevious", () => {
    it("首次调用应聚焦最后一条消息", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.focusPrevious();
      });

      expect(result.current.focusedIndex).toBe(4); // 最后一条
    });

    it("应移动到上一条消息", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.setFocusedIndex(3);
      });

      act(() => {
        result.current.focusPrevious();
      });

      expect(result.current.focusedIndex).toBe(2);
    });

    it("在开头时不应越界", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.setFocusedIndex(0); // 第一条
      });

      act(() => {
        result.current.focusPrevious();
      });

      expect(result.current.focusedIndex).toBe(0); // 保持不变
    });
  });

  describe("setFocusedIndex", () => {
    it("应设置指定索引", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.setFocusedIndex(3);
      });

      expect(result.current.focusedIndex).toBe(3);
    });

    it("应限制索引在有效范围内", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.setFocusedIndex(10); // 超出范围
      });

      expect(result.current.focusedIndex).toBe(4); // 限制为最后一条
    });

    it("应允许设置 -1 清除焦点", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.setFocusedIndex(2);
      });

      act(() => {
        result.current.setFocusedIndex(-1);
      });

      expect(result.current.focusedIndex).toBe(-1);
      expect(result.current.hasFocus).toBe(false);
    });
  });

  describe("clearFocus", () => {
    it("应清除焦点", () => {
      const { result } = renderHook(() =>
        useMessageFocus({ messageCount: 5 })
      );

      act(() => {
        result.current.setFocusedIndex(2);
      });

      act(() => {
        result.current.clearFocus();
      });

      expect(result.current.focusedIndex).toBe(-1);
      expect(result.current.hasFocus).toBe(false);
    });
  });

  describe("消息数量变化", () => {
    it("焦点超出新范围时应调整", () => {
      const { result, rerender } = renderHook(
        ({ messageCount }) => useMessageFocus({ messageCount }),
        { initialProps: { messageCount: 10 } }
      );

      act(() => {
        result.current.setFocusedIndex(8);
      });

      // 减少消息数量
      rerender({ messageCount: 5 });

      expect(result.current.focusedIndex).toBe(4); // 调整为最后一条
    });

    it("消息清空时应清除焦点", () => {
      const { result, rerender } = renderHook(
        ({ messageCount }) => useMessageFocus({ messageCount }),
        { initialProps: { messageCount: 5 } }
      );

      act(() => {
        result.current.setFocusedIndex(2);
      });

      rerender({ messageCount: 0 });

      expect(result.current.focusedIndex).toBe(-1);
    });
  });
});
