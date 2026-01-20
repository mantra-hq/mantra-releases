/**
 * useCompressMode Tests - 压缩模式状态管理 Hook 测试
 * Story 10.1: Task 6.2
 */

import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useCompressMode } from "./use-compress-mode";

// Mock storage
const mockSessionStorage: Record<string, string> = {};
const mockLocalStorage: Record<string, string> = {};

beforeEach(() => {
  // Clear mocks
  Object.keys(mockSessionStorage).forEach(
    (key) => delete mockSessionStorage[key]
  );
  Object.keys(mockLocalStorage).forEach((key) => delete mockLocalStorage[key]);

  // Mock sessionStorage
  vi.spyOn(Storage.prototype, "getItem").mockImplementation((key: string) => {
    if (key.startsWith("mantra-player-mode-")) {
      return mockSessionStorage[key] || null;
    }
    if (key === "mantra-compress-guide-dismissed") {
      return mockLocalStorage[key] || null;
    }
    return null;
  });

  vi.spyOn(Storage.prototype, "setItem").mockImplementation(
    (key: string, value: string) => {
      if (key.startsWith("mantra-player-mode-")) {
        mockSessionStorage[key] = value;
      }
      if (key === "mantra-compress-guide-dismissed") {
        mockLocalStorage[key] = value;
      }
    }
  );
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("useCompressMode", () => {
  describe("initial state", () => {
    it("should default to playback mode", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-1" })
      );

      expect(result.current.mode).toBe("playback");
    });

    it("should load persisted mode from sessionStorage", () => {
      mockSessionStorage["mantra-player-mode-test-session-2"] = "compress";

      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-2" })
      );

      expect(result.current.mode).toBe("compress");
    });

    it("should default to playback if stored value is invalid", () => {
      mockSessionStorage["mantra-player-mode-test-session-3"] = "invalid";

      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-3" })
      );

      expect(result.current.mode).toBe("playback");
    });
  });

  describe("setMode", () => {
    it("should update mode state", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-4" })
      );

      act(() => {
        result.current.setMode("compress");
      });

      expect(result.current.mode).toBe("compress");
    });

    it("should persist mode to sessionStorage", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-5" })
      );

      act(() => {
        result.current.setMode("compress");
      });

      expect(mockSessionStorage["mantra-player-mode-test-session-5"]).toBe(
        "compress"
      );
    });

    it("should switch back to playback mode", () => {
      mockSessionStorage["mantra-player-mode-test-session-6"] = "compress";

      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-6" })
      );

      act(() => {
        result.current.setMode("playback");
      });

      expect(result.current.mode).toBe("playback");
      expect(mockSessionStorage["mantra-player-mode-test-session-6"]).toBe(
        "playback"
      );
    });
  });

  describe("isFirstTimeCompress", () => {
    it("should be true when switching to compress mode for first time", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-7" })
      );

      act(() => {
        result.current.setMode("compress");
      });

      expect(result.current.isFirstTimeCompress).toBe(true);
    });

    it("should be false when guide has been dismissed", () => {
      mockLocalStorage["mantra-compress-guide-dismissed"] = "true";

      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-8" })
      );

      act(() => {
        result.current.setMode("compress");
      });

      expect(result.current.isFirstTimeCompress).toBe(false);
    });

    it("should be false in playback mode", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-9" })
      );

      expect(result.current.isFirstTimeCompress).toBe(false);
    });
  });

  describe("dismissGuide", () => {
    it("should set guide as dismissed", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-10" })
      );

      act(() => {
        result.current.setMode("compress");
      });

      expect(result.current.isFirstTimeCompress).toBe(true);

      act(() => {
        result.current.dismissGuide();
      });

      expect(result.current.isFirstTimeCompress).toBe(false);
    });

    it("should persist dismissed state to localStorage", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-11" })
      );

      act(() => {
        result.current.dismissGuide();
      });

      expect(mockLocalStorage["mantra-compress-guide-dismissed"]).toBe("true");
    });
  });

  describe("sessionId changes", () => {
    it("should reload mode when sessionId changes", () => {
      mockSessionStorage["mantra-player-mode-session-a"] = "playback";
      mockSessionStorage["mantra-player-mode-session-b"] = "compress";

      const { result, rerender } = renderHook(
        ({ sessionId }) => useCompressMode({ sessionId }),
        { initialProps: { sessionId: "session-a" } }
      );

      expect(result.current.mode).toBe("playback");

      rerender({ sessionId: "session-b" });

      expect(result.current.mode).toBe("compress");
    });
  });
});
