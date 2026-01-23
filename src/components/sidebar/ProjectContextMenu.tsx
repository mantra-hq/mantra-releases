/**
 * ProjectContextMenu Component - 项目上下文菜单
 * Story 2.19: Task 1
 * Story 2.26: 国际化支持
 * Story 1.12: 移除强制解析选项（已弃用）
 *
 * 项目管理菜单，包含同步、重命名、移除操作
 * 注意：此组件保留用于向后兼容，新代码请使用 LogicalProjectContextMenu
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw, Pencil, Trash2, Loader2, Settings, Info } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

/**
 * ProjectContextMenu Props
 */
export interface ProjectContextMenuProps {
  /** 同步回调（增量同步） */
  onSync: () => Promise<void>;
  /** 重命名回调 */
  onRename: () => void;
  /** 移除回调 */
  onRemove: () => void;
  /** 查看详情回调 (Story 2.27 AC1) */
  onViewInfo: () => void;
  /** 菜单打开状态变化 */
  onOpenChange?: (open: boolean) => void;
}

/**
 * ProjectContextMenu 组件
 * 项目管理菜单，包含同步更新、重命名、从 Mantra 移除操作
 */
export function ProjectContextMenu({
  onSync,
  onRename,
  onRemove,
  onViewInfo,
  onOpenChange,
}: ProjectContextMenuProps) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = React.useState(false);
  const [isSyncing, setIsSyncing] = React.useState(false);

  const handleOpenChange = (open: boolean) => {
    setIsOpen(open);
    onOpenChange?.(open);
  };

  const handleSync = async (e: React.MouseEvent) => {
    e.preventDefault();
    setIsSyncing(true);
    try {
      await onSync();
    } finally {
      setIsSyncing(false);
      setIsOpen(false);
    }
  };

  const handleRename = (e: React.MouseEvent) => {
    e.preventDefault();
    onRename();
    setIsOpen(false);
  };

  const handleRemove = (e: React.MouseEvent) => {
    e.preventDefault();
    onRemove();
    setIsOpen(false);
  };

  // Story 2.27 AC1: 处理查看详情
  const handleViewInfo = (e: React.MouseEvent) => {
    e.preventDefault();
    onViewInfo();
    setIsOpen(false);
  };

  return (
    <DropdownMenu open={isOpen} onOpenChange={handleOpenChange}>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className="h-6 w-6 flex items-center justify-center rounded-sm hover:bg-muted shrink-0 cursor-pointer"
          aria-label={t("project.settings")}
          data-testid="project-context-menu-trigger"
        >
          <Settings className="h-3.5 w-3.5 text-muted-foreground" />
        </button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align="start" className="w-48">
        {/* 同步更新 (AC2) */}
        <DropdownMenuItem onClick={handleSync} disabled={isSyncing}>
          {isSyncing ? (
            <Loader2
              className="h-4 w-4 mr-2 animate-spin"
              data-testid="sync-loading"
            />
          ) : (
            <RefreshCw className="h-4 w-4 mr-2" />
          )}
          {t("project.syncUpdate")}
        </DropdownMenuItem>

        {/* Story 2.27 AC1: 查看详情 */}
        <DropdownMenuItem onClick={handleViewInfo}>
          <Info className="h-4 w-4 mr-2" />
          {t("projectInfo.viewDetails", "查看详情")}
        </DropdownMenuItem>

        {/* 重命名 (AC2) */}
        <DropdownMenuItem onClick={handleRename}>
          <Pencil className="h-4 w-4 mr-2" />
          {t("project.rename")}
        </DropdownMenuItem>

        {/* 分隔线 */}
        <DropdownMenuSeparator />

        {/* 从 Mantra 移除 (AC2, AC3) - 危险操作 */}
        <DropdownMenuItem onClick={handleRemove} variant="destructive">
          <div className="flex flex-col items-start">
            <div className="flex items-center">
              <Trash2 className="h-4 w-4 mr-2" />
              {t("project.removeFromMantra")}
            </div>
            <span className="text-xs text-muted-foreground ml-6">
              {t("project.removeNote")}
            </span>
          </div>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
