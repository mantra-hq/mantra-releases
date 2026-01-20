/**
 * ModeSwitch Tests - Player 模式切换组件测试
 * Story 10.1: Task 6.1
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ModeSwitch, type PlayerMode } from "./ModeSwitch";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "player.modeSwitch": "Mode Switch",
        "player.playbackMode": "Playback",
        "player.compressMode": "Compress",
      };
      return translations[key] || key;
    },
  }),
}));

describe("ModeSwitch", () => {
  const defaultProps = {
    mode: "playback" as PlayerMode,
    onModeChange: vi.fn(),
  };

  it("should render mode switch component with both tabs", () => {
    render(<ModeSwitch {...defaultProps} />);

    expect(screen.getByRole("tablist")).toBeInTheDocument();
    expect(screen.getByText("Playback")).toBeInTheDocument();
    expect(screen.getByText("Compress")).toBeInTheDocument();
  });

  it("should have playback tab selected by default", () => {
    render(<ModeSwitch {...defaultProps} mode="playback" />);

    const playbackTab = screen.getByRole("tab", { name: /playback/i });
    const compressTab = screen.getByRole("tab", { name: /compress/i });

    expect(playbackTab).toHaveAttribute("aria-selected", "true");
    expect(compressTab).toHaveAttribute("aria-selected", "false");
  });

  it("should have compress tab selected when mode is compress", () => {
    render(<ModeSwitch {...defaultProps} mode="compress" />);

    const playbackTab = screen.getByRole("tab", { name: /playback/i });
    const compressTab = screen.getByRole("tab", { name: /compress/i });

    expect(playbackTab).toHaveAttribute("aria-selected", "false");
    expect(compressTab).toHaveAttribute("aria-selected", "true");
  });

  it("should call onModeChange with 'compress' when compress tab is clicked", () => {
    const onModeChange = vi.fn();
    render(<ModeSwitch {...defaultProps} onModeChange={onModeChange} />);

    const compressTab = screen.getByRole("tab", { name: /compress/i });
    fireEvent.click(compressTab);

    expect(onModeChange).toHaveBeenCalledWith("compress");
    expect(onModeChange).toHaveBeenCalledTimes(1);
  });

  it("should call onModeChange with 'playback' when playback tab is clicked", () => {
    const onModeChange = vi.fn();
    render(
      <ModeSwitch {...defaultProps} mode="compress" onModeChange={onModeChange} />
    );

    const playbackTab = screen.getByRole("tab", { name: /playback/i });
    fireEvent.click(playbackTab);

    expect(onModeChange).toHaveBeenCalledWith("playback");
    expect(onModeChange).toHaveBeenCalledTimes(1);
  });

  it("should apply custom className", () => {
    render(<ModeSwitch {...defaultProps} className="custom-class" />);

    const tablist = screen.getByRole("tablist");
    expect(tablist).toHaveClass("custom-class");
  });

  it("should have proper aria-controls attributes", () => {
    render(<ModeSwitch {...defaultProps} />);

    const playbackTab = screen.getByRole("tab", { name: /playback/i });
    const compressTab = screen.getByRole("tab", { name: /compress/i });

    expect(playbackTab).toHaveAttribute("aria-controls", "playback-panel");
    expect(compressTab).toHaveAttribute("aria-controls", "compress-panel");
  });

  it("should have transition classes for smooth animation", () => {
    render(<ModeSwitch {...defaultProps} />);

    const playbackTab = screen.getByRole("tab", { name: /playback/i });
    const compressTab = screen.getByRole("tab", { name: /compress/i });

    // Both tabs should have transition classes
    expect(playbackTab).toHaveClass("transition-all");
    expect(playbackTab).toHaveClass("duration-150");
    expect(compressTab).toHaveClass("transition-all");
    expect(compressTab).toHaveClass("duration-150");
  });
});
