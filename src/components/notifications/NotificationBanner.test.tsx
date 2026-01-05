/**
 * NotificationBanner Tests - Banner 组件测试
 * Tech-Spec: 通知系统
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { NotificationBanner } from "./NotificationBanner";
import type { BannerNotification } from "@/types/notification";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "notifications.dismissBanner": "Dismiss notification",
        "notifications.moreOptions": "More options",
        "notifications.dismissOnce": "Dismiss for now",
        "notifications.dismissForever": "Don't show again",
      };
      return translations[key] || key;
    },
  }),
}));

describe("NotificationBanner", () => {
  const mockBanner: BannerNotification = {
    id: "banner-1",
    category: "banner",
    title: "Test Banner",
    body: "This is a test banner message",
    createdAt: "2026-01-05T10:00:00Z",
    dismissible: true,
    priority: "normal",
  };

  const mockOnDismiss = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("基础渲染", () => {
    it("应该渲染 Banner", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      expect(screen.getByTestId("notification-banner-banner-1")).toBeInTheDocument();
    });

    it("应该显示标题", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      expect(screen.getByText("Test Banner")).toBeInTheDocument();
    });

    it("应该显示正文", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      expect(screen.getByText("This is a test banner message")).toBeInTheDocument();
    });

    it("应该有 role=alert", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });

    it("应该有正确的 aria-label", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      expect(screen.getByRole("alert")).toHaveAttribute("aria-label", "Test Banner");
    });
  });

  describe("优先级样式", () => {
    it("应该使用不透明卡片背景", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      const banner = screen.getByTestId("notification-banner-banner-1");
      expect(banner).toHaveClass("bg-card");
    });

    it("普通优先级应该使用主色边框", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      const banner = screen.getByTestId("notification-banner-banner-1");
      expect(banner).toHaveClass("border-primary/50");
    });

    it("高优先级应该使用警告色边框", () => {
      const highPriorityBanner = { ...mockBanner, priority: "high" as const };
      render(<NotificationBanner banner={highPriorityBanner} onDismiss={mockOnDismiss} />);
      const banner = screen.getByTestId("notification-banner-banner-1");
      expect(banner).toHaveClass("border-destructive/50");
    });
  });

  describe("关闭按钮", () => {
    it("可关闭时应该显示关闭按钮", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      expect(screen.getByLabelText("Dismiss notification")).toBeInTheDocument();
    });

    it("不可关闭时不应该显示关闭按钮", () => {
      const nonDismissibleBanner = { ...mockBanner, dismissible: false };
      render(<NotificationBanner banner={nonDismissibleBanner} onDismiss={mockOnDismiss} />);
      expect(screen.queryByLabelText("Dismiss notification")).not.toBeInTheDocument();
    });

    it("点击关闭按钮应该调用 onDismiss(false)", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      fireEvent.click(screen.getByLabelText("Dismiss notification"));
      expect(mockOnDismiss).toHaveBeenCalledWith(false);
    });
  });

  describe("下拉菜单", () => {
    it("应该显示更多选项按钮", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      expect(screen.getByLabelText("More options")).toBeInTheDocument();
    });

    it("更多选项按钮应该有 dropdown trigger 属性", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      const trigger = screen.getByLabelText("More options");
      expect(trigger).toHaveAttribute("aria-haspopup", "menu");
      expect(trigger).toHaveAttribute("data-state", "closed");
    });
  });

  describe("动画", () => {
    it("默认应该有进入动画类", () => {
      render(<NotificationBanner banner={mockBanner} onDismiss={mockOnDismiss} />);
      const banner = screen.getByTestId("notification-banner-banner-1");
      expect(banner).toHaveClass("animate-in");
      expect(banner).toHaveClass("slide-in-from-top");
      expect(banner).toHaveAttribute("data-state", "open");
    });

    it("isExiting=true 时应该有退出样式", () => {
      render(
        <NotificationBanner
          banner={mockBanner}
          onDismiss={mockOnDismiss}
          isExiting={true}
        />
      );
      const banner = screen.getByTestId("notification-banner-banner-1");
      expect(banner).toHaveClass("opacity-0");
      expect(banner).toHaveAttribute("data-state", "closed");
    });
  });
});
