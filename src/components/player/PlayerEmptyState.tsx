/**
 * PlayerEmptyState Component - Player 空状态组件
 * Story 2.21: Task 1
 *
 * 无会话时显示的引导界面
 * 支持两种模式：有项目 vs 无项目
 */

import { Play, FolderOpen, Rocket } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/**
 * PlayerEmptyState Props
 */
export interface PlayerEmptyStateProps {
  /** 是否有项目 */
  hasProjects: boolean;
  /** 打开抽屉回调 */
  onOpenDrawer: () => void;
  /** 打开导入向导回调 */
  onImport: () => void;
}

/**
 * PlayerEmptyState 组件
 * Player 页面空状态，引导用户选择或导入会话
 */
export function PlayerEmptyState({
  hasProjects,
  onOpenDrawer,
  onImport,
}: PlayerEmptyStateProps) {
  if (hasProjects) {
    // 有项目时的 UI (AC #4-8)
    return (
      <div
        data-testid="player-empty-state"
        className={cn(
          "flex flex-col items-center justify-center",
          "h-full py-12 px-6",
          "text-center"
        )}
      >
        {/* Play 图标 (AC #5 - 48px+) */}
        <div
          data-testid="empty-state-icon"
          className={cn(
            "w-24 h-24 mb-6",
            "flex items-center justify-center",
            "rounded-2xl",
            "bg-muted/50"
          )}
        >
          <Play className="w-12 h-12 text-primary/70" />
        </div>

        {/* 主标题 (AC #6) */}
        <h2 className="text-xl font-semibold text-foreground mb-2">
          选择一个会话开始回放
        </h2>

        {/* 副标题 (AC #7) */}
        <p className="text-sm text-muted-foreground max-w-md mb-8">
          从左侧项目列表中选择，或导入新的 AI 编程会话
        </p>

        {/* CTA 按钮组 (AC #8) */}
        <div className="flex items-center gap-3">
          <Button onClick={onOpenDrawer} variant="default" size="lg">
            打开项目列表
          </Button>
          <Button onClick={onImport} variant="outline" size="lg">
            导入项目
          </Button>
        </div>
      </div>
    );
  }

  // 无项目时的 UI (AC #9)
  return (
    <div
      data-testid="player-empty-state"
      className={cn(
        "flex flex-col items-center justify-center",
        "h-full py-12 px-6",
        "text-center"
      )}
    >
      {/* Folder 图标 (AC #5 - 48px+) */}
      <div
        data-testid="empty-state-icon"
        className={cn(
          "w-24 h-24 mb-6",
          "flex items-center justify-center",
          "rounded-2xl",
          "bg-muted/50"
        )}
      >
        <FolderOpen className="w-12 h-12 text-muted-foreground/70" />
      </div>

      {/* 主标题 (AC #9) */}
      <h2 className="text-xl font-semibold text-foreground mb-2">
        还没有导入任何项目
      </h2>

      {/* 副标题 (AC #9) */}
      <p className="text-sm text-muted-foreground max-w-md mb-8">
        导入你的 AI 编程会话，开始探索和回放心法
      </p>

      {/* 单个 CTA (AC #9) */}
      <Button onClick={onImport} size="lg" className="gap-2 mb-6">
        <Rocket className="w-4 h-4" />
        导入第一个项目
      </Button>

      {/* 支持说明 (AC #9) */}
      <p className="text-xs text-muted-foreground">
        支持: Claude Code · Cursor · Gemini CLI · Codex
      </p>
    </div>
  );
}
