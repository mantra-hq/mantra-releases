/**
 * useCompressMode Tests - 压缩模式引导弹窗状态管理 Hook 测试
 * Story 10.1: Task 6.2
 * Story 10.11: 重构 - 使用统一的 useAppModeStore
 */

import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useCompressMode } from "./use-compress-mode";
import { useAppModeStore } from "@/stores/useAppModeStore";

// Mock storage
const mockLocalStorage: Record<string, string> = {};

beforeEach(() => {
  // Reset unified store
  useAppModeStore.setState({ mode: "playback" });
  
  // Clear mocks
  Object.keys(mockLocalStorage).forEach((key) => delete mockLocalStorage[key]);

  // Mock localStorage only (no longer using sessionStorage)
  vi.spyOn(Storage.prototype, "getItem").mockImplementation((key: string) => {
    if (key === "mantra-compress-guide-dismissed") {
      return mockLocalStorage[key] || null;
    }
    // Return stored value for zustand persist (mantra-app-mode)
    if (key === "mantra-app-mode") {
      return null; // Let zustand use default
    }
    return null;
  });

  vi.spyOn(Storage.prototype, "setItem").mockImplementation(
    (key: string, value: string) => {
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
  describe("mode integration with useAppModeStore", () => {
    it("should return mode from unified store", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-1" })
      );

      expect(result.current.mode).toBe("playback");
    });

    it("should update unified store when setMode is called", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-2" })
      );

      act(() => {
        result.current.setMode("compress");
      });

      expect(result.current.mode).toBe("compress");
      expect(useAppModeStore.getState().mode).toBe("compress");
    });

    it("should support all three modes", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-3" })
      );

      act(() => {
        result.current.setMode("analytics");
      });
      expect(result.current.mode).toBe("analytics");

      act(() => {
        result.current.setMode("compress");
      });
      expect(result.current.mode).toBe("compress");

      act(() => {
        result.current.setMode("playback");
      });
      expect(result.current.mode).toBe("playback");
    });
  });

  describe("AC6: no sessionId fallback", () => {
    it("should fallback to playback when no sessionId and mode is compress", () => {
      // Set compress mode first
      useAppModeStore.setState({ mode: "compress" });
      
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "" })
      );

      // Should have fallen back to playback
      expect(result.current.mode).toBe("playback");
    });

    it("should allow analytics mode without sessionId", () => {
      useAppModeStore.setState({ mode: "analytics" });
      
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "" })
      );

      // Analytics should remain (no fallback)
      expect(result.current.mode).toBe("analytics");
    });
  });

  describe("isFirstTimeCompress", () => {
    it("should be true when in compress mode with sessionId and not dismissed", () => {
      useAppModeStore.setState({ mode: "compress" });
      
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-4" })
      );

      expect(result.current.isFirstTimeCompress).toBe(true);
    });

    it("should be false when guide has been dismissed", () => {
      mockLocalStorage["mantra-compress-guide-dismissed"] = "true";
      useAppModeStore.setState({ mode: "compress" });

      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-5" })
      );

      expect(result.current.isFirstTimeCompress).toBe(false);
    });

    it("should be false in playback mode", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-6" })
      );

      expect(result.current.isFirstTimeCompress).toBe(false);
    });

    it("should be false without sessionId even in compress mode", () => {
      // First set compress mode
      useAppModeStore.setState({ mode: "compress" });
      
      // Then render hook without sessionId - it should fallback
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "" })
      );

      // Mode falls back to playback, so isFirstTimeCompress is false
      expect(result.current.isFirstTimeCompress).toBe(false);
    });
  });

  describe("hideGuide", () => {
    it("should temporarily hide the guide", () => {
      useAppModeStore.setState({ mode: "compress" });
      
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-7" })
      );

      expect(result.current.isFirstTimeCompress).toBe(true);

      act(() => {
        result.current.hideGuide();
      });

      expect(result.current.isFirstTimeCompress).toBe(false);
    });

    it("should reset hidden state when mode changes from compress", () => {
      useAppModeStore.setState({ mode: "compress" });
      
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-8" })
      );

      act(() => {
        result.current.hideGuide();
      });
      expect(result.current.isFirstTimeCompress).toBe(false);

      // Switch to playback
      act(() => {
        result.current.setMode("playback");
      });

      // Switch back to compress - guide should show again
      act(() => {
        result.current.setMode("compress");
      });

      expect(result.current.isFirstTimeCompress).toBe(true);
    });
  });

  describe("dismissGuide", () => {
    it("should permanently dismiss the guide", () => {
      useAppModeStore.setState({ mode: "compress" });
      
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-9" })
      );

      expect(result.current.isFirstTimeCompress).toBe(true);

      act(() => {
        result.current.dismissGuide();
      });

      expect(result.current.isFirstTimeCompress).toBe(false);
    });

    it("should persist dismissed state to localStorage", () => {
      const { result } = renderHook(() =>
        useCompressMode({ sessionId: "test-session-10" })
      );

      act(() => {
        result.current.dismissGuide();
      });

      expect(mockLocalStorage["mantra-compress-guide-dismissed"]).toBe("true");
    });
  });

  describe("sessionId changes", () => {
    it("should reset hidden state when sessionId changes", () => {
      useAppModeStore.setState({ mode: "compress" });
      
      const { result, rerender } = renderHook(
        ({ sessionId }) => useCompressMode({ sessionId }),
        { initialProps: { sessionId: "session-a" } }
      );

      // Hide guide
      act(() => {
        result.current.hideGuide();
      });
      expect(result.current.isFirstTimeCompress).toBe(false);

      // Change session - should reset hidden state
      rerender({ sessionId: "session-b" });

      // Guide should show again (hidden state reset)
      expect(result.current.isFirstTimeCompress).toBe(true);
    });
  });
});
