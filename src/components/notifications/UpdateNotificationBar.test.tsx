/**
 * UpdateNotificationBar Tests
 * Story 14.6: AC #8
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { UpdateNotificationBar } from "./UpdateNotificationBar";
import type { UpdateStatus, UpdateInfo } from "@/hooks/useUpdateChecker";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, string>) => {
      const translations: Record<string, string> = {
        "updater.readyToInstall": `Version ${params?.version ?? ""} is ready to install`,
        "updater.restartToUpdate": "Restart to Update",
        "updater.releaseNotes": "Release Notes",
        "updater.dismiss": "Dismiss",
      };
      return translations[key] || key;
    },
  }),
}));

describe("UpdateNotificationBar", () => {
  const defaultProps = {
    updateStatus: "ready" as UpdateStatus,
    updateInfo: {
      version: "1.2.0",
      body: "Bug fixes and improvements",
    } as UpdateInfo,
    onRestart: vi.fn().mockResolvedValue(undefined),
    onDismiss: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("条件渲染 (AC #3)", () => {
    it("updateStatus='idle' 时不渲染", () => {
      render(<UpdateNotificationBar {...defaultProps} updateStatus="idle" />);
      expect(screen.queryByTestId("update-notification-bar")).not.toBeInTheDocument();
    });

    it("updateStatus='checking' 时不渲染", () => {
      render(<UpdateNotificationBar {...defaultProps} updateStatus="checking" />);
      expect(screen.queryByTestId("update-notification-bar")).not.toBeInTheDocument();
    });

    it("updateStatus='downloading' 时不渲染", () => {
      render(<UpdateNotificationBar {...defaultProps} updateStatus="downloading" />);
      expect(screen.queryByTestId("update-notification-bar")).not.toBeInTheDocument();
    });

    it("updateStatus='error' 时不渲染", () => {
      render(<UpdateNotificationBar {...defaultProps} updateStatus="error" />);
      expect(screen.queryByTestId("update-notification-bar")).not.toBeInTheDocument();
    });

    it("updateStatus='ready' 时正确渲染通知条", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      expect(screen.getByTestId("update-notification-bar")).toBeInTheDocument();
    });
  });

  describe("版本号显示 (i18n 插值)", () => {
    it("应显示正确的版本号", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      expect(screen.getByText("Version 1.2.0 is ready to install")).toBeInTheDocument();
    });

    it("updateInfo 为 null 时版本号为空", () => {
      render(<UpdateNotificationBar {...defaultProps} updateInfo={null} />);
      expect(screen.getByText(/Version\s+is ready to install/)).toBeInTheDocument();
    });
  });

  describe("按钮交互 (AC #3)", () => {
    it("点击 [重启更新] 按钮调用 onRestart", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      fireEvent.click(screen.getByTestId("update-restart-btn"));
      expect(defaultProps.onRestart).toHaveBeenCalledTimes(1);
    });

    it("onRestart 失败时不抛出 unhandled rejection", async () => {
      const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
      const failingRestart = vi.fn().mockRejectedValue(new Error("relaunch failed"));
      render(<UpdateNotificationBar {...defaultProps} onRestart={failingRestart} />);

      fireEvent.click(screen.getByTestId("update-restart-btn"));
      // Wait for the async handler to settle
      await vi.waitFor(() => {
        expect(consoleError).toHaveBeenCalledWith(
          "[UpdateNotificationBar] restart failed:",
          expect.any(Error)
        );
      });

      consoleError.mockRestore();
    });

    it("点击关闭按钮触发退出动画后调用 onDismiss", () => {
      vi.useFakeTimers();
      render(<UpdateNotificationBar {...defaultProps} />);
      fireEvent.click(screen.getByTestId("update-dismiss-btn"));

      // Before timeout: exit animation applied but onDismiss not yet called
      expect(defaultProps.onDismiss).not.toHaveBeenCalled();

      // After timeout: onDismiss called
      vi.advanceTimersByTime(200);
      expect(defaultProps.onDismiss).toHaveBeenCalledTimes(1);
      vi.useRealTimers();
    });
  });

  describe("更新日志展开/折叠 (AC #3)", () => {
    it("点击 [更新日志] 按钮展开 release notes", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      expect(screen.queryByTestId("update-release-notes-content")).not.toBeInTheDocument();

      fireEvent.click(screen.getByTestId("update-release-notes-btn"));
      expect(screen.getByTestId("update-release-notes-content")).toBeInTheDocument();
      expect(screen.getByText("Bug fixes and improvements")).toBeInTheDocument();
    });

    it("再次点击 [更新日志] 按钮折叠 release notes", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      const btn = screen.getByTestId("update-release-notes-btn");

      // Open
      fireEvent.click(btn);
      expect(screen.getByTestId("update-release-notes-content")).toBeInTheDocument();

      // Close
      fireEvent.click(btn);
      expect(screen.queryByTestId("update-release-notes-content")).not.toBeInTheDocument();
    });

    it("updateInfo.body 为空时不显示 [更新日志] 按钮", () => {
      render(
        <UpdateNotificationBar
          {...defaultProps}
          updateInfo={{ version: "1.2.0" }}
        />
      );
      expect(screen.queryByTestId("update-release-notes-btn")).not.toBeInTheDocument();
    });
  });

  describe("动画类和 data-state (AC #4)", () => {
    it("默认有入场动画类", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      const bar = screen.getByTestId("update-notification-bar");
      expect(bar).toHaveClass("animate-in");
      expect(bar).toHaveClass("slide-in-from-top");
      expect(bar).toHaveAttribute("data-state", "open");
    });

    it("点击关闭后有退出动画类", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      fireEvent.click(screen.getByTestId("update-dismiss-btn"));

      const bar = screen.getByTestId("update-notification-bar");
      expect(bar).toHaveClass("opacity-0");
      expect(bar).toHaveClass("scale-95");
      expect(bar).toHaveAttribute("data-state", "closed");
    });
  });

  describe("无障碍属性 (AC #8)", () => {
    it("应有 role='status'（非侵入式通知）", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      expect(screen.getByRole("status")).toBeInTheDocument();
    });

    it("应有正确的 aria-label", () => {
      render(<UpdateNotificationBar {...defaultProps} />);
      expect(screen.getByRole("status")).toHaveAttribute(
        "aria-label",
        "Version 1.2.0 is ready to install"
      );
    });
  });
});
