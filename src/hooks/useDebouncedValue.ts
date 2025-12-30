/**
 * useDebouncedValue Hook - 防抖值
 * Story 2.8: Task 6
 *
 * 延迟更新值，用于搜索输入等场景减少频繁触发
 */

import { useState, useEffect } from "react";

/**
 * useDebouncedValue Hook
 * @param value 需要防抖的值
 * @param delay 延迟时间 (毫秒)
 * @returns 防抖后的值
 */
export function useDebouncedValue<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    // 设置定时器
    const timer = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    // 清理函数 - 取消定时器
    return () => {
      clearTimeout(timer);
    };
  }, [value, delay]);

  return debouncedValue;
}

