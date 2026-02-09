/**
 * GeneralSettings Tests - 通用设置页面测试
 * Story 2-35: Task 3.1
 * Story 14.7: Task 4 - 关于与更新区域测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { GeneralSettings } from "./GeneralSettings";
import type { UseUpdateCheckerResult } from "@/hooks/useUpdateChecker";

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

// Mock useUpdateCheckerContext (replaces direct useUpdateChecker mock)
const mockCheckForUpdate = vi.fn().mockResolvedValue(undefined);
const mockRestartToUpdate = vi.fn().mockResolvedValue(undefined);
const mockDismissUpdate = vi.fn();
const mockSetAutoUpdateEnabled = vi.fn();

const defaultUpdateCheckerState: UseUpdateCheckerResult = {
  updateAvailable: false,
  updateInfo: null,
  downloadProgress: 0,
  updateStatus: "idle",
  errorMessage: null,
  autoUpdateEnabled: true,
  checkForUpdate: mockCheckForUpdate,
  downloadAndInstall: vi.fn(),
  restartToUpdate: mockRestartToUpdate,
  dismissUpdate: mockDismissUpdate,
  setAutoUpdateEnabled: mockSetAutoUpdateEnabled,
};

let mockUpdateCheckerReturn: UseUpdateCheckerResult = { ...defaultUpdateCheckerState };

vi.mock("@/contexts/UpdateCheckerContext", () => ({
  useUpdateCheckerContext: () => mockUpdateCheckerReturn,
}));

// Mock Progress component
vi.mock("@/components/ui/progress", () => ({
  Progress: ({ value, className }: { value: number; className?: string }) => (
    <div data-testid="progress-bar" data-value={value} className={className} role="progressbar" />
  ),
}));

/**
 * Helper: render and flush the async getVersion() microtask
 * to eliminate act() warnings from the useEffect state update.
 */
async function renderSettled() {
  await act(async () => {
    render(<GeneralSettings />);
  });
}

describe("GeneralSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUpdateCheckerReturn = { ...defaultUpdateCheckerState };
  });

  // --- Existing tests ---
  it("renders LanguageSwitcher component", async () => {
    await renderSettled();
    expect(screen.getByTestId("language-switcher")).toBeInTheDocument();
  });

  it("renders help section with title", async () => {
    await renderSettled();
    expect(screen.getByText("帮助")).toBeInTheDocument();
  });

  it("renders official website button", async () => {
    await renderSettled();
    expect(screen.getByTestId("official-website-button")).toBeInTheDocument();
  });

  it("renders documentation button", async () => {
    await renderSettled();
    expect(screen.getByTestId("documentation-button")).toBeInTheDocument();
  });

  it("renders copy logs button", async () => {
    await renderSettled();
    expect(screen.getByTestId("copy-logs-button")).toBeInTheDocument();
  });

  it("calls openUrl when official website button is clicked", async () => {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await renderSettled();

    fireEvent.click(screen.getByTestId("official-website-button"));
    expect(openUrl).toHaveBeenCalledWith("https://mantra.gonewx.com");
  });

  it("calls openUrl when documentation button is clicked", async () => {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await renderSettled();

    fireEvent.click(screen.getByTestId("documentation-button"));
    expect(openUrl).toHaveBeenCalledWith("https://docs.mantra.gonewx.com");
  });

  it("copies logs when copy button is clicked", async () => {
    mockCopyToClipboard.mockResolvedValueOnce(true);
    await renderSettled();

    fireEvent.click(screen.getByTestId("copy-logs-button"));
    await waitFor(() => {
      expect(mockCopyToClipboard).toHaveBeenCalled();
    });
  });

  // --- About Mantra section tests (Story 14.7) ---
  describe("About Mantra section", () => {
    it("renders the about section with title", async () => {
      await renderSettled();
      expect(screen.getByTestId("about-mantra-section")).toBeInTheDocument();
      expect(screen.getByText("关于 Mantra")).toBeInTheDocument();
    });

    it("displays the current version number", async () => {
      await renderSettled();
      expect(screen.getByTestId("app-version")).toHaveTextContent("v0.7.1");
    });

    it("renders check for updates button", async () => {
      await renderSettled();
      expect(screen.getByTestId("check-update-button")).toBeInTheDocument();
      expect(screen.getByText("检查更新")).toBeInTheDocument();
    });

    it("calls checkForUpdate when button is clicked", async () => {
      await renderSettled();

      await act(async () => {
        fireEvent.click(screen.getByTestId("check-update-button"));
      });
      expect(mockCheckForUpdate).toHaveBeenCalled();
    });

    it("shows disabled button with spinner when checking", async () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "checking",
      };
      await renderSettled();

      const button = screen.getByTestId("check-update-button");
      expect(button).toBeDisabled();
      expect(screen.getByText("正在检查更新...")).toBeInTheDocument();
    });

    it("shows up-to-date status after check completes with no update", async () => {
      await renderSettled();

      // Click check → triggers hasChecked = true
      await act(async () => {
        fireEvent.click(screen.getByTestId("check-update-button"));
      });

      await waitFor(() => {
        expect(screen.getByTestId("up-to-date-status")).toBeInTheDocument();
        expect(screen.getByText("已是最新版本")).toBeInTheDocument();
      });
    });

    it("shows update available status", async () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateAvailable: true,
        updateInfo: { version: "0.8.0" },
      };
      await renderSettled();

      expect(screen.getByTestId("update-available-status")).toBeInTheDocument();
      expect(screen.getByText("新版本 0.8.0 可用")).toBeInTheDocument();
    });

    it("shows downloading status with progress bar", async () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "downloading",
        downloadProgress: 65,
      };
      await renderSettled();

      expect(screen.getByTestId("downloading-status")).toBeInTheDocument();
      expect(screen.getByTestId("progress-bar")).toHaveAttribute("data-value", "65");
      expect(screen.getByText("下载中... 65%")).toBeInTheDocument();
      expect(screen.getByTestId("check-update-button")).toBeDisabled();
    });

    it("shows ready status with restart button", async () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "ready",
        updateInfo: { version: "0.8.0" },
      };
      await renderSettled();

      expect(screen.getByTestId("ready-status")).toBeInTheDocument();
      expect(screen.getByTestId("restart-to-update-button")).toBeInTheDocument();
      expect(screen.getByText("重启以更新")).toBeInTheDocument();
      expect(screen.getByTestId("check-update-button")).toBeDisabled();
    });

    it("calls restartToUpdate when restart button is clicked", async () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "ready",
        updateInfo: { version: "0.8.0" },
      };
      await renderSettled();

      fireEvent.click(screen.getByTestId("restart-to-update-button"));
      expect(mockRestartToUpdate).toHaveBeenCalled();
    });

    it("shows error status", async () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "error",
        errorMessage: "Network error",
      };
      await renderSettled();

      expect(screen.getByTestId("error-status")).toBeInTheDocument();
      expect(screen.getByText("检查更新失败，请稍后重试")).toBeInTheDocument();
    });

    // --- Story 14.10: 自动更新开关测试 ---

    it("renders auto-update switch checked by default", async () => {
      await renderSettled();
      const switchEl = screen.getByTestId("auto-update-switch");
      expect(switchEl).toBeInTheDocument();
      expect(switchEl).toHaveAttribute("data-state", "checked");
    });

    it("renders auto-update switch unchecked when disabled", async () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        autoUpdateEnabled: false,
      };
      await renderSettled();
      const switchEl = screen.getByTestId("auto-update-switch");
      expect(switchEl).toHaveAttribute("data-state", "unchecked");
    });

    it("calls setAutoUpdateEnabled when switch is toggled", async () => {
      await renderSettled();
      const switchEl = screen.getByTestId("auto-update-switch");
      fireEvent.click(switchEl);
      expect(mockSetAutoUpdateEnabled).toHaveBeenCalledWith(false);
    });

    // --- Story 14.10: 查看更新日志按钮测试 ---

    it("shows changelog button in ready status", async () => {
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "ready",
        updateInfo: { version: "0.8.0" },
      };
      await renderSettled();
      expect(screen.getByTestId("view-changelog-button")).toBeInTheDocument();
    });

    it("opens changelog URL when changelog button is clicked", async () => {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      mockUpdateCheckerReturn = {
        ...defaultUpdateCheckerState,
        updateStatus: "ready",
        updateInfo: { version: "0.8.0" },
      };
      await renderSettled();

      fireEvent.click(screen.getByTestId("view-changelog-button"));
      expect(openUrl).toHaveBeenCalledWith(
        "https://github.com/mantra-hq/mantra-releases/blob/main/CHANGELOG.md"
      );
    });

    it("does not show changelog button in idle status", async () => {
      await renderSettled();
      expect(screen.queryByTestId("view-changelog-button")).not.toBeInTheDocument();
    });
  });
});
