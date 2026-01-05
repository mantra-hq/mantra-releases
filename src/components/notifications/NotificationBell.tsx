/**
 * NotificationBell - 铃铛图标组件
 * Tech-Spec: 通知系统 Task 9
 *
 * 导航栏铃铛按钮，含未读角标:
 * - 显示具体数字
 * - 超过 9 显示 "9+"
 */

import { Bell } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useNotificationStore } from "@/stores/useNotificationStore";

/**
 * NotificationBell Props
 */
export interface NotificationBellProps {
  /** 自定义类名 */
  className?: string;
}

/**
 * NotificationBell 组件
 */
export function NotificationBell({ className }: NotificationBellProps) {
  const { t } = useTranslation();
  const unreadCount = useNotificationStore((state) => state.unreadCount);
  const setInboxOpen = useNotificationStore((state) => state.setInboxOpen);

  // 格式化未读数显示
  const displayCount = unreadCount > 9 ? "9+" : unreadCount.toString();

  const handleClick = () => {
    setInboxOpen(true);
  };

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleClick}
          aria-label={t("notifications.openInbox", { count: unreadCount })}
          data-testid="notification-bell"
          className={cn("h-8 w-8 relative", className)}
        >
          <Bell className="h-4 w-4" />
          {unreadCount > 0 && (
            <span
              className={cn(
                "absolute -top-1 -right-1",
                "min-w-[20px] h-[20px] px-1.5",
                "flex items-center justify-center",
                "text-[11px] font-semibold text-white",
                "bg-red-500 rounded-full",
                "pointer-events-none",
                "ring-2 ring-background"
              )}
              data-testid="notification-badge"
              aria-hidden="true"
            >
              {displayCount}
            </span>
          )}
        </Button>
      </TooltipTrigger>
      <TooltipContent side="bottom">
        <p>
          {unreadCount > 0
            ? t("notifications.unreadCount", { count: unreadCount })
            : t("notifications.title")}
        </p>
      </TooltipContent>
    </Tooltip>
  );
}
