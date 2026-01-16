/**
 * TopBarActions Component - TopBar 右侧操作按钮
 * Story 2.17: Task 4
 * Story 2.21: Task 4.2-4.4 (添加全局搜索按钮、设置按钮)
 * Story 2-26: i18n 国际化
 * Story 3.4: 脱敏预览主视图原生模式
 * Tech-Spec: 通知系统 Task 13
 *
 * 按钮顺序：搜索 → 脱敏预览 → 同步 → 导入 → 通知 → 设置 → 主题切换
 */

import { useEffect } from "react";
import { RefreshCw, Plus, Search, Settings, Shield } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { ThemeToggle } from "@/components/theme-toggle";
import { ModeSwitch } from "@/components/layout/ModeSwitch";
import { useSearchStore } from "@/stores/useSearchStore";
import { useSanitizePreviewStore } from "@/stores/useSanitizePreviewStore";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

/**
 * TopBarActions Props
 */
export interface TopBarActionsProps {
  /** 当前会话 ID (用于脱敏预览) */
  sessionId?: string;
  /** 同步回调 (AC10) */
  onSync: () => void;
  /** 导入回调 (AC11) */
  onImport: () => void;
  /** 是否正在同步 */
  isSyncing?: boolean;
  /** 是否显示同步按钮 - 空状态时隐藏 */
  showSync?: boolean;
}

/**
 * TopBarActions 组件
 * TopBar 右侧操作按钮组
 */
export function TopBarActions({
  sessionId,
  onSync,
  onImport,
  isSyncing = false,
  showSync = true,
}: TopBarActionsProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const openSearch = useSearchStore((state) => state.open);

  // 脱敏预览 Store
  const mode = useSanitizePreviewStore((state) => state.mode);
  const isLoading = useSanitizePreviewStore((state) => state.isLoading);
  const setSessionId = useSanitizePreviewStore((state) => state.setSessionId);
  const enterPreviewMode = useSanitizePreviewStore((state) => state.enterPreviewMode);

  // 同步 sessionId 到 store
  useEffect(() => {
    setSessionId(sessionId ?? null);
  }, [sessionId, setSessionId]);

  // 是否处于预览模式
  const isInPreviewMode = mode === 'preview';

  return (
    <div
      className="flex items-center gap-1 shrink-0"
      data-testid="topbar-actions"
    >
      <TooltipProvider delayDuration={300}>
        {/* 全局搜索按钮 (Story 2.21 AC #15) */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={openSearch}
              aria-label={t("topbar.globalSearchShortcut")}
              data-testid="topbar-search-button"
              className="h-8 w-8"
            >
              <Search className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t("topbar.globalSearchShortcut")}</p>
          </TooltipContent>
        </Tooltip>

        {/* 脱敏预览按钮 - 有会话时显示 (AC3, AC4) */}
        {sessionId && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={isInPreviewMode ? "secondary" : "ghost"}
                size="sm"
                onClick={enterPreviewMode}
                disabled={isLoading || isInPreviewMode}
                aria-label={t("topbar.securityCheck")}
                data-testid="topbar-sanitize-button"
                className="h-8 gap-1.5 px-2"
              >
                <Shield className="h-4 w-4" />
                <span className="text-sm">{t("topbar.securityCheck")}</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p>{t("topbar.securityCheckTooltip")}</p>
            </TooltipContent>
          </Tooltip>
        )}

        {/* 同步按钮 (AC10) - 空状态时隐藏 */}
        {showSync && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={onSync}
                disabled={isSyncing}
                aria-label={t("sync.syncProject")}
                data-testid="topbar-sync-button"
                className="h-8 w-8"
              >
                <RefreshCw
                  className={`h-4 w-4 ${isSyncing ? "animate-spin" : ""}`}
                />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p>{t("sync.syncProject")}</p>
            </TooltipContent>
          </Tooltip>
        )}

        {/* 导入按钮 (AC11) */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={onImport}
              aria-label={t("import.importSession")}
              data-testid="topbar-import-button"
              className="h-8 w-8"
            >
              <Plus className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t("import.importSession")}</p>
          </TooltipContent>
        </Tooltip>

        {/* TODO: 通知功能暂未开放 */}
        {/* <NotificationBell /> */}

        {/* 设置按钮 (Story 2.21 AC #16) */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => navigate("/settings")}
              aria-label={t("settings.title")}
              data-testid="topbar-settings-button"
              className="h-8 w-8"
            >
              <Settings className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t("settings.title")}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>

      {/* Story 2.34: 模式切换 */}
      <ModeSwitch />

      {/* 主题切换 (AC12) */}
      <ThemeToggle />

      {/* TODO: 通知功能暂未开放 */}
      {/* <NotificationInbox /> */}
    </div>
  );
}
