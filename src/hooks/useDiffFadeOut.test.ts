/**
 * useDiffFadeOut - Diff 淡出 Hook 测试
 * Story 2.7: Code Review - 补充测试覆盖
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useDiffFadeOut } from "./useDiffFadeOut";

describe("useDiffFadeOut", () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe("初始状态", () => {
        it("shouldShow 初始应为 false", () => {
            const { result } = renderHook(() => useDiffFadeOut());

            expect(result.current.shouldShow).toBe(false);
        });

        it("应该返回 triggerFadeOut 和 cancelFadeOut 方法", () => {
            const { result } = renderHook(() => useDiffFadeOut());

            expect(result.current.triggerFadeOut).toBeDefined();
            expect(result.current.cancelFadeOut).toBeDefined();
        });
    });

    describe("triggerFadeOut", () => {
        it("调用后 shouldShow 应为 true", () => {
            const { result } = renderHook(() => useDiffFadeOut());

            act(() => {
                result.current.triggerFadeOut();
            });

            expect(result.current.shouldShow).toBe(true);
        });

        it("默认 3 秒后 shouldShow 应为 false", () => {
            const { result } = renderHook(() => useDiffFadeOut());

            act(() => {
                result.current.triggerFadeOut();
            });

            expect(result.current.shouldShow).toBe(true);

            // 快进 2.9 秒
            act(() => {
                vi.advanceTimersByTime(2900);
            });

            expect(result.current.shouldShow).toBe(true);

            // 快进到 3 秒
            act(() => {
                vi.advanceTimersByTime(100);
            });

            expect(result.current.shouldShow).toBe(false);
        });

        it("自定义延迟应该生效", () => {
            const { result } = renderHook(() => useDiffFadeOut(1000));

            act(() => {
                result.current.triggerFadeOut();
            });

            expect(result.current.shouldShow).toBe(true);

            // 快进 1 秒
            act(() => {
                vi.advanceTimersByTime(1000);
            });

            expect(result.current.shouldShow).toBe(false);
        });

        it("连续调用应该重置定时器", () => {
            const { result } = renderHook(() => useDiffFadeOut(1000));

            // 第一次触发
            act(() => {
                result.current.triggerFadeOut();
            });

            // 快进 500ms
            act(() => {
                vi.advanceTimersByTime(500);
            });

            expect(result.current.shouldShow).toBe(true);

            // 第二次触发 (重置定时器)
            act(() => {
                result.current.triggerFadeOut();
            });

            // 再快进 500ms (总共 1000ms)
            act(() => {
                vi.advanceTimersByTime(500);
            });

            // 因为定时器被重置，所以还是 true
            expect(result.current.shouldShow).toBe(true);

            // 再快进 500ms
            act(() => {
                vi.advanceTimersByTime(500);
            });

            // 现在应该是 false
            expect(result.current.shouldShow).toBe(false);
        });
    });

    describe("cancelFadeOut", () => {
        it("应该立即隐藏并清除定时器", () => {
            const { result } = renderHook(() => useDiffFadeOut(3000));

            // 触发
            act(() => {
                result.current.triggerFadeOut();
            });

            expect(result.current.shouldShow).toBe(true);

            // 取消
            act(() => {
                result.current.cancelFadeOut();
            });

            expect(result.current.shouldShow).toBe(false);

            // 快进超过原定时间，不应该有任何影响
            act(() => {
                vi.advanceTimersByTime(5000);
            });

            expect(result.current.shouldShow).toBe(false);
        });
    });

    describe("清理", () => {
        it("卸载时应该清除定时器", () => {
            const { result, unmount } = renderHook(() => useDiffFadeOut(3000));

            act(() => {
                result.current.triggerFadeOut();
            });

            expect(result.current.shouldShow).toBe(true);

            // 卸载
            unmount();

            // 不应该有错误
            expect(() => {
                vi.advanceTimersByTime(5000);
            }).not.toThrow();
        });
    });
});
