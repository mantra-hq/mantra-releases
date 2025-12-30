/**
 * EmptyDashboard Tests - 空状态组件测试
 * Story 2.8: Task 5
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { EmptyDashboard } from "./EmptyDashboard";

describe("EmptyDashboard", () => {
  const mockOnImport = vi.fn();

  describe("UI 展示", () => {
    it("应该显示标题", () => {
      render(<EmptyDashboard onImport={mockOnImport} />);
      expect(screen.getByText(/开始|Get Started|欢迎/i)).toBeInTheDocument();
    });

    it("应该显示描述文本", () => {
      render(<EmptyDashboard onImport={mockOnImport} />);
      expect(
        screen.getByText(/导入.*会话日志|编程心法/i)
      ).toBeInTheDocument();
    });

    it("应该显示导入按钮", () => {
      render(<EmptyDashboard onImport={mockOnImport} />);
      expect(
        screen.getByRole("button", { name: /导入|import/i })
      ).toBeInTheDocument();
    });

    it("应该显示支持的格式说明", () => {
      render(<EmptyDashboard onImport={mockOnImport} />);
      // 应该提到 Claude, Gemini, 和 Cursor
      expect(screen.getByText(/Claude Code/i)).toBeInTheDocument();
      expect(screen.getByText(/Gemini CLI/i)).toBeInTheDocument();
      expect(screen.getByText(/Cursor/i)).toBeInTheDocument();
    });

    it("应该有装饰性图标", () => {
      render(<EmptyDashboard onImport={mockOnImport} />);
      const svg = document.querySelector("svg");
      expect(svg).toBeInTheDocument();
    });
  });

  describe("交互", () => {
    it("点击导入按钮应该调用 onImport", async () => {
      const user = userEvent.setup();
      render(<EmptyDashboard onImport={mockOnImport} />);

      const button = screen.getByRole("button", { name: /导入|import/i });
      await user.click(button);

      expect(mockOnImport).toHaveBeenCalledTimes(1);
    });
  });

  describe("样式", () => {
    it("应该居中显示", () => {
      render(<EmptyDashboard onImport={mockOnImport} />);
      const container = screen.getByTestId("empty-dashboard");
      expect(container).toHaveClass("flex");
      expect(container).toHaveClass("items-center");
      expect(container).toHaveClass("justify-center");
    });
  });
});

