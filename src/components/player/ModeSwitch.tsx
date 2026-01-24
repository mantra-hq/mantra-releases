/**
 * ModeSwitch - 模式切换 Tab 组件
 * Story 10.1: AC #1
 *
 * 在会话详情页切换"回放模式"和"压缩模式"
 * - 回放模式: 查看历史会话消息和代码快照
 * - 压缩模式: 优化会话上下文，对抗自动截断
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Play, Minimize2 } from "lucide-react";
import { cn } from "@/lib/utils";

export type PlayerMode = "playback" | "compress";

export interface ModeSwitchProps {
  /** 当前模式 */
  mode: PlayerMode;
  /** 模式切换回调 */
  onModeChange: (mode: PlayerMode) => void;
  /** 自定义 className */
  className?: string;
}

/**
 * ModeSwitch 组件
 * 显示回放/压缩模式切换 Tab
 */
export function ModeSwitch({
  mode,
  onModeChange,
  className,
}: ModeSwitchProps) {
  const { t } = useTranslation();

  return (
    <div
      className={cn(
        "inline-flex items-center rounded-lg bg-muted/50 p-1",
        className
      )}
      role="tablist"
      aria-label={t("player.modeSwitch")}
    >
      {/* 回放模式 Tab */}
      <button
        type="button"
        role="tab"
        aria-selected={mode === "playback"}
        aria-controls="playback-panel"
        onClick={() => onModeChange("playback")}
        data-testid="mode-playback"
        className={cn(
          "inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md",
          "transition-all duration-150",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
          mode === "playback"
            ? "bg-background text-foreground shadow-sm"
            : "text-muted-foreground hover:text-foreground hover:bg-muted"
        )}
      >
        <Play className="h-4 w-4" />
        <span>{t("player.playbackMode")}</span>
      </button>

      {/* 压缩模式 Tab */}
      <button
        type="button"
        role="tab"
        aria-selected={mode === "compress"}
        aria-controls="compress-panel"
        onClick={() => onModeChange("compress")}
        data-testid="mode-compress"
        className={cn(
          "inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md",
          "transition-all duration-150",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
          mode === "compress"
            ? "bg-background text-foreground shadow-sm"
            : "text-muted-foreground hover:text-foreground hover:bg-muted"
        )}
      >
        <Minimize2 className="h-4 w-4" />
        <span>{t("player.compressMode")}</span>
      </button>
    </div>
  );
}

ModeSwitch.displayName = "ModeSwitch";

export default ModeSwitch;
