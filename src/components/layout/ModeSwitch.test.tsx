/**
 * ModeSwitch Tests - 三态模式切换组件测试
 * Story 2.34: Task 6.1
 * Story 10.11: 三态模式支持
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ModeSwitch } from "./ModeSwitch";
import { useAppModeStore } from "@/stores/useAppModeStore";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("ModeSwitch", () => {
  beforeEach(() => {
    // Reset store to initial state
    useAppModeStore.setState({ mode: "playback" });
  });

  it("should render mode switch component", () => {
    render(<ModeSwitch />);
    expect(screen.getByTestId("mode-switch")).toBeInTheDocument();
  });

  describe("three-state rendering", () => {
    it("should render all three mode buttons by default", () => {
      render(<ModeSwitch />);
      expect(screen.getByTestId("mode-switch-analytics")).toBeInTheDocument();
      expect(screen.getByTestId("mode-switch-playback")).toBeInTheDocument();
      expect(screen.getByTestId("mode-switch-compress")).toBeInTheDocument();
    });

    it("should hide compress button when disableCompress is true", () => {
      render(<ModeSwitch disableCompress />);
      expect(screen.getByTestId("mode-switch-analytics")).toBeInTheDocument();
      expect(screen.getByTestId("mode-switch-playback")).toBeInTheDocument();
      expect(screen.queryByTestId("mode-switch-compress")).not.toBeInTheDocument();
    });
  });

  describe("initial state", () => {
    it("should have playback selected by default", () => {
      render(<ModeSwitch />);
      const playbackButton = screen.getByTestId("mode-switch-playback");
      expect(playbackButton).toHaveAttribute("aria-selected", "true");
    });
  });

  describe("mode switching", () => {
    it("should switch to analytics mode when clicked", () => {
      render(<ModeSwitch />);

      const analyticsButton = screen.getByTestId("mode-switch-analytics");
      fireEvent.click(analyticsButton);

      expect(useAppModeStore.getState().mode).toBe("analytics");
    });

    it("should switch to compress mode when clicked", () => {
      render(<ModeSwitch />);

      const compressButton = screen.getByTestId("mode-switch-compress");
      fireEvent.click(compressButton);

      expect(useAppModeStore.getState().mode).toBe("compress");
    });

    it("should switch back to playback mode when clicked", () => {
      useAppModeStore.setState({ mode: "analytics" });
      render(<ModeSwitch />);

      const playbackButton = screen.getByTestId("mode-switch-playback");
      fireEvent.click(playbackButton);

      expect(useAppModeStore.getState().mode).toBe("playback");
    });
  });

  describe("aria-selected updates", () => {
    it("should update aria-selected when switching to analytics", () => {
      render(<ModeSwitch />);

      const playbackButton = screen.getByTestId("mode-switch-playback");
      const analyticsButton = screen.getByTestId("mode-switch-analytics");
      
      expect(playbackButton).toHaveAttribute("aria-selected", "true");
      expect(analyticsButton).toHaveAttribute("aria-selected", "false");

      fireEvent.click(analyticsButton);

      // Re-query buttons as component re-renders
      const updatedPlayback = screen.getByTestId("mode-switch-playback");
      const updatedAnalytics = screen.getByTestId("mode-switch-analytics");
      expect(updatedPlayback).toHaveAttribute("aria-selected", "false");
      expect(updatedAnalytics).toHaveAttribute("aria-selected", "true");
    });

    it("should update aria-selected when switching to compress", () => {
      render(<ModeSwitch />);

      const compressButton = screen.getByTestId("mode-switch-compress");
      fireEvent.click(compressButton);

      // Re-query buttons
      const updatedPlayback = screen.getByTestId("mode-switch-playback");
      const updatedCompress = screen.getByTestId("mode-switch-compress");
      expect(updatedPlayback).toHaveAttribute("aria-selected", "false");
      expect(updatedCompress).toHaveAttribute("aria-selected", "true");
    });
  });

  describe("same mode click", () => {
    it("should not change mode when same mode is clicked", () => {
      const initialMode = useAppModeStore.getState().mode;
      render(<ModeSwitch />);

      const playbackButton = screen.getByTestId("mode-switch-playback");
      fireEvent.click(playbackButton);

      expect(useAppModeStore.getState().mode).toBe(initialMode);
    });
  });

  describe("custom className", () => {
    it("should apply custom className", () => {
      render(<ModeSwitch className="custom-class" />);
      expect(screen.getByTestId("mode-switch")).toHaveClass("custom-class");
    });
  });

  describe("AC3: responsive layout", () => {
    it("should render labels with sr-only class for responsive hiding", () => {
      render(<ModeSwitch />);
      
      // Check that span elements exist with sr-only class (visible on sm+ screens)
      const playbackButton = screen.getByTestId("mode-switch-playback");
      const labelSpan = playbackButton.querySelector("span");
      expect(labelSpan).toHaveClass("sr-only", "sm:not-sr-only");
    });
  });
});
