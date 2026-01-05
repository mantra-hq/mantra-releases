/**
 * useHideEmptyProjects Hook
 * Story 2.29: AC3, AC4, AC5
 * 
 * 管理"隐藏空项目"偏好设置，支持 localStorage 持久化
 */

import { useState, useEffect, useCallback } from "react";

const STORAGE_KEY = "mantra.hideEmptyProjects";

/**
 * useHideEmptyProjects Hook
 * @returns [hideEmptyProjects, setHideEmptyProjects]
 */
export function useHideEmptyProjects(): [boolean, (value: boolean) => void] {
    // 初始化时从 localStorage 读取，默认为 true（隐藏空项目）
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

    // 持久化到 localStorage
    useEffect(() => {
        try {
            localStorage.setItem(STORAGE_KEY, String(hideEmptyProjects));
        } catch {
            // 忽略存储错误
        }
    }, [hideEmptyProjects]);

    const setHideEmptyProjects = useCallback((value: boolean) => {
        setHideEmptyProjectsState(value);
    }, []);

    return [hideEmptyProjects, setHideEmptyProjects];
}
