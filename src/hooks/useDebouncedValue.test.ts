/**
 * useDebouncedValue Tests - 防抖值 Hook 测试
 * Story 2.8: Task 6
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useDebouncedValue } from "./useDebouncedValue";

describe("useDebouncedValue", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("应该立即返回初始值", () => {
    const { result } = renderHook(() => useDebouncedValue("initial", 300));
    expect(result.current).toBe("initial");
  });

  it("应该在延迟后更新值", () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebouncedValue(value, delay),
      { initialProps: { value: "initial", delay: 300 } }
    );

    expect(result.current).toBe("initial");

    // 更新值
    rerender({ value: "updated", delay: 300 });

    // 立即检查 - 应该还是旧值
    expect(result.current).toBe("initial");

    // 快进 299ms - 还是旧值
    act(() => {
      vi.advanceTimersByTime(299);
    });
    expect(result.current).toBe("initial");

    // 快进到 300ms - 应该更新
    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(result.current).toBe("updated");
  });

  it("应该在连续更新时重置定时器", () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebouncedValue(value, delay),
      { initialProps: { value: "v1", delay: 300 } }
    );

    // 第一次更新
    rerender({ value: "v2", delay: 300 });
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(result.current).toBe("v1");

    // 第二次更新 - 重置定时器
    rerender({ value: "v3", delay: 300 });
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(result.current).toBe("v1"); // 还是 v1

    // 完成等待
    act(() => {
      vi.advanceTimersByTime(100);
    });
    expect(result.current).toBe("v3"); // 直接跳到 v3
  });

  it("应该支持自定义延迟时间", () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebouncedValue(value, delay),
      { initialProps: { value: "initial", delay: 500 } }
    );

    rerender({ value: "updated", delay: 500 });

    act(() => {
      vi.advanceTimersByTime(400);
    });
    expect(result.current).toBe("initial");

    act(() => {
      vi.advanceTimersByTime(100);
    });
    expect(result.current).toBe("updated");
  });

  it("应该清理定时器", () => {
    const { unmount, rerender } = renderHook(
      ({ value, delay }) => useDebouncedValue(value, delay),
      { initialProps: { value: "initial", delay: 300 } }
    );

    rerender({ value: "updated", delay: 300 });

    // 卸载组件
    unmount();

    // 应该不会抛出错误
    act(() => {
      vi.advanceTimersByTime(500);
    });
  });

  it("应该支持不同类型的值", () => {
    // 数字
    const { result: numResult } = renderHook(() => useDebouncedValue(42, 300));
    expect(numResult.current).toBe(42);

    // 对象
    const obj = { foo: "bar" };
    const { result: objResult } = renderHook(() => useDebouncedValue(obj, 300));
    expect(objResult.current).toEqual({ foo: "bar" });

    // 布尔值
    const { result: boolResult } = renderHook(() => useDebouncedValue(true, 300));
    expect(boolResult.current).toBe(true);
  });
});

