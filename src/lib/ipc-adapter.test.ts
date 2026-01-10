/**
 * IPC Adapter Tests - Story 9.2: Task 1
 *
 * 测试 IPC 适配器的核心功能:
 * - 环境检测
 * - Mock 注入
 * - invoke 路由
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  isTestEnv,
  setMockInvoke,
  clearMockInvoke,
  invoke,
  hasMockHandler,
} from "./ipc-adapter";

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue("tauri-result"),
}));

describe("ipc-adapter", () => {
  // 保存原始 window 状态
  const originalPlaywrightTest = window.__PLAYWRIGHT_TEST__;

  beforeEach(() => {
    // 重置状态
    clearMockInvoke();
    delete window.__PLAYWRIGHT_TEST__;
  });

  afterEach(() => {
    // 恢复原始状态
    if (originalPlaywrightTest !== undefined) {
      window.__PLAYWRIGHT_TEST__ = originalPlaywrightTest;
    } else {
      delete window.__PLAYWRIGHT_TEST__;
    }
  });

  describe("isTestEnv", () => {
    it("应该在 __PLAYWRIGHT_TEST__ 未设置时返回 false", () => {
      expect(isTestEnv()).toBe(false);
    });

    it("应该在 __PLAYWRIGHT_TEST__ 为 false 时返回 false", () => {
      window.__PLAYWRIGHT_TEST__ = false;
      expect(isTestEnv()).toBe(false);
    });

    it("应该在 __PLAYWRIGHT_TEST__ 为 true 时返回 true", () => {
      window.__PLAYWRIGHT_TEST__ = true;
      expect(isTestEnv()).toBe(true);
    });
  });

  describe("setMockInvoke / clearMockInvoke / hasMockHandler", () => {
    it("初始状态应该没有 mock 处理器", () => {
      expect(hasMockHandler()).toBe(false);
    });

    it("设置 mock 处理器后应该返回 true", () => {
      const mockHandler = vi.fn();
      setMockInvoke(mockHandler);
      expect(hasMockHandler()).toBe(true);
    });

    it("清除 mock 处理器后应该返回 false", () => {
      const mockHandler = vi.fn();
      setMockInvoke(mockHandler);
      clearMockInvoke();
      expect(hasMockHandler()).toBe(false);
    });
  });

  describe("invoke", () => {
    it("在非测试环境应该调用真实 Tauri invoke", async () => {
      const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");

      const result = await invoke("test_command", { param: "value" });

      expect(tauriInvoke).toHaveBeenCalledWith("test_command", {
        param: "value",
      });
      expect(result).toBe("tauri-result");
    });

    it("在测试环境但无 mock 处理器时应该调用真实 Tauri invoke", async () => {
      window.__PLAYWRIGHT_TEST__ = true;
      const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");

      const result = await invoke("test_command");

      expect(tauriInvoke).toHaveBeenCalledWith("test_command", undefined);
      expect(result).toBe("tauri-result");
    });

    it("在测试环境且有 mock 处理器时应该调用 mock", async () => {
      window.__PLAYWRIGHT_TEST__ = true;
      const mockHandler = vi.fn().mockResolvedValue("mock-result");
      setMockInvoke(mockHandler);

      const result = await invoke("test_command", { param: "value" });

      expect(mockHandler).toHaveBeenCalledWith("test_command", {
        param: "value",
      });
      expect(result).toBe("mock-result");
    });

    it("mock 处理器应该支持泛型返回类型", async () => {
      interface TestResult {
        id: string;
        name: string;
      }

      window.__PLAYWRIGHT_TEST__ = true;
      const mockHandler = vi.fn().mockResolvedValue({
        id: "123",
        name: "test",
      });
      setMockInvoke(mockHandler);

      const result = await invoke<TestResult>("get_test");

      expect(result).toEqual({ id: "123", name: "test" });
    });

    it("mock 处理器抛出错误时应该正常传播", async () => {
      window.__PLAYWRIGHT_TEST__ = true;
      const mockHandler = vi.fn().mockRejectedValue(new Error("Mock error"));
      setMockInvoke(mockHandler);

      await expect(invoke("test_command")).rejects.toThrow("Mock error");
    });
  });
});
