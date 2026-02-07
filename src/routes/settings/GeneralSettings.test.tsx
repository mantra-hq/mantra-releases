/**
 * GeneralSettings Tests - 通用设置页面测试
 * Story 2-35: Task 3.1
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { GeneralSettings } from "./GeneralSettings";

// Mock @tauri-apps/plugin-opener
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

// Mock LanguageSwitcher
vi.mock("@/components/settings/LanguageSwitcher", () => ({
  LanguageSwitcher: () => <div data-testid="language-switcher">LanguageSwitcher</div>,
}));

// Mock useLogStore
const mockCopyToClipboard = vi.fn();
vi.mock("@/stores", () => ({
  useLogStore: (selector: (state: { copyToClipboard: () => Promise<boolean> }) => unknown) =>
    selector({ copyToClipboard: mockCopyToClipboard }),
}));

describe("GeneralSettings", () => {
  it("renders LanguageSwitcher component", () => {
    render(<GeneralSettings />);
    expect(screen.getByTestId("language-switcher")).toBeInTheDocument();
  });

  it("renders help section with title", () => {
    render(<GeneralSettings />);
    expect(screen.getByText("帮助")).toBeInTheDocument();
  });

  it("renders official website button", () => {
    render(<GeneralSettings />);
    expect(screen.getByTestId("official-website-button")).toBeInTheDocument();
  });

  it("renders documentation button", () => {
    render(<GeneralSettings />);
    expect(screen.getByTestId("documentation-button")).toBeInTheDocument();
  });

  it("renders copy logs button", () => {
    render(<GeneralSettings />);
    expect(screen.getByTestId("copy-logs-button")).toBeInTheDocument();
  });

  it("calls openUrl when official website button is clicked", async () => {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    render(<GeneralSettings />);

    fireEvent.click(screen.getByTestId("official-website-button"));
    expect(openUrl).toHaveBeenCalledWith("https://mantra.gonewx.com");
  });

  it("calls openUrl when documentation button is clicked", async () => {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    render(<GeneralSettings />);

    fireEvent.click(screen.getByTestId("documentation-button"));
    expect(openUrl).toHaveBeenCalledWith("https://docs.mantra.gonewx.com");
  });

  it("copies logs when copy button is clicked", async () => {
    mockCopyToClipboard.mockResolvedValueOnce(true);
    render(<GeneralSettings />);

    fireEvent.click(screen.getByTestId("copy-logs-button"));
    await waitFor(() => {
      expect(mockCopyToClipboard).toHaveBeenCalled();
    });
  });
});
