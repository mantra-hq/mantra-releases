/**
 * NotificationCard Tests - 消息卡片组件测试
 * Tech-Spec: 通知系统
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { NotificationCard } from "./NotificationCard";
import type { InboxNotification } from "@/types/notification";

// Mock react-router-dom
const mockNavigate = vi.fn();
vi.mock("react-router-dom", () => ({
  useNavigate: () => mockNavigate,
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "notifications.unread": "Unread",
        "time.unknownTime": "Unknown time",
      };
      return translations[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

// Mock date-fns
vi.mock("date-fns", () => ({
  formatDistanceToNow: () => "2 hours ago",
}));

vi.mock("date-fns/locale", () => ({
  zhCN: {},
  enUS: {},
}));

describe("NotificationCard", () => {
  const mockNotification: InboxNotification = {
    id: "test-1",
    category: "inbox",
    type: "follow",
    title: "New Follower",
    body: "User X followed you",
    createdAt: "2026-01-05T09:00:00Z",
    isRead: false,
    icon: "UserPlus",
    actions: [
      {
        id: "follow-back",
        label: "Follow Back",
        variant: "primary",
        actionType: "api",
        payload: "/api/users/x/follow",
      },
      {
        id: "view-profile",
        label: "View Profile",
        variant: "secondary",
        actionType: "navigate",
        payload: "/users/x",
      },
    ],
    link: "/users/x",
  };

  const mockOnClick = vi.fn();
  const mockOnAction = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("基础渲染", () => {
    it("应该渲染卡片", () => {
      render(<NotificationCard notification={mockNotification} />);
      expect(screen.getByTestId("notification-card-test-1")).toBeInTheDocument();
    });

    it("应该显示标题", () => {
      render(<NotificationCard notification={mockNotification} />);
      expect(screen.getByText("New Follower")).toBeInTheDocument();
    });

    it("应该显示正文", () => {
      render(<NotificationCard notification={mockNotification} />);
      expect(screen.getByText("User X followed you")).toBeInTheDocument();
    });

    it("应该显示相对时间", () => {
      render(<NotificationCard notification={mockNotification} />);
      expect(screen.getByText("2 hours ago")).toBeInTheDocument();
    });

    it("应该有正确的 aria-label", () => {
      render(<NotificationCard notification={mockNotification} />);
      const card = screen.getByTestId("notification-card-test-1");
      expect(card).toHaveAttribute("aria-label", "New Follower: User X followed you");
    });
  });

  describe("未读标记", () => {
    it("未读消息应该显示蓝色圆点", () => {
      render(<NotificationCard notification={mockNotification} />);
      const unreadDot = screen.getByLabelText("Unread");
      expect(unreadDot).toBeInTheDocument();
      expect(unreadDot).toHaveClass("bg-blue-500");
    });

    it("已读消息不应该显示蓝色圆点", () => {
      const readNotification = { ...mockNotification, isRead: true };
      render(<NotificationCard notification={readNotification} />);
      expect(screen.queryByLabelText("Unread")).not.toBeInTheDocument();
    });

    it("未读消息标题应该加粗", () => {
      render(<NotificationCard notification={mockNotification} />);
      const title = screen.getByText("New Follower");
      expect(title).toHaveClass("font-medium");
    });
  });

  describe("操作按钮", () => {
    it("应该渲染操作按钮", () => {
      render(<NotificationCard notification={mockNotification} />);
      expect(screen.getByText("Follow Back")).toBeInTheDocument();
      expect(screen.getByText("View Profile")).toBeInTheDocument();
    });

    it("点击操作按钮应该调用 onAction", () => {
      render(
        <NotificationCard
          notification={mockNotification}
          onAction={mockOnAction}
        />
      );
      fireEvent.click(screen.getByText("Follow Back"));
      expect(mockOnAction).toHaveBeenCalledWith("follow-back");
    });

    it("点击 navigate 类型按钮应该导航", () => {
      render(
        <NotificationCard
          notification={mockNotification}
          onAction={mockOnAction}
        />
      );
      fireEvent.click(screen.getByText("View Profile"));
      expect(mockNavigate).toHaveBeenCalledWith("/users/x");
    });

    it("没有操作按钮时不应该渲染按钮区域", () => {
      const noActionNotification = { ...mockNotification, actions: undefined };
      render(<NotificationCard notification={noActionNotification} />);
      expect(screen.queryByText("Follow Back")).not.toBeInTheDocument();
    });
  });

  describe("点击交互", () => {
    it("点击卡片应该调用 onClick", () => {
      render(
        <NotificationCard
          notification={mockNotification}
          onClick={mockOnClick}
        />
      );
      fireEvent.click(screen.getByTestId("notification-card-test-1"));
      expect(mockOnClick).toHaveBeenCalled();
    });

    it("点击卡片应该导航到 link", () => {
      render(<NotificationCard notification={mockNotification} />);
      fireEvent.click(screen.getByTestId("notification-card-test-1"));
      expect(mockNavigate).toHaveBeenCalledWith("/users/x");
    });

    it("按 Enter 键应该触发点击", () => {
      render(
        <NotificationCard
          notification={mockNotification}
          onClick={mockOnClick}
        />
      );
      fireEvent.keyDown(screen.getByTestId("notification-card-test-1"), {
        key: "Enter",
      });
      expect(mockOnClick).toHaveBeenCalled();
    });

    it("按空格键应该触发点击", () => {
      render(
        <NotificationCard
          notification={mockNotification}
          onClick={mockOnClick}
        />
      );
      fireEvent.keyDown(screen.getByTestId("notification-card-test-1"), {
        key: " ",
      });
      expect(mockOnClick).toHaveBeenCalled();
    });
  });

  describe("无 link 的卡片", () => {
    it("没有 link 时点击不应该导航", () => {
      const noLinkNotification = { ...mockNotification, link: undefined };
      render(
        <NotificationCard
          notification={noLinkNotification}
          onClick={mockOnClick}
        />
      );
      fireEvent.click(screen.getByTestId("notification-card-test-1"));
      expect(mockOnClick).toHaveBeenCalled();
      expect(mockNavigate).not.toHaveBeenCalled();
    });
  });
});
