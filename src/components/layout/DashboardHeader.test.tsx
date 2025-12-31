/**
 * DashboardHeader Tests - Dashboard 头部组件测试
 * Story 2.8: Task 7
 * Story 3-3: Task 5.4 (Settings Entry - Updated)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { DashboardHeader } from "./DashboardHeader";

// Mock ThemeToggle
vi.mock("@/components/theme-toggle", () => ({
  ThemeToggle: () => <button data-testid="theme-toggle">Toggle Theme</button>,
}));

// 测试包装器
const renderWithRouter = (ui: React.ReactElement) => {
  return render(<MemoryRouter>{ui}</MemoryRouter>);
};


describe("DashboardHeader", () => {
  const mockOnSearch = vi.fn();
  const mockOnImport = vi.fn();

  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("UI 展示", () => {
    it("应该显示应用标题 Mantra", () => {
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );
      expect(screen.getByText("Mantra")).toBeInTheDocument();
    });

    it("应该显示中文副标题 心法", () => {
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );
      expect(screen.getByText("心法")).toBeInTheDocument();
    });

    it("应该包含搜索框", () => {
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );
      expect(screen.getByPlaceholderText(/搜索.*项目/i)).toBeInTheDocument();
    });

    it("应该包含主题切换按钮", () => {
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );
      expect(screen.getByTestId("theme-toggle")).toBeInTheDocument();
    });

    it("应该包含导入按钮", () => {
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );
      expect(
        screen.getByRole("button", { name: /导入|import/i })
      ).toBeInTheDocument();
    });

    it("应该包含设置按钮", () => {
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );
      expect(screen.getByTestId("settings-button")).toBeInTheDocument();
    });
  });

  describe("交互", () => {
    it("搜索输入应该触发 onSearch", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const { unmount } = renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );

      const input = screen.getByPlaceholderText(/搜索.*项目/i);
      await user.type(input, "test");

      await vi.advanceTimersByTimeAsync(300);
      await waitFor(() => {
        expect(mockOnSearch).toHaveBeenCalledWith("test");
      });

      // 清理以避免 act() 警告
      unmount();
    });

    it("点击导入按钮应该触发 onImport", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );

      const button = screen.getByRole("button", { name: /导入|import/i });
      await user.click(button);

      expect(mockOnImport).toHaveBeenCalledTimes(1);
    });
  });

  describe("样式", () => {
    it("应该是 sticky 定位", () => {
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );
      const header = screen.getByTestId("dashboard-header");
      expect(header).toHaveClass("sticky");
    });

    it("应该有边框", () => {
      renderWithRouter(
        <DashboardHeader onSearch={mockOnSearch} onImport={mockOnImport} />
      );
      const header = screen.getByTestId("dashboard-header");
      expect(header).toHaveClass("border-b");
    });
  });
});


