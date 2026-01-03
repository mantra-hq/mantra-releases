/**
 * ProjectTreeItem Component - 项目树节点
 * Story 2.18: Task 3
 * Story 2.19: Task 10 - 集成项目管理功能
 *
 * 项目节点，可展开/折叠显示会话列表
 */

import * as React from "react";
import { ChevronRight, FolderOpen, Settings, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { SessionTreeItem } from "./SessionTreeItem";
import { HighlightText } from "./DrawerSearch";
import { ProjectRenameInput } from "./ProjectRenameInput";
import type { Project } from "@/types/project";
import type { SessionSummary } from "./types";

/**
 * ProjectTreeItem Props
 */
export interface ProjectTreeItemProps {
  /** 项目信息 */
  project: Project;
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
  /** 齿轮按钮点击回调（预留给 2.19） */
  onSettingsClick?: () => void;
  /** Story 2.19: 是否处于重命名模式 */
  isRenaming?: boolean;
  /** Story 2.19: 重命名保存回调 */
  onRename?: (newName: string) => void;
  /** Story 2.19: 重命名取消回调 */
  onRenameCancel?: () => void;
  /** Story 2.19: 设置菜单组件（替换默认齿轮按钮） */
  settingsMenu?: React.ReactNode;
}

/**
 * ProjectTreeItem 组件
 * 项目节点，可展开显示会话列表
 */
export function ProjectTreeItem({
  project,
  isExpanded,
  isLoading = false,
  sessions,
  currentSessionId,
  searchKeyword,
  onToggle,
  onSessionSelect,
  onSettingsClick,
  isRenaming = false,
  onRename,
  onRenameCancel,
  settingsMenu,
}: ProjectTreeItemProps) {
  // 悬停状态（用于显示齿轮按钮）
  const [isHovered, setIsHovered] = React.useState(false);

  return (
    <div data-testid={`project-tree-item-${project.id}`}>
      {/* 项目节点 - 使用 div 包裹避免嵌套 button */}
      <div
        className={cn(
          "w-full flex items-center gap-2 px-4 py-2",
          "hover:bg-muted/50 transition-colors",
          "group relative"
        )}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        {/* 可点击区域 - 展开/折叠 */}
        <button
          type="button"
          onClick={onToggle}
          className="flex items-center gap-2 flex-1 text-left min-w-0"
          data-testid={`project-toggle-${project.id}`}
          disabled={isRenaming}
        >
          {/* 展开/折叠图标 */}
          <ChevronRight
            className={cn(
              "h-4 w-4 shrink-0 text-muted-foreground transition-transform duration-200",
              isExpanded && "rotate-90"
            )}
          />

          {/* 项目图标 */}
          <FolderOpen className="h-4 w-4 shrink-0 text-muted-foreground" />

          {/* 项目名称 - 重命名模式或普通显示 */}
          {isRenaming && onRename && onRenameCancel ? (
            <ProjectRenameInput
              initialName={project.name}
              onSave={onRename}
              onCancel={onRenameCancel}
            />
          ) : (
            <>
              <span className="flex-1 truncate text-sm">
                <HighlightText text={project.name} keyword={searchKeyword} />
              </span>

              {/* 会话数量 */}
              <span className="text-xs text-muted-foreground shrink-0">
                {project.session_count}
              </span>
            </>
          )}
        </button>

        {/* 齿轮按钮 (AC9: 预留给 2.19) - 作为兄弟元素避免嵌套 */}
        {/* Story 2.19: 如果提供了 settingsMenu，渲染它；否则渲染默认按钮 */}
        {isHovered && !isRenaming && (
          settingsMenu ? (
            <div className="shrink-0" onClick={(e) => e.stopPropagation()}>
              {settingsMenu}
            </div>
          ) : onSettingsClick ? (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onSettingsClick();
              }}
              className="h-6 w-6 flex items-center justify-center rounded-sm hover:bg-muted shrink-0"
              aria-label="项目设置"
              data-testid={`project-settings-${project.id}`}
            >
              <Settings className="h-3.5 w-3.5 text-muted-foreground" />
            </button>
          ) : null
        )}
      </div>

      {/* 会话列表 (展开时显示) */}
      {isExpanded && (
        <div
          className={cn(
            "overflow-hidden transition-all duration-200",
            isExpanded ? "max-h-[1000px] opacity-100" : "max-h-0 opacity-0"
          )}
        >
          {isLoading ? (
            <div className="flex items-center gap-2 pl-10 pr-4 py-2 text-sm text-muted-foreground">
              <Loader2 className="h-3 w-3 animate-spin" />
              加载中...
            </div>
          ) : sessions.length === 0 ? (
            <div className="pl-10 pr-4 py-2 text-sm text-muted-foreground">
              暂无会话
            </div>
          ) : (
            sessions.map((session) => (
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
