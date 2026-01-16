/**
 * ModeSwitch Tests - 模式切换组件测试
 * Story 2.34: Task 6.1
 */

import { describe, it, expect, beforeEach } from "vitest";
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

  it("should render playback and statistics buttons", () => {
    render(<ModeSwitch />);
    expect(screen.getByTestId("mode-switch-playback")).toBeInTheDocument();
    expect(screen.getByTestId("mode-switch-statistics")).toBeInTheDocument();
  });

  it("should have playback selected by default", () => {
    render(<ModeSwitch />);
    const playbackButton = screen.getByTestId("mode-switch-playback");
    expect(playbackButton).toHaveAttribute("aria-selected", "true");
  });

  it("should switch to statistics mode when clicked", () => {
    render(<ModeSwitch />);

    const statisticsButton = screen.getByTestId("mode-switch-statistics");
    fireEvent.click(statisticsButton);

    expect(useAppModeStore.getState().mode).toBe("statistics");
  });

  it("should switch back to playback mode when clicked", () => {
    useAppModeStore.setState({ mode: "statistics" });
    render(<ModeSwitch />);

    const playbackButton = screen.getByTestId("mode-switch-playback");
    fireEvent.click(playbackButton);

    expect(useAppModeStore.getState().mode).toBe("playback");
  });

  it("should update aria-selected when mode changes", () => {
    render(<ModeSwitch />);

    // Initially playback is selected
    let playbackButton = screen.getByTestId("mode-switch-playback");
    let statisticsButton = screen.getByTestId("mode-switch-statistics");
    expect(playbackButton).toHaveAttribute("aria-selected", "true");
    expect(statisticsButton).toHaveAttribute("aria-selected", "false");

    // Click statistics
    fireEvent.click(statisticsButton);

    // Re-query buttons as component re-renders
    playbackButton = screen.getByTestId("mode-switch-playback");
    statisticsButton = screen.getByTestId("mode-switch-statistics");
    expect(playbackButton).toHaveAttribute("aria-selected", "false");
    expect(statisticsButton).toHaveAttribute("aria-selected", "true");
  });

  it("should not change mode when same mode is clicked", () => {
    const initialMode = useAppModeStore.getState().mode;
    render(<ModeSwitch />);

    const playbackButton = screen.getByTestId("mode-switch-playback");
    fireEvent.click(playbackButton);

    expect(useAppModeStore.getState().mode).toBe(initialMode);
  });

  it("should apply custom className", () => {
    render(<ModeSwitch className="custom-class" />);
    expect(screen.getByTestId("mode-switch")).toHaveClass("custom-class");
  });
});
