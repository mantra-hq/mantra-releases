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

// Mock Tauri invoke (for useProjects internal use)
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

// Test data
const mockProjects: Project[] = [
  {
    id: "project-1",
    name: "my-awesome-project",
    path: "/home/user/projects/my-awesome-project",
    sessions: [
      {
        id: "session-1",
        title: "实现用户认证功能",
        source: "claude",
        messageCount: 42,
        startTime: Date.now() - 3600000,
        endTime: Date.now() - 1800000,
      },
      {
        id: "session-2",
        title: "修复登录 Bug",
        source: "gemini",
        messageCount: 15,
        startTime: Date.now() - 7200000,
        endTime: Date.now() - 5400000,
      },
    ],
    lastActivity: Date.now() - 1800000,
  },
  {
    id: "project-2",
    name: "another-project",
    path: "/home/user/projects/another-project",
    sessions: [],
    lastActivity: Date.now() - 86400000,
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
    it("点击项目应该展开会话列表", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      mockUseProjects.mockReturnValue({
        projects: mockProjects,
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      });

      renderWithRouter(<Dashboard />);

      // 初始状态不显示会话
      expect(screen.queryByText("实现用户认证功能")).not.toBeInTheDocument();

      // 点击项目
      const projectButton = screen.getByRole("button", {
        name: /my-awesome-project/i,
      });
      await user.click(projectButton);

      // 应该显示会话列表
      await waitFor(() => {
        expect(screen.getByText("实现用户认证功能")).toBeInTheDocument();
        expect(screen.getByText("修复登录 Bug")).toBeInTheDocument();
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
        expect(screen.getByText(/未找到匹配的项目/i)).toBeInTheDocument();
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

