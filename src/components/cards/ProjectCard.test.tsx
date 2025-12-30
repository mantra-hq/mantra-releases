/**
 * ProjectCard Tests - 项目卡片组件测试
 * Story 2.8: Task 2
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ProjectCard } from "./ProjectCard";
import type { Project, Session } from "@/types/project";

// Mock date-fns formatDistanceToNow
vi.mock("date-fns", () => ({
  formatDistanceToNow: vi.fn((date: Date) => {
    const now = Date.now();
    const diff = now - date.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    if (hours < 1) return "刚刚";
    if (hours < 24) return `${hours} 小时前`;
    const days = Math.floor(hours / 24);
    return `${days} 天前`;
  }),
}));

// Mock session data
const mockSessions: Session[] = [
  {
    id: "session-1",
    title: "实现用户认证功能",
    source: "claude",
    messageCount: 42,
    startTime: Date.now() - 3600000, // 1 hour ago
    endTime: Date.now() - 1800000, // 30 minutes ago
  },
  {
    id: "session-2",
    title: "修复登录 Bug",
    source: "gemini",
    messageCount: 15,
    startTime: Date.now() - 7200000, // 2 hours ago
    endTime: Date.now() - 5400000, // 1.5 hours ago
  },
];

// Mock project data
const mockProject: Project = {
  id: "project-1",
  name: "my-awesome-project",
  path: "/home/user/projects/my-awesome-project",
  sessions: mockSessions,
  lastActivity: Date.now() - 1800000, // 30 minutes ago
};

describe("ProjectCard", () => {
  const defaultProps = {
    project: mockProject,
    isExpanded: false,
    onToggle: vi.fn(),
    onSessionClick: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("项目信息展示", () => {
    it("应该显示项目名称", () => {
      render(<ProjectCard {...defaultProps} />);
      expect(screen.getByText("my-awesome-project")).toBeInTheDocument();
    });

    it("应该显示 Folder 图标", () => {
      render(<ProjectCard {...defaultProps} />);
      // Folder icon should be rendered (lucide-react uses data-testid)
      const icon = document.querySelector('[data-slot="icon"]');
      expect(icon || document.querySelector("svg")).toBeInTheDocument();
    });

    it("应该显示会话数量", () => {
      render(<ProjectCard {...defaultProps} />);
      expect(screen.getByText(/2.*会话|2 sessions/i)).toBeInTheDocument();
    });

    it("应该显示相对时间", () => {
      render(<ProjectCard {...defaultProps} />);
      // formatDistanceToNow mock returns "刚刚" or "X 小时前" or "X 天前"
      // 由于 mock 会根据时间差返回不同值，我们匹配常见格式
      expect(screen.getByText(/刚刚|前|ago/i)).toBeInTheDocument();
    });
  });

  describe("展开/折叠交互", () => {
    it("点击卡片头部应该调用 onToggle", async () => {
      const user = userEvent.setup();
      const onToggle = vi.fn();
      render(<ProjectCard {...defaultProps} onToggle={onToggle} />);

      const header = screen.getByRole("button", { name: /my-awesome-project/i });
      await user.click(header);

      expect(onToggle).toHaveBeenCalledTimes(1);
    });

    it("折叠状态下不应该显示会话列表", () => {
      render(<ProjectCard {...defaultProps} isExpanded={false} />);
      expect(screen.queryByText("实现用户认证功能")).not.toBeInTheDocument();
    });

    it("展开状态下应该显示会话列表", () => {
      render(<ProjectCard {...defaultProps} isExpanded={true} />);
      expect(screen.getByText("实现用户认证功能")).toBeInTheDocument();
      expect(screen.getByText("修复登录 Bug")).toBeInTheDocument();
    });
  });

  describe("展开状态指示器", () => {
    it("折叠状态下箭头应该朝下", () => {
      render(<ProjectCard {...defaultProps} isExpanded={false} />);
      const chevron = document.querySelector('[data-expanded="false"]');
      expect(chevron).toBeInTheDocument();
    });

    it("展开状态下箭头应该旋转", () => {
      render(<ProjectCard {...defaultProps} isExpanded={true} />);
      const chevron = document.querySelector('[data-expanded="true"]');
      expect(chevron).toBeInTheDocument();
    });
  });

  describe("无会话项目", () => {
    it("应该显示 0 会话", () => {
      const emptyProject: Project = {
        ...mockProject,
        sessions: [],
      };
      render(<ProjectCard {...defaultProps} project={emptyProject} />);
      expect(screen.getByText(/0.*会话|0 sessions/i)).toBeInTheDocument();
    });
  });

  describe("样式", () => {
    it("应该有正确的卡片容器样式", () => {
      render(<ProjectCard {...defaultProps} />);
      const card = screen.getByTestId("project-card");
      expect(card).toHaveClass("rounded-lg");
    });
  });
});

