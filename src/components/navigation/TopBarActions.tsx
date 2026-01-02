/**
 * TopBarActions Component - TopBar 右侧操作按钮
 * Story 2.17: Task 4
 *
 * 包含同步按钮、导入按钮和主题切换
 */

import { RefreshCw, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ThemeToggle } from "@/components/theme-toggle";
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
  /** 同步回调 (AC10) */
  onSync: () => void;
  /** 导入回调 (AC11) */
  onImport: () => void;
  /** 是否正在同步 */
  isSyncing?: boolean;
}

/**
 * TopBarActions 组件
 * TopBar 右侧操作按钮组
 */
export function TopBarActions({
  onSync,
  onImport,
  isSyncing = false,
}: TopBarActionsProps) {
  return (
    <div
      className="flex items-center gap-1 shrink-0"
      data-testid="topbar-actions"
    >
      <TooltipProvider delayDuration={300}>
        {/* 同步按钮 (AC10) */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={onSync}
              disabled={isSyncing}
              aria-label="同步项目"
              data-testid="topbar-sync-button"
              className="h-8 w-8"
            >
              <RefreshCw
                className={`h-4 w-4 ${isSyncing ? "animate-spin" : ""}`}
              />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>同步项目</p>
          </TooltipContent>
        </Tooltip>

        {/* 导入按钮 (AC11) */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={onImport}
              aria-label="导入会话"
              data-testid="topbar-import-button"
              className="h-8 w-8"
            >
              <Plus className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>导入会话</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>

      {/* 主题切换 (AC12) */}
      <ThemeToggle />
    </div>
  );
}
