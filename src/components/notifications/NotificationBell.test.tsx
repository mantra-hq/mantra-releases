/**
 * NotificationBell Tests - 铃铛图标组件测试
 * Tech-Spec: 通知系统
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { NotificationBell } from "./NotificationBell";
import { useNotificationStore } from "@/stores/useNotificationStore";

// Mock useNotificationStore
vi.mock("@/stores/useNotificationStore", () => ({
  useNotificationStore: vi.fn(),
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { count?: number }) => {
      const translations: Record<string, string> = {
        "notifications.openInbox": `Open notifications (${options?.count ?? 0} unread)`,
        "notifications.unreadCount": `${options?.count ?? 0} unread notifications`,
        "notifications.title": "Notifications",
      };
      return translations[key] || key;
    },
  }),
}));

// Helper to render with TooltipProvider
const renderWithProvider = (ui: React.ReactElement) => {
  return render(<TooltipProvider>{ui}</TooltipProvider>);
};

describe("NotificationBell", () => {
  const mockSetInboxOpen = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useNotificationStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(
      (selector: (state: { unreadCount: number; setInboxOpen: typeof mockSetInboxOpen }) => unknown) =>
        selector({ unreadCount: 0, setInboxOpen: mockSetInboxOpen })
    );
  });

  describe("基础渲染", () => {
    it("应该渲染铃铛按钮", () => {
      renderWithProvider(<NotificationBell />);
      expect(screen.getByTestId("notification-bell")).toBeInTheDocument();
    });

    it("应该有正确的 aria-label", () => {
      renderWithProvider(<NotificationBell />);
      const button = screen.getByTestId("notification-bell");
      expect(button).toHaveAttribute("aria-label", "Open notifications (0 unread)");
    });
  });

  describe("未读角标", () => {
    it("没有未读时不应该显示角标", () => {
      renderWithProvider(<NotificationBell />);
      expect(screen.queryByTestId("notification-badge")).not.toBeInTheDocument();
    });

    it("有未读时应该显示角标", () => {
      (useNotificationStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(
        (selector: (state: { unreadCount: number; setInboxOpen: typeof mockSetInboxOpen }) => unknown) =>
          selector({ unreadCount: 5, setInboxOpen: mockSetInboxOpen })
      );

      renderWithProvider(<NotificationBell />);
      const badge = screen.getByTestId("notification-badge");
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveTextContent("5");
    });

    it("超过 9 个未读时应该显示 9+", () => {
      (useNotificationStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(
        (selector: (state: { unreadCount: number; setInboxOpen: typeof mockSetInboxOpen }) => unknown) =>
          selector({ unreadCount: 15, setInboxOpen: mockSetInboxOpen })
      );

      renderWithProvider(<NotificationBell />);
      const badge = screen.getByTestId("notification-badge");
      expect(badge).toHaveTextContent("9+");
    });

    it("正好 9 个未读时应该显示 9", () => {
      (useNotificationStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(
        (selector: (state: { unreadCount: number; setInboxOpen: typeof mockSetInboxOpen }) => unknown) =>
          selector({ unreadCount: 9, setInboxOpen: mockSetInboxOpen })
      );

      renderWithProvider(<NotificationBell />);
      const badge = screen.getByTestId("notification-badge");
      expect(badge).toHaveTextContent("9");
    });
  });

  describe("点击交互", () => {
    it("点击应该打开 Inbox", () => {
      renderWithProvider(<NotificationBell />);
      const button = screen.getByTestId("notification-bell");
      fireEvent.click(button);
      expect(mockSetInboxOpen).toHaveBeenCalledWith(true);
    });
  });

  describe("自定义样式", () => {
    it("应该接受自定义 className", () => {
      renderWithProvider(<NotificationBell className="custom-class" />);
      const button = screen.getByTestId("notification-bell");
      expect(button).toHaveClass("custom-class");
    });
  });
});
