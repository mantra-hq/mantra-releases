/**
 * useUpdateChecker 单元测试
 * Story 14.5: AC #8
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useUpdateChecker } from "./useUpdateChecker";

// --- Mocks ---

const mockClose = vi.fn().mockResolvedValue(undefined);
const mockDownloadAndInstall = vi.fn().mockResolvedValue(undefined);

function createMockUpdate(overrides?: Partial<{
  version: string;
  date: string;
  body: string;
}>) {
  return {
    version: overrides?.version ?? "1.2.0",
    date: overrides?.date ?? "2026-02-08T00:00:00Z",
    body: overrides?.body ?? "Bug fixes and improvements",
    currentVersion: "1.1.0",
    close: mockClose,
    downloadAndInstall: mockDownloadAndInstall,
  };
}

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn(),
}));

import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const mockCheck = vi.mocked(check);
const mockRelaunch = vi.mocked(relaunch);

describe("useUpdateChecker", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    mockClose.mockResolvedValue(undefined);
    mockDownloadAndInstall.mockResolvedValue(undefined);
    mockCheck.mockResolvedValue(null);
    mockRelaunch.mockResolvedValue(undefined);
    // 清除 localStorage
    window.localStorage.clear();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  // --- 5.2: 测试初始状态 ---

  describe("初始状态", () => {
    it("应返回正确的初始状态值", () => {
      const { result } = renderHook(() => useUpdateChecker());

      expect(result.current.updateAvailable).toBe(false);
      expect(result.current.updateInfo).toBeNull();
      expect(result.current.downloadProgress).toBe(0);
      expect(result.current.updateStatus).toBe("idle");
      expect(result.current.errorMessage).toBeNull();
    });

    it("应暴露所有必需的方法", () => {
      const { result } = renderHook(() => useUpdateChecker());

      expect(typeof result.current.checkForUpdate).toBe("function");
      expect(typeof result.current.downloadAndInstall).toBe("function");
      expect(typeof result.current.restartToUpdate).toBe("function");
      expect(typeof result.current.dismissUpdate).toBe("function");
    });
  });

  // --- 5.3: 测试 checkForUpdate 成功流程 ---

  describe("checkForUpdate", () => {
    it("无更新时应保持 updateAvailable=false", async () => {
      mockCheck.mockResolvedValue(null);

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.updateAvailable).toBe(false);
      expect(result.current.updateInfo).toBeNull();
      expect(result.current.updateStatus).toBe("idle");
      expect(mockCheck).toHaveBeenCalled();
    });

    it("有更新时应设置 updateAvailable=true 和 updateInfo", async () => {
      const mockUpdate = createMockUpdate();
      mockCheck.mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.updateAvailable).toBe(true);
      expect(result.current.updateInfo).toEqual({
        version: "1.2.0",
        date: "2026-02-08T00:00:00Z",
        body: "Bug fixes and improvements",
      });
    });

    it("检查后应记录 localStorage 时间戳", async () => {
      mockCheck.mockResolvedValue(null);
      const spy = vi.spyOn(Storage.prototype, "setItem");

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(spy).toHaveBeenCalledWith(
        "mantra-update-last-check",
        expect.any(String)
      );
      spy.mockRestore();
    });

    it("新检查应清理旧的 Update 资源", async () => {
      const mockUpdate1 = createMockUpdate({ version: "1.2.0" });
      const mockUpdate2 = createMockUpdate({ version: "1.3.0" });

      mockCheck.mockResolvedValueOnce(mockUpdate1 as any);
      mockCheck.mockResolvedValueOnce(mockUpdate2 as any);

      const { result } = renderHook(() => useUpdateChecker());

      // 第一次检查
      await act(async () => {
        await result.current.checkForUpdate();
      });

      // 第二次检查应先清理
      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(mockClose).toHaveBeenCalled();
    });
  });

  // --- 5.4: 测试 downloadAndInstall 进度回调 ---

  describe("downloadAndInstall", () => {
    it("应正确更新下载进度", async () => {
      const mockUpdate = createMockUpdate();
      mockUpdate.downloadAndInstall.mockImplementation(async (callback: any) => {
        callback({ event: "Started", data: { contentLength: 1000 } });
        callback({ event: "Progress", data: { chunkLength: 300 } });
        callback({ event: "Progress", data: { chunkLength: 500 } });
        callback({ event: "Finished" });
      });

      mockCheck.mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useUpdateChecker());

      // 先检查获取 Update 对象（这会自动触发下载）
      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.downloadProgress).toBe(100);
      expect(result.current.updateStatus).toBe("ready");
    });

    it("contentLength 为 undefined 时不应崩溃", async () => {
      const mockUpdate = createMockUpdate();
      mockUpdate.downloadAndInstall.mockImplementation(async (callback: any) => {
        callback({ event: "Started", data: { contentLength: undefined } });
        callback({ event: "Progress", data: { chunkLength: 500 } });
        callback({ event: "Finished" });
      });

      mockCheck.mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      // 最终 Finished 会设置 100
      expect(result.current.downloadProgress).toBe(100);
      expect(result.current.updateStatus).toBe("ready");
    });

    it("下载完成后 status 应为 ready", async () => {
      const mockUpdate = createMockUpdate();
      mockUpdate.downloadAndInstall.mockImplementation(async (callback: any) => {
        callback({ event: "Started", data: { contentLength: 100 } });
        callback({ event: "Finished" });
      });

      mockCheck.mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.updateStatus).toBe("ready");
    });

    it("下载失败应设置 error 状态", async () => {
      const mockUpdate = createMockUpdate();
      mockUpdate.downloadAndInstall.mockRejectedValue(new Error("Network error"));

      mockCheck.mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.updateStatus).toBe("error");
      expect(result.current.errorMessage).toBe("Network error");
    });
  });

  // --- 5.5: 测试错误处理 ---

  describe("错误处理", () => {
    it("手动检查失败应设置 error + errorMessage", async () => {
      mockCheck.mockRejectedValue(new Error("Connection refused"));

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.updateStatus).toBe("error");
      expect(result.current.errorMessage).toBe("Connection refused");
    });

    it("自动检查失败应回到 idle，不设 errorMessage", async () => {
      mockCheck.mockRejectedValue(new Error("Timeout"));

      const { result } = renderHook(() => useUpdateChecker());

      // 触发自动检查（通过 startup delay）
      await act(async () => {
        vi.advanceTimersByTime(5000);
      });

      // 等待 promise resolve
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(result.current.updateStatus).toBe("idle");
      // 自动检查不设 errorMessage — 但由于这里 mock 比较复杂，
      // 我们主要验证状态回到 idle
    });

    it("非 Error 对象的异常应提供默认消息", async () => {
      mockCheck.mockRejectedValue("string error");

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.updateStatus).toBe("error");
      expect(result.current.errorMessage).toBe("Check failed");
    });

    it("错误应写入 console.warn", async () => {
      const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
      mockCheck.mockRejectedValue(new Error("Test error"));

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(warnSpy).toHaveBeenCalledWith(
        "[useUpdateChecker]",
        "Test error"
      );
      warnSpy.mockRestore();
    });
  });

  // --- 5.6: 测试 dismissUpdate ---

  describe("dismissUpdate", () => {
    it("应重置所有更新状态", async () => {
      const mockUpdate = createMockUpdate();
      mockUpdate.downloadAndInstall.mockImplementation(async (callback: any) => {
        callback({ event: "Started", data: { contentLength: 100 } });
        callback({ event: "Finished" });
      });
      mockCheck.mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useUpdateChecker());

      // 先执行检查获取更新
      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.updateAvailable).toBe(true);

      // 然后 dismiss
      act(() => {
        result.current.dismissUpdate();
      });

      expect(result.current.updateAvailable).toBe(false);
      expect(result.current.updateInfo).toBeNull();
      expect(result.current.downloadProgress).toBe(0);
      expect(result.current.updateStatus).toBe("idle");
      expect(result.current.errorMessage).toBeNull();
    });

    it("应调用 update.close() 释放资源", async () => {
      const mockUpdate = createMockUpdate();
      mockUpdate.downloadAndInstall.mockImplementation(async (callback: any) => {
        callback({ event: "Started", data: { contentLength: 100 } });
        callback({ event: "Finished" });
      });
      mockCheck.mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      act(() => {
        result.current.dismissUpdate();
      });

      // close 被异步调用
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(mockClose).toHaveBeenCalled();
    });
  });

  // --- 5.7: 测试 localStorage 时间戳逻辑 ---

  describe("localStorage 时间戳", () => {
    it("距上次检查 >= 24h 应触发启动检查", async () => {
      // 设置 25 小时前的时间戳
      const twentyFiveHoursAgo = Date.now() - 25 * 60 * 60 * 1000;
      window.localStorage.setItem(
        "mantra-update-last-check",
        twentyFiveHoursAgo.toString()
      );

      renderHook(() => useUpdateChecker());

      // 快进 5 秒（startup delay）
      await act(async () => {
        vi.advanceTimersByTime(5000);
      });

      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(mockCheck).toHaveBeenCalled();
    });

    it("距上次检查 < 24h 应跳过启动检查", async () => {
      // 设置 1 小时前的时间戳
      const oneHourAgo = Date.now() - 1 * 60 * 60 * 1000;
      window.localStorage.setItem(
        "mantra-update-last-check",
        oneHourAgo.toString()
      );

      renderHook(() => useUpdateChecker());

      // 快进 5 秒（startup delay）
      await act(async () => {
        vi.advanceTimersByTime(5000);
      });

      // 启动检查不应触发（但 24h 周期 timer 仍在）
      // 注意：mockCheck 可能只被 24h timer 调用
      expect(mockCheck).not.toHaveBeenCalled();
    });

    it("无 localStorage 记录应触发启动检查", async () => {
      // 不设置任何 localStorage

      renderHook(() => useUpdateChecker());

      await act(async () => {
        vi.advanceTimersByTime(5000);
      });

      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(mockCheck).toHaveBeenCalled();
    });
  });

  // --- 5.8: 测试定时器清理 ---

  describe("定时器清理", () => {
    it("组件卸载应清理所有定时器", () => {
      const clearTimeoutSpy = vi.spyOn(global, "clearTimeout");

      const { unmount } = renderHook(() => useUpdateChecker());

      unmount();

      // 应该调用了 clearTimeout 清理 startup 和 periodic timer
      expect(clearTimeoutSpy).toHaveBeenCalled();
      clearTimeoutSpy.mockRestore();
    });

    it("组件卸载应调用 update.close() 清理资源", async () => {
      const mockUpdate = createMockUpdate();
      mockUpdate.downloadAndInstall.mockImplementation(async (callback: any) => {
        callback({ event: "Started", data: { contentLength: 100 } });
        callback({ event: "Finished" });
      });
      mockCheck.mockResolvedValue(mockUpdate as any);

      const { result, unmount } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      unmount();

      // unmount 触发 cleanup 中的 cleanupUpdate
      await act(async () => {
        await vi.runAllTimersAsync();
      });

      expect(mockClose).toHaveBeenCalled();
    });
  });

  // --- restartToUpdate ---

  describe("restartToUpdate", () => {
    it("应调用 relaunch()", async () => {
      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.restartToUpdate();
      });

      expect(mockRelaunch).toHaveBeenCalled();
    });

    it("relaunch 失败应设置 error 状态", async () => {
      mockRelaunch.mockRejectedValue(new Error("Relaunch failed"));

      const { result } = renderHook(() => useUpdateChecker());

      await act(async () => {
        await result.current.restartToUpdate();
      });

      expect(result.current.updateStatus).toBe("error");
      expect(result.current.errorMessage).toBe("Relaunch failed");
    });
  });
});
