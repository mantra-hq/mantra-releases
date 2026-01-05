/**
 * useNotificationInit - 通知系统初始化 Hook
 * Tech-Spec: 通知系统 Task 14
 *
 * 在应用启动时初始化通知数据
 */

import { useEffect } from "react";
import { useNotificationStore } from "@/stores/useNotificationStore";

/**
 * 初始化通知系统
 * 在应用根级别调用，加载通知数据
 */
export function useNotificationInit() {
  const fetchAll = useNotificationStore((state) => state.fetchAll);

  useEffect(() => {
    // 应用启动时加载通知数据
    fetchAll();
  }, [fetchAll]);
}
