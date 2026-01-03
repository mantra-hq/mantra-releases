/**
 * ProjectContextMenu Component - 项目上下文菜单
 * Story 2.19: Task 1
 *
 * 项目管理菜单，包含同步、重命名、移除操作
 */

import * as React from "react";
import { RefreshCw, RotateCcw, Pencil, Trash2, Loader2, Settings } from "lucide-react";
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
  /** 强制重新解析回调（完全重新解析所有会话） */
  onForceSync: () => Promise<void>;
  /** 重命名回调 */
  onRename: () => void;
  /** 移除回调 */
  onRemove: () => void;
  /** 菜单打开状态变化 */
  onOpenChange?: (open: boolean) => void;
}

/**
 * ProjectContextMenu 组件
 * 项目管理菜单，包含同步更新、重命名、从 Mantra 移除操作
 */
export function ProjectContextMenu({
  onSync,
  onForceSync,
  onRename,
  onRemove,
  onOpenChange,
}: ProjectContextMenuProps) {
  const [isOpen, setIsOpen] = React.useState(false);
  const [isSyncing, setIsSyncing] = React.useState(false);
  const [isForceSyncing, setIsForceSyncing] = React.useState(false);

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

  const handleForceSync = async (e: React.MouseEvent) => {
    e.preventDefault();
    setIsForceSyncing(true);
    try {
      await onForceSync();
    } finally {
      setIsForceSyncing(false);
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

  return (
    <DropdownMenu open={isOpen} onOpenChange={handleOpenChange}>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className="h-6 w-6 flex items-center justify-center rounded-sm hover:bg-muted shrink-0"
          aria-label="项目设置"
          data-testid="project-context-menu-trigger"
        >
          <Settings className="h-3.5 w-3.5 text-muted-foreground" />
        </button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align="start" className="w-48">
        {/* 同步更新 (AC2) */}
        <DropdownMenuItem onClick={handleSync} disabled={isSyncing || isForceSyncing}>
          {isSyncing ? (
            <Loader2
              className="h-4 w-4 mr-2 animate-spin"
              data-testid="sync-loading"
            />
          ) : (
            <RefreshCw className="h-4 w-4 mr-2" />
          )}
          同步更新
        </DropdownMenuItem>

        {/* 强制重新解析 - 用于修复解析 bug 后恢复数据 */}
        <DropdownMenuItem onClick={handleForceSync} disabled={isSyncing || isForceSyncing}>
          {isForceSyncing ? (
            <Loader2
              className="h-4 w-4 mr-2 animate-spin"
              data-testid="force-sync-loading"
            />
          ) : (
            <RotateCcw className="h-4 w-4 mr-2" />
          )}
          <div className="flex flex-col items-start">
            <span>强制重新解析</span>
            <span className="text-xs text-muted-foreground">
              重新解析所有会话
            </span>
          </div>
        </DropdownMenuItem>

        {/* 重命名 (AC2) */}
        <DropdownMenuItem onClick={handleRename}>
          <Pencil className="h-4 w-4 mr-2" />
          重命名
        </DropdownMenuItem>

        {/* 分隔线 */}
        <DropdownMenuSeparator />

        {/* 从 Mantra 移除 (AC2, AC3) - 危险操作 */}
        <DropdownMenuItem onClick={handleRemove} variant="destructive">
          <div className="flex flex-col items-start">
            <div className="flex items-center">
              <Trash2 className="h-4 w-4 mr-2" />
              从 Mantra 移除
            </div>
            <span className="text-xs text-muted-foreground ml-6">
              (不会删除源项目)
            </span>
          </div>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
