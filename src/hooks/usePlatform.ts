/**
 * usePlatform - 平台检测 Hook
 * 
 * 检测当前运行平台 (Mac/Other)
 * 用于处理平台差异，如快捷键 (⌘ vs Ctrl)
 */

import * as React from "react";

export type Platform = "mac" | "other";

/**
 * 检测是否为 Mac 平台
 * 支持新旧 API，优先使用 userAgentData
 */
function detectPlatform(): Platform {
  // 优先使用新的 userAgentData API (如果可用)
  if (typeof navigator !== "undefined") {
    // @ts-expect-error - userAgentData 是较新的 API，类型定义可能不完整
    const userAgentData = navigator.userAgentData;
    if (userAgentData?.platform) {
      return userAgentData.platform.toLowerCase().includes("mac") ? "mac" : "other";
    }

    // 回退到 navigator.platform (已弃用但仍广泛支持)
    if (navigator.platform) {
      return navigator.platform.toUpperCase().indexOf("MAC") >= 0 ? "mac" : "other";
    }
  }

  return "other";
}

/**
 * 平台检测 Hook
 * 
 * @returns 当前平台类型 ("mac" | "other")
 * 
 * @example
 * ```tsx
 * const platform = usePlatform();
 * const shortcutKey = platform === "mac" ? "⌘" : "Ctrl";
 * ```
 */
export function usePlatform(): Platform {
  const [platform, setPlatform] = React.useState<Platform>("other");

  React.useEffect(() => {
    setPlatform(detectPlatform());
  }, []);

  return platform;
}

/**
 * 获取修饰键显示文本
 * 
 * @param platform 平台类型
 * @returns 修饰键符号 (⌘ for Mac, Ctrl for others)
 */
export function getModifierKey(platform: Platform): string {
  return platform === "mac" ? "⌘" : "Ctrl";
}

/**
 * 获取 Shift 键显示文本
 * 
 * @param platform 平台类型
 * @returns Shift 键符号 (⇧ for Mac, Shift for others)
 */
export function getShiftKey(platform: Platform): string {
  return platform === "mac" ? "⇧" : "Shift+";
}

export default usePlatform;
