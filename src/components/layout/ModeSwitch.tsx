/**
 * ModeSwitch Component - 模式切换组件
 * Story 2.34: Task 6.1
 *
 * 在回放模式和统计模式之间切换
 * 使用 Segmented Control 样式
 */

import { useTranslation } from "react-i18next";
import { Play, BarChart2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppModeStore, type AppMode } from "@/stores/useAppModeStore";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

/**
 * ModeSwitch Props
 */
export interface ModeSwitchProps {
  /** 自定义 className */
  className?: string;
}

/**
 * ModeSwitch 组件
 *
 * Segmented Control 样式的模式切换按钮
 * 支持回放模式和统计模式
 */
export function ModeSwitch({ className }: ModeSwitchProps) {
  const { t } = useTranslation();
  const mode = useAppModeStore((state) => state.mode);
  const setMode = useAppModeStore((state) => state.setMode);

  const handleModeChange = (newMode: AppMode) => {
    if (newMode !== mode) {
      setMode(newMode);
    }
  };

  return (
    <TooltipProvider delayDuration={300}>
      <div
        className={cn(
          "inline-flex items-center rounded-lg bg-muted p-0.5 h-8",
          className
        )}
        role="tablist"
        aria-label={t("analytics.modeSwitch")}
        data-testid="mode-switch"
      >
        {/* 回放模式按钮 */}
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              role="tab"
              aria-selected={mode === "playback"}
              aria-controls="playback-panel"
              onClick={() => handleModeChange("playback")}
              className={cn(
                "inline-flex items-center justify-center gap-1 rounded-md px-2 py-1 text-sm font-medium transition-all cursor-pointer",
                "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                mode === "playback"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
              data-testid="mode-switch-playback"
            >
              <Play className="h-3.5 w-3.5" />
              <span className="sr-only sm:not-sr-only">
                {t("analytics.mode.playback")}
              </span>
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t("analytics.mode.playbackTooltip")}</p>
          </TooltipContent>
        </Tooltip>

        {/* 统计模式按钮 */}
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              role="tab"
              aria-selected={mode === "statistics"}
              aria-controls="statistics-panel"
              onClick={() => handleModeChange("statistics")}
              className={cn(
                "inline-flex items-center justify-center gap-1 rounded-md px-2 py-1 text-sm font-medium transition-all cursor-pointer",
                "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                mode === "statistics"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
              data-testid="mode-switch-statistics"
            >
              <BarChart2 className="h-3.5 w-3.5" />
              <span className="sr-only sm:not-sr-only">
                {t("analytics.mode.statistics")}
              </span>
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t("analytics.mode.statisticsTooltip")}</p>
          </TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>
  );
}
