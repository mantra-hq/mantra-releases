/**
 * NotificationBannerStack - Banner 堆叠容器
 * Tech-Spec: 通知系统 Task 12
 *
 * 堆叠容器:
 * - 最多显示 3 条
 * - 垂直排列
 * - z-index: 40（低于 Dialog/Sheet 的 50）
 * - 固定在视口顶部
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import { useNotificationStore } from "@/stores/useNotificationStore";
import { NotificationBanner } from "./NotificationBanner";

/** 最多显示的 Banner 数量 */
const MAX_VISIBLE_BANNERS = 3;

/**
 * NotificationBannerStack 组件
 */
export function NotificationBannerStack() {
  const banners = useNotificationStore((state) => state.banners);
  const dismissBanner = useNotificationStore((state) => state.dismissBanner);

  // 正在退出的 Banner ID 集合
  const [exitingIds, setExitingIds] = React.useState<Set<string>>(new Set());

  // 只显示最新的 3 条
  const visibleBanners = React.useMemo(() => {
    return banners
      .filter((b) => !exitingIds.has(b.id))
      .slice(0, MAX_VISIBLE_BANNERS);
  }, [banners, exitingIds]);

  // 处理关闭
  const handleDismiss = React.useCallback(
    (id: string, permanent: boolean) => {
      // 先标记为退出状态（播放退出动画）
      setExitingIds((prev) => new Set(prev).add(id));

      // 动画结束后真正移除
      setTimeout(() => {
        dismissBanner(id, permanent);
        setExitingIds((prev) => {
          const next = new Set(prev);
          next.delete(id);
          return next;
        });
      }, 200); // 与 fade-out 动画时长匹配
    },
    [dismissBanner]
  );

  // 没有 Banner 时不渲染
  if (visibleBanners.length === 0) {
    return null;
  }

  return (
    <div
      className={cn(
        "fixed top-0 left-0 right-0 z-40",
        "px-4 pt-2 pb-1",
        "pointer-events-none"
      )}
      data-testid="notification-banner-stack"
    >
      <div className="max-w-2xl mx-auto space-y-2 pointer-events-auto">
        {visibleBanners.map((banner) => (
          <NotificationBanner
            key={banner.id}
            banner={banner}
            onDismiss={(permanent) => handleDismiss(banner.id, permanent)}
            isExiting={exitingIds.has(banner.id)}
          />
        ))}
      </div>
    </div>
  );
}
