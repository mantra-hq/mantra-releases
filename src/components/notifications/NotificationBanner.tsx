/**
 * NotificationBanner - 单条 Banner 组件
 * Tech-Spec: 通知系统 Task 11
 *
 * 单条 Banner，含:
 * - 关闭按钮 + DropdownMenu（本次隐藏/永久隐藏）
 * - 高优先级用红色边框，普通用蓝色边框
 * - 进入/退出动画
 */

import * as React from "react";
import { X, ChevronDown } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { BannerNotification } from "@/types/notification";

/**
 * NotificationBanner Props
 */
export interface NotificationBannerProps {
  /** Banner 数据 */
  banner: BannerNotification;
  /** 关闭回调 */
  onDismiss: (permanent: boolean) => void;
  /** 是否正在退出 */
  isExiting?: boolean;
}

/**
 * NotificationBanner 组件
 */
export function NotificationBanner({
  banner,
  onDismiss,
  isExiting = false,
}: NotificationBannerProps) {
  const { t } = useTranslation();
  const isHighPriority = banner.priority === "high";

  // 处理快速关闭（本次隐藏）
  const handleQuickDismiss = React.useCallback(() => {
    onDismiss(false);
  }, [onDismiss]);

  // 处理永久隐藏
  const handlePermanentDismiss = React.useCallback(() => {
    onDismiss(true);
  }, [onDismiss]);

  return (
    <div
      className={cn(
        "relative flex items-start gap-3 px-4 py-3 rounded-lg border",
        // 优先级样式
        isHighPriority
          ? "bg-destructive/10 border-destructive/20"
          : "bg-blue-500/10 border-blue-500/20",
        // 动画 - 使用 data-state 模式
        "transition-all duration-200",
        !isExiting && "animate-in slide-in-from-top duration-300",
        isExiting && "opacity-0 scale-95 translate-y-[-10px]"
      )}
      data-state={isExiting ? "closed" : "open"}
      role="alert"
      aria-label={banner.title}
      data-testid={`notification-banner-${banner.id}`}
    >
      {/* 内容 */}
      <div className="flex-1 min-w-0">
        <h4 className="text-sm font-medium">{banner.title}</h4>
        <p className="text-sm text-muted-foreground mt-0.5">{banner.body}</p>
      </div>

      {/* 关闭按钮组 */}
      {banner.dismissible && (
        <div className="flex items-center shrink-0">
          {/* 快速关闭按钮 */}
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={handleQuickDismiss}
            aria-label={t("notifications.dismissBanner")}
          >
            <X className="h-4 w-4" />
          </Button>

          {/* 下拉菜单 */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 -ml-1"
                aria-label={t("notifications.moreOptions")}
              >
                <ChevronDown className="h-3 w-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={handleQuickDismiss}>
                {t("notifications.dismissOnce")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handlePermanentDismiss}>
                {t("notifications.dismissForever")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      )}
    </div>
  );
}
