/**
 * useUndoableAction Hook Tests
 * Story 2.19: Task 4.5
 *
 * 测试可撤销操作 Hook
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, cleanup } from "@testing-library/react";
import { useUndoableAction } from "./useUndoableAction";

describe("useUndoableAction", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    cleanup();
  });

  it("executes action immediately", async () => {
    const execute = vi.fn().mockResolvedValue("result");
    const undo = vi.fn().mockResolvedValue(undefined);

    const { result } = renderHook(() => useUndoableAction());

    await act(async () => {
      await result.current.trigger({ execute, undo });
    });

    expect(execute).toHaveBeenCalled();
  });

  it("sets isPending to true during action", async () => {
    let resolveExecute: () => void = () => {};
    const execute = vi.fn().mockImplementation(
      () => new Promise<void>((resolve) => {
        resolveExecute = resolve;
      })
    );
    const undo = vi.fn();

    const { result } = renderHook(() => useUndoableAction());

    act(() => {
      result.current.trigger({ execute, undo });
    });

    expect(result.current.isPending).toBe(true);

    await act(async () => {
      resolveExecute();
    });

    expect(result.current.isPending).toBe(false);
  });

  it("calls undo when cancel is invoked within timeout", async () => {
    const execute = vi.fn().mockResolvedValue(undefined);
    const undo = vi.fn().mockResolvedValue(undefined);

    const { result } = renderHook(() => useUndoableAction());

    await act(async () => {
      await result.current.trigger({ execute, undo, timeoutMs: 5000 });
    });

    expect(execute).toHaveBeenCalled();
    expect(undo).not.toHaveBeenCalled();

    await act(async () => {
      await result.current.cancel();
    });

    expect(undo).toHaveBeenCalled();
  });

  it("does not call undo after timeout expires", async () => {
    const execute = vi.fn().mockResolvedValue(undefined);
    const undo = vi.fn().mockResolvedValue(undefined);

    const { result } = renderHook(() => useUndoableAction());

    await act(async () => {
      await result.current.trigger({ execute, undo, timeoutMs: 5000 });
    });

    // 超时后撤销
    act(() => {
      vi.advanceTimersByTime(6000);
    });

    await act(async () => {
      await result.current.cancel();
    });

    expect(undo).not.toHaveBeenCalled();
  });

  it("uses default timeout of 5000ms", async () => {
    const execute = vi.fn().mockResolvedValue(undefined);
    const undo = vi.fn().mockResolvedValue(undefined);

    const { result } = renderHook(() => useUndoableAction());

    await act(async () => {
      await result.current.trigger({ execute, undo });
    });

    // 4秒后应该还能撤销
    act(() => {
      vi.advanceTimersByTime(4000);
    });

    await act(async () => {
      await result.current.cancel();
    });

    expect(undo).toHaveBeenCalled();
  });

  it("sets canUndo to true after execute and false after timeout", async () => {
    const execute = vi.fn().mockResolvedValue(undefined);
    const undo = vi.fn().mockResolvedValue(undefined);

    const { result } = renderHook(() => useUndoableAction());

    expect(result.current.canUndo).toBe(false);

    await act(async () => {
      await result.current.trigger({ execute, undo, timeoutMs: 5000 });
    });

    expect(result.current.canUndo).toBe(true);

    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    expect(result.current.canUndo).toBe(false);
  });

  it("clears undo state after successful undo", async () => {
    const execute = vi.fn().mockResolvedValue(undefined);
    const undo = vi.fn().mockResolvedValue(undefined);

    const { result } = renderHook(() => useUndoableAction());

    await act(async () => {
      await result.current.trigger({ execute, undo });
    });

    expect(result.current.canUndo).toBe(true);

    await act(async () => {
      await result.current.cancel();
    });

    expect(result.current.canUndo).toBe(false);
  });
});
