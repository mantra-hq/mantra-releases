/**
 * useNavigationGuard - 导航拦截 Hook
 * Story 10.9: Task 5
 *
 * 使用 React Router 的 useBlocker 实现导航拦截
 * 当有未保存更改时阻止导航并显示确认对话框
 */

import * as React from "react";
import { useBlocker, type BlockerFunction } from "react-router-dom";

export interface UseNavigationGuardOptions {
  /** 是否应该阻止导航 */
  shouldBlock: boolean;
  /** 导航被阻止时的回调 */
  onBlock?: () => void;
}

export interface UseNavigationGuardResult {
  /** 是否正在阻止导航 */
  isBlocked: boolean;
  /** 继续导航（放弃阻止） */
  proceed: () => void;
  /** 取消导航（保持当前页面） */
  reset: () => void;
}

/**
 * 导航拦截 Hook
 *
 * @param options 配置选项
 * @returns 导航拦截控制对象
 *
 * @example
 * ```tsx
 * const { isBlocked, proceed, reset } = useNavigationGuard({
 *   shouldBlock: hasUnsavedChanges,
 *   onBlock: () => setShowDialog(true),
 * });
 * ```
 */
export function useNavigationGuard({
  shouldBlock,
  onBlock,
}: UseNavigationGuardOptions): UseNavigationGuardResult {
  // 使用 useBlocker 拦截导航
  const blockerFn: BlockerFunction = React.useCallback(
    ({ currentLocation, nextLocation }) => {
      // 只有在 shouldBlock 为 true 且路径发生变化时才阻止
      return shouldBlock && currentLocation.pathname !== nextLocation.pathname;
    },
    [shouldBlock]
  );

  const blocker = useBlocker(blockerFn);

  // 当导航被阻止时调用 onBlock 回调
  React.useEffect(() => {
    if (blocker.state === "blocked" && onBlock) {
      onBlock();
    }
  }, [blocker.state, onBlock]);

  // 继续导航
  const proceed = React.useCallback(() => {
    if (blocker.state === "blocked" && blocker.proceed) {
      blocker.proceed();
    }
  }, [blocker]);

  // 取消导航
  const reset = React.useCallback(() => {
    if (blocker.state === "blocked" && blocker.reset) {
      blocker.reset();
    }
  }, [blocker]);

  return {
    isBlocked: blocker.state === "blocked",
    proceed,
    reset,
  };
}

export default useNavigationGuard;
