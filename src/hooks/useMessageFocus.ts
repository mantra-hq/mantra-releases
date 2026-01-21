/**
 * useMessageFocus - 消息焦点管理 Hook
 * Story 10.10: Task 1
 *
 * 管理压缩模式下消息列表的焦点状态
 * 支持键盘导航 (↑/↓) 和点击聚焦
 */

import * as React from "react";

/**
 * useMessageFocus 选项
 */
export interface UseMessageFocusOptions {
  /** 消息总数 */
  messageCount: number;
  /** 滚动容器 ref (用于 scrollIntoView) */
  containerRef?: React.RefObject<HTMLElement | null>;
}

/**
 * useMessageFocus 返回值
 */
export interface UseMessageFocusReturn {
  /** 当前焦点索引 (-1 表示无焦点) */
  focusedIndex: number;
  /** 设置焦点索引 */
  setFocusedIndex: (index: number) => void;
  /** 移动到下一条消息 */
  focusNext: () => void;
  /** 移动到上一条消息 */
  focusPrevious: () => void;
  /** 清除焦点 */
  clearFocus: () => void;
  /** 焦点是否激活 */
  hasFocus: boolean;
}

/**
 * 消息焦点管理 Hook
 *
 * @param options 选项
 * @returns 焦点状态和操作方法
 *
 * @example
 * ```tsx
 * const containerRef = useRef<HTMLDivElement>(null);
 * const { focusedIndex, focusNext, focusPrevious } = useMessageFocus({
 *   messageCount: messages.length,
 *   containerRef,
 * });
 * ```
 */
export function useMessageFocus(options: UseMessageFocusOptions): UseMessageFocusReturn {
  const { messageCount, containerRef } = options;

  // 焦点索引状态 (-1 表示无焦点)
  const [focusedIndex, setFocusedIndexState] = React.useState(-1);

  // 设置焦点索引 (带边界检查)
  const setFocusedIndex = React.useCallback(
    (index: number) => {
      if (messageCount === 0) {
        setFocusedIndexState(-1);
        return;
      }
      // 确保索引在有效范围内
      const clampedIndex = Math.max(-1, Math.min(index, messageCount - 1));
      setFocusedIndexState(clampedIndex);
    },
    [messageCount]
  );

  // 移动到下一条消息
  const focusNext = React.useCallback(() => {
    if (messageCount === 0) return;

    setFocusedIndexState((prev) => {
      // 首次聚焦第一条消息
      if (prev < 0) return 0;
      // 已在末尾，保持不变
      if (prev >= messageCount - 1) return prev;
      // 移动到下一条
      return prev + 1;
    });
  }, [messageCount]);

  // 移动到上一条消息
  const focusPrevious = React.useCallback(() => {
    if (messageCount === 0) return;

    setFocusedIndexState((prev) => {
      // 首次聚焦最后一条消息
      if (prev < 0) return messageCount - 1;
      // 已在开头，保持不变
      if (prev <= 0) return 0;
      // 移动到上一条
      return prev - 1;
    });
  }, [messageCount]);

  // 清除焦点
  const clearFocus = React.useCallback(() => {
    setFocusedIndexState(-1);
  }, []);

  // 焦点是否激活
  const hasFocus = focusedIndex >= 0;

  // 消息数量变化时调整焦点索引
  React.useEffect(() => {
    if (focusedIndex >= messageCount && messageCount > 0) {
      // 焦点超出范围，移动到最后一条
      setFocusedIndexState(messageCount - 1);
    } else if (messageCount === 0 && focusedIndex >= 0) {
      // 列表清空，清除焦点
      setFocusedIndexState(-1);
    }
  }, [messageCount, focusedIndex]);

  // 焦点变化时滚动到可见区域
  React.useEffect(() => {
    if (focusedIndex >= 0 && containerRef?.current) {
      const focusedElement = containerRef.current.querySelector(
        `[data-index="${focusedIndex}"]`
      );
      if (focusedElement) {
        focusedElement.scrollIntoView({
          behavior: "smooth",
          block: "nearest",
        });
      }
    }
  }, [focusedIndex, containerRef]);

  return {
    focusedIndex,
    setFocusedIndex,
    focusNext,
    focusPrevious,
    clearFocus,
    hasFocus,
  };
}

export default useMessageFocus;
