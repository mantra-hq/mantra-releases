/**
 * Dashboard Page Tests - Dashboard 页面集成测试
 * Story 2.8: Task 9
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import Dashboard from "./Dashboard";
import type { Project } from "@/types/project";

// Mock useProjects hook
const mockRefetch = vi.fn();
vi.mock("@/hooks", async () => {
  const actual = await vi.importActual("@/hooks");
  return {
    ...actual,
    useProjects: vi.fn(() => ({
      projects: [],
      isLoading: false,
      error: null,
      refetch: mockRefetch,
    })),
  };
});

// Mock Tauri invoke (for useProjects and ProjectCard internal use)
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve([])),
}));

// Mock ThemeToggle
vi.mock("@/components/theme-toggle", () => ({
  ThemeToggle: () => <button data-testid="theme-toggle">Toggle Theme</button>,
}));

// Mock date-fns
vi.mock("date-fns", () => ({
  formatDistanceToNow: vi.fn(() => "刚刚"),
}));

// Import after mocking
import { useProjects } from "@/hooks";
const mockUseProjects = vi.mocked(useProjects);

// Test data (Rust Project format - snake_case, ISO dates)
const mockProjects: Project[] = [
  {
    id: "project-1",
    name: "my-awesome-project",
    cwd: "/home/user/projects/my-awesome-project",
    session_count: 2,
    created_at: new Date(Date.now() - 86400000).toISOString(),
    last_activity: new Date(Date.now() - 1800000).toISOString(),
  },
  {
    id: "project-2",
    name: "another-project",
    cwd: "/home/user/projects/another-project",
    session_count: 0,
    created_at: new Date(Date.now() - 172800000).toISOString(),
    last_activity: new Date(Date.now() - 86400000).toISOString(),
  },
];

// Wrapper with Router - returns render result including unmount
function renderWithRouter(ui: React.ReactElement) {
  return render(<MemoryRouter>{ui}</MemoryRouter>);
}

describe("Dashboard Page", () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("AC1 - 项目列表展示", () => {
    it("应该显示项目列表", async () => {
      mockUseProjects.mockReturnValue({
        projects: mockProjects,
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      await waitFor(() => {
        expect(screen.getByText("my-awesome-project")).toBeInTheDocument();
        expect(screen.getByText("another-project")).toBeInTheDocument();
      });
    });

    it("加载时应该显示骨架屏", () => {
      mockUseProjects.mockReturnValue({
        projects: [],
        isLoading: true,
        error: null,
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      // 骨架屏有多个 Skeleton 元素
      const skeletons = document.querySelectorAll(".animate-pulse");
      expect(skeletons.length).toBeGreaterThan(0);
    });
  });

  describe("AC2 - 项目卡片信息", () => {
    it("应该显示项目名称和会话数量", async () => {
      mockUseProjects.mockReturnValue({
        projects: mockProjects,
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      await waitFor(() => {
        expect(screen.getByText("my-awesome-project")).toBeInTheDocument();
        expect(screen.getByText("2 会话")).toBeInTheDocument();
        expect(screen.getByText("0 会话")).toBeInTheDocument();
      });
    });
  });

  describe("AC3 - 会话列表展开", () => {
    it("点击项目应该触发展开", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      mockUseProjects.mockReturnValue({
        projects: mockProjects,
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      // 点击项目
      const projectButton = screen.getByRole("button", {
        name: /my-awesome-project/i,
      });
      await user.click(projectButton);

      // 展开后 collapsible content 应该可见
      // 由于会话是按需加载的，这里检查加载状态或空状态
      await waitFor(() => {
        // 展开后会显示加载中或会话内容区域
        const expandedContent = document.querySelector(
          '[data-state="open"]'
        );
        expect(expandedContent).toBeInTheDocument();
      });
    });
  });

  describe("AC5 - 空状态引导", () => {
    it("无项目时应该显示空状态引导", async () => {
      mockUseProjects.mockReturnValue({
        projects: [],
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      await waitFor(() => {
        expect(screen.getByText("开始使用 Mantra")).toBeInTheDocument();
        expect(
          screen.getByRole("button", { name: /导入日志/i })
        ).toBeInTheDocument();
      });
    });
  });

  describe("AC6 - 搜索与筛选", () => {
    it("搜索应该过滤项目列表", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      mockUseProjects.mockReturnValue({
        projects: mockProjects,
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      const { unmount } = renderWithRouter(<Dashboard />);

      // 输入搜索词
      const searchInput = screen.getByPlaceholderText(/搜索.*项目/i);
      await user.type(searchInput, "awesome");

      // 等待防抖
      await vi.advanceTimersByTimeAsync(300);

      // 应该只显示匹配的项目
      await waitFor(() => {
        expect(screen.getByText("my-awesome-project")).toBeInTheDocument();
        expect(screen.queryByText("another-project")).not.toBeInTheDocument();
      });

      // 清理以避免 act() 警告
      unmount();
    });

    it("搜索无结果时应该显示提示", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      mockUseProjects.mockReturnValue({
        projects: mockProjects,
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      const { unmount } = renderWithRouter(<Dashboard />);

      const searchInput = screen.getByPlaceholderText(/搜索.*项目/i);
      await user.type(searchInput, "nonexistent");

      await vi.advanceTimersByTimeAsync(300);

      await waitFor(() => {
        expect(screen.getByText(/没有找到/i)).toBeInTheDocument();
      });

      // 清理以避免 act() 警告
      unmount();
    });
  });

  describe("错误处理", () => {
    it("加载失败时应该显示错误信息", () => {
      mockUseProjects.mockReturnValue({
        projects: [],
        isLoading: false,
        error: "网络错误",
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      expect(screen.getByText("网络错误")).toBeInTheDocument();
      expect(screen.getByText("重试")).toBeInTheDocument();
    });

    it("点击重试应该重新加载", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      mockUseProjects.mockReturnValue({
        projects: [],
        isLoading: false,
        error: "网络错误",
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      const retryButton = screen.getByText("重试");
      await user.click(retryButton);

      expect(mockRefetch).toHaveBeenCalledTimes(1);
    });
  });

  describe("Header 功能", () => {
    it("应该显示 Header 组件", () => {
      mockUseProjects.mockReturnValue({
        projects: mockProjects,
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      expect(screen.getByText("Mantra")).toBeInTheDocument();
      expect(screen.getByText("心法")).toBeInTheDocument();
      expect(screen.getByTestId("theme-toggle")).toBeInTheDocument();
    });

    it("应该有导入按钮", () => {
      mockUseProjects.mockReturnValue({
        projects: mockProjects,
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      // Header 中的导入按钮
      expect(screen.getByRole("button", { name: /导入/i })).toBeInTheDocument();
    });
  });
});
