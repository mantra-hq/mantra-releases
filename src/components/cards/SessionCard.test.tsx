/**
 * SessionCard Tests - 会话卡片组件测试
 * Story 2.8: Task 3
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SessionCard } from "./SessionCard";
import type { Session } from "@/types/project";

// Mock date-fns formatDistanceToNow
vi.mock("date-fns", () => ({
  formatDistanceToNow: vi.fn(() => "2 小时前"),
}));

// Mock session data for different sources
const createMockSession = (source: Session["source"]): Session => ({
  id: `session-${source}`,
  source,
  message_count: 25,
  created_at: new Date(Date.now() - 7200000).toISOString(), // 2 hours ago
  updated_at: new Date(Date.now() - 3600000).toISOString(), // 1 hour ago
});

describe("SessionCard", () => {
  const mockOnClick = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("会话信息展示", () => {
    it("应该显示会话标题（包含来源名称）", () => {
      const session = createMockSession("claude");
      render(<SessionCard session={session} onClick={mockOnClick} />);
      // 标题格式：{Source} 会话 · {时间}
      expect(screen.getByText(/Claude 会话/)).toBeInTheDocument();
    });

    it("应该显示消息数量", () => {
      const session = createMockSession("claude");
      render(<SessionCard session={session} onClick={mockOnClick} />);
      expect(screen.getByText(/25.*消息|25 条消息/i)).toBeInTheDocument();
    });

    it("应该显示相对时间", () => {
      const session = createMockSession("claude");
      render(<SessionCard session={session} onClick={mockOnClick} />);
      expect(screen.getByText(/2 小时前/)).toBeInTheDocument();
    });
  });

  describe("来源图标", () => {
    it("应该显示 Claude 来源图标 (橙色 Sparkles)", () => {
      const session = createMockSession("claude");
      render(<SessionCard session={session} onClick={mockOnClick} />);
      const icon = document.querySelector(".text-orange-500");
      expect(icon).toBeInTheDocument();
    });

    it("应该显示 Gemini 来源图标 (蓝色 MessageSquare)", () => {
      const session = createMockSession("gemini");
      render(<SessionCard session={session} onClick={mockOnClick} />);
      const icon = document.querySelector(".text-blue-500");
      expect(icon).toBeInTheDocument();
    });

    it("应该显示 Cursor 来源图标 (紫色 Terminal)", () => {
      const session = createMockSession("cursor");
      render(<SessionCard session={session} onClick={mockOnClick} />);
      const icon = document.querySelector(".text-purple-500");
      expect(icon).toBeInTheDocument();
    });
  });

  describe("交互", () => {
    it("点击卡片应该调用 onClick", async () => {
      const user = userEvent.setup();
      const session = createMockSession("claude");
      render(<SessionCard session={session} onClick={mockOnClick} />);

      const card = screen.getByTestId("session-card");
      await user.click(card);

      expect(mockOnClick).toHaveBeenCalledTimes(1);
    });

    it("应该有 hover 样式", () => {
      const session = createMockSession("claude");
      render(<SessionCard session={session} onClick={mockOnClick} />);
      const card = screen.getByTestId("session-card");
      expect(card).toHaveClass("hover:bg-muted");
    });
  });

  describe("样式", () => {
    it("应该有正确的卡片容器样式", () => {
      const session = createMockSession("claude");
      render(<SessionCard session={session} onClick={mockOnClick} />);
      const card = screen.getByTestId("session-card");
      expect(card).toHaveClass("rounded-md");
      expect(card).toHaveClass("cursor-pointer");
    });
  });
});
