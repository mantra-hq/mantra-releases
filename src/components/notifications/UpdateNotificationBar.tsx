/**
 * UpdateNotificationBar - 更新就绪通知条
 * Story 14.6: AC #1-#6
 *
 * 非侵入式通知条，在应用顶部显示更新就绪状态。
 * 消费 useUpdateChecker Hook 的状态。
 */

import { useState, useCallback, useRef, useEffect } from "react";
import { Download, RefreshCw, FileText, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import type { UpdateStatus, UpdateInfo } from "@/hooks/useUpdateChecker";

export interface UpdateNotificationBarProps {
  updateStatus: UpdateStatus;
  updateInfo: UpdateInfo | null;
  onRestart: () => Promise<void>;
  onDismiss: () => void;
}

export function UpdateNotificationBar({
  updateStatus,
  updateInfo,
  onRestart,
  onDismiss,
}: UpdateNotificationBarProps) {
  const { t } = useTranslation();
  const [showNotes, setShowNotes] = useState(false);
  const [isExiting, setIsExiting] = useState(false);
  const dismissTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // [H2 fix] Cleanup dismiss timer on unmount to prevent setState on unmounted component
  useEffect(() => {
    return () => {
      if (dismissTimerRef.current) {
        clearTimeout(dismissTimerRef.current);
        dismissTimerRef.current = null;
      }
    };
  }, []);

  const handleDismiss = useCallback(() => {
    setIsExiting(true);
    // Wait for exit animation before calling onDismiss
    dismissTimerRef.current = setTimeout(() => {
      dismissTimerRef.current = null;
      onDismiss();
    }, 200);
  }, [onDismiss]);

  // [H1 fix] Wrap async onRestart to catch unhandled rejections
  const handleRestart = useCallback(async () => {
    try {
      await onRestart();
    } catch (err) {
      console.error('[UpdateNotificationBar] restart failed:', err);
    }
  }, [onRestart]);

  const handleToggleNotes = useCallback(() => {
    setShowNotes((prev) => !prev);
  }, []);

  // AC #3: Only show when updateStatus === 'ready'
  if (updateStatus !== "ready") {
    return null;
  }

  return (
    <div
      className={cn(
        "relative flex flex-col",
        "bg-card border border-primary/50 rounded-lg",
        // Left color bar
        "before:absolute before:left-0 before:top-0 before:bottom-0 before:w-1 before:rounded-l-lg before:bg-primary",
        // Animation
        "transition-all duration-200",
        !isExiting && "animate-in slide-in-from-top duration-300",
        isExiting && "opacity-0 scale-95 translate-y-[-10px]"
      )}
      data-state={isExiting ? "closed" : "open"}
      data-testid="update-notification-bar"
      role="status"
      aria-label={t("updater.readyToInstall", { version: updateInfo?.version ?? "" })}
    >
      {/* Main row */}
      <div className="flex items-center gap-3 px-4 py-3">
        {/* Icon */}
        <Download className="h-4 w-4 text-primary shrink-0" />

        {/* Text */}
        <span className="text-sm text-foreground flex-1">
          {t("updater.readyToInstall", { version: updateInfo?.version ?? "" })}
        </span>

        {/* Action buttons */}
        <div className="flex items-center gap-1 shrink-0">
          {/* Release Notes button - only show when body exists */}
          {updateInfo?.body && (
            <Button
              variant="ghost"
              size="sm"
              className="h-7 text-xs"
              onClick={handleToggleNotes}
              data-testid="update-release-notes-btn"
            >
              <FileText className="h-3.5 w-3.5 mr-1" />
              {t("updater.releaseNotes")}
            </Button>
          )}

          {/* Restart to Update button */}
          <Button
            variant="default"
            size="sm"
            className="h-7 text-xs"
            onClick={handleRestart}
            data-testid="update-restart-btn"
          >
            <RefreshCw className="h-3.5 w-3.5 mr-1" />
            {t("updater.restartToUpdate")}
          </Button>

          {/* Dismiss button */}
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={handleDismiss}
            aria-label={t("updater.dismiss")}
            data-testid="update-dismiss-btn"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Release notes expandable area */}
      {showNotes && updateInfo?.body && (
        <div
          className="px-4 pb-3 text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto border-t border-border/50 pt-2 ml-1"
          data-testid="update-release-notes-content"
        >
          {updateInfo.body}
        </div>
      )}
    </div>
  );
}
