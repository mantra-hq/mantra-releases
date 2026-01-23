/**
 * LogicalProjectTreeItem Component - 逻辑项目树节点
 * Story 1.12: Phase 5 - Task 11
 *
 * 显示基于物理路径聚合的逻辑项目，支持多来源会话聚合。
 * - 显示从路径提取的 display_name
 * - 显示路径类型图标和"需关联"状态
 * - 支持展开显示所有来源的会话
 * - 支持重命名（仅单项目时）
 */

import * as React from "react";
import { ChevronRight, FolderOpen, Loader2, Link2, GitBranch, Globe } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { SessionTreeItem } from "./SessionTreeItem";
import { HighlightText } from "./DrawerSearch";
import { ProjectRenameInput } from "./ProjectRenameInput";
import type { LogicalProjectStats } from "@/types/project";
import type { SessionSummary } from "./types";

/**
 * LogicalProjectTreeItem Props
 */
export interface LogicalProjectTreeItemProps {
  /** 逻辑项目统计信息 */
  logicalProject: LogicalProjectStats;
  /** 是否展开 */
  isExpanded: boolean;
  /** 是否正在加载会话 */
  isLoading?: boolean;
  /** 会话列表 */
  sessions: SessionSummary[];
  /** 当前选中的会话 ID */
  currentSessionId?: string;
  /** 搜索关键词（用于高亮） */
  searchKeyword?: string;
  /** 展开/折叠回调 */
  onToggle: () => void;
  /** 会话点击回调 */
  onSessionSelect: (sessionId: string) => void;
  /** 点击项目（用于打开详情页）回调 */
  onProjectClick?: () => void;
  /** 设置菜单组件 */
  settingsMenu?: React.ReactNode;
  /** 菜单是否打开（用于保持按钮可见） */
  isSettingsMenuOpen?: boolean;
  /** 是否隐藏空会话 */
  hideEmptySessions?: boolean;
  /** 是否处于重命名模式（仅单项目时有效） */
  isRenaming?: boolean;
  /** 重命名保存回调 */
  onRename?: (newName: string) => void;
  /** 重命名取消回调 */
  onRenameCancel?: () => void;
}

/**
 * 获取路径类型图标
 */
function getPathTypeIcon(pathType: string, needsAssociation: boolean) {
  if (needsAssociation) {
    return <Link2 className="h-4 w-4 shrink-0 text-yellow-500" />;
  }
  
  switch (pathType) {
    case "remote":
      return <Globe className="h-4 w-4 shrink-0 text-blue-400" />;
    default:
      return <FolderOpen className="h-4 w-4 shrink-0 text-muted-foreground" />;
  }
}

/**
 * LogicalProjectTreeItem 组件
 * 逻辑项目节点，显示聚合后的项目视图
 */
export function LogicalProjectTreeItem({
  logicalProject,
  isExpanded,
  isLoading = false,
  sessions,
  currentSessionId,
  searchKeyword,
  onToggle,
  onSessionSelect,
  onProjectClick,
  settingsMenu,
  isSettingsMenuOpen = false,
  hideEmptySessions = false,
  isRenaming = false,
  onRename,
  onRenameCancel,
}: LogicalProjectTreeItemProps) {
  const { t } = useTranslation();
  
  // Filter out empty sessions if hideEmptySessions is enabled
  const filteredSessions = React.useMemo(() => {
    if (!hideEmptySessions) return sessions;
    return sessions.filter((session) => !session.is_empty);
  }, [sessions, hideEmptySessions]);

  // Generate a unique test ID from the physical path
  const testId = React.useMemo(() => {
    return logicalProject.physical_path
      .replace(/[^a-zA-Z0-9]/g, "-")
      .slice(0, 50);
  }, [logicalProject.physical_path]);

  return (
    <div data-testid={`logical-project-${testId}`}>
      {/* 项目节点 */}
      <div
        className={cn(
          "w-full flex items-center gap-2 px-4 py-2",
          "hover:bg-muted/50 transition-colors",
          "group"
        )}
      >
        {/* 可点击区域 - 展开/折叠 */}
        <button
          type="button"
          onClick={onToggle}
          className="flex items-center gap-2 flex-1 text-left min-w-0 cursor-pointer"
          data-testid={`logical-project-toggle-${testId}`}
          disabled={isRenaming}
        >
          {/* 展开/折叠图标 */}
          <ChevronRight
            className={cn(
              "h-4 w-4 shrink-0 text-muted-foreground transition-transform duration-200",
              isExpanded && "rotate-90"
            )}
          />

          {/* 路径类型图标 */}
          {getPathTypeIcon(logicalProject.path_type, logicalProject.needs_association)}

          {/* 项目名称 - 重命名模式或普通显示 */}
          {isRenaming && onRename && onRenameCancel ? (
            <ProjectRenameInput
              initialName={logicalProject.display_name}
              onSave={onRename}
              onCancel={onRenameCancel}
            />
          ) : (
            <>
              <span
                className={cn(
                  "flex-1 truncate text-sm",
                  logicalProject.needs_association && "text-yellow-500"
                )}
                title={logicalProject.physical_path}
                onClick={(e) => {
                  // 点击名称时打开详情页（如果有回调）
                  if (onProjectClick) {
                    e.stopPropagation();
                    onProjectClick();
                  }
                }}
              >
                <HighlightText text={logicalProject.display_name} keyword={searchKeyword} />
              </span>

              {/* 多来源指示器 */}
              {logicalProject.project_count > 1 && (
                <span
                  className="text-[10px] px-1 py-0.5 rounded bg-primary/10 text-primary shrink-0"
                  title={t("project.multiSource", { count: logicalProject.project_count })}
                >
                  {logicalProject.project_count}
                </span>
              )}

              {/* Git 状态指示器 */}
              {logicalProject.has_git_repo && (
                <GitBranch className="h-3 w-3 text-muted-foreground shrink-0" />
              )}

              {/* 会话数量 */}
              <span className="text-xs text-muted-foreground shrink-0">
                {logicalProject.total_sessions}
              </span>
            </>
          )}
        </button>

        {/* 设置菜单 */}
        {settingsMenu && !isRenaming && (
          <div
            className={cn(
              "shrink-0 transition-opacity",
              "opacity-0 group-hover:opacity-100",
              isSettingsMenuOpen && "opacity-100"
            )}
            onClick={(e) => e.stopPropagation()}
          >
            {settingsMenu}
          </div>
        )}
      </div>

      {/* "需关联"状态提示 */}
      {logicalProject.needs_association && isExpanded && (
        <div className="pl-10 pr-4 py-1 text-xs text-yellow-600 dark:text-yellow-500">
          {logicalProject.path_type === "virtual"
            ? t("project.needsRealPath", "虚拟路径，点击关联真实目录")
            : t("project.pathNotFound", "路径不存在，点击更新")}
        </div>
      )}

      {/* 会话列表 (展开时显示) */}
      {isExpanded && (
        <div className="transition-opacity duration-200 opacity-100">
          {isLoading ? (
            <div className="flex items-center gap-2 pl-10 pr-4 py-2 text-sm text-muted-foreground">
              <Loader2 className="h-3 w-3 animate-spin" />
              {t("common.loading")}...
            </div>
          ) : filteredSessions.length === 0 ? (
            <div className="pl-10 pr-4 py-2 text-sm text-muted-foreground">
              {t("project.noSessions", "暂无会话")}
            </div>
          ) : (
            filteredSessions.map((session) => (
              <SessionTreeItem
                key={session.id}
                session={session}
                isCurrent={session.id === currentSessionId}
                searchKeyword={searchKeyword}
                onClick={() => onSessionSelect(session.id)}
              />
            ))
          )}
        </div>
      )}
    </div>
  );
}
