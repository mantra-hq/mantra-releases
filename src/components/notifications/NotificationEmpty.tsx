/**
 * NotificationEmpty - 空状态组件
 * Tech-Spec: 通知系统 Task 7
 *
 * 当没有通知时显示的空状态
 */

import { BellOff } from "lucide-react";
import { useTranslation } from "react-i18next";

/**
 * NotificationEmpty 组件
 * 显示空状态图标和提示文案
 */
export function NotificationEmpty() {
  const { t } = useTranslation();

  return (
    <div
      className="flex flex-col items-center justify-center h-full py-12 px-6 text-center"
      data-testid="notification-empty"
    >
      <div className="w-14 h-14 mb-4 flex items-center justify-center rounded-xl bg-muted/50">
        <BellOff className="h-7 w-7 text-muted-foreground/70" />
      </div>
      <p className="text-sm font-medium text-foreground mb-1">
        {t("notifications.empty")}
      </p>
      <p className="text-xs text-muted-foreground">
        {t("notifications.emptyHint")}
      </p>
    </div>
  );
}
