/**
 * ModeSwitch Component - 三态模式切换组件
 * Story 2.34: Task 6.1 - 初始实现
 * Story 10.11: 三态模式支持 (playback/analytics/compress)
 *
 * 在回放模式、统计模式和压缩模式之间切换
 * 使用 Segmented Control 样式
 * AC3: 响应式布局 - 小屏幕仅显示图标
 */

import { useTranslation } from "react-i18next";
import { Play, BarChart2, Minimize2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppModeStore, type AppMode } from "@/stores/useAppModeStore";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

/**
 * 模式配置项
 */
interface ModeConfig {
  value: AppMode;
  icon: React.ComponentType<{ className?: string }>;
  labelKey: string;
  tooltipKey: string;
}

/**
 * 所有可用模式配置
 */
const ALL_MODES: ModeConfig[] = [
  {
    value: "analytics",
    icon: BarChart2,
    labelKey: "analytics.mode.analytics",
    tooltipKey: "analytics.mode.analyticsTooltip",
  },
  {
    value: "playback",
    icon: Play,
    labelKey: "analytics.mode.playback",
    tooltipKey: "analytics.mode.playbackTooltip",
  },
  {
    value: "compress",
    icon: Minimize2,
    labelKey: "analytics.mode.compress",
    tooltipKey: "analytics.mode.compressTooltip",
  },
];

/**
 * ModeSwitch Props
 */
export interface ModeSwitchProps {
  /** 自定义 className */
  className?: string;
  /** 是否禁用压缩模式（无 sessionId 时） */
  disableCompress?: boolean;
}

/**
 * ModeSwitch 组件
 *
 * Segmented Control 样式的模式切换按钮
 * 支持回放模式、统计模式和压缩模式
 * AC3: 响应式布局 - 768px 以下仅显示图标
 */
export function ModeSwitch({ className, disableCompress = false }: ModeSwitchProps) {
  const { t } = useTranslation();
  const mode = useAppModeStore((state) => state.mode);
  const setMode = useAppModeStore((state) => state.setMode);

  const handleModeChange = (newMode: AppMode) => {
    if (newMode !== mode) {
      setMode(newMode);
    }
  };

  // AC6: 过滤可用模式 - 无 sessionId 时隐藏压缩模式
  const availableModes = disableCompress
    ? ALL_MODES.filter((m) => m.value !== "compress")
    : ALL_MODES;

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
        {availableModes.map(({ value, icon: Icon, labelKey, tooltipKey }) => {
          const isActive = mode === value;

          return (
            <Tooltip key={value}>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  role="tab"
                  aria-selected={isActive}
                  aria-controls={`${value}-panel`}
                  onClick={() => handleModeChange(value)}
                  className={cn(
                    "inline-flex items-center justify-center gap-1 rounded-md px-2 py-1 text-sm font-medium transition-all duration-150 cursor-pointer",
                    "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                    isActive
                      ? "bg-background text-foreground shadow-sm"
                      : "text-muted-foreground hover:text-foreground"
                  )}
                  data-testid={`mode-switch-${value}`}
                >
                  <Icon className="h-3.5 w-3.5" />
                  {/* AC3: 响应式 - 小屏幕仅显示图标 */}
                  <span className="sr-only sm:not-sr-only">
                    {t(labelKey)}
                  </span>
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom">
                <p>{t(tooltipKey)}</p>
              </TooltipContent>
            </Tooltip>
          );
        })}
      </div>
    </TooltipProvider>
  );
}
