/**
 * useCollapsible 测试
 * Story 2.15: Task 7.6
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useCollapsible } from "./useCollapsible";

// Mock IntersectionObserver
vi.stubGlobal("IntersectionObserver", class MockIntersectionObserver {
    callback: IntersectionObserverCallback;
    constructor(callback: IntersectionObserverCallback) {
        this.callback = callback;
    }
    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
});

describe("useCollapsible", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    it("默认应该是折叠状态", () => {
        const { result } = renderHook(() => useCollapsible());

        expect(result.current.isExpanded).toBe(false);
        expect(result.current.showFloatingBar).toBe(false);
    });

    it("defaultExpanded=true 时应该是展开状态", () => {
        const { result } = renderHook(() =>
            useCollapsible({ defaultExpanded: true })
        );

        expect(result.current.isExpanded).toBe(true);
    });

    it("toggle 应该切换展开状态", () => {
        const { result } = renderHook(() => useCollapsible());

        expect(result.current.isExpanded).toBe(false);

        act(() => {
            result.current.toggle();
        });

        expect(result.current.isExpanded).toBe(true);

        act(() => {
            result.current.toggle();
        });

        expect(result.current.isExpanded).toBe(false);
    });

    it("expand 应该展开", () => {
        const { result } = renderHook(() => useCollapsible());

        act(() => {
            result.current.expand();
        });

        expect(result.current.isExpanded).toBe(true);
    });

    it("collapse 应该折叠", () => {
        const { result } = renderHook(() =>
            useCollapsible({ defaultExpanded: true })
        );

        act(() => {
            result.current.collapse();
        });

        expect(result.current.isExpanded).toBe(false);
    });

    it("collapse 应该调用 onCollapse 回调", () => {
        const onCollapse = vi.fn();
        const { result } = renderHook(() =>
            useCollapsible({ defaultExpanded: true, onCollapse })
        );

        act(() => {
            result.current.collapse();
        });

        expect(onCollapse).toHaveBeenCalled();
    });

    it("collapse 应该重置 showFloatingBar", () => {
        const { result } = renderHook(() =>
            useCollapsible({ defaultExpanded: true })
        );

        // 模拟 showFloatingBar 为 true 的情况
        act(() => {
            result.current.collapse();
        });

        expect(result.current.showFloatingBar).toBe(false);
    });

    it("Escape 键应该触发折叠", () => {
        const { result } = renderHook(() =>
            useCollapsible({ defaultExpanded: true })
        );

        expect(result.current.isExpanded).toBe(true);

        act(() => {
            const event = new KeyboardEvent("keydown", { key: "Escape" });
            document.dispatchEvent(event);
        });

        expect(result.current.isExpanded).toBe(false);
    });

    it("折叠状态下 Escape 键不应该有效果", () => {
        const onCollapse = vi.fn();
        const { result } = renderHook(() =>
            useCollapsible({ defaultExpanded: false, onCollapse })
        );

        act(() => {
            const event = new KeyboardEvent("keydown", { key: "Escape" });
            document.dispatchEvent(event);
        });

        expect(onCollapse).not.toHaveBeenCalled();
    });

    it("应该返回 collapseButtonRef", () => {
        const { result } = renderHook(() => useCollapsible());

        expect(result.current.collapseButtonRef).toBeDefined();
        expect(result.current.collapseButtonRef.current).toBeNull();
    });

    it("应该返回 contentRef", () => {
        const { result } = renderHook(() => useCollapsible());

        expect(result.current.contentRef).toBeDefined();
        expect(result.current.contentRef.current).toBeNull();
    });

    it("scrollToTop 应该滚动到顶部", () => {
        const mockScrollTo = vi.fn();
        const mockElement = { scrollTo: mockScrollTo };

        const { result } = renderHook(() => useCollapsible());

        // 手动设置 ref
        Object.defineProperty(result.current.contentRef, "current", {
            value: mockElement,
            writable: true,
        });

        act(() => {
            result.current.scrollToTop();
        });

        expect(mockScrollTo).toHaveBeenCalledWith({
            top: 0,
            behavior: "smooth",
        });
    });
});
