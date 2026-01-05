/**
 * useNotificationStore - 通知状态管理
 * Tech-Spec: 通知系统 ADR-002
 *
 * 管理通知系统的所有状态:
 * - Banner 和 Inbox 通知列表
 * - 未读数计算
 * - Inbox 面板开关
 * - 已读标记和 Banner 关闭
 */

import { create } from "zustand";
import { toast } from "sonner";
import type { BannerNotification, InboxNotification } from "@/types/notification";
import { notificationStorage } from "@/lib/notification-storage";
import { getMockNotifications } from "@/lib/mock-notifications";

/**
 * 通知状态接口
 */
export interface NotificationState {
  // ======== 状态 ========
  /** Banner 通知列表 */
  banners: BannerNotification[];
  /** Inbox 通知列表 */
  inbox: InboxNotification[];
  /** 未读数量 */
  unreadCount: number;
  /** Inbox 面板是否打开 */
  inboxOpen: boolean;
  /** 是否正在加载 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;

  // ======== Actions ========
  /** 获取所有通知 */
  fetchAll: () => Promise<void>;
  /** 关闭 Banner (可选永久隐藏) */
  dismissBanner: (id: string, permanent: boolean) => void;
  /** 标记单条通知已读 */
  markAsRead: (id: string) => void;
  /** 标记所有通知已读 */
  markAllAsRead: () => void;
  /** 设置 Inbox 面板开关 */
  setInboxOpen: (open: boolean) => void;
  /** 执行通知操作按钮 */
  executeAction: (notificationId: string, actionId: string) => Promise<void>;
  /** 重置状态 (用于登出) */
  reset: () => void;
  /** 清除错误 */
  clearError: () => void;
  /** WebSocket 订阅预留 - TODO: 实现实时推送 */
  subscribe: () => () => void;
}

/** 初始状态 */
const initialState = {
  banners: [] as BannerNotification[],
  inbox: [] as InboxNotification[],
  unreadCount: 0,
  inboxOpen: false,
  isLoading: false,
  error: null as string | null,
};

/**
 * 计算未读数量
 */
function calculateUnreadCount(inbox: InboxNotification[]): number {
  return inbox.filter((n) => !n.isRead).length;
}

/**
 * 通知状态 Store
 */
export const useNotificationStore = create<NotificationState>()((set, get) => ({
  ...initialState,

  fetchAll: async () => {
    set({ isLoading: true, error: null });

    try {
      // MVP 阶段从 mock 数据加载
      // TODO: 替换为真实 API 调用
      // const response = await fetch('/api/v1/notifications');
      // const data = await response.json();

      const { banners, inbox } = getMockNotifications();

      // 从 localStorage 恢复已读状态
      const readIds = notificationStorage.getReadIds();
      const dismissedBanners = notificationStorage.getDismissedBanners();

      // 过滤已永久关闭的 Banner
      const activeBanners = banners.filter((b) => !dismissedBanners.includes(b.id));

      // 应用已读状态到 inbox
      const inboxWithReadState = inbox.map((n) => ({
        ...n,
        isRead: n.isRead || readIds.includes(n.id),
      }));

      set({
        banners: activeBanners,
        inbox: inboxWithReadState,
        unreadCount: calculateUnreadCount(inboxWithReadState),
        isLoading: false,
      });
    } catch (error) {
      set({
        error: (error as Error).message || "加载通知失败",
        isLoading: false,
      });
    }
  },

  dismissBanner: (id: string, permanent: boolean) => {
    set((state) => ({
      banners: state.banners.filter((b) => b.id !== id),
    }));

    if (permanent) {
      notificationStorage.dismissBanner(id);
    }
  },

  markAsRead: (id: string) => {
    set((state) => {
      const updatedInbox = state.inbox.map((n) =>
        n.id === id ? { ...n, isRead: true } : n
      );
      return {
        inbox: updatedInbox,
        unreadCount: calculateUnreadCount(updatedInbox),
      };
    });

    // 持久化到 localStorage
    notificationStorage.addReadId(id);
  },

  markAllAsRead: () => {
    const { inbox } = get();
    const unreadIds = inbox.filter((n) => !n.isRead).map((n) => n.id);

    set((state) => ({
      inbox: state.inbox.map((n) => ({ ...n, isRead: true })),
      unreadCount: 0,
    }));

    // 批量持久化
    notificationStorage.addReadIds(unreadIds);
  },

  setInboxOpen: (open: boolean) => {
    set({ inboxOpen: open });
  },

  executeAction: async (notificationId: string, actionId: string) => {
    const { inbox } = get();
    const notification = inbox.find((n) => n.id === notificationId);
    const action = notification?.actions?.find((a) => a.id === actionId);

    if (!action) return;

    try {
      switch (action.actionType) {
        case "api":
          // Mock API 调用
          // TODO: 替换为真实 API
          // await fetch(action.payload, { method: 'POST' });
          toast.success(`操作已执行: ${action.label}`);
          break;

        case "navigate":
          // 导航由组件层处理，这里只标记已读
          break;

        case "dismiss":
          // 关闭通知
          set((state) => ({
            inbox: state.inbox.filter((n) => n.id !== notificationId),
            unreadCount: calculateUnreadCount(
              state.inbox.filter((n) => n.id !== notificationId)
            ),
          }));
          break;
      }

      // 执行操作后标记已读
      get().markAsRead(notificationId);
    } catch (error) {
      toast.error(`操作失败: ${(error as Error).message}`);
    }
  },

  reset: () => {
    set(initialState);
    notificationStorage.clear();
  },

  clearError: () => {
    set({ error: null });
  },

  subscribe: () => {
    // TODO: 实现 WebSocket 实时推送
    // 预留结构：
    // const ws = new WebSocket('wss://api.mantra.dev/notifications/ws');
    // ws.onmessage = (event) => {
    //   const notification = JSON.parse(event.data);
    //   if (notification.category === 'banner') {
    //     set((state) => ({ banners: [notification, ...state.banners] }));
    //   } else {
    //     set((state) => ({
    //       inbox: [notification, ...state.inbox],
    //       unreadCount: state.unreadCount + 1,
    //     }));
    //   }
    // };
    // return () => ws.close();

    // MVP 阶段返回空的清理函数
    return () => {};
  },
}));

export default useNotificationStore;
