/**
 * Player Page Tests - 会话回放页面测试
 * Story 2.8: Task 9 (Code Review Fix)
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import Player from "./Player";

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
    it("应该渲染页面标题 Mantra 心法", () => {
      renderWithRouter(<Player />);
      expect(screen.getByText("Mantra")).toBeInTheDocument();
      expect(screen.getByText("心法")).toBeInTheDocument();
    });

    it("应该显示 session ID", () => {
      renderWithRouter(<Player />, { route: "/session/my-session-456" });
      expect(screen.getByText(/Session:.*my-session-456/)).toBeInTheDocument();
    });

    it("应该渲染 DualStreamLayout", () => {
      renderWithRouter(<Player />);
      expect(screen.getByTestId("dual-stream-layout")).toBeInTheDocument();
    });

    it("应该渲染返回按钮", () => {
      renderWithRouter(<Player />);
      // ArrowLeft icon is in a button
      const buttons = screen.getAllByRole("button");
      expect(buttons.length).toBeGreaterThanOrEqual(1);
    });

    it("应该渲染主题切换按钮", () => {
      renderWithRouter(<Player />);
      expect(screen.getByTestId("theme-toggle")).toBeInTheDocument();
    });
  });

  describe("导航", () => {
    it("点击返回按钮应该导航回 Dashboard", async () => {
      const user = userEvent.setup();
      renderWithRouter(<Player />);

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

