/**
 * useCompressMode - 压缩模式状态管理 Hook
 * Story 10.1: AC #1, #3, #4
 *
 * 管理回放/压缩模式切换状态
 * - 会话级别状态持久化 (sessionStorage)
 * - 引导弹窗偏好管理 (localStorage)
 */

import * as React from "react";
import type { PlayerMode } from "@/components/player/ModeSwitch";

// Storage keys
const MODE_STORAGE_PREFIX = "mantra-player-mode-";
const GUIDE_DISMISSED_KEY = "mantra-compress-guide-dismissed";

export interface UseCompressModeOptions {
  /** 会话 ID，用于会话级别状态持久化 */
  sessionId: string;
}

export interface UseCompressModeReturn {
  /** 当前模式 */
  mode: PlayerMode;
  /** 设置模式 */
  setMode: (mode: PlayerMode) => void;
  /** 是否是首次进入压缩模式（需要显示引导弹窗） */
  isFirstTimeCompress: boolean;
  /** 临时隐藏引导弹窗（不勾选"不再提示"时使用） */
  hideGuide: () => void;
  /** 关闭引导弹窗并记住偏好 */
  dismissGuide: () => void;
}

/**
 * 获取模式存储 key
 */
function getModeStorageKey(sessionId: string): string {
  return `${MODE_STORAGE_PREFIX}${sessionId}`;
}

/**
 * 从 sessionStorage 读取模式
 */
function loadMode(sessionId: string): PlayerMode {
  try {
    const stored = sessionStorage.getItem(getModeStorageKey(sessionId));
    if (stored === "playback" || stored === "compress") {
      return stored;
    }
  } catch {
    // sessionStorage 不可用，返回默认值
  }
  return "playback";
}

/**
 * 保存模式到 sessionStorage
 */
function saveMode(sessionId: string, mode: PlayerMode): void {
  try {
    sessionStorage.setItem(getModeStorageKey(sessionId), mode);
  } catch {
    // sessionStorage 不可用，静默失败
  }
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
  // 从 sessionStorage 初始化模式状态 (AC #4: 会话级别持久化)
  const [mode, setModeState] = React.useState<PlayerMode>(() =>
    loadMode(sessionId)
  );

  // 引导弹窗是否已永久关闭
  const [guideDismissed, setGuideDismissed] = React.useState<boolean>(() =>
    isGuideDismissed()
  );

  // 引导弹窗是否临时隐藏（当前会话内）
  const [guideHidden, setGuideHidden] = React.useState<boolean>(false);

  // 是否已经切换过到压缩模式（用于判断是否显示引导）
  const [hasEnteredCompress, setHasEnteredCompress] = React.useState<boolean>(
    () => loadMode(sessionId) === "compress"
  );

  // 当 sessionId 变化时，重新加载模式状态
  React.useEffect(() => {
    const storedMode = loadMode(sessionId);
    setModeState(storedMode);
    setHasEnteredCompress(storedMode === "compress");
    // 重置临时隐藏状态
    setGuideHidden(false);
  }, [sessionId]);

  // 设置模式并持久化
  const setMode = React.useCallback(
    (newMode: PlayerMode) => {
      setModeState(newMode);
      saveMode(sessionId, newMode);

      // 首次进入压缩模式时标记
      if (newMode === "compress" && !hasEnteredCompress) {
        setHasEnteredCompress(true);
      }

      // 切换回 playback 时重置临时隐藏状态，下次进入 compress 时可再次显示
      if (newMode === "playback") {
        setGuideHidden(false);
      }
    },
    [sessionId, hasEnteredCompress]
  );

  // 临时隐藏引导弹窗（不勾选"不再提示"）
  const hideGuide = React.useCallback(() => {
    setGuideHidden(true);
  }, []);

  // 永久关闭引导弹窗
  const dismissGuide = React.useCallback(() => {
    setGuideDismissed(true);
    saveGuideDismissed();
  }, []);

  // 是否需要显示引导弹窗 (AC #2: 首次切换到压缩模式)
  // 条件: 当前是压缩模式 + 引导未被永久关闭 + 未被临时隐藏
  const isFirstTimeCompress = mode === "compress" && !guideDismissed && !guideHidden;

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
