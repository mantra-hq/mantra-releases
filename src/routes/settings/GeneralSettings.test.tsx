/**
 * GeneralSettings Tests - 通用设置页面测试
 * Story 2-35: Task 3.1
 * Story 14.7: Task 4 - 关于与更新区域测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { GeneralSettings } from "./GeneralSettings";

// Mock @tauri-apps/plugin-opener
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

// Mock @tauri-apps/api/app
vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn().mockResolvedValue("0.7.1"),
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

// Mock useUpdateChecker
const mockCheckForUpdate = vi.fn().mockResolvedValue(undefined);
const mockRestartToUpdate = vi.fn().mockResolvedValue(undefined);
const mockDismissUpdate = vi.fn();

const defaultUpdateCheckerState = {
  updateAvailable: false,
  updateInfo: null,
  downloadProgress: 0,
  updateStatus: "idle" as const,
  errorMessage: null,
  checkForUpdate: mockCheckForUpdate,
  downloadAndInstall: vi.fn(),
  restartToUpdate: mockRestartToUpdate,
  dismissUpdate: mockDismissUpdate,
};

let mockUpdateCheckerReturn = { ...defaultUpdateCheckerState };

vi.mock("@/hooks", () => ({
  useUpdateChecker: () => mockUpdateCheckerReturn,
}));

// Mock Progress component
vi.mock("@/components/ui/progress", () => ({
  Progress: ({ value, className }: { value: number; className?: string }) => (
    <div data-testid="progress-bar" data-value={value} className={className} role="progressbar" />
  ),
}));

describe("GeneralSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUpdateCheckerReturn = { ...defaultUpdateCheckerState };
  });

  // --- Existing tests ---
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

  // --- About Mantra section tests (Story 14.7) ---
  describe("About Mantra section", () => {
    it("renders the about section with title", () => {
      render(<GeneralSettings />);
      expect(screen.getByTestId("about-mantra-section")).toBeInTheDocument();
      expect(screen.getByText("关于 Mantra")).toBeInTheDocument();
    });

    it("displays the current version number", async () => {
      render(<GeneralSettings />);
      await waitFor(() => {
        expect(screen.getByTestId("app-version")).toHaveTextContent("v0.7.1");
      });
    });

    it("renders check for updates button", () => {
      render(<GeneralSettings />);
      expect(screen.getByTestId("check-update-button")).toBeInTheDocument();
      expect(screen.getByText("检查更新")).toBeInTheDocument();
    });

    it("calls checkForUpdate when button is clicked", async () => {
      render(<GeneralSettings />);

      fireEvent.click(screen.getByTestId("check-update-button"));
      await waitFor(() => {
        expect(mockCheckForUpdate).toHaveBeenCalled();
      });
    });

    it("shows disabled button with spinner when checking", () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "checking",
      };
      render(<GeneralSettings />);

      const button = screen.getByTestId("check-update-button");
      expect(button).toBeDisabled();
      expect(screen.getByText("正在检查更新...")).toBeInTheDocument();
    });

    it("shows up-to-date status after check completes with no update", async () => {
      render(<GeneralSettings />);

      // Click check → triggers hasChecked = true
      fireEvent.click(screen.getByTestId("check-update-button"));

      await waitFor(() => {
        expect(screen.getByTestId("up-to-date-status")).toBeInTheDocument();
        expect(screen.getByText("已是最新版本")).toBeInTheDocument();
      });
    });

    it("shows update available status", () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateAvailable: true,
        updateInfo: { version: "0.8.0", date: null, body: null },
      };
      render(<GeneralSettings />);

      expect(screen.getByTestId("update-available-status")).toBeInTheDocument();
      expect(screen.getByText("新版本 0.8.0 可用")).toBeInTheDocument();
    });

    it("shows downloading status with progress bar", () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "downloading",
        downloadProgress: 65,
      };
      render(<GeneralSettings />);

      expect(screen.getByTestId("downloading-status")).toBeInTheDocument();
      expect(screen.getByTestId("progress-bar")).toBeInTheDocument();
      expect(screen.getByText("下载中... 65%")).toBeInTheDocument();
    });

    it("shows ready status with restart button", () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "ready",
        updateInfo: { version: "0.8.0", date: null, body: null },
      };
      render(<GeneralSettings />);

      expect(screen.getByTestId("ready-status")).toBeInTheDocument();
      expect(screen.getByTestId("restart-to-update-button")).toBeInTheDocument();
      expect(screen.getByText("重启以更新")).toBeInTheDocument();
    });

    it("calls restartToUpdate when restart button is clicked", () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "ready",
        updateInfo: { version: "0.8.0", date: null, body: null },
      };
      render(<GeneralSettings />);

      fireEvent.click(screen.getByTestId("restart-to-update-button"));
      expect(mockRestartToUpdate).toHaveBeenCalled();
    });

    it("shows error status", () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "error",
        errorMessage: "Network error",
      };
      render(<GeneralSettings />);

      expect(screen.getByTestId("error-status")).toBeInTheDocument();
      expect(screen.getByText("检查更新失败，请稍后重试")).toBeInTheDocument();
    });
  });
});
