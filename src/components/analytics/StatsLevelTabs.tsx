/**
 * StatsLevelTabs Component - 统计视图层级切换
 * Story 2.34: 项目/会话统计切换
 *
 * 在有选中会话时，允许用户切换查看项目统计或会话统计
 */

import { useTranslation } from "react-i18next";
import { FolderOpen, MessageSquare } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/**
 * 统计层级类型
 */
export type StatsLevel = "project" | "session";

/**
 * StatsLevelTabs Props
 */
export interface StatsLevelTabsProps {
  /** 当前选中的层级 */
  value: StatsLevel;
  /** 层级变化回调 */
  onChange: (level: StatsLevel) => void;
  /** 项目名称 */
  projectName?: string;
  /** 会话名称 */
  sessionName?: string;
  /** 自定义 className */
  className?: string;
}

/**
 * StatsLevelTabs 组件
 *
 * 项目/会话统计视图切换 Tabs
 */
export function StatsLevelTabs({
  value,
  onChange,
  projectName,
  sessionName,
  className,
}: StatsLevelTabsProps) {
  const { t } = useTranslation();

  return (
    <div
      className={cn(
        "flex items-center gap-1 p-1 rounded-lg bg-muted/50 border border-border/50",
        className
      )}
      data-testid="stats-level-tabs"
    >
      {/* 项目统计 Tab */}
      <Button
        variant={value === "project" ? "secondary" : "ghost"}
        size="sm"
        onClick={() => onChange("project")}
        className={cn(
          "gap-1.5 h-7 px-3 text-xs",
          value === "project"
            ? "bg-background shadow-sm text-foreground"
            : "text-muted-foreground hover:text-foreground"
        )}
        data-testid="stats-level-project"
      >
        <FolderOpen className="h-3.5 w-3.5" />
        <span className="hidden sm:inline">
          {projectName || t("analytics.projectStats")}
        </span>
        <span className="sm:hidden">{t("analytics.projectStats")}</span>
      </Button>

      {/* 会话统计 Tab */}
      <Button
        variant={value === "session" ? "secondary" : "ghost"}
        size="sm"
        onClick={() => onChange("session")}
        className={cn(
          "gap-1.5 h-7 px-3 text-xs",
          value === "session"
            ? "bg-background shadow-sm text-foreground"
            : "text-muted-foreground hover:text-foreground"
        )}
        data-testid="stats-level-session"
      >
        <MessageSquare className="h-3.5 w-3.5" />
        <span className="hidden sm:inline truncate max-w-[150px]">
          {sessionName || t("analytics.sessionStats")}
        </span>
        <span className="sm:hidden">{t("analytics.sessionStats")}</span>
      </Button>
    </div>
  );
}
