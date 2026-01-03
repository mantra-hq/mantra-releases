/**
 * useUndoableAction Hook - 可撤销操作
 * Story 2.19: Task 4
 *
 * 提供执行后可撤销的操作能力，支持自定义超时时间
 */

import { useState, useCallback, useRef, useEffect } from "react";

/**
 * 可撤销操作定义
 */
export interface UndoableAction<T> {
  /** 执行操作 */
  execute: () => Promise<T>;
  /** 撤销操作 */
  undo: () => Promise<void>;
  /** 撤销窗口时间（毫秒），默认 5000ms */
  timeoutMs?: number;
}

/**
 * useUndoableAction Hook 返回值
 */
export interface UseUndoableActionReturn<T> {
  /** 触发可撤销操作 */
  trigger: (action: UndoableAction<T>) => Promise<T | void>;
  /** 是否正在执行 */
  isPending: boolean;
  /** 是否可撤销 */
  canUndo: boolean;
  /** 撤销操作 */
  cancel: () => Promise<void>;
}

const DEFAULT_TIMEOUT = 5000;

/**
 * 可撤销操作 Hook
 *
 * @example
 * ```tsx
 * const { trigger, isPending, canUndo, cancel } = useUndoableAction();
 *
 * const handleRemove = () => {
 *   trigger({
 *     execute: async () => {
 *       await removeProject(projectId);
 *     },
 *     undo: async () => {
 *       await restoreProject(projectId);
 *     },
 *     timeoutMs: 5000,
 *   });
 * };
 * ```
 */
export function useUndoableAction<T = void>(): UseUndoableActionReturn<T> {
  const [isPending, setIsPending] = useState(false);
  const [canUndo, setCanUndo] = useState(false);

  const undoFnRef = useRef<(() => Promise<void>) | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  // 清理超时
  const clearUndoTimeout = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  // 组件卸载时清理
  useEffect(() => {
    return () => {
      clearUndoTimeout();
    };
  }, [clearUndoTimeout]);

  // 触发可撤销操作
  const trigger = useCallback(
    async (action: UndoableAction<T>): Promise<T | void> => {
      const { execute, undo, timeoutMs = DEFAULT_TIMEOUT } = action;

      // 清理之前的状态
      clearUndoTimeout();
      undoFnRef.current = null;
      setCanUndo(false);

      setIsPending(true);

      try {
        const result = await execute();

        // 存储撤销函数
        undoFnRef.current = undo;
        setCanUndo(true);

        // 设置超时
        timeoutRef.current = setTimeout(() => {
          undoFnRef.current = null;
          setCanUndo(false);
        }, timeoutMs);

        return result;
      } finally {
        setIsPending(false);
      }
    },
    [clearUndoTimeout]
  );

  // 撤销操作
  const cancel = useCallback(async (): Promise<void> => {
    const undoFn = undoFnRef.current;

    if (!undoFn) {
      return;
    }

    // 清理状态
    clearUndoTimeout();
    undoFnRef.current = null;
    setCanUndo(false);

    // 执行撤销
    await undoFn();
  }, [clearUndoTimeout]);

  return {
    trigger,
    isPending,
    canUndo,
    cancel,
  };
}
