/**
 * useGlobalShortcut Tests - 全局快捷键测试
 * Story 2.10: Task 8.4
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useGlobalShortcut } from "./useGlobalShortcut";
import { useSearchStore } from "@/stores/useSearchStore";

describe("useGlobalShortcut", () => {
    beforeEach(() => {
        // Reset store
        act(() => {
            useSearchStore.setState({
                isOpen: false,
                query: "",
                results: [],
                isLoading: false,
                selectedIndex: 0,
                recentSessions: [],
            });
        });
    });

    afterEach(() => {
        vi.clearAllMocks();
    });

    it("should open search on Ctrl+K", () => {
        renderHook(() => useGlobalShortcut());

        act(() => {
            const event = new KeyboardEvent("keydown", {
                key: "k",
                ctrlKey: true,
                bubbles: true,
            });
            document.dispatchEvent(event);
        });

        expect(useSearchStore.getState().isOpen).toBe(true);
    });

    it("should open search on Meta+K (macOS)", () => {
        renderHook(() => useGlobalShortcut());

        act(() => {
            const event = new KeyboardEvent("keydown", {
                key: "k",
                metaKey: true,
                bubbles: true,
            });
            document.dispatchEvent(event);
        });

        expect(useSearchStore.getState().isOpen).toBe(true);
    });

    it("should close search when open and Ctrl+K pressed", () => {
        act(() => {
            useSearchStore.getState().open();
        });

        renderHook(() => useGlobalShortcut());

        act(() => {
            const event = new KeyboardEvent("keydown", {
                key: "k",
                ctrlKey: true,
                bubbles: true,
            });
            document.dispatchEvent(event);
        });

        expect(useSearchStore.getState().isOpen).toBe(false);
    });

    it("should not trigger on other key combinations", () => {
        renderHook(() => useGlobalShortcut());

        act(() => {
            // Just 'k' without modifier
            document.dispatchEvent(
                new KeyboardEvent("keydown", { key: "k", bubbles: true })
            );
        });

        expect(useSearchStore.getState().isOpen).toBe(false);

        act(() => {
            // Ctrl+J
            document.dispatchEvent(
                new KeyboardEvent("keydown", { key: "j", ctrlKey: true, bubbles: true })
            );
        });

        expect(useSearchStore.getState().isOpen).toBe(false);
    });

    it("should prevent default browser behavior", () => {
        renderHook(() => useGlobalShortcut());

        const event = new KeyboardEvent("keydown", {
            key: "k",
            ctrlKey: true,
            bubbles: true,
            cancelable: true,
        });
        const preventDefaultSpy = vi.spyOn(event, "preventDefault");

        act(() => {
            document.dispatchEvent(event);
        });

        expect(preventDefaultSpy).toHaveBeenCalled();
    });
});
