/**
 * useNavigationGuard - 导航拦截 Hook
 * Story 10.9: Task 5
 *
 * 注意: React Router 的 useBlocker 需要 data router (createBrowserRouter)
 * 当前应用使用 BrowserRouter，因此使用 beforeunload 事件作为后备方案
 *
 * 未来迁移到 data router 后可以启用完整的导航拦截功能
 */

import * as React from "react";

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
 * 当前实现: 使用 beforeunload 事件拦截页面关闭/刷新
 * 注意: 由于 BrowserRouter 限制，无法拦截 SPA 内部导航
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
  onBlock: _onBlock,
}: UseNavigationGuardOptions): UseNavigationGuardResult {
  // 当前使用简化实现，不使用 useBlocker（需要 data router）
  // beforeunload 事件已在 CompressModeContent 中处理

  const [isBlocked, setIsBlocked] = React.useState(false);

  // 继续导航
  const proceed = React.useCallback(() => {
    setIsBlocked(false);
  }, []);

  // 取消导航
  const reset = React.useCallback(() => {
    setIsBlocked(false);
  }, []);

  // 当 shouldBlock 变为 false 时重置状态
  React.useEffect(() => {
    if (!shouldBlock) {
      setIsBlocked(false);
    }
  }, [shouldBlock]);

  return {
    isBlocked,
    proceed,
    reset,
  };
}

export default useNavigationGuard;
