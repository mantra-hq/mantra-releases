/**
 * useResponsiveLayout - 响应式布局检测 Hook
 * Story 2.2: AC #5, Task 3
 *
 * 断点定义:
 * - Desktop: ≥ 1024px (双流分栏)
 * - Tablet: 768px - 1023px (Tab 切换)
 * - Mobile: < 768px (单视图 + 底部 Tab Bar)
 */

import * as React from "react";

export type LayoutMode = "desktop" | "tablet" | "mobile";

export interface Breakpoints {
  /** Mobile 断点 (< 此值为 Mobile) */
  mobile: number;
  /** Tablet 断点 (≥ 此值为 Desktop) */
  tablet: number;
}

const DEFAULT_BREAKPOINTS: Breakpoints = {
  mobile: 768,
  tablet: 1024,
};

/**
 * 根据窗口宽度计算布局模式
 */
function getLayoutMode(width: number, breakpoints: Breakpoints): LayoutMode {
  if (width >= breakpoints.tablet) {
    return "desktop";
  }
  if (width >= breakpoints.mobile) {
    return "tablet";
  }
  return "mobile";
}

/**
 * 获取当前窗口宽度 (SSR 安全)
 */
function getWindowWidth(): number {
  if (typeof window === "undefined") {
    return 1024; // SSR 默认 Desktop
  }
  return window.innerWidth;
}

export interface UseResponsiveLayoutOptions {
  /** 自定义断点 */
  breakpoints?: Partial<Breakpoints>;
}

/**
 * 响应式布局检测 Hook
 *
 * @param options - 配置选项
 * @returns 当前布局模式
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const layoutMode = useResponsiveLayout();
 *
 *   if (layoutMode === 'desktop') {
 *     return <DesktopLayout />;
 *   }
 *   // ...
 * }
 * ```
 */
export function useResponsiveLayout(
  options?: UseResponsiveLayoutOptions
): LayoutMode {
  const breakpoints = React.useMemo(
    () => ({
      ...DEFAULT_BREAKPOINTS,
      ...options?.breakpoints,
    }),
    [options?.breakpoints]
  );

  // 初始化状态
  const [layoutMode, setLayoutMode] = React.useState<LayoutMode>(() =>
    getLayoutMode(getWindowWidth(), breakpoints)
  );

  React.useEffect(() => {
    // 创建 media query listeners
    const desktopQuery = window.matchMedia(
      `(min-width: ${breakpoints.tablet}px)`
    );
    const tabletQuery = window.matchMedia(
      `(min-width: ${breakpoints.mobile}px)`
    );

    // 更新布局模式
    const updateLayoutMode = () => {
      const width = getWindowWidth();
      setLayoutMode(getLayoutMode(width, breakpoints));
    };

    // 监听变化
    desktopQuery.addEventListener("change", updateLayoutMode);
    tabletQuery.addEventListener("change", updateLayoutMode);

    // 初始更新 (处理 SSR hydration)
    updateLayoutMode();

    // 清理
    return () => {
      desktopQuery.removeEventListener("change", updateLayoutMode);
      tabletQuery.removeEventListener("change", updateLayoutMode);
    };
  }, [breakpoints]);

  return layoutMode;
}

/**
 * 获取断点常量
 */
export function getBreakpoints(): Readonly<Breakpoints> {
  return DEFAULT_BREAKPOINTS;
}

export default useResponsiveLayout;

