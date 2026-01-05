/**
 * Notification Types - 通知系统类型定义
 * Tech-Spec: 通知系统
 *
 * 定义通知相关的数据结构
 */

/**
 * 通知操作按钮
 */
export interface NotificationAction {
  /** 操作唯一标识 */
  id: string;
  /** 按钮文案 */
  label: string;
  /** 按钮样式 */
  variant: "primary" | "secondary" | "destructive";
  /** 操作类型 */
  actionType: "api" | "navigate" | "dismiss";
  /** API endpoint 或 路由路径 */
  payload: string;
}

/**
 * 通知类型枚举
 */
export type NotificationType =
  | "system"
  | "follow"
  | "comment"
  | "like"
  | "transaction"
  | "invite"
  | "review";

/**
 * 通知基类
 * 时间戳格式：ISO 8601
 */
export interface BaseNotification {
  /** 通知唯一标识 */
  id: string;
  /** 通知标题 */
  title: string;
  /** 通知正文 */
  body: string;
  /** 创建时间 (ISO 8601 格式，如 "2026-01-05T10:30:00Z") */
  createdAt: string;
}

/**
 * Banner 通知 (系统公告)
 */
export interface BannerNotification extends BaseNotification {
  /** 通知类别 */
  category: "banner";
  /** 是否可关闭 */
  dismissible: boolean;
  /** 优先级 */
  priority?: "high" | "normal";
  /** 过期时间 (ISO 8601 格式) */
  expiresAt?: string;
}

/**
 * Inbox 通知 (消息收件箱)
 */
export interface InboxNotification extends BaseNotification {
  /** 通知类别 */
  category: "inbox";
  /** 通知类型 */
  type: NotificationType;
  /** 是否已读 */
  isRead: boolean;
  /** 图标名称 */
  icon?: string;
  /** 操作按钮 */
  actions?: NotificationAction[];
  /** 跳转链接 */
  link?: string;
  /** 元数据 */
  metadata?: Record<string, unknown>;
}

/**
 * 通知联合类型
 */
export type Notification = BannerNotification | InboxNotification;
