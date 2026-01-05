/**
 * NotificationInbox - 消息收件箱组件
 * Tech-Spec: 通知系统 Task 10
 *
 * Sheet 侧栏组件（side="right", modal={false}）:
 * - 宽度 360px
 * - 无遮罩层，可同时操作其他区域
 * - 包含标题栏、"全部已读"按钮、消息列表、空状态
 */

import * as React from "react";
import * as SheetPrimitive from "@radix-ui/react-dialog";
import { X, CheckCheck, AlertCircle, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { useNotificationStore } from "@/stores/useNotificationStore";
import { NotificationCard } from "./NotificationCard";
import { NotificationEmpty } from "./NotificationEmpty";

/**
 * NotificationInbox 组件
 */
export function NotificationInbox() {
  const { t } = useTranslation();
  const inboxOpen = useNotificationStore((state) => state.inboxOpen);
  const setInboxOpen = useNotificationStore((state) => state.setInboxOpen);
  const inbox = useNotificationStore((state) => state.inbox);
  const unreadCount = useNotificationStore((state) => state.unreadCount);
  const isLoading = useNotificationStore((state) => state.isLoading);
  const error = useNotificationStore((state) => state.error);
  const fetchAll = useNotificationStore((state) => state.fetchAll);
  const clearError = useNotificationStore((state) => state.clearError);
  const markAsRead = useNotificationStore((state) => state.markAsRead);
  const markAllAsRead = useNotificationStore((state) => state.markAllAsRead);
  const executeAction = useNotificationStore((state) => state.executeAction);

  // 处理卡片点击
  const handleCardClick = React.useCallback(
    (notificationId: string) => {
      markAsRead(notificationId);
    },
    [markAsRead]
  );

  // 处理操作按钮点击
  const handleAction = React.useCallback(
    (notificationId: string, actionId: string) => {
      executeAction(notificationId, actionId);
    },
    [executeAction]
  );

  // 处理重试加载
  const handleRetry = React.useCallback(() => {
    clearError();
    fetchAll();
  }, [clearError, fetchAll]);

  return (
    <SheetPrimitive.Root open={inboxOpen} onOpenChange={setInboxOpen}>
      <SheetPrimitive.Portal>
        {/* 无遮罩层 - modal={false} 效果 */}
        <SheetPrimitive.Content
          className={cn(
            "bg-background fixed z-50 flex flex-col shadow-lg border-l",
            "inset-y-0 right-0 h-full w-[360px]",
            "data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right",
            "data-[state=open]:duration-[250ms] data-[state=closed]:duration-[250ms]",
            "ease-out"
          )}
          onInteractOutside={(e) => {
            // 阻止点击外部关闭，允许用户操作其他区域
            e.preventDefault();
          }}
          onPointerDownOutside={(e) => {
            e.preventDefault();
          }}
          data-testid="notification-inbox"
        >
          {/* 标题栏 */}
          <div className="flex items-center justify-between px-4 py-3 border-b shrink-0">
            <SheetPrimitive.Title className="text-base font-semibold">
              {t("notifications.title")}
            </SheetPrimitive.Title>
            <div className="flex items-center gap-1">
              {/* 全部已读按钮 */}
              {unreadCount > 0 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={markAllAsRead}
                  className="h-8 px-2 text-xs"
                  data-testid="mark-all-read-button"
                >
                  <CheckCheck className="h-4 w-4 mr-1" />
                  {t("notifications.markAllRead")}
                </Button>
              )}
              {/* 关闭按钮 */}
              <SheetPrimitive.Close asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8"
                  aria-label={t("common.close")}
                >
                  <X className="h-4 w-4" />
                </Button>
              </SheetPrimitive.Close>
            </div>
          </div>

          {/* 隐藏的描述（无障碍访问） */}
          <SheetPrimitive.Description className="sr-only">
            {t("notifications.inboxDescription")}
          </SheetPrimitive.Description>

          {/* 内容区域 */}
          <ScrollArea className="flex-1">
            {isLoading ? (
              // 加载状态骨架屏
              <div className="p-4 space-y-3">
                {[1, 2, 3].map((i) => (
                  <div key={i} className="flex gap-3">
                    <Skeleton className="h-9 w-9 rounded-full shrink-0" />
                    <div className="flex-1 space-y-2">
                      <Skeleton className="h-4 w-3/4" />
                      <Skeleton className="h-3 w-full" />
                      <div className="flex gap-2 pt-1">
                        <Skeleton className="h-7 w-16" />
                        <Skeleton className="h-7 w-16" />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : error ? (
              // 错误状态
              <div
                className="flex flex-col items-center justify-center h-full py-12 px-6 text-center"
                data-testid="notification-error"
              >
                <div className="w-14 h-14 mb-4 flex items-center justify-center rounded-xl bg-destructive/10">
                  <AlertCircle className="h-7 w-7 text-destructive" />
                </div>
                <p className="text-sm font-medium text-foreground mb-1">
                  {t("notifications.loadError")}
                </p>
                <p className="text-xs text-muted-foreground mb-4">
                  {error}
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleRetry}
                  className="gap-2"
                >
                  <RefreshCw className="h-4 w-4" />
                  {t("common.retry")}
                </Button>
              </div>
            ) : inbox.length === 0 ? (
              <NotificationEmpty />
            ) : (
              <div className="divide-y divide-border">
                {inbox.map((notification) => (
                  <NotificationCard
                    key={notification.id}
                    notification={notification}
                    onClick={() => handleCardClick(notification.id)}
                    onAction={(actionId) => handleAction(notification.id, actionId)}
                  />
                ))}
              </div>
            )}
          </ScrollArea>
        </SheetPrimitive.Content>
      </SheetPrimitive.Portal>
    </SheetPrimitive.Root>
  );
}
