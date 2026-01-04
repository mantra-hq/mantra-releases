/**
 * SourceSelector Component - 导入来源选择
 * Story 2.9: Task 2
 * Story 2.24: AC5 官方品牌图标
 *
 * 显示可用的导入来源选项：
 * - Claude Code (已支持)
 * - Gemini CLI (已支持 - Story 1.6)
 * - Cursor (已支持 - Story 1.7)
 */

import * as React from "react";
import { cn } from "@/lib/utils";

// Story 2.24: AC5 官方品牌图标
import { ClaudeIcon } from "./SourceIcons";  // SVG 组件
import cursorIcon from "@/assets/source-icons/cursor.png";
import geminiIcon from "@/assets/source-icons/gemini.png";

/** 导入来源类型 */
export type ImportSource = "claude" | "gemini" | "cursor";

/** 来源配置 */
interface SourceConfig {
  id: ImportSource;
  name: string;
  defaultPath: string;
  /** 图标图片路径 (PNG) 或 React 组件 */
  iconSrc?: string;
  iconComponent?: React.ComponentType<{ className?: string }>;
  disabled: boolean;
  badge?: string;
}

/** 来源配置列表 */
const SOURCES: SourceConfig[] = [
  {
    id: "claude",
    name: "Claude Code",
    defaultPath: "~/.claude/projects",
    iconComponent: ClaudeIcon,
    disabled: false,
  },
  {
    id: "gemini",
    name: "Gemini CLI",
    defaultPath: "~/.gemini/tmp",
    iconSrc: geminiIcon,
    disabled: false,
  },
  {
    id: "cursor",
    name: "Cursor",
    defaultPath: "~/.config/Cursor (按工作区)",
    iconSrc: cursorIcon,
    disabled: false,
  },
];

/** SourceSelector Props */
export interface SourceSelectorProps {
  /** 当前选中的来源 */
  value: ImportSource | null;
  /** 选择变更回调 */
  onChange: (source: ImportSource) => void;
}

/**
 * 来源卡片组件
 */
function SourceCard({
  source,
  selected,
  onClick,
}: {
  source: SourceConfig;
  selected: boolean;
  onClick: () => void;
}) {
  const IconComponent = source.iconComponent;

  const handleClick = () => {
    if (!source.disabled) {
      onClick();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.key === "Enter" || e.key === " ") && !source.disabled) {
      e.preventDefault();
      onClick();
    }
  };

  return (
    <div
      data-testid={`source-card-${source.id}`}
      data-selected={selected ? "true" : "false"}
      data-disabled={source.disabled ? "true" : "false"}
      role="radio"
      aria-checked={selected}
      aria-disabled={source.disabled}
      tabIndex={source.disabled ? -1 : 0}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      className={cn(
        "flex flex-col items-center p-6 rounded-xl border-2 transition-all cursor-pointer",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        // 默认状态
        "border-border hover:border-primary/50",
        // 选中状态
        selected && "border-primary bg-primary/5",
        // 禁用状态
        source.disabled && "opacity-50 cursor-not-allowed hover:border-border"
      )}
    >
      {/* 图标 */}
      <div
        data-slot="source-icon"
        className={cn(
          "w-12 h-12 rounded-lg flex items-center justify-center mb-3",
          source.id === "claude" && "bg-orange-500/10 text-orange-500",
          source.id === "gemini" && "bg-blue-500/10",
          source.id === "cursor" && "bg-purple-500/10"
        )}
      >
        {source.iconSrc ? (
          <img
            src={source.iconSrc}
            alt={`${source.name} 图标`}
            className="w-8 h-8 object-contain"
          />
        ) : IconComponent ? (
          <IconComponent className="w-6 h-6" />
        ) : null}
      </div>

      {/* 名称 */}
      <span className="text-sm font-semibold text-foreground mb-1">
        {source.name}
      </span>

      {/* 默认路径 */}
      <span className="text-xs text-muted-foreground font-mono">
        {source.defaultPath}
      </span>

      {/* 徽章 */}
      {source.badge && (
        <span className="mt-2 px-2 py-0.5 rounded text-[10px] bg-muted text-muted-foreground">
          {source.badge}
        </span>
      )}
    </div>
  );
}

/**
 * SourceSelector 组件
 * 选择导入来源
 */
export function SourceSelector({ value, onChange }: SourceSelectorProps) {
  return (
    <div
      data-testid="source-selector"
      role="radiogroup"
      aria-label="选择导入来源"
      className="grid grid-cols-3 gap-4"
    >
      {SOURCES.map((source) => (
        <SourceCard
          key={source.id}
          source={source}
          selected={value === source.id}
          onClick={() => onChange(source.id)}
        />
      ))}
    </div>
  );
}
