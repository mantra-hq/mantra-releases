/**
 * ProjectSearch Tests - 项目搜索组件测试
 * Story 2.8: Task 6
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ProjectSearch } from "./ProjectSearch";

describe("ProjectSearch", () => {
  const mockOnSearch = vi.fn();

  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("UI 展示", () => {
    it("应该渲染搜索输入框", () => {
      render(<ProjectSearch onSearch={mockOnSearch} />);
      expect(
        screen.getByPlaceholderText(/搜索.*项目|search.*project/i)
      ).toBeInTheDocument();
    });

    it("应该显示搜索图标", () => {
      render(<ProjectSearch onSearch={mockOnSearch} />);
      const svg = document.querySelector("svg");
      expect(svg).toBeInTheDocument();
    });

    it("应该支持自定义 placeholder", () => {
      render(
        <ProjectSearch onSearch={mockOnSearch} placeholder="自定义占位符" />
      );
      expect(screen.getByPlaceholderText("自定义占位符")).toBeInTheDocument();
    });
  });

  describe("搜索功能", () => {
    it("输入文本应该触发搜索", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const { unmount } = render(<ProjectSearch onSearch={mockOnSearch} />);

      const input = screen.getByPlaceholderText(/搜索.*项目/i);
      await user.type(input, "test");

      // 等待防抖
      await vi.advanceTimersByTimeAsync(300);
      await waitFor(() => {
        expect(mockOnSearch).toHaveBeenCalledWith("test");
      });

      // 清理以避免 act() 警告
      unmount();
    });

    it("应该防抖搜索请求", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const { unmount } = render(<ProjectSearch onSearch={mockOnSearch} />);

      const input = screen.getByPlaceholderText(/搜索.*项目/i);

      // 快速输入多个字符
      await user.type(input, "abc");

      // 在防抖期间内不应该调用 (因为 debounce + 首次挂载跳过)
      expect(mockOnSearch).not.toHaveBeenCalled();

      // 等待防抖完成
      await vi.advanceTimersByTimeAsync(300);
      await waitFor(() => {
        // 应该只调用一次，传入最终值
        expect(mockOnSearch).toHaveBeenCalledTimes(1);
        expect(mockOnSearch).toHaveBeenCalledWith("abc");
      });

      // 清理以避免 act() 警告
      unmount();
    });

    it("清空输入应该触发空字符串搜索", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const { unmount } = render(<ProjectSearch onSearch={mockOnSearch} />);

      const input = screen.getByPlaceholderText(/搜索.*项目/i);
      await user.type(input, "test");
      await vi.advanceTimersByTimeAsync(300);

      // 清空输入
      await user.clear(input);
      await vi.advanceTimersByTimeAsync(300);

      await waitFor(() => {
        expect(mockOnSearch).toHaveBeenLastCalledWith("");
      });

      // 清理以避免 act() 警告
      unmount();
    });
  });

  describe("样式", () => {
    it("应该有正确的容器样式", () => {
      render(<ProjectSearch onSearch={mockOnSearch} />);
      const container = screen.getByTestId("project-search");
      expect(container).toHaveClass("relative");
    });
  });
});

