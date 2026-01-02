/**
 * TopBarActions Tests - TopBar 操作按钮组件测试
 * Story 2.17: Task 4
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TopBarActions } from "./TopBarActions";

// Mock ThemeToggle
vi.mock("@/components/theme-toggle", () => ({
  ThemeToggle: () => <button data-testid="theme-toggle">Toggle Theme</button>,
}));

const defaultProps = {
  onSync: vi.fn(),
  onImport: vi.fn(),
};

describe("TopBarActions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("UI 展示", () => {
    it("应该显示同步按钮", () => {
      render(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-sync-button")).toBeInTheDocument();
    });

    it("应该显示导入按钮", () => {
      render(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-import-button")).toBeInTheDocument();
    });

    it("应该显示主题切换按钮", () => {
      render(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("theme-toggle")).toBeInTheDocument();
    });
  });

  describe("交互", () => {
    it("点击同步按钮应该触发 onSync", async () => {
      const user = userEvent.setup();
      render(<TopBarActions {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-sync-button"));
      expect(defaultProps.onSync).toHaveBeenCalledTimes(1);
    });

    it("点击导入按钮应该触发 onImport", async () => {
      const user = userEvent.setup();
      render(<TopBarActions {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-import-button"));
      expect(defaultProps.onImport).toHaveBeenCalledTimes(1);
    });
  });

  describe("同步状态", () => {
    it("isSyncing=true 时同步按钮应该禁用", () => {
      render(<TopBarActions {...defaultProps} isSyncing />);
      expect(screen.getByTestId("topbar-sync-button")).toBeDisabled();
    });

    it("isSyncing=true 时同步图标应该有动画", () => {
      render(<TopBarActions {...defaultProps} isSyncing />);
      const button = screen.getByTestId("topbar-sync-button");
      const icon = button.querySelector("svg");
      expect(icon).toHaveClass("animate-spin");
    });

    it("isSyncing=false 时同步按钮应该启用", () => {
      render(<TopBarActions {...defaultProps} isSyncing={false} />);
      expect(screen.getByTestId("topbar-sync-button")).not.toBeDisabled();
    });
  });

  describe("无障碍", () => {
    it("同步按钮应该有 aria-label", () => {
      render(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-sync-button")).toHaveAttribute(
        "aria-label",
        "同步项目"
      );
    });

    it("导入按钮应该有 aria-label", () => {
      render(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-import-button")).toHaveAttribute(
        "aria-label",
        "导入会话"
      );
    });
  });
});
