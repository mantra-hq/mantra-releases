/**
 * Mock Notifications - 通知系统模拟数据
 * Tech-Spec: 通知系统 Task 3
 *
 * 覆盖所有 7 种消息类型 + Banner 数据
 */

import type { BannerNotification, InboxNotification } from "@/types/notification";

/**
 * Mock Banner 通知数据
 */
export const mockBanners: BannerNotification[] = [
  {
    id: "banner-1",
    category: "banner",
    title: "🎉 Mantra v1.0 正式发布",
    body: "全球首个 AI 编程过程资产分享社区现已上线！立即探索海量编程心法。",
    createdAt: "2026-01-05T08:00:00Z",
    dismissible: true,
    priority: "high",
  },
  {
    id: "banner-2",
    category: "banner",
    title: "📢 系统维护通知",
    body: "计划于 1 月 10 日 02:00-04:00 进行系统升级，届时服务可能暂时中断。",
    createdAt: "2026-01-04T10:00:00Z",
    dismissible: true,
    priority: "normal",
    expiresAt: "2026-01-10T04:00:00Z",
  },
  {
    id: "banner-3",
    category: "banner",
    title: "💡 新功能上线：代码片段分享",
    body: "现在可以将会话中的精彩代码片段一键分享给社区。",
    createdAt: "2026-01-03T14:00:00Z",
    dismissible: true,
    priority: "normal",
  },
];

/**
 * Mock Inbox 通知数据
 * 覆盖所有 7 种消息类型
 */
export const mockInboxNotifications: InboxNotification[] = [
  // system - 系统公告
  {
    id: "inbox-1",
    category: "inbox",
    type: "system",
    title: "账户安全提醒",
    body: "检测到您的账户在新设备上登录，如非本人操作请及时修改密码。",
    createdAt: "2026-01-05T09:30:00Z",
    isRead: false,
    icon: "Shield",
    actions: [
      {
        id: "view-detail",
        label: "查看详情",
        variant: "primary",
        actionType: "navigate",
        payload: "/settings/security",
      },
    ],
  },
  // follow - 关注通知
  {
    id: "inbox-2",
    category: "inbox",
    type: "follow",
    title: "新粉丝",
    body: "Alex Chen 关注了你",
    createdAt: "2026-01-05T08:45:00Z",
    isRead: false,
    icon: "UserPlus",
    actions: [
      {
        id: "follow-back",
        label: "回关",
        variant: "primary",
        actionType: "api",
        payload: "/api/v1/users/alex-chen/follow",
      },
      {
        id: "view-profile",
        label: "查看主页",
        variant: "secondary",
        actionType: "navigate",
        payload: "/users/alex-chen",
      },
    ],
    link: "/users/alex-chen",
    metadata: { userId: "alex-chen", avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=alex" },
  },
  // comment - 评论回复
  {
    id: "inbox-3",
    category: "inbox",
    type: "comment",
    title: "新评论",
    body: 'Emma Wang 评论了你的心法「React 性能优化实战」：这个 useMemo 的用法太实用了！',
    createdAt: "2026-01-05T07:20:00Z",
    isRead: false,
    icon: "MessageCircle",
    actions: [
      {
        id: "reply",
        label: "回复",
        variant: "primary",
        actionType: "navigate",
        payload: "/mantras/react-perf-001?comment=reply",
      },
      {
        id: "view-context",
        label: "查看上下文",
        variant: "secondary",
        actionType: "navigate",
        payload: "/mantras/react-perf-001#comment-123",
      },
    ],
    link: "/mantras/react-perf-001#comment-123",
  },
  // like - 点赞收藏
  {
    id: "inbox-4",
    category: "inbox",
    type: "like",
    title: "收到点赞",
    body: "你的心法「TypeScript 高级类型体操」获得了 10 个新点赞",
    createdAt: "2026-01-04T22:15:00Z",
    isRead: true,
    icon: "Heart",
    actions: [
      {
        id: "view",
        label: "查看",
        variant: "secondary",
        actionType: "navigate",
        payload: "/mantras/ts-types-001",
      },
    ],
    link: "/mantras/ts-types-001",
  },
  // transaction - 交易通知
  {
    id: "inbox-5",
    category: "inbox",
    type: "transaction",
    title: "交易成功",
    body: "用户 Mike Lee 购买了你的心法「全栈 AI 应用开发」，收入 ¥29.00",
    createdAt: "2026-01-04T18:30:00Z",
    isRead: false,
    icon: "Wallet",
    actions: [
      {
        id: "view-order",
        label: "查看订单",
        variant: "primary",
        actionType: "navigate",
        payload: "/orders/order-456",
      },
    ],
    link: "/orders/order-456",
    metadata: { orderId: "order-456", amount: 29.0, currency: "CNY" },
  },
  // invite - 邀请协作
  {
    id: "inbox-6",
    category: "inbox",
    type: "invite",
    title: "协作邀请",
    body: "Sophie Zhang 邀请你加入项目「AI 编程助手开发」",
    createdAt: "2026-01-04T15:00:00Z",
    isRead: false,
    icon: "Users",
    actions: [
      {
        id: "accept",
        label: "接受",
        variant: "primary",
        actionType: "api",
        payload: "/api/v1/invites/inv-789/accept",
      },
      {
        id: "decline",
        label: "拒绝",
        variant: "destructive",
        actionType: "api",
        payload: "/api/v1/invites/inv-789/decline",
      },
    ],
    metadata: { inviteId: "inv-789", projectId: "proj-ai-assistant" },
  },
  // review - 审核结果
  {
    id: "inbox-7",
    category: "inbox",
    type: "review",
    title: "审核通过",
    body: "你的心法「Rust 并发编程精要」已通过审核，现已上架",
    createdAt: "2026-01-04T10:00:00Z",
    isRead: true,
    icon: "CheckCircle",
    actions: [
      {
        id: "view-detail",
        label: "查看详情",
        variant: "primary",
        actionType: "navigate",
        payload: "/mantras/rust-concurrency-001",
      },
    ],
    link: "/mantras/rust-concurrency-001",
  },
  // 额外的未读通知用于测试
  {
    id: "inbox-8",
    category: "inbox",
    type: "comment",
    title: "新评论",
    body: 'David Liu 回复了你的评论：完全同意，这种模式在大型项目中特别有用。',
    createdAt: "2026-01-03T16:45:00Z",
    isRead: false,
    icon: "MessageCircle",
    actions: [
      {
        id: "reply",
        label: "回复",
        variant: "primary",
        actionType: "navigate",
        payload: "/mantras/react-perf-001?comment=reply",
      },
    ],
  },
  {
    id: "inbox-9",
    category: "inbox",
    type: "follow",
    title: "新粉丝",
    body: "Jessica Wu 关注了你",
    createdAt: "2026-01-03T12:00:00Z",
    isRead: true,
    icon: "UserPlus",
    actions: [
      {
        id: "follow-back",
        label: "回关",
        variant: "primary",
        actionType: "api",
        payload: "/api/v1/users/jessica-wu/follow",
      },
    ],
  },
  {
    id: "inbox-10",
    category: "inbox",
    type: "review",
    title: "审核未通过",
    body: "你的心法「xxx」未通过审核，请修改后重新提交",
    createdAt: "2026-01-02T09:00:00Z",
    isRead: false,
    icon: "XCircle",
    actions: [
      {
        id: "view-detail",
        label: "查看详情",
        variant: "primary",
        actionType: "navigate",
        payload: "/mantras/draft-001/review",
      },
      {
        id: "appeal",
        label: "申诉",
        variant: "secondary",
        actionType: "navigate",
        payload: "/mantras/draft-001/appeal",
      },
    ],
  },
];

/**
 * 获取所有 Mock 通知
 */
export function getMockNotifications() {
  return {
    banners: mockBanners,
    inbox: mockInboxNotifications,
  };
}
