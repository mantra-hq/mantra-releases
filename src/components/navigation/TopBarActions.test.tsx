/**
 * TopBarActions Tests - TopBar 操作按钮组件测试
 * Story 2.17: Task 4
 * Story 2.21: Task 4.5 (添加搜索、设置按钮测试)
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BrowserRouter } from "react-router-dom";
import { TopBarActions } from "./TopBarActions";

// Mock ThemeToggle
vi.mock("@/components/theme-toggle", () => ({
  ThemeToggle: () => <button data-testid="theme-toggle">Toggle Theme</button>,
}));

// Mock useSearchStore
const mockOpenSearch = vi.fn();
vi.mock("@/stores/useSearchStore", () => ({
  useSearchStore: (selector: (state: { open: () => void }) => unknown) =>
    selector({ open: mockOpenSearch }),
}));

// Mock useNavigate
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

const defaultProps = {
  onSync: vi.fn(),
  onImport: vi.fn(),
};

// Wrapper with Router
function renderWithRouter(ui: React.ReactElement) {
  return render(<BrowserRouter>{ui}</BrowserRouter>);
}

describe("TopBarActions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("UI 展示", () => {
    it("应该显示搜索按钮 (Story 2.21 AC #15)", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-search-button")).toBeInTheDocument();
    });

    it("应该显示同步按钮", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-sync-button")).toBeInTheDocument();
    });

    it("应该显示导入按钮", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-import-button")).toBeInTheDocument();
    });

    it("应该显示设置按钮 (Story 2.21 AC #16)", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-settings-button")).toBeInTheDocument();
    });

    it("应该显示主题切换按钮", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("theme-toggle")).toBeInTheDocument();
    });

    it("showSync=false 时不应该显示同步按钮", () => {
      renderWithRouter(<TopBarActions {...defaultProps} showSync={false} />);
      expect(screen.queryByTestId("topbar-sync-button")).not.toBeInTheDocument();
    });
  });

  describe("交互", () => {
    it("点击搜索按钮应该触发 openSearch", async () => {
      const user = userEvent.setup();
      renderWithRouter(<TopBarActions {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-search-button"));
      expect(mockOpenSearch).toHaveBeenCalledTimes(1);
    });

    it("点击同步按钮应该触发 onSync", async () => {
      const user = userEvent.setup();
      renderWithRouter(<TopBarActions {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-sync-button"));
      expect(defaultProps.onSync).toHaveBeenCalledTimes(1);
    });

    it("点击导入按钮应该触发 onImport", async () => {
      const user = userEvent.setup();
      renderWithRouter(<TopBarActions {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-import-button"));
      expect(defaultProps.onImport).toHaveBeenCalledTimes(1);
    });

    it("点击设置按钮应该导航到 /settings", async () => {
      const user = userEvent.setup();
      renderWithRouter(<TopBarActions {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-settings-button"));
      expect(mockNavigate).toHaveBeenCalledWith("/settings");
    });
  });

  describe("同步状态", () => {
    it("isSyncing=true 时同步按钮应该禁用", () => {
      renderWithRouter(<TopBarActions {...defaultProps} isSyncing />);
      expect(screen.getByTestId("topbar-sync-button")).toBeDisabled();
    });

    it("isSyncing=true 时同步图标应该有动画", () => {
      renderWithRouter(<TopBarActions {...defaultProps} isSyncing />);
      const button = screen.getByTestId("topbar-sync-button");
      const icon = button.querySelector("svg");
      expect(icon).toHaveClass("animate-spin");
    });

    it("isSyncing=false 时同步按钮应该启用", () => {
      renderWithRouter(<TopBarActions {...defaultProps} isSyncing={false} />);
      expect(screen.getByTestId("topbar-sync-button")).not.toBeDisabled();
    });
  });

  describe("无障碍", () => {
    it("搜索按钮应该有 aria-label", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-search-button")).toHaveAttribute(
        "aria-label",
        "全局搜索 (⌘K)"
      );
    });

    it("同步按钮应该有 aria-label", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-sync-button")).toHaveAttribute(
        "aria-label",
        "同步项目"
      );
    });

    it("导入按钮应该有 aria-label", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-import-button")).toHaveAttribute(
        "aria-label",
        "导入会话"
      );
    });

    it("设置按钮应该有 aria-label", () => {
      renderWithRouter(<TopBarActions {...defaultProps} />);
      expect(screen.getByTestId("topbar-settings-button")).toHaveAttribute(
        "aria-label",
        "设置"
      );
    });
  });
});
