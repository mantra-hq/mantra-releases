/**
 * ProjectDrawer Component - 项目抽屉
 * Story 2.18: Task 2
 * Story 2.19: Task 10 - 集成项目管理功能
 *
 * 侧边抽屉，用于浏览和管理所有项目
 * - 从左侧滑入，宽度 320px
 * - 包含标题栏、搜索框、项目树列表和导入按钮
 * - 支持项目同步、重命名、移除操作
 */

import * as React from "react";
import { FolderOpen, Plus, FolderSearch } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { DrawerSearch } from "./DrawerSearch";
import { ProjectTreeItem } from "./ProjectTreeItem";
import { ProjectContextMenu } from "./ProjectContextMenu";
import { RemoveProjectDialog } from "./RemoveProjectDialog";
import { showSyncResult } from "./SyncResultToast";
import { useUndoableAction } from "@/hooks/useUndoableAction";
import {
  syncProject,
  removeProject,
  restoreProject,
  renameProject,
} from "@/lib/project-ipc";
import { toast } from "sonner";
import type { Project } from "@/types/project";
import type { SessionSummary } from "./types";

/**
 * ProjectDrawer Props
 */
export interface ProjectDrawerProps {
  /** 抽屉是否打开 */
  isOpen: boolean;
  /** 抽屉开关回调 */
  onOpenChange: (open: boolean) => void;
  /** 项目列表 */
  projects: Project[];
  /** 是否正在加载 */
  isLoading?: boolean;
  /** 当前会话 ID（用于高亮当前选中） */
  currentSessionId?: string;
  /** 会话点击回调 */
  onSessionSelect: (sessionId: string, projectId: string) => void;
  /** 导入按钮点击回调 */
  onImportClick: () => void;
  /** 获取项目会话列表 */
  getProjectSessions: (projectId: string) => Promise<SessionSummary[]>;
  /** Story 2.19: 项目列表变更回调（用于刷新列表） */
  onProjectsChange?: () => void;
}

/**
 * ProjectDrawer 组件
 * 项目抽屉，从左侧滑入显示项目树
 */
export function ProjectDrawer({
  isOpen,
  onOpenChange,
  projects,
  isLoading = false,
  currentSessionId,
  onSessionSelect,
  onImportClick,
  getProjectSessions,
  onProjectsChange,
}: ProjectDrawerProps) {
  // 搜索关键词状态
  const [searchKeyword, setSearchKeyword] = React.useState("");
  // 展开的项目 ID 集合
  const [expandedProjects, setExpandedProjects] = React.useState<Set<string>>(
    new Set()
  );
  // 项目会话缓存
  const [projectSessions, setProjectSessions] = React.useState<
    Record<string, SessionSummary[]>
  >({});
  // 加载中的项目 ID 集合
  const [loadingProjects, setLoadingProjects] = React.useState<Set<string>>(
    new Set()
  );

  // Story 2.19: 项目管理状态
  // 当前操作的项目（用于对话框）
  const [activeProject, setActiveProject] = React.useState<Project | null>(null);
  // 移除对话框打开状态
  const [isRemoveDialogOpen, setIsRemoveDialogOpen] = React.useState(false);
  // 重命名中的项目 ID
  const [renamingProjectId, setRenamingProjectId] = React.useState<string | null>(null);

  // 可撤销操作 Hook
  const undoableAction = useUndoableAction();

  // 过滤后的项目列表
  const filteredProjects = React.useMemo(() => {
    if (!searchKeyword.trim()) return projects;

    const keyword = searchKeyword.toLowerCase();
    return projects.filter((project) => {
      // 匹配项目名
      if (project.name.toLowerCase().includes(keyword)) return true;

      // 匹配项目路径
      if (project.cwd.toLowerCase().includes(keyword)) return true;

      // 匹配会话（如果已加载）
      const sessions = projectSessions[project.id];
      if (sessions?.some((s) => s.id.toLowerCase().includes(keyword))) {
        return true;
      }

      return false;
    });
  }, [projects, searchKeyword, projectSessions]);

  // 切换项目展开状态
  const handleToggleProject = React.useCallback(
    async (projectId: string) => {
      setExpandedProjects((prev) => {
        const next = new Set(prev);
        if (next.has(projectId)) {
          next.delete(projectId);
        } else {
          next.add(projectId);
        }
        return next;
      });

      // 如果展开且尚未加载会话，则加载
      if (
        !expandedProjects.has(projectId) &&
        !projectSessions[projectId] &&
        !loadingProjects.has(projectId)
      ) {
        setLoadingProjects((prev) => new Set(prev).add(projectId));
        try {
          const sessions = await getProjectSessions(projectId);
          setProjectSessions((prev) => ({ ...prev, [projectId]: sessions }));
        } catch (error) {
          console.error("Failed to load project sessions:", error);
        } finally {
          setLoadingProjects((prev) => {
            const next = new Set(prev);
            next.delete(projectId);
            return next;
          });
        }
      }
    },
    [expandedProjects, projectSessions, loadingProjects, getProjectSessions]
  );

  // 处理会话选择
  const handleSessionSelect = React.useCallback(
    (sessionId: string, projectId: string) => {
      onSessionSelect(sessionId, projectId);
      onOpenChange(false); // AC11: 导航后自动关闭抽屉
    },
    [onSessionSelect, onOpenChange]
  );

  // 处理导入按钮点击
  const handleImportClick = React.useCallback(() => {
    onImportClick();
    onOpenChange(false);
  }, [onImportClick, onOpenChange]);

  // Story 2.19: 处理重命名保存
  const handleRenameSave = React.useCallback(
    async (newName: string) => {
      if (!renamingProjectId) return;

      try {
        await renameProject(renamingProjectId, newName);
        onProjectsChange?.();
        setRenamingProjectId(null);
      } catch (error) {
        toast.error("重命名失败", {
          description: (error as Error).message,
        });
      }
    },
    [renamingProjectId, onProjectsChange]
  );

  // Story 2.19: 处理重命名取消
  const handleRenameCancel = React.useCallback(() => {
    setRenamingProjectId(null);
  }, []);

  // Story 2.19: 处理移除确认
  const handleRemoveConfirm = React.useCallback(async () => {
    if (!activeProject) return;

    const projectToRemove = activeProject;
    setIsRemoveDialogOpen(false);

    // 使用可撤销操作
    undoableAction.trigger({
      execute: async () => {
        await removeProject(projectToRemove.id);
        onProjectsChange?.();
        toast.success(`已移除「${projectToRemove.name}」`, {
          action: {
            label: "撤销",
            onClick: () => undoableAction.cancel(),
          },
          duration: 5000,
        });
      },
      undo: async () => {
        await restoreProject(projectToRemove.id);
        onProjectsChange?.();
        toast.success("已恢复项目");
      },
      timeoutMs: 5000,
    });
  }, [activeProject, undoableAction, onProjectsChange]);

  // 空状态
  const isEmpty = !isLoading && projects.length === 0;

  return (
    <Sheet open={isOpen} onOpenChange={onOpenChange}>
      <SheetContent
        side="left"
        className={cn(
          "w-[320px] p-0 flex flex-col",
          // AC5: 250ms ease-out 动画
          "data-[state=open]:duration-[250ms] data-[state=closed]:duration-[250ms]",
          "data-[state=open]:ease-out data-[state=closed]:ease-out"
        )}
        data-testid="project-drawer"
      >
        {/* 标题栏 */}
        <SheetHeader className="px-4 py-3 border-b shrink-0">
          <SheetTitle className="flex items-center gap-2 text-base">
            <FolderOpen className="h-5 w-5" />
            我的项目
          </SheetTitle>
          <SheetDescription className="sr-only">
            浏览和管理所有项目及会话
          </SheetDescription>
        </SheetHeader>

        {/* 搜索框 */}
        <div className="px-4 py-2 border-b shrink-0">
          <DrawerSearch
            value={searchKeyword}
            onChange={setSearchKeyword}
            placeholder="搜索项目或会话..."
          />
        </div>

        {/* 项目树列表 */}
        <div className="flex-1 overflow-y-auto">
          {isLoading ? (
            <div className="flex items-center justify-center h-32 text-muted-foreground">
              加载中...
            </div>
          ) : isEmpty ? (
            // AC14: 空项目列表
            <div
              className="flex flex-col items-center justify-center h-full p-6 text-center"
              data-testid="project-drawer-empty"
            >
              <FolderSearch className="h-12 w-12 text-muted-foreground/50 mb-4" />
              <p className="text-muted-foreground mb-4">
                还没有导入任何项目
              </p>
              <Button onClick={handleImportClick} size="sm">
                <Plus className="h-4 w-4 mr-2" />
                导入项目
              </Button>
            </div>
          ) : filteredProjects.length === 0 ? (
            <div className="flex items-center justify-center h-32 text-muted-foreground">
              未找到匹配的项目
            </div>
          ) : (
            <div className="py-2">
              {filteredProjects.map((project) => (
                <ProjectTreeItem
                  key={project.id}
                  project={project}
                  isExpanded={expandedProjects.has(project.id)}
                  isLoading={loadingProjects.has(project.id)}
                  sessions={projectSessions[project.id] || []}
                  currentSessionId={currentSessionId}
                  searchKeyword={searchKeyword}
                  onToggle={() => handleToggleProject(project.id)}
                  onSessionSelect={(sessionId) =>
                    handleSessionSelect(sessionId, project.id)
                  }
                  // Story 2.19: 重命名相关
                  isRenaming={renamingProjectId === project.id}
                  onRename={handleRenameSave}
                  onRenameCancel={handleRenameCancel}
                  // Story 2.19: 设置菜单
                  settingsMenu={
                    <ProjectContextMenu
                      onSync={async () => {
                        try {
                          const result = await syncProject(project.id);
                          showSyncResult(project.name, result);
                          if (result.new_sessions.length > 0 || result.updated_sessions.length > 0) {
                            const sessions = await getProjectSessions(project.id);
                            setProjectSessions((prev) => ({ ...prev, [project.id]: sessions }));
                            onProjectsChange?.();
                          }
                        } catch (error) {
                          showSyncResult(project.name, null, error as Error);
                        }
                      }}
                      onRename={() => {
                        setRenamingProjectId(project.id);
                      }}
                      onRemove={() => {
                        setActiveProject(project);
                        setIsRemoveDialogOpen(true);
                      }}
                    />
                  }
                />
              ))}
            </div>
          )}
        </div>

        {/* 底部导入按钮 */}
        {!isEmpty && (
          <div className="px-4 py-3 border-t shrink-0">
            <Button
              variant="outline"
              className="w-full"
              onClick={handleImportClick}
              data-testid="project-drawer-import-button"
            >
              <Plus className="h-4 w-4 mr-2" />
              导入新项目
            </Button>
          </div>
        )}
      </SheetContent>

      {/* Story 2.19: 移除确认对话框 */}
      <RemoveProjectDialog
        isOpen={isRemoveDialogOpen}
        onOpenChange={setIsRemoveDialogOpen}
        projectName={activeProject?.name ?? ""}
        onConfirm={handleRemoveConfirm}
      />
    </Sheet>
  );
}
