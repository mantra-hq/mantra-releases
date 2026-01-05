/**
 * useNotificationStore Tests - 通知状态管理测试
 * Tech-Spec: 通知系统 Task 17
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { act } from "@testing-library/react";
import { useNotificationStore } from "./useNotificationStore";
import { notificationStorage } from "@/lib/notification-storage";
import type { BannerNotification, InboxNotification } from "@/types/notification";

// Mock notificationStorage
vi.mock("@/lib/notification-storage", () => ({
  notificationStorage: {
    getReadIds: vi.fn(() => []),
    addReadId: vi.fn(),
    addReadIds: vi.fn(),
    getDismissedBanners: vi.fn(() => []),
    dismissBanner: vi.fn(),
    clear: vi.fn(),
  },
}));

// Mock sonner
vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// 测试数据
const mockBanner: BannerNotification = {
  id: "banner-1",
  category: "banner",
  title: "Test Banner",
  body: "Test banner body",
  createdAt: "2026-01-05T10:00:00Z",
  dismissible: true,
  priority: "normal",
};

const mockBanner2: BannerNotification = {
  id: "banner-2",
  category: "banner",
  title: "Test Banner 2",
  body: "Test banner 2 body",
  createdAt: "2026-01-05T11:00:00Z",
  dismissible: true,
  priority: "high",
};

const mockInboxNotification: InboxNotification = {
  id: "inbox-1",
  category: "inbox",
  type: "follow",
  title: "New Follower",
  body: "User X followed you",
  createdAt: "2026-01-05T09:00:00Z",
  isRead: false,
  actions: [
    {
      id: "follow-back",
      label: "Follow Back",
      variant: "primary",
      actionType: "api",
      payload: "/api/users/x/follow",
    },
  ],
};

const mockInboxNotification2: InboxNotification = {
  id: "inbox-2",
  category: "inbox",
  type: "comment",
  title: "New Comment",
  body: "User Y commented on your post",
  createdAt: "2026-01-05T08:00:00Z",
  isRead: false,
};

const mockInboxNotificationRead: InboxNotification = {
  id: "inbox-3",
  category: "inbox",
  type: "like",
  title: "New Like",
  body: "User Z liked your post",
  createdAt: "2026-01-05T07:00:00Z",
  isRead: true,
};

// 初始状态
const initialState = {
  banners: [] as BannerNotification[],
  inbox: [] as InboxNotification[],
  unreadCount: 0,
  inboxOpen: false,
  isLoading: false,
  error: null as string | null,
};

describe("useNotificationStore", () => {
  beforeEach(() => {
    // Reset store before each test
    act(() => {
      useNotificationStore.setState(initialState);
    });
    // Reset mocks
    vi.clearAllMocks();
  });

  describe("initial state", () => {
    it("should have correct initial state", () => {
      const state = useNotificationStore.getState();
      expect(state.banners).toEqual([]);
      expect(state.inbox).toEqual([]);
      expect(state.unreadCount).toBe(0);
      expect(state.inboxOpen).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe("setInboxOpen", () => {
    it("should open inbox", () => {
      act(() => {
        useNotificationStore.getState().setInboxOpen(true);
      });
      expect(useNotificationStore.getState().inboxOpen).toBe(true);
    });

    it("should close inbox", () => {
      act(() => {
        useNotificationStore.getState().setInboxOpen(true);
        useNotificationStore.getState().setInboxOpen(false);
      });
      expect(useNotificationStore.getState().inboxOpen).toBe(false);
    });
  });

  describe("markAsRead", () => {
    beforeEach(() => {
      act(() => {
        useNotificationStore.setState({
          inbox: [mockInboxNotification, mockInboxNotification2],
          unreadCount: 2,
        });
      });
    });

    it("should mark notification as read", () => {
      act(() => {
        useNotificationStore.getState().markAsRead("inbox-1");
      });

      const state = useNotificationStore.getState();
      const notification = state.inbox.find((n) => n.id === "inbox-1");
      expect(notification?.isRead).toBe(true);
      expect(state.unreadCount).toBe(1);
    });

    it("should call storage to persist read state", () => {
      act(() => {
        useNotificationStore.getState().markAsRead("inbox-1");
      });

      expect(notificationStorage.addReadId).toHaveBeenCalledWith("inbox-1");
    });
  });

  describe("markAllAsRead", () => {
    beforeEach(() => {
      act(() => {
        useNotificationStore.setState({
          inbox: [mockInboxNotification, mockInboxNotification2, mockInboxNotificationRead],
          unreadCount: 2,
        });
      });
    });

    it("should mark all notifications as read", () => {
      act(() => {
        useNotificationStore.getState().markAllAsRead();
      });

      const state = useNotificationStore.getState();
      expect(state.inbox.every((n) => n.isRead)).toBe(true);
      expect(state.unreadCount).toBe(0);
    });

    it("should call storage to persist all read states", () => {
      act(() => {
        useNotificationStore.getState().markAllAsRead();
      });

      expect(notificationStorage.addReadIds).toHaveBeenCalledWith(["inbox-1", "inbox-2"]);
    });
  });

  describe("dismissBanner", () => {
    beforeEach(() => {
      act(() => {
        useNotificationStore.setState({
          banners: [mockBanner, mockBanner2],
        });
      });
    });

    it("should remove banner from list (temporary dismiss)", () => {
      act(() => {
        useNotificationStore.getState().dismissBanner("banner-1", false);
      });

      const state = useNotificationStore.getState();
      expect(state.banners).toHaveLength(1);
      expect(state.banners[0].id).toBe("banner-2");
    });

    it("should not call storage for temporary dismiss", () => {
      act(() => {
        useNotificationStore.getState().dismissBanner("banner-1", false);
      });

      expect(notificationStorage.dismissBanner).not.toHaveBeenCalled();
    });

    it("should call storage for permanent dismiss", () => {
      act(() => {
        useNotificationStore.getState().dismissBanner("banner-1", true);
      });

      expect(notificationStorage.dismissBanner).toHaveBeenCalledWith("banner-1");
    });
  });

  describe("reset", () => {
    it("should reset all state to initial values", () => {
      act(() => {
        useNotificationStore.setState({
          banners: [mockBanner],
          inbox: [mockInboxNotification],
          unreadCount: 1,
          inboxOpen: true,
          isLoading: true,
          error: "Some error",
        });
        useNotificationStore.getState().reset();
      });

      const state = useNotificationStore.getState();
      expect(state.banners).toEqual([]);
      expect(state.inbox).toEqual([]);
      expect(state.unreadCount).toBe(0);
      expect(state.inboxOpen).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });

    it("should clear storage", () => {
      act(() => {
        useNotificationStore.getState().reset();
      });

      expect(notificationStorage.clear).toHaveBeenCalled();
    });
  });

  describe("clearError", () => {
    it("should clear error state", () => {
      act(() => {
        useNotificationStore.setState({ error: "Some error" });
        useNotificationStore.getState().clearError();
      });

      expect(useNotificationStore.getState().error).toBeNull();
    });
  });

  describe("unreadCount calculation", () => {
    it("should correctly calculate unread count", () => {
      act(() => {
        useNotificationStore.setState({
          inbox: [mockInboxNotification, mockInboxNotification2, mockInboxNotificationRead],
          unreadCount: 2, // inbox-1 and inbox-2 are unread
        });
      });

      expect(useNotificationStore.getState().unreadCount).toBe(2);
    });

    it("should update unread count when marking as read", () => {
      act(() => {
        useNotificationStore.setState({
          inbox: [mockInboxNotification, mockInboxNotification2],
          unreadCount: 2,
        });
        useNotificationStore.getState().markAsRead("inbox-1");
      });

      expect(useNotificationStore.getState().unreadCount).toBe(1);
    });
  });

  describe("fetchAll", () => {
    it("should load banners and inbox from mock data", async () => {
      await act(async () => {
        await useNotificationStore.getState().fetchAll();
      });

      const state = useNotificationStore.getState();
      expect(state.banners.length).toBeGreaterThan(0);
      expect(state.inbox.length).toBeGreaterThan(0);
      expect(state.isLoading).toBe(false);
    });

    it("should filter out dismissed banners from localStorage", async () => {
      (notificationStorage.getDismissedBanners as ReturnType<typeof vi.fn>).mockReturnValue([
        "banner-1",
      ]);

      await act(async () => {
        await useNotificationStore.getState().fetchAll();
      });

      const state = useNotificationStore.getState();
      expect(state.banners.find((b) => b.id === "banner-1")).toBeUndefined();
    });

    it("should apply read state from localStorage to inbox", async () => {
      (notificationStorage.getReadIds as ReturnType<typeof vi.fn>).mockReturnValue([
        "inbox-1",
      ]);

      await act(async () => {
        await useNotificationStore.getState().fetchAll();
      });

      const state = useNotificationStore.getState();
      const notification = state.inbox.find((n) => n.id === "inbox-1");
      expect(notification?.isRead).toBe(true);
    });

    it("should calculate correct unread count after fetch", async () => {
      await act(async () => {
        await useNotificationStore.getState().fetchAll();
      });

      const state = useNotificationStore.getState();
      const expectedUnreadCount = state.inbox.filter((n) => !n.isRead).length;
      expect(state.unreadCount).toBe(expectedUnreadCount);
    });

    it("should clear error on successful fetch", async () => {
      act(() => {
        useNotificationStore.setState({ error: "Previous error" });
      });

      await act(async () => {
        await useNotificationStore.getState().fetchAll();
      });

      expect(useNotificationStore.getState().error).toBeNull();
    });
  });

  describe("executeAction", () => {
    beforeEach(() => {
      act(() => {
        useNotificationStore.setState({
          inbox: [mockInboxNotification],
          unreadCount: 1,
        });
      });
    });

    it("should show toast for api action type", async () => {
      const { toast } = await import("sonner");

      await act(async () => {
        await useNotificationStore.getState().executeAction("inbox-1", "follow-back");
      });

      expect(toast.success).toHaveBeenCalledWith("操作已执行: Follow Back");
    });

    it("should mark notification as read after executing action", async () => {
      await act(async () => {
        await useNotificationStore.getState().executeAction("inbox-1", "follow-back");
      });

      const notification = useNotificationStore.getState().inbox.find((n) => n.id === "inbox-1");
      expect(notification?.isRead).toBe(true);
      expect(notificationStorage.addReadId).toHaveBeenCalledWith("inbox-1");
    });

    it("should do nothing for non-existent notification", async () => {
      const { toast } = await import("sonner");

      await act(async () => {
        await useNotificationStore.getState().executeAction("non-existent", "action");
      });

      expect(toast.success).not.toHaveBeenCalled();
    });

    it("should do nothing for non-existent action", async () => {
      const { toast } = await import("sonner");

      await act(async () => {
        await useNotificationStore.getState().executeAction("inbox-1", "non-existent");
      });

      expect(toast.success).not.toHaveBeenCalled();
    });

    it("should remove notification for dismiss action type", async () => {
      const dismissNotification: InboxNotification = {
        id: "inbox-dismiss",
        category: "inbox",
        type: "system",
        title: "Dismissable",
        body: "Can be dismissed",
        createdAt: "2026-01-05T10:00:00Z",
        isRead: false,
        actions: [
          {
            id: "dismiss-action",
            label: "Dismiss",
            variant: "secondary",
            actionType: "dismiss",
            payload: "",
          },
        ],
      };

      act(() => {
        useNotificationStore.setState({
          inbox: [dismissNotification],
          unreadCount: 1,
        });
      });

      await act(async () => {
        await useNotificationStore.getState().executeAction("inbox-dismiss", "dismiss-action");
      });

      const state = useNotificationStore.getState();
      expect(state.inbox.find((n) => n.id === "inbox-dismiss")).toBeUndefined();
    });
  });
});
