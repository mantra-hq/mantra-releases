/**
 * EmptyDashboard Component - 空状态组件
 * Story 2.8: Task 5
 *
 * 无项目时显示的引导界面
 */

import * as React from "react";
import { FolderOpen, Sparkles, MessageSquare, Terminal, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/**
 * EmptyDashboard Props
 */
export interface EmptyDashboardProps {
  /** 点击导入按钮回调 */
  onImport: () => void;
}

/**
 * 支持的格式列表
 */
const supportedFormats = [
  { name: "Claude Code", icon: Sparkles, color: "text-orange-500" },
  { name: "Gemini CLI", icon: MessageSquare, color: "text-blue-500" },
  { name: "Cursor", icon: Terminal, color: "text-purple-500" },
];

/**
 * EmptyDashboard 组件
 * 显示空状态引导，包含导入按钮和支持格式说明
 */
export function EmptyDashboard({ onImport }: EmptyDashboardProps) {
  return (
    <div
      data-testid="empty-dashboard"
      className={cn(
        "flex flex-col items-center justify-center",
        "min-h-[400px] py-12 px-6",
        "text-center"
      )}
    >
      {/* 装饰图标 */}
      <div
        className={cn(
          "w-20 h-20 mb-6",
          "flex items-center justify-center",
          "rounded-2xl",
          "bg-muted/50"
        )}
      >
        <FolderOpen className="w-10 h-10 text-muted-foreground/50" />
      </div>

      {/* 标题 */}
      <h2 className="text-xl font-semibold text-foreground mb-2">
        开始使用 Mantra
      </h2>

      {/* 描述 */}
      <p className="text-sm text-muted-foreground max-w-sm mb-6">
        导入你的 AI 编程会话日志，回顾和分享你的编程心法
      </p>

      {/* 导入按钮 */}
      <Button
        onClick={onImport}
        size="lg"
        className="gap-2 mb-8"
      >
        <Upload className="w-4 h-4" />
        导入日志
      </Button>

      {/* 支持的格式 */}
      <div className="flex flex-col items-center gap-3">
        <span className="text-xs text-muted-foreground">支持的格式</span>
        <div className="flex items-center gap-4">
          {supportedFormats.map((format) => (
            <div
              key={format.name}
              className="flex items-center gap-1.5 text-sm text-muted-foreground"
            >
              <format.icon className={cn("w-4 h-4", format.color)} />
              <span>{format.name}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

