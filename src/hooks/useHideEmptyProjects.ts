/**
 * useHideEmptyProjects Hook
 * Story 2.29 V2
 *
 * 管理"隐藏空会话"偏好设置，支持 localStorage 持久化
 * 当勾选时，隐藏所有 is_empty=true 的项目（所有会话都是空会话的项目）
 *
 * 通过 storage 事件和自定义事件实现跨组件状态同步
 */

import { useState, useEffect, useCallback } from "react";

const STORAGE_KEY = "mantra.hideEmptyProjects";
const SYNC_EVENT = "mantra:hideEmptyProjectsChanged";

/**
 * useHideEmptyProjects Hook
 * @returns [hideEmptyProjects, setHideEmptyProjects]
 */
export function useHideEmptyProjects(): [boolean, (value: boolean) => void] {
    // 初始化时从 localStorage 读取，默认为 true（隐藏空会话）
    const [hideEmptyProjects, setHideEmptyProjectsState] = useState<boolean>(() => {
        try {
            const stored = localStorage.getItem(STORAGE_KEY);
            // 如果没有存储值，默认为 true（AC3: 默认勾选）
            if (stored === null) return true;
            return stored === "true";
        } catch {
            return true;
        }
    });

    // 监听自定义同步事件（同一窗口内的组件同步）
    useEffect(() => {
        const handleSyncEvent = (e: CustomEvent<boolean>) => {
            setHideEmptyProjectsState(e.detail);
        };

        window.addEventListener(SYNC_EVENT, handleSyncEvent as EventListener);
        return () => {
            window.removeEventListener(SYNC_EVENT, handleSyncEvent as EventListener);
        };
    }, []);

    // 监听 storage 事件（跨标签页同步）
    useEffect(() => {
        const handleStorageChange = (e: StorageEvent) => {
            if (e.key === STORAGE_KEY && e.newValue !== null) {
                setHideEmptyProjectsState(e.newValue === "true");
            }
        };

        window.addEventListener("storage", handleStorageChange);
        return () => {
            window.removeEventListener("storage", handleStorageChange);
        };
    }, []);

    const setHideEmptyProjects = useCallback((value: boolean) => {
        setHideEmptyProjectsState(value);
        // 持久化到 localStorage
        try {
            localStorage.setItem(STORAGE_KEY, String(value));
        } catch {
            // 忽略存储错误
        }
        // 触发自定义事件通知其他组件（同一窗口内）
        window.dispatchEvent(new CustomEvent(SYNC_EVENT, { detail: value }));
    }, []);

    return [hideEmptyProjects, setHideEmptyProjects];
}
