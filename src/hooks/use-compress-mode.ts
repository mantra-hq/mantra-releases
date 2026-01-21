/**
 * useCompressMode - 压缩模式引导弹窗状态管理 Hook
 * Story 10.1: AC #1, #3, #4
 * Story 10.11: 重构 - 使用统一的 useAppModeStore
 *
 * 职责:
 * - 引导弹窗偏好管理 (localStorage)
 * - 无 sessionId 时自动回退到 playback
 * 
 * 注意: 模式状态现由 useAppModeStore 统一管理
 */

import * as React from "react";
import { useAppModeStore, type AppMode } from "@/stores/useAppModeStore";

// Storage keys
const GUIDE_DISMISSED_KEY = "mantra-compress-guide-dismissed";

export interface UseCompressModeOptions {
  /** 会话 ID */
  sessionId: string;
}

export interface UseCompressModeReturn {
  /** 当前模式 */
  mode: AppMode;
  /** 设置模式 */
  setMode: (mode: AppMode) => void;
  /** 是否是首次进入压缩模式（需要显示引导弹窗） */
  isFirstTimeCompress: boolean;
  /** 临时隐藏引导弹窗（不勾选"不再提示"时使用） */
  hideGuide: () => void;
  /** 关闭引导弹窗并记住偏好 */
  dismissGuide: () => void;
}

/**
 * 检查引导弹窗是否已被关闭
 */
function isGuideDismissed(): boolean {
  try {
    return localStorage.getItem(GUIDE_DISMISSED_KEY) === "true";
  } catch {
    return false;
  }
}

/**
 * 保存引导弹窗关闭偏好
 */
function saveGuideDismissed(): void {
  try {
    localStorage.setItem(GUIDE_DISMISSED_KEY, "true");
  } catch {
    // localStorage 不可用，静默失败
  }
}

/**
 * useCompressMode Hook
 *
 * @param options - Hook 配置选项
 * @returns 模式状态和控制函数
 */
export function useCompressMode({
  sessionId,
}: UseCompressModeOptions): UseCompressModeReturn {
  // Story 10.11: 使用统一的 AppMode store
  const mode = useAppModeStore((state) => state.mode);
  const setMode = useAppModeStore((state) => state.setMode);

  // 引导弹窗是否已永久关闭
  const [guideDismissed, setGuideDismissed] = React.useState<boolean>(() =>
    isGuideDismissed()
  );

  // 引导弹窗是否临时隐藏（当前会话内）
  const [guideHidden, setGuideHidden] = React.useState<boolean>(false);

  // 切换会话时重置临时隐藏状态
  React.useEffect(() => {
    setGuideHidden(false);
  }, [sessionId]);

  // AC6: 无 sessionId 时，如果当前是压缩模式则回退到回放模式
  React.useEffect(() => {
    if (!sessionId && mode === "compress") {
      setMode("playback");
    }
  }, [sessionId, mode, setMode]);

  // 当模式从 compress 切换到其他模式时，重置临时隐藏状态
  React.useEffect(() => {
    if (mode !== "compress") {
      setGuideHidden(false);
    }
  }, [mode]);

  // 临时隐藏引导弹窗（不勾选"不再提示"）
  const hideGuide = React.useCallback(() => {
    setGuideHidden(true);
  }, []);

  // 永久关闭引导弹窗
  const dismissGuide = React.useCallback(() => {
    setGuideDismissed(true);
    saveGuideDismissed();
  }, []);

  // 是否需要显示引导弹窗
  // 条件: 当前是压缩模式 + 有 sessionId + 引导未被永久关闭 + 未被临时隐藏
  const isFirstTimeCompress = 
    mode === "compress" && 
    Boolean(sessionId) &&
    !guideDismissed && 
    !guideHidden;

  return {
    mode,
    setMode,
    isFirstTimeCompress,
    hideGuide,
    dismissGuide,
  };
}

// 保持向后兼容的别名
export const useRefineMode = useCompressMode;
export type UseRefineModeOptions = UseCompressModeOptions;
export type UseRefineModeReturn = UseCompressModeReturn;

export default useCompressMode;
