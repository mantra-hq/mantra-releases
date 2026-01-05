/**
 * NotificationCard - 消息卡片组件
 * Tech-Spec: 通知系统 Task 8
 *
 * 显示单条通知消息，支持:
 * - 图标、标题、正文、时间戳
 * - 未读标记（左侧蓝色圆点）
 * - 内嵌操作按钮
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { formatDistanceToNow } from "date-fns";
import { zhCN, enUS } from "date-fns/locale";
import * as Icons from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import type { InboxNotification, NotificationAction } from "@/types/notification";

/**
 * NotificationCard Props
 */
export interface NotificationCardProps {
  /** 通知数据 */
  notification: InboxNotification;
  /** 点击卡片回调 */
  onClick?: () => void;
  /** 执行操作回调 */
  onAction?: (actionId: string) => void;
}

/**
 * 动态获取 Lucide 图标
 */
function getIcon(iconName?: string): React.ElementType {
  if (!iconName) return Icons.Bell;
  // 使用类型断言获取图标
  const IconMap: Record<string, React.ElementType> = {
    Shield: Icons.Shield,
    UserPlus: Icons.UserPlus,
    MessageCircle: Icons.MessageCircle,
    Heart: Icons.Heart,
    Wallet: Icons.Wallet,
    Users: Icons.Users,
    CheckCircle: Icons.CheckCircle,
    XCircle: Icons.XCircle,
    Bell: Icons.Bell,
    Megaphone: Icons.Megaphone,
  };
  return IconMap[iconName] || Icons.Bell;
}

/**
 * 获取通知类型对应的默认图标
 */
function getTypeIcon(type: InboxNotification["type"]): React.ElementType {
  const iconMap: Record<InboxNotification["type"], React.ElementType> = {
    system: Icons.Megaphone,
    follow: Icons.UserPlus,
    comment: Icons.MessageCircle,
    like: Icons.Heart,
    transaction: Icons.Wallet,
    invite: Icons.Users,
    review: Icons.CheckCircle,
  };
  return iconMap[type] || Icons.Bell;
}

/**
 * 操作按钮变体映射
 */
function getButtonVariant(variant: NotificationAction["variant"]) {
  const variantMap: Record<NotificationAction["variant"], "default" | "secondary" | "destructive"> = {
    primary: "default",
    secondary: "secondary",
    destructive: "destructive",
  };
  return variantMap[variant];
}

/**
 * NotificationCard 组件
 */
export function NotificationCard({
  notification,
  onClick,
  onAction,
}: NotificationCardProps) {
  const { i18n, t } = useTranslation();
  const navigate = useNavigate();

  // 格式化相对时间
  const relativeTime = React.useMemo(() => {
    try {
      return formatDistanceToNow(new Date(notification.createdAt), {
        addSuffix: true,
        locale: i18n.language === "zh-CN" ? zhCN : enUS,
      });
    } catch {
      return t("time.unknownTime");
    }
  }, [notification.createdAt, i18n.language, t]);

  // 获取图标组件
  const IconComponent = notification.icon
    ? getIcon(notification.icon)
    : getTypeIcon(notification.type);

  // 处理卡片点击
  const handleClick = React.useCallback(() => {
    onClick?.();
    if (notification.link) {
      navigate(notification.link);
    }
  }, [onClick, notification.link, navigate]);

  // 处理操作按钮点击
  const handleActionClick = React.useCallback(
    (e: React.MouseEvent, action: NotificationAction) => {
      e.stopPropagation();
      onAction?.(action.id);
      if (action.actionType === "navigate") {
        navigate(action.payload);
      }
    },
    [onAction, navigate]
  );

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={handleClick}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleClick();
        }
      }}
      className={cn(
        "relative flex gap-3 px-4 py-3",
        "hover:bg-muted/50 transition-colors cursor-pointer",
        "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-inset",
        "border-b border-border last:border-b-0"
      )}
      aria-label={`${notification.title}: ${notification.body}`}
      data-testid={`notification-card-${notification.id}`}
    >
      {/* 未读标记 */}
      {!notification.isRead && (
        <span
          className="absolute left-1.5 top-1/2 -translate-y-1/2 w-2 h-2 rounded-full bg-blue-500"
          aria-label={t("notifications.unread")}
        />
      )}

      {/* 图标 */}
      <div className="shrink-0 w-9 h-9 flex items-center justify-center rounded-full bg-muted">
        <IconComponent className="h-4 w-4 text-muted-foreground" />
      </div>

      {/* 内容 */}
      <div className="flex-1 min-w-0">
        <div className="flex items-start justify-between gap-2">
          <h4 className={cn(
            "text-sm truncate",
            !notification.isRead && "font-medium"
          )}>
            {notification.title}
          </h4>
          <span className="text-xs text-muted-foreground shrink-0">
            {relativeTime}
          </span>
        </div>
        <p className="text-sm text-muted-foreground line-clamp-2 mt-0.5">
          {notification.body}
        </p>

        {/* 操作按钮 */}
        {notification.actions && notification.actions.length > 0 && (
          <div className="flex items-center gap-2 mt-2">
            {notification.actions.map((action) => (
              <Button
                key={action.id}
                variant={getButtonVariant(action.variant)}
                size="sm"
                className="h-7 px-2.5 text-xs"
                onClick={(e) => handleActionClick(e, action)}
              >
                {action.label}
              </Button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
