/**
 * useAppModeStore Tests - 应用模式状态测试
 * Story 2.34: Task 6.2
 * Story 10.11: 三态模式测试
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useAppModeStore } from "./useAppModeStore";

describe("useAppModeStore", () => {
  beforeEach(() => {
    // Reset store to initial state
    useAppModeStore.setState({ mode: "playback" });
  });

  describe("initial state", () => {
    it("should default to playback mode", () => {
      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("playback");
    });

    it("should not be in analytics mode by default", () => {
      const { isAnalyticsMode } = useAppModeStore.getState();
      expect(isAnalyticsMode()).toBe(false);
    });

    it("should not be in compress mode by default", () => {
      const { isCompressMode } = useAppModeStore.getState();
      expect(isCompressMode()).toBe(false);
    });
  });

  describe("setMode", () => {
    it("should set mode to analytics", () => {
      const { setMode } = useAppModeStore.getState();
      setMode("analytics");

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("analytics");
    });

    it("should set mode to compress", () => {
      const { setMode } = useAppModeStore.getState();
      setMode("compress");

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("compress");
    });

    it("should set mode to playback", () => {
      const { setMode } = useAppModeStore.getState();
      setMode("analytics");
      setMode("playback");

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("playback");
    });
  });

  describe("toggleMode", () => {
    it("should toggle from playback to analytics", () => {
      const { toggleMode } = useAppModeStore.getState();
      toggleMode();

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("analytics");
    });

    it("should toggle from analytics to playback", () => {
      const { setMode, toggleMode } = useAppModeStore.getState();
      setMode("analytics");
      toggleMode();

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("playback");
    });

    it("should toggle from compress to playback", () => {
      const { setMode, toggleMode } = useAppModeStore.getState();
      setMode("compress");
      toggleMode();

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("playback");
    });

    it("should toggle back and forth between playback and analytics", () => {
      const { toggleMode } = useAppModeStore.getState();

      toggleMode(); // playback -> analytics
      expect(useAppModeStore.getState().mode).toBe("analytics");

      toggleMode(); // analytics -> playback
      expect(useAppModeStore.getState().mode).toBe("playback");

      toggleMode(); // playback -> analytics
      expect(useAppModeStore.getState().mode).toBe("analytics");
    });
  });

  describe("isAnalyticsMode", () => {
    it("should return false in playback mode", () => {
      const { isAnalyticsMode } = useAppModeStore.getState();
      expect(isAnalyticsMode()).toBe(false);
    });

    it("should return true in analytics mode", () => {
      const { setMode, isAnalyticsMode } = useAppModeStore.getState();
      setMode("analytics");

      expect(isAnalyticsMode()).toBe(true);
    });

    it("should return false in compress mode", () => {
      const { setMode, isAnalyticsMode } = useAppModeStore.getState();
      setMode("compress");

      expect(isAnalyticsMode()).toBe(false);
    });
  });

  describe("isCompressMode", () => {
    it("should return false in playback mode", () => {
      const { isCompressMode } = useAppModeStore.getState();
      expect(isCompressMode()).toBe(false);
    });

    it("should return false in analytics mode", () => {
      const { setMode, isCompressMode } = useAppModeStore.getState();
      setMode("analytics");

      expect(isCompressMode()).toBe(false);
    });

    it("should return true in compress mode", () => {
      const { setMode, isCompressMode } = useAppModeStore.getState();
      setMode("compress");

      expect(isCompressMode()).toBe(true);
    });
  });
});
