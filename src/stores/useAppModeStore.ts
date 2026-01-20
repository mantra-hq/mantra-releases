/**
 * useAppModeStore - 应用模式状态管理
 * Story 2.34: 全局模式切换
 *
 * 管理应用的显示模式:
 * - playback: 回放模式（默认）
 * - analytics: 分析模式
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";

/**
 * 应用显示模式
 * - playback: 回放模式（查看会话消息和代码快照）
 * - analytics: 分析模式（查看分析统计数据）
 */
export type AppMode = "playback" | "analytics";

/**
 * 应用模式状态接口
 */
export interface AppModeState {
  // ======== 状态 ========
  /** 当前模式 */
  mode: AppMode;

  // ======== Actions ========
  /** 设置模式 */
  setMode: (mode: AppMode) => void;
  /** 切换模式 */
  toggleMode: () => void;
  /** 是否为分析模式 */
  isAnalyticsMode: () => boolean;
}

/**
 * 应用模式 Store
 *
 * 使用 persist 中间件将模式持久化到 localStorage
 */
export const useAppModeStore = create<AppModeState>()(
  persist(
    (set, get) => ({
      mode: "playback",

      setMode: (mode) => set({ mode }),

      toggleMode: () =>
        set((state) => ({
          mode: state.mode === "playback" ? "analytics" : "playback",
        })),

      isAnalyticsMode: () => get().mode === "analytics",
    }),
    {
      name: "mantra-app-mode",
    }
  )
);
