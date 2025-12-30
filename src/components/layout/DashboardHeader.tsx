/**
 * DashboardHeader Component - Dashboard 头部组件
 * Story 2.8: Task 7
 *
 * 包含 Logo、搜索框、主题切换和导入按钮
 */

import { Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ThemeToggle } from "@/components/theme-toggle";
import { ProjectSearch } from "@/components/search";
import { cn } from "@/lib/utils";

/**
 * DashboardHeader Props
 */
export interface DashboardHeaderProps {
  /** 搜索回调 */
  onSearch: (query: string) => void;
  /** 导入回调 */
  onImport: () => void;
}

/**
 * DashboardHeader 组件
 * Dashboard 页面头部，包含品牌、搜索和操作按钮
 */
export function DashboardHeader({ onSearch, onImport }: DashboardHeaderProps) {
  return (
    <header
      data-testid="dashboard-header"
      className={cn(
        "sticky top-0 z-50 w-full",
        "border-b border-border",
        "bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
      )}
    >
      <div className="flex h-14 items-center justify-between px-4 gap-4">
        {/* 左侧: Logo + 标题 */}
        <div className="flex items-center gap-2 shrink-0">
          <span className="text-xl font-bold text-foreground">
            Mantra <span className="text-primary">心法</span>
          </span>
        </div>

        {/* 中间: 搜索框 */}
        <div className="flex-1 max-w-md mx-4">
          <ProjectSearch onSearch={onSearch} />
        </div>

        {/* 右侧: 操作按钮 */}
        <div className="flex items-center gap-2 shrink-0">
          <Button
            variant="outline"
            size="sm"
            onClick={onImport}
            className="gap-1.5"
          >
            <Upload className="w-4 h-4" />
            导入
          </Button>
          <ThemeToggle />
        </div>
      </div>
    </header>
  );
}

