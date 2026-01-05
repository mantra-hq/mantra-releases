/**
 * TopBar Component - Player 页面顶部导航栏
 * Story 2.17: Task 1
 * Story 2.21: Task 4 (添加 Logo, 全局搜索按钮, 设置按钮)
 *
 * 面包屑导航：☰ [Logo] 📁 项目名 › 💬 会话名 (消息数)
 * 支持响应式截断
 */

import { ChevronRight, Menu, FolderOpen } from "lucide-react";
import { cn } from "@/lib/utils";
import { BreadcrumbItem } from "./BreadcrumbItem";
import { SessionDropdown } from "./SessionDropdown";
import { TopBarActions } from "./TopBarActions";
import mantraLogo from "@/assets/mantra.png";

/**
 * 会话摘要信息
 */
export interface SessionSummary {
  id: string;
  name: string;
  messageCount: number;
  lastActiveAt: number; // Unix timestamp (ms)
}

/**
 * TopBar Props
 */
export interface TopBarProps {
  /** 当前会话 ID */
  sessionId: string;
  /** 当前会话名称 */
  sessionName: string;
  /** 当前会话消息数 */
  messageCount: number;

  /** 当前项目 ID */
  projectId: string;
  /** 当前项目名称 */
  projectName: string;

  /** 同项目会话列表 */
  sessions: SessionSummary[];

  /** 打开 ProjectDrawer 回调 */
  onDrawerOpen: () => void;
  /** 会话切换回调 */
  onSessionSelect: (sessionId: string) => void;
  /** 同步项目回调 */
  onSync: () => void;
  /** 导入回调 */
  onImport: () => void;
  /** 是否正在同步 */
  isSyncing?: boolean;
}

/**
 * TopBar 组件
 * Player 页面顶部导航栏，包含面包屑和操作按钮
 */
export function TopBar({
  sessionId,
  sessionName,
  messageCount,
  projectId: _projectId, // 预留接口，未来可用于项目级操作
  projectName,
  sessions,
  onDrawerOpen,
  onSessionSelect,
  onSync,
  onImport,
  isSyncing = false,
}: TopBarProps) {
  return (
    <header
      data-testid="top-bar"
      className={cn(
        "shrink-0 sticky top-0 z-50 w-full",
        "border-b border-border",
        "bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
      )}
    >
      <div className="flex h-14 items-center justify-between px-4 gap-2">
        {/* 左侧: 汉堡菜单 + Logo + 面包屑 */}
        <div className="flex items-center gap-1 min-w-0 flex-1">
          {/* 汉堡菜单 (AC2) */}
          <BreadcrumbItem
            icon={<Menu className="h-4 w-4" />}
            onClick={onDrawerOpen}
            aria-label="打开项目抽屉"
            testId="topbar-menu-button"
          />

          {/* Logo (Story 2.21 AC #14) */}
          <div className="flex items-center gap-1.5 mr-2">
            <img
              src={mantraLogo}
              alt="Mantra"
              className="h-6 w-6 rounded"
            />
            <span className="text-sm font-bold text-foreground whitespace-nowrap">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>

          {/* 项目名 (AC3) */}
          <BreadcrumbItem
            icon={<FolderOpen className="h-4 w-4" />}
            label={projectName}
            onClick={onDrawerOpen}
            truncate
            testId="topbar-project-name"
          />

          {/* 分隔符 */}
          <ChevronRight
            className="h-4 w-4 text-muted-foreground shrink-0"
            aria-hidden="true"
          />

          {/* 会话名 + 下拉选择器 (AC4) */}
          <SessionDropdown
            currentSessionId={sessionId}
            currentSessionName={sessionName}
            messageCount={messageCount}
            sessions={sessions}
            onSessionSelect={onSessionSelect}
          />
        </div>

        {/* 右侧: 操作按钮 (AC10, AC11, AC12) */}
        <TopBarActions onSync={onSync} onImport={onImport} isSyncing={isSyncing} />
      </div>
    </header>
  );
}
