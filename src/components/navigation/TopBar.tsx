/**
 * TopBar Component - 统一的顶部导航栏
 * Story 2.17: Task 1
 * Story 2.21: Task 4 (添加 Logo, 全局搜索按钮, 设置按钮)
 * Refactor: 支持多种模式 (full, minimal, loading, error)
 *
 * 面包屑导航：☰ [Logo] 📁 项目名 › 💬 会话名 (消息数)
 * 支持响应式截断
 */

import { ChevronRight, Menu, FolderOpen, ArrowLeft, Loader2, AlertCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { BreadcrumbItem } from "./BreadcrumbItem";
import { SessionDropdown } from "./SessionDropdown";
import { TopBarActions } from "./TopBarActions";
import { Button } from "@/components/ui/button";
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
 * TopBar 模式
 * - full: 完整模式（有会话信息、同步按钮）
 * - minimal: 最小模式（无会话信息，用于空状态）
 * - loading: 加载中状态
 * - error: 错误状态
 */
export type TopBarMode = "full" | "minimal" | "loading" | "error";

/**
 * TopBar Props
 */
export interface TopBarProps {
  /** 显示模式 */
  mode?: TopBarMode;

  /** 当前会话 ID */
  sessionId?: string;
  /** 当前会话名称 */
  sessionName?: string;
  /** 当前会话消息数 */
  messageCount?: number;

  /** 当前项目 ID */
  projectId?: string;
  /** 当前项目名称 */
  projectName?: string;

  /** 同项目会话列表 */
  sessions?: SessionSummary[];

  /** 打开 ProjectDrawer 回调 */
  onDrawerOpen?: () => void;
  /** 会话切换回调 */
  onSessionSelect?: (sessionId: string) => void;
  /** 同步项目回调 */
  onSync?: () => void;
  /** 导入回调 */
  onImport?: () => void;
  /** 返回回调（用于 loading/error 状态） */
  onBack?: () => void;
  /** 是否正在同步 */
  isSyncing?: boolean;
}

/**
 * TopBar 组件
 * 统一的顶部导航栏，支持多种显示模式
 */
export function TopBar({
  mode = "full",
  sessionId,
  sessionName,
  messageCount = 0,
  projectId: _projectId, // 预留接口，未来可用于项目级操作
  projectName,
  sessions = [],
  onDrawerOpen,
  onSessionSelect,
  onSync,
  onImport,
  onBack,
  isSyncing = false,
}: TopBarProps) {
  // 渲染 Logo 区域
  const renderLogo = () => (
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
  );

  // 渲染左侧内容（根据模式不同）
  const renderLeftContent = () => {
    switch (mode) {
      case "loading":
      case "error":
        // loading/error: 返回按钮 + Logo
        return (
          <>
            <Button
              variant="ghost"
              size="icon"
              onClick={onBack}
              className="h-8 w-8"
              data-testid="topbar-back-button"
            >
              <ArrowLeft className="h-5 w-5" />
            </Button>
            {renderLogo()}
          </>
        );

      case "minimal":
        // minimal: 汉堡菜单 + Logo（无面包屑）
        return (
          <>
            <Button
              variant="ghost"
              size="icon"
              onClick={onDrawerOpen}
              aria-label="打开项目抽屉"
              className="h-8 w-8"
              data-testid="topbar-menu-button"
            >
              <Menu className="h-4 w-4" />
            </Button>
            {renderLogo()}
          </>
        );

      case "full":
      default:
        // full: 汉堡菜单 + Logo + 面包屑
        return (
          <>
            {/* 汉堡菜单 (AC2) */}
            <BreadcrumbItem
              icon={<Menu className="h-4 w-4" />}
              onClick={onDrawerOpen}
              aria-label="打开项目抽屉"
              testId="topbar-menu-button"
            />

            {/* Logo (Story 2.21 AC #14) */}
            {renderLogo()}

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
              currentSessionId={sessionId || ""}
              currentSessionName={sessionName || ""}
              messageCount={messageCount}
              sessions={sessions}
              onSessionSelect={onSessionSelect || (() => { })}
            />
          </>
        );
    }
  };

  // 渲染右侧内容（根据模式不同）
  const renderRightContent = () => {
    switch (mode) {
      case "loading":
        // loading: 只显示加载动画提示
        return (
          <div className="flex items-center gap-2 text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            <span className="text-sm">加载中...</span>
          </div>
        );

      case "error":
        // error: 只显示错误提示
        return (
          <div className="flex items-center gap-2 text-destructive">
            <AlertCircle className="h-4 w-4" />
            <span className="text-sm">加载失败</span>
          </div>
        );

      case "minimal":
        // minimal: 操作按钮（无同步）
        return (
          <TopBarActions
            sessionId={undefined}
            onSync={onSync || (() => { })}
            onImport={onImport || (() => { })}
            isSyncing={false}
            showSync={false}
          />
        );

      case "full":
      default:
        // full: 完整操作按钮组
        return (
          <TopBarActions
            sessionId={sessionId}
            onSync={onSync || (() => { })}
            onImport={onImport || (() => { })}
            isSyncing={isSyncing}
          />
        );
    }
  };

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
        {/* 左侧内容 */}
        <div className="flex items-center gap-1 min-w-0 flex-1">
          {renderLeftContent()}
        </div>

        {/* 右侧内容 */}
        {renderRightContent()}
      </div>
    </header>
  );
}
