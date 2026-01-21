/**
 * useCompressPersistStore - 压缩状态持久化 Store
 * Story 10.9: Task 1
 *
 * 管理压缩模式下的编辑状态持久化，支持模式切换时保留状态。
 * 使用 sessionStorage 实现会话级别持久化。
 */

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { StateSnapshot, CompressOperation } from "@/hooks/useCompressState";

// ===== 序列化类型 =====

/**
 * 序列化后的快照类型
 * Map 类型需要转换为数组才能存储到 sessionStorage
 */
export interface SerializedSnapshot {
  operations: [string, CompressOperation][];
  insertions: [number, CompressOperation][];
}

// ===== Store 接口 =====

/**
 * 压缩状态持久化 Store 接口
 */
export interface CompressPersistState {
  // ======== 状态 ========
  /** 当前会话 ID */
  sessionId: string | null;
  /** 序列化后的状态快照 */
  snapshot: SerializedSnapshot | null;

  // ======== Actions ========
  /**
   * 保存状态到 store
   * @param sessionId 会话 ID
   * @param snapshot 状态快照
   */
  saveState: (sessionId: string, snapshot: StateSnapshot) => void;

  /**
   * 从 store 加载状态
   * @param sessionId 会话 ID
   * @returns 状态快照，如果不存在或 sessionId 不匹配返回 null
   */
  loadState: (sessionId: string) => StateSnapshot | null;

  /**
   * 清除存储的状态
   */
  clearState: () => void;

  /**
   * 检查是否有指定会话的已保存状态
   * @param sessionId 会话 ID
   */
  hasState: (sessionId: string) => boolean;
}

// ===== 序列化工具函数 =====

/**
 * 将 StateSnapshot 序列化为可存储格式
 */
function serializeSnapshot(snapshot: StateSnapshot): SerializedSnapshot {
  return {
    operations: Array.from(snapshot.operations.entries()),
    insertions: Array.from(snapshot.insertions.entries()),
  };
}

/**
 * 将序列化格式反序列化为 StateSnapshot
 */
function deserializeSnapshot(serialized: SerializedSnapshot): StateSnapshot {
  return {
    operations: new Map(serialized.operations),
    insertions: new Map(serialized.insertions),
  };
}

// ===== Store 实现 =====

/**
 * 压缩状态持久化 Store
 *
 * 使用 persist 中间件将状态持久化到 sessionStorage
 * - 会话级别持久化，关闭浏览器后清除
 * - 支持 Map 类型的序列化/反序列化
 */
export const useCompressPersistStore = create<CompressPersistState>()(
  persist(
    (set, get) => ({
      sessionId: null,
      snapshot: null,

      saveState: (sessionId: string, snapshot: StateSnapshot) => {
        // 序列化 Map 为数组
        const serialized = serializeSnapshot(snapshot);
        set({
          sessionId,
          snapshot: serialized,
        });
      },

      loadState: (sessionId: string): StateSnapshot | null => {
        const state = get();

        // 检查 sessionId 是否匹配
        if (state.sessionId !== sessionId || !state.snapshot) {
          return null;
        }

        try {
          // 反序列化数组为 Map
          return deserializeSnapshot(state.snapshot);
        } catch (error) {
          // 序列化失败时清除损坏的存储
          console.warn("[CompressPersistStore] Failed to deserialize snapshot:", error);
          set({ sessionId: null, snapshot: null });
          return null;
        }
      },

      clearState: () => {
        set({
          sessionId: null,
          snapshot: null,
        });
      },

      hasState: (sessionId: string): boolean => {
        const state = get();
        return state.sessionId === sessionId && state.snapshot !== null;
      },
    }),
    {
      name: "mantra-compress-persist",
      storage: createJSONStorage(() => {
        // 安全检查: sessionStorage 可能不可用
        try {
          if (typeof window !== "undefined" && window.sessionStorage) {
            return sessionStorage;
          }
        } catch {
          console.warn("[CompressPersistStore] sessionStorage is not available");
        }
        // 降级: 返回内存存储
        return {
          getItem: () => null,
          setItem: () => {},
          removeItem: () => {},
        };
      }),
    }
  )
);

export default useCompressPersistStore;
