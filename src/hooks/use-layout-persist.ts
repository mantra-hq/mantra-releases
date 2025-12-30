/**
 * useLayoutPersist - 布局持久化 Hook
 * Story 2.2: AC #4, Task 2
 *
 * 功能:
 * - 保存布局比例到 localStorage
 * - 初始化时读取并应用保存的布局
 * - Debounce 保存 (300ms)
 * - 处理 localStorage 不可用的降级场景
 */

import * as React from "react";

export interface UseLayoutPersistOptions {
  /** localStorage 存储键名 */
  storageKey: string;
  /** 默认布局比例 */
  defaultLayout: [number, number];
  /** Debounce 延迟 (ms) */
  debounceMs?: number;
}

export interface UseLayoutPersistReturn {
  /** 当前布局 */
  layout: [number, number];
  /** 更新布局 (会自动 debounce 保存) */
  setLayout: (layout: [number, number]) => void;
  /** 重置为默认布局 */
  resetLayout: () => void;
}

/**
 * 安全读取 localStorage
 */
function safeGetItem(key: string): string | null {
  try {
    if (typeof window === "undefined") return null;
    return window.localStorage.getItem(key);
  } catch {
    // localStorage 可能在隐私模式下不可用
    return null;
  }
}

/**
 * 安全写入 localStorage
 */
function safeSetItem(key: string, value: string): boolean {
  try {
    if (typeof window === "undefined") return false;
    window.localStorage.setItem(key, value);
    return true;
  } catch {
    // localStorage 可能在隐私模式下不可用或已满
    return false;
  }
}

/**
 * 验证布局数据格式
 */
function isValidLayout(data: unknown): data is [number, number] {
  if (!Array.isArray(data) || data.length !== 2) return false;
  const [left, right] = data;
  return (
    typeof left === "number" &&
    typeof right === "number" &&
    left >= 0 &&
    left <= 100 &&
    right >= 0 &&
    right <= 100
  );
}

/**
 * 从 localStorage 读取布局
 */
function loadLayout(
  storageKey: string,
  defaultLayout: [number, number]
): [number, number] {
  const stored = safeGetItem(storageKey);
  if (!stored) return defaultLayout;

  try {
    const parsed = JSON.parse(stored);
    if (isValidLayout(parsed)) {
      return parsed;
    }
  } catch {
    // JSON 解析失败，使用默认值
  }

  return defaultLayout;
}

export function useLayoutPersist({
  storageKey,
  defaultLayout,
  debounceMs = 300,
}: UseLayoutPersistOptions): UseLayoutPersistReturn {
  // 初始化时从 localStorage 读取
  const [layout, setLayoutState] = React.useState<[number, number]>(() =>
    loadLayout(storageKey, defaultLayout)
  );

  // 保存 timeout ref
  const saveTimeoutRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);

  // Debounced 保存到 localStorage
  const saveToStorage = React.useCallback(
    (newLayout: [number, number]) => {
      // 清除之前的 timeout
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }

      // 设置新的 debounced 保存
      saveTimeoutRef.current = setTimeout(() => {
        safeSetItem(storageKey, JSON.stringify(newLayout));
      }, debounceMs);
    },
    [storageKey, debounceMs]
  );

  // 设置布局并触发保存
  const setLayout = React.useCallback(
    (newLayout: [number, number]) => {
      setLayoutState(newLayout);
      saveToStorage(newLayout);
    },
    [saveToStorage]
  );

  // 重置为默认布局
  const resetLayout = React.useCallback(() => {
    setLayoutState(defaultLayout);
    safeSetItem(storageKey, JSON.stringify(defaultLayout));
  }, [defaultLayout, storageKey]);

  // 清理 timeout
  React.useEffect(() => {
    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, []);

  return {
    layout,
    setLayout,
    resetLayout,
  };
}

export default useLayoutPersist;

