/**
 * useCompressPersistence - 压缩状态持久化 Hook
 * Story 10.9: Task 3
 *
 * 处理压缩模式下的状态持久化逻辑：
 * - 模式切换时保存/恢复状态
 * - 会话切换时清除状态
 */

import * as React from "react";
import { useCompressState } from "./useCompressState";
import { useCompressPersistStore } from "@/stores";

interface UseCompressPersistenceOptions {
  /** 当前会话 ID */
  sessionId: string;
  /** 是否处于压缩模式 */
  isCompressMode: boolean;
}

/**
 * 压缩状态持久化 Hook
 *
 * 必须在 CompressStateProvider 内部使用
 */
export function useCompressPersistence({
  sessionId,
  isCompressMode,
}: UseCompressPersistenceOptions) {
  const {
    hasAnyChanges,
    initializeFromSnapshot,
    exportSnapshot,
    resetAll,
  } = useCompressState();

  const persistStore = useCompressPersistStore();

  // 追踪上一次的模式和 sessionId
  const prevModeRef = React.useRef(isCompressMode);
  const prevSessionIdRef = React.useRef(sessionId);
  // 追踪是否已初始化（避免重复初始化）
  const isInitializedRef = React.useRef(false);

  // 进入压缩模式时恢复状态
  React.useEffect(() => {
    // 只在模式从非压缩切换到压缩时执行
    if (isCompressMode && !prevModeRef.current && !isInitializedRef.current) {
      const savedSnapshot = persistStore.loadState(sessionId);
      if (savedSnapshot) {
        initializeFromSnapshot(savedSnapshot);
        console.log("[CompressPersistence] 已恢复压缩状态:", sessionId);
      }
      isInitializedRef.current = true;
    }

    // 更新 ref
    prevModeRef.current = isCompressMode;
  }, [isCompressMode, sessionId, persistStore, initializeFromSnapshot]);

  // 退出压缩模式时保存状态
  React.useEffect(() => {
    // 只在模式从压缩切换到非压缩时执行
    if (!isCompressMode && prevModeRef.current) {
      if (hasAnyChanges) {
        const snapshot = exportSnapshot();
        persistStore.saveState(sessionId, snapshot);
        console.log("[CompressPersistence] 已保存压缩状态:", sessionId);
      }
      isInitializedRef.current = false;
    }
  }, [isCompressMode, sessionId, hasAnyChanges, exportSnapshot, persistStore]);

  // sessionId 变化时清除状态
  React.useEffect(() => {
    if (prevSessionIdRef.current !== sessionId) {
      // 会话切换，清除旧状态
      persistStore.clearState();
      resetAll();
      isInitializedRef.current = false;
      console.log("[CompressPersistence] 会话切换，已清除压缩状态");
    }
    prevSessionIdRef.current = sessionId;
  }, [sessionId, persistStore, resetAll]);

  // 组件卸载时保存状态（如果有更改）
  React.useEffect(() => {
    return () => {
      // 注意：这里使用闭包捕获的值，需要使用 ref 获取最新值
      // 但由于这是清理函数，我们依赖于正常的模式切换保存逻辑
    };
  }, []);

  return {
    /** 是否有未保存的更改 */
    hasUnsavedChanges: hasAnyChanges,
  };
}

export default useCompressPersistence;
