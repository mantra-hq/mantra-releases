/**
 * LogicalProjectContextMenu Component - 逻辑项目上下文菜单
 * Story 1.12: Phase 6 - Task 13, 14
 *
 * 逻辑项目管理菜单：
 * - 同步更新（同步所有关联的存储层项目）
 * - 重命名（仅单项目时启用）
 * - 查看详情
 * - 移除
 *
 * Task 13/14: 不包含关联操作，关联只通过详情页进行
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw, Trash2, Loader2, Settings, Info, Pencil } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { LogicalProjectStats } from "@/types/project";

/**
 * LogicalProjectContextMenu Props
 */
export interface LogicalProjectContextMenuProps {
  /** 逻辑项目信息 */
  logicalProject: LogicalProjectStats;
  /** 同步回调 */
  onSync: () => Promise<void>;
  /** 重命名回调（仅单项目时启用） */
  onRename?: () => void;
  /** 移除回调 */
  onRemove: () => void;
  /** 查看详情回调 */
  onViewInfo: () => void;
  /** 菜单打开状态变化 */
  onOpenChange?: (open: boolean) => void;
}

/**
 * LogicalProjectContextMenu 组件
 * 逻辑项目管理菜单
 */
export function LogicalProjectContextMenu({
  logicalProject,
  onSync,
  onRename,
  onRemove,
  onViewInfo,
  onOpenChange,
}: LogicalProjectContextMenuProps) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = React.useState(false);
  const [isSyncing, setIsSyncing] = React.useState(false);

  // 仅单项目时启用重命名
  const canRename = logicalProject.project_count === 1 && !!onRename;

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
    onRename?.();
    setIsOpen(false);
  };

  const handleRemove = (e: React.MouseEvent) => {
    e.preventDefault();
    onRemove();
    setIsOpen(false);
  };

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
          data-testid="logical-project-context-menu-trigger"
        >
          <Settings className="h-3.5 w-3.5 text-muted-foreground" />
        </button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align="start" className="w-48">
        {/* 同步更新 */}
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
          {/* 多来源提示 */}
          {logicalProject.project_count > 1 && (
            <span className="ml-1 text-xs text-muted-foreground">
              ({logicalProject.project_count})
            </span>
          )}
        </DropdownMenuItem>

        {/* 查看详情 - Task 15: 关联入口统一到详情页 */}
        {/* 始终显示"查看详情"，关联操作在详情页内进行 */}
        <DropdownMenuItem onClick={handleViewInfo}>
          <Info className="h-4 w-4 mr-2" />
          {t("projectInfo.viewDetails", "查看详情")}
        </DropdownMenuItem>

        {/* 重命名 - 仅单项目时启用 */}
        {canRename && (
          <DropdownMenuItem onClick={handleRename}>
            <Pencil className="h-4 w-4 mr-2" />
            {t("project.rename")}
          </DropdownMenuItem>
        )}

        {/* 分隔线 */}
        <DropdownMenuSeparator />

        {/* 移除 - 危险操作 */}
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
