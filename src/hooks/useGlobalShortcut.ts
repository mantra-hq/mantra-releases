/**
 * useGlobalShortcut - 全局快捷键 Hook
 * Story 2.10: Task 2
 *
 * 监听全局键盘快捷键 Cmd+K (macOS) / Ctrl+K (Windows/Linux)
 * 用于打开/关闭全局搜索框
 */

import { useEffect } from "react";
import { useSearchStore } from "@/stores/useSearchStore";

/**
 * 全局搜索快捷键 Hook
 * 监听 Cmd+K / Ctrl+K 打开搜索框
 */
export function useGlobalShortcut() {
    const open = useSearchStore((state) => state.open);
    const close = useSearchStore((state) => state.close);
    const isOpen = useSearchStore((state) => state.isOpen);

    useEffect(() => {
        function handleKeyDown(e: KeyboardEvent) {
            // Cmd+K (macOS) 或 Ctrl+K (Windows/Linux)
            if ((e.metaKey || e.ctrlKey) && e.key === "k") {
                e.preventDefault();
                if (isOpen) {
                    close();
                } else {
                    open();
                }
            }
        }

        document.addEventListener("keydown", handleKeyDown);
        return () => document.removeEventListener("keydown", handleKeyDown);
    }, [open, close, isOpen]);
}

export default useGlobalShortcut;
