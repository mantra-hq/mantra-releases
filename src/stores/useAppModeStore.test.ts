/**
 * useAppModeStore Tests - 应用模式状态测试
 * Story 2.34: Task 6.2
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

    it("should not be in statistics mode by default", () => {
      const { isStatisticsMode } = useAppModeStore.getState();
      expect(isStatisticsMode()).toBe(false);
    });
  });

  describe("setMode", () => {
    it("should set mode to statistics", () => {
      const { setMode } = useAppModeStore.getState();
      setMode("statistics");

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("statistics");
    });

    it("should set mode to playback", () => {
      const { setMode } = useAppModeStore.getState();
      setMode("statistics");
      setMode("playback");

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("playback");
    });
  });

  describe("toggleMode", () => {
    it("should toggle from playback to statistics", () => {
      const { toggleMode } = useAppModeStore.getState();
      toggleMode();

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("statistics");
    });

    it("should toggle from statistics to playback", () => {
      const { setMode, toggleMode } = useAppModeStore.getState();
      setMode("statistics");
      toggleMode();

      const { mode } = useAppModeStore.getState();
      expect(mode).toBe("playback");
    });

    it("should toggle back and forth", () => {
      const { toggleMode } = useAppModeStore.getState();

      toggleMode(); // playback -> statistics
      expect(useAppModeStore.getState().mode).toBe("statistics");

      toggleMode(); // statistics -> playback
      expect(useAppModeStore.getState().mode).toBe("playback");

      toggleMode(); // playback -> statistics
      expect(useAppModeStore.getState().mode).toBe("statistics");
    });
  });

  describe("isStatisticsMode", () => {
    it("should return false in playback mode", () => {
      const { isStatisticsMode } = useAppModeStore.getState();
      expect(isStatisticsMode()).toBe(false);
    });

    it("should return true in statistics mode", () => {
      const { setMode, isStatisticsMode } = useAppModeStore.getState();
      setMode("statistics");

      expect(isStatisticsMode()).toBe(true);
    });
  });
});
