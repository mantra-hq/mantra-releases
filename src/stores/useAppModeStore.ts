/**
 * useAppModeStore - 应用模式状态管理
 * Story 2.34: 全局模式切换
 * Story 10.11: 三态模式统一 (playback/analytics/compress)
 *
 * 管理应用的显示模式:
 * - playback: 回放模式（默认）- 查看会话消息和代码快照
 * - analytics: 分析模式 - 查看分析统计数据
 * - compress: 压缩模式 - 优化会话上下文
 */

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

/**
 * 应用显示模式
 * - playback: 回放模式（查看会话消息和代码快照）
 * - analytics: 分析模式（查看分析统计数据）
 * - compress: 压缩模式（优化会话上下文）
 */
export type AppMode = "playback" | "analytics" | "compress";

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
  /** 切换模式（playback <-> analytics，compress 单独控制） */
  toggleMode: () => void;
  /** 是否为分析模式 */
  isAnalyticsMode: () => boolean;
  /** 是否为压缩模式 */
  isCompressMode: () => boolean;
}

// 迁移函数：处理旧版本数据迁移
interface PersistedState {
  mode: string;
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
          // 在 playback 和 analytics 之间切换，compress 模式单独控制
          mode: state.mode === "playback" ? "analytics" : 
                state.mode === "analytics" ? "playback" : 
                "playback", // 从 compress 返回 playback
        })),

      isAnalyticsMode: () => get().mode === "analytics",
      isCompressMode: () => get().mode === "compress",
    }),
    {
      name: "mantra-app-mode",
      storage: createJSONStorage(() => localStorage),
      version: 2, // 版本升级触发迁移
      migrate: (persistedState, version) => {
        const state = persistedState as PersistedState;
        
        // v1 -> v2: 处理可能存在的 "statistics" 旧值
        if (version < 2) {
          if (state.mode === "statistics") {
            return { ...state, mode: "analytics" } as AppModeState;
          }
          // 验证 mode 值有效性，无效则重置
          if (!["playback", "analytics", "compress"].includes(state.mode)) {
            return { ...state, mode: "playback" } as AppModeState;
          }
        }
        
        return state as AppModeState;
      },
    }
  )
);
