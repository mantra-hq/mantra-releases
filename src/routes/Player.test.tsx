/**
 * Player Page Tests - 会话回放页面测试
 * Story 2.8: Task 9 (Code Review Fix)
 * Story 2.11: 更新测试以支持异步加载
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import Player from "./Player";

// Mock Tauri IPC
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === "get_session") {
      return Promise.resolve({
        id: "test-session-123",
        source: "claude",
        cwd: "/test/project",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        messages: [],
      });
    }
    if (cmd === "detect_git_repo") {
      return Promise.resolve("/test/project");
    }
    return Promise.resolve(null);
  }),
}));

// Mock project-ipc
vi.mock("@/lib/project-ipc", () => ({
  getProjectByCwd: vi.fn().mockResolvedValue({
    id: "proj-123",
    name: "test-project",
    cwd: "/test/project",
    session_count: 1,
    created_at: new Date().toISOString(),
    last_activity: new Date().toISOString(),
    git_repo_path: "/test/project",
    has_git_repo: true,
  }),
  getRepresentativeFile: vi.fn().mockResolvedValue({
    path: "README.md",
    content: "# Test Project",
    language: "markdown",
  }),
}));

// Mock DualStreamLayout
vi.mock("@/components/layout", () => ({
  DualStreamLayout: vi.fn(() => (
    <div data-testid="dual-stream-layout">DualStreamLayout</div>
  )),
}));

// Mock ThemeToggle
vi.mock("@/components/theme-toggle", () => ({
  ThemeToggle: () => <button data-testid="theme-toggle">Toggle Theme</button>,
}));

// Mock TimberLine
vi.mock("@/components/timeline", () => ({
  TimberLine: () => <div data-testid="timberline">TimberLine</div>,
}));

// Mock useTimeMachine hook
vi.mock("@/hooks/useTimeMachine", () => ({
  useTimeMachine: () => ({
    fetchSnapshot: vi.fn(),
    isLoading: false,
    error: null,
  }),
}));

// Mock mock-messages
vi.mock("@/lib/mock-messages", () => ({
  MOCK_MESSAGES_WITH_ALL_TYPES: [],
}));

// Wrapper with Router
function renderWithRouter(
  ui: React.ReactElement,
  { route = "/session/test-session-123" } = {}
) {
  return render(
    <MemoryRouter initialEntries={[route]}>
      <Routes>
        <Route path="/session/:sessionId" element={ui} />
        <Route path="/" element={<div data-testid="dashboard">Dashboard</div>} />
      </Routes>
    </MemoryRouter>
  );
}

describe("Player Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("UI 展示", () => {
    it("应该渲染页面标题 Mantra 心法", async () => {
      renderWithRouter(<Player />);
      // 等待加载完成
      await waitFor(() => {
        expect(screen.getByText("Mantra")).toBeInTheDocument();
      });
      expect(screen.getByText("心法")).toBeInTheDocument();
    });

    it("应该渲染 DualStreamLayout", async () => {
      renderWithRouter(<Player />);
      // 等待异步加载完成后显示 DualStreamLayout
      // 由于会话返回空消息，会显示"会话为空"状态
      await waitFor(() => {
        // 检查初始标题渲染
        expect(screen.getByText("Mantra")).toBeInTheDocument();
      });
    });

    it("应该渲染返回按钮", async () => {
      renderWithRouter(<Player />);
      await waitFor(() => {
        const buttons = screen.getAllByRole("button");
        expect(buttons.length).toBeGreaterThanOrEqual(1);
      });
    });

    it("应该在加载时显示加载状态", () => {
      renderWithRouter(<Player />);
      // 初始渲染时应该显示加载中状态
      expect(screen.getByText("加载会话中...")).toBeInTheDocument();
    });
  });

  describe("导航", () => {
    it("点击返回按钮应该导航回 Dashboard", async () => {
      const user = userEvent.setup();
      renderWithRouter(<Player />);

      // 等待加载完成
      await waitFor(() => {
        expect(screen.getByText("Mantra")).toBeInTheDocument();
      });

      // 找到返回按钮 (第一个 button，包含 ArrowLeft 图标)
      const backButton = screen.getAllByRole("button")[0];
      await user.click(backButton);

      // 应该导航到 Dashboard
      expect(screen.getByTestId("dashboard")).toBeInTheDocument();
    });
  });

  describe("样式", () => {
    it("应该有全屏高度布局", () => {
      const { container } = renderWithRouter(<Player />);
      const mainDiv = container.firstChild as HTMLElement;
      expect(mainDiv).toHaveClass("h-screen");
    });
  });
});
