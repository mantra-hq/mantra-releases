/**
 * useEditorKeyboard - 编辑器快捷键 Hook 测试
 * Story 2.13: Task 7 验证
 * AC: #15 Cmd+P, #16 Cmd+W, #17 Cmd+Tab, #18 Cmd+Shift+Tab
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useEditorKeyboard } from "./useEditorKeyboard";
import { useEditorStore } from "@/stores/useEditorStore";

describe("useEditorKeyboard", () => {
    // 模拟 window.addEventListener
    let addEventListenerSpy: ReturnType<typeof vi.spyOn>;
    let removeEventListenerSpy: ReturnType<typeof vi.spyOn>;

    beforeEach(() => {
        useEditorStore.getState().closeAllTabs();
        addEventListenerSpy = vi.spyOn(window, "addEventListener");
        removeEventListenerSpy = vi.spyOn(window, "removeEventListener");
    });

    afterEach(() => {
        addEventListenerSpy.mockRestore();
        removeEventListenerSpy.mockRestore();
    });

    describe("注册监听器", () => {
        it("应该注册 keydown 事件监听器", () => {
            renderHook(() => useEditorKeyboard());

            expect(addEventListenerSpy).toHaveBeenCalledWith(
                "keydown",
                expect.any(Function)
            );
        });

        it("卸载时应移除监听器", () => {
            const { unmount } = renderHook(() => useEditorKeyboard());

            unmount();

            expect(removeEventListenerSpy).toHaveBeenCalledWith(
                "keydown",
                expect.any(Function)
            );
        });

        it("enabled=false 时不响应快捷键", () => {
            const onQuickOpen = vi.fn();
            renderHook(() => useEditorKeyboard({ onQuickOpen, enabled: false }));

            // 模拟 Cmd+P
            const event = new KeyboardEvent("keydown", {
                key: "p",
                metaKey: true,
                bubbles: true,
            });
            window.dispatchEvent(event);

            expect(onQuickOpen).not.toHaveBeenCalled();
        });
    });

    describe("AC #15: Cmd/Ctrl+P → Quick Open", () => {
        it("按下 Cmd+P 应调用 onQuickOpen", () => {
            const onQuickOpen = vi.fn();
            renderHook(() => useEditorKeyboard({ onQuickOpen }));

            const event = new KeyboardEvent("keydown", {
                key: "p",
                metaKey: true,
                bubbles: true,
            });
            window.dispatchEvent(event);

            expect(onQuickOpen).toHaveBeenCalled();
        });

        it("按下 Ctrl+P 应调用 onQuickOpen", () => {
            const onQuickOpen = vi.fn();
            renderHook(() => useEditorKeyboard({ onQuickOpen }));

            const event = new KeyboardEvent("keydown", {
                key: "p",
                ctrlKey: true,
                bubbles: true,
            });
            window.dispatchEvent(event);

            expect(onQuickOpen).toHaveBeenCalled();
        });
    });

    describe("AC #16: Cmd/Ctrl+W → 关闭当前标签", () => {
        it("按下 Cmd+W 应关闭当前标签", () => {
            // 先打开一些标签
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            
            renderHook(() => useEditorKeyboard());

            const event = new KeyboardEvent("keydown", {
                key: "w",
                metaKey: true,
                bubbles: true,
            });
            window.dispatchEvent(event);

            expect(useEditorStore.getState().tabs).toHaveLength(1);
        });
    });

    describe("AC #17: Cmd/Ctrl+Tab → 下一标签", () => {
        it("按下 Cmd+Tab 应切换到下一标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().openTab("src/c.ts");
            useEditorStore.getState().setActiveTab("src/a.ts");

            renderHook(() => useEditorKeyboard());

            const event = new KeyboardEvent("keydown", {
                key: "Tab",
                metaKey: true,
                bubbles: true,
            });
            window.dispatchEvent(event);

            expect(useEditorStore.getState().activeTabId).toBe("src/b.ts");
        });
    });

    describe("AC #18: Cmd/Ctrl+Shift+Tab → 上一标签", () => {
        it("按下 Cmd+Shift+Tab 应切换到上一标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().openTab("src/c.ts");
            useEditorStore.getState().setActiveTab("src/b.ts");

            renderHook(() => useEditorKeyboard());

            const event = new KeyboardEvent("keydown", {
                key: "Tab",
                metaKey: true,
                shiftKey: true,
                bubbles: true,
            });
            window.dispatchEvent(event);

            expect(useEditorStore.getState().activeTabId).toBe("src/a.ts");
        });
    });

    describe("Cmd/Ctrl+B → 切换侧边栏", () => {
        it("按下 Cmd+B 应切换侧边栏", () => {
            expect(useEditorStore.getState().sidebarOpen).toBe(false);

            renderHook(() => useEditorKeyboard());

            const event = new KeyboardEvent("keydown", {
                key: "b",
                metaKey: true,
                bubbles: true,
            });
            window.dispatchEvent(event);

            expect(useEditorStore.getState().sidebarOpen).toBe(true);
        });
    });
});




