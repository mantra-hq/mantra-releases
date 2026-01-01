/**
 * useEditorKeyboard - 编辑器快捷键 Hook
 * Story 2.13: Task 7 - AC #15, #16, #17, #18
 *
 * 功能:
 * - Cmd/Ctrl+P → Quick Open
 * - Cmd/Ctrl+W → 关闭当前标签
 * - Cmd/Ctrl+Tab → 下一标签
 * - Cmd/Ctrl+Shift+Tab → 上一标签
 * - Cmd/Ctrl+B → 切换侧边栏
 */

import { useEffect, useCallback } from "react";
import { useEditorStore } from "@/stores/useEditorStore";

export interface UseEditorKeyboardOptions {
    /** Quick Open 回调 */
    onQuickOpen?: () => void;
    /** 是否启用快捷键 */
    enabled?: boolean;
}

/**
 * 编辑器快捷键 Hook
 */
export function useEditorKeyboard(options: UseEditorKeyboardOptions = {}) {
    const { onQuickOpen, enabled = true } = options;
    // 使用独立的选择器确保引用稳定
    const nextTab = useEditorStore((state) => state.nextTab);
    const prevTab = useEditorStore((state) => state.prevTab);
    const closeCurrentTab = useEditorStore((state) => state.closeCurrentTab);
    const toggleSidebar = useEditorStore((state) => state.toggleSidebar);

    const handleKeyDown = useCallback(
        (e: KeyboardEvent) => {
            if (!enabled) return;

            const isMod = e.metaKey || e.ctrlKey;

            // Cmd/Ctrl+P → Quick Open
            if (isMod && e.key === "p") {
                e.preventDefault();
                onQuickOpen?.();
                return;
            }

            // Cmd/Ctrl+W → 关闭当前标签
            if (isMod && e.key === "w") {
                e.preventDefault();
                closeCurrentTab();
                return;
            }

            // Cmd/Ctrl+B → 切换侧边栏
            if (isMod && e.key === "b") {
                e.preventDefault();
                toggleSidebar();
                return;
            }

            // Cmd/Ctrl+Tab → 下一标签
            if (isMod && e.key === "Tab" && !e.shiftKey) {
                e.preventDefault();
                nextTab();
                return;
            }

            // Cmd/Ctrl+Shift+Tab → 上一标签
            if (isMod && e.key === "Tab" && e.shiftKey) {
                e.preventDefault();
                prevTab();
                return;
            }
        },
        [enabled, onQuickOpen, closeCurrentTab, toggleSidebar, nextTab, prevTab]
    );

    useEffect(() => {
        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [handleKeyDown]);
}

export default useEditorKeyboard;


