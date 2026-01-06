/**
 * ProjectDrawer Component - 项目抽屉
 * Story 2.18: Task 2
 * Story 2.19: Task 10 - 集成项目管理功能
 * Story 2-26: i18n 国际化
 *
 * 侧边抽屉，用于浏览和管理所有项目
 * - 从左侧滑入，宽度 320px
 * - 包含标题栏、搜索框、项目树列表和导入按钮
 * - 支持项目同步、重命名、移除操作
 */

import * as React from "react";
import { FolderOpen, Plus, Rocket, ChevronsDownUp } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { DrawerSearch } from "./DrawerSearch";
import { ProjectTreeItem } from "./ProjectTreeItem";
import { ProjectContextMenu } from "./ProjectContextMenu";
import { RemoveProjectDialog } from "./RemoveProjectDialog";
import { ProjectInfoDialog } from "./ProjectInfoDialog";
import { showSyncResult } from "./SyncResultToast";
import {
  syncProject,
  removeProject,
  renameProject,
} from "@/lib/project-ipc";
import { toast } from "sonner";
import { appLog } from "@/lib/log-actions";
import { useHideEmptyProjects } from "@/hooks/useHideEmptyProjects";
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
  /** 当前项目 ID（用于检测移除当前项目） */
  currentProjectId?: string;
  /** 会话点击回调 */
  onSessionSelect: (sessionId: string, projectId: string) => void;
  /** 导入按钮点击回调 */
  onImportClick: () => void;
  /** 获取项目会话列表 */
  getProjectSessions: (projectId: string) => Promise<SessionSummary[]>;
  /** Story 2.19: 项目列表变更回调（用于刷新列表） */
  onProjectsChange?: () => void;
  /** 当前项目被移除时的回调（用于导航到空状态） */
  onCurrentProjectRemoved?: () => void;
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
  currentProjectId,
  onSessionSelect,
  onImportClick,
  getProjectSessions,
  onProjectsChange,
  onCurrentProjectRemoved,
}: ProjectDrawerProps) {
  const { t } = useTranslation();
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
  // Story 2.18 fix: 菜单打开的项目 ID（用于保持按钮可见）
  const [menuOpenProjectId, setMenuOpenProjectId] = React.useState<string | null>(null);
  // Story 2.27: 显示详情对话框的项目
  const [infoProject, setInfoProject] = React.useState<Project | null>(null);
  // Story 2.29: 隐藏空项目偏好
  const [hideEmptyProjects, setHideEmptyProjects] = useHideEmptyProjects();

  // 一键折叠所有项目
  const handleCollapseAll = React.useCallback(() => {
    setExpandedProjects(new Set());
  }, []);

  // 过滤后的项目列表
  const filteredProjects = React.useMemo(() => {
    let result = projects;

    // Story 2.29 V2: 使用项目的 is_empty 字段进行过滤（加载时已确定）
    if (hideEmptyProjects) {
      result = result.filter((project) => !project.is_empty);
    }

    // 搜索过滤
    if (searchKeyword.trim()) {
      const keyword = searchKeyword.toLowerCase();
      result = result.filter((project) => {
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
    }

    return result;
  }, [projects, searchKeyword, projectSessions, hideEmptyProjects]);

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
        // Story 2.28: 记录重命名日志
        const oldProject = projects.find((p) => p.id === renamingProjectId);
        appLog.projectRenamed(oldProject?.name || renamingProjectId, newName);
        onProjectsChange?.();
        setRenamingProjectId(null);
      } catch (error) {
        toast.error(t("project.renameFailed", "重命名失败"), {
          description: (error as Error).message,
        });
      }
    },
    [renamingProjectId, onProjectsChange, t]
  );

  // Story 2.19: 处理重命名取消
  const handleRenameCancel = React.useCallback(() => {
    setRenamingProjectId(null);
  }, []);

  // Story 2.19: 处理移除确认
  const handleRemoveConfirm = React.useCallback(async () => {
    if (!activeProject) return;

    const projectToRemove = activeProject;
    const isRemovingCurrentProject = projectToRemove.id === currentProjectId;
    setIsRemoveDialogOpen(false);

    try {
      await removeProject(projectToRemove.id);
      onProjectsChange?.();
      // 如果移除的是当前正在查看的项目，导航到空状态
      if (isRemovingCurrentProject) {
        onCurrentProjectRemoved?.();
      }
      toast.success(t("project.removed", { name: projectToRemove.name }));
      // Story 2.28: 记录移除日志
      appLog.projectRemoved(projectToRemove.name);
    } catch (error) {
      toast.error(t("project.removeFailed"), {
        description: (error as Error).message,
      });
    }
  }, [activeProject, currentProjectId, onProjectsChange, onCurrentProjectRemoved, t]);

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
            {t("project.myProjects")}
            {expandedProjects.size > 0 && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6 ml-1"
                onClick={handleCollapseAll}
                aria-label={t("common.collapseAll")}
                title={t("common.collapseAll")}
              >
                <ChevronsDownUp className="h-4 w-4" />
              </Button>
            )}
          </SheetTitle>
          <SheetDescription className="sr-only">
            {t("project.browseAndManage")}
          </SheetDescription>
        </SheetHeader>

        {/* 搜索框 */}
        <div className="px-4 py-2 border-b shrink-0">
          <DrawerSearch
            value={searchKeyword}
            onChange={setSearchKeyword}
            placeholder={t("import.searchProjectOrSession") + "..."}
          />
          {/* Story 2.29 V2: 隐藏空会话复选框（默认勾选）*/}
          <label className="flex items-center gap-2 mt-2 cursor-pointer">
            <Checkbox
              checked={hideEmptyProjects}
              onCheckedChange={(checked) => setHideEmptyProjects(checked === true)}
              data-testid="hide-empty-projects-checkbox"
            />
            <span className="text-xs text-muted-foreground">
              {t("project.hideEmptyProjects")}
            </span>
          </label>
        </div>

        {/* 项目树列表 */}
        <div className="flex-1 overflow-y-auto pb-4">
          {isLoading ? (
            <div className="flex items-center justify-center h-32 text-muted-foreground">
              {t("common.loading")}...
            </div>
          ) : isEmpty ? (
            // AC14: 空项目列表 (Story 2.21 AC #17: 与 Player 空状态样式一致)
            <div
              className="flex flex-col items-center justify-center h-full p-6 text-center"
              data-testid="project-drawer-empty"
            >
              {/* 图标容器 - 与 PlayerEmptyState 一致 */}
              <div className="w-16 h-16 mb-4 flex items-center justify-center rounded-xl bg-muted/50">
                <FolderOpen className="h-8 w-8 text-muted-foreground/70" />
              </div>
              {/* 主标题 */}
              <p className="text-sm font-medium text-foreground mb-1">
                {t("project.noProjects")}
              </p>
              {/* 副标题 */}
              <p className="text-xs text-muted-foreground mb-4">
                {t("project.importPrompt")}
              </p>
              {/* CTA 按钮 */}
              <Button onClick={handleImportClick} size="sm" className="gap-1.5">
                <Rocket className="h-3.5 w-3.5" />
                {t("import.importFirstProject")}
              </Button>
              {/* 支持说明 */}
              <p className="text-[10px] text-muted-foreground/70 mt-3">
                {t("import.supportedTools")}
              </p>
            </div>
          ) : filteredProjects.length === 0 ? (
            <div className="flex items-center justify-center h-32 text-muted-foreground">
              {t("project.noMatch")}
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
                  // Story 2.18 fix: 菜单打开状态
                  isSettingsMenuOpen={menuOpenProjectId === project.id}
                  // Story 2.29 V2: 隐藏空会话
                  hideEmptySessions={hideEmptyProjects}
                  // Story 2.19: 设置菜单
                  settingsMenu={
                    <ProjectContextMenu
                      onOpenChange={(open) => {
                        setMenuOpenProjectId(open ? project.id : null);
                      }}
                      onSync={async () => {
                        appLog.syncStart(project.name);
                        try {
                          const result = await syncProject(project.id);
                          showSyncResult(project.name, result);
                          appLog.syncComplete(project.name, result.new_sessions.length, result.updated_sessions.length);
                          if (result.new_sessions.length > 0 || result.updated_sessions.length > 0) {
                            const sessions = await getProjectSessions(project.id);
                            setProjectSessions((prev) => ({ ...prev, [project.id]: sessions }));
                            onProjectsChange?.();
                          }
                        } catch (error) {
                          showSyncResult(project.name, null, error as Error);
                          appLog.syncError(project.name, (error as Error).message);
                        }
                      }}
                      onForceSync={async () => {
                        appLog.syncStart(project.name + " (force)");
                        try {
                          const result = await syncProject(project.id, true);
                          showSyncResult(project.name, result, undefined, true);
                          appLog.syncComplete(project.name, result.new_sessions.length, result.updated_sessions.length);
                          // 强制重新解析后总是刷新会话列表
                          const sessions = await getProjectSessions(project.id);
                          setProjectSessions((prev) => ({ ...prev, [project.id]: sessions }));
                          onProjectsChange?.();
                        } catch (error) {
                          showSyncResult(project.name, null, error as Error);
                          appLog.syncError(project.name, (error as Error).message);
                        }
                      }}
                      onRename={() => {
                        setRenamingProjectId(project.id);
                      }}
                      onRemove={() => {
                        setActiveProject(project);
                        setIsRemoveDialogOpen(true);
                      }}
                      onViewInfo={() => {
                        setInfoProject(project);
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
              {t("project.importNew")}
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

      {/* Story 2.27: 项目元信息对话框 */}
      {/* Story 1.9: Task 8.4 - 添加 onProjectUpdated 回调支持项目 cwd 更新 */}
      <ProjectInfoDialog
        isOpen={infoProject !== null}
        onOpenChange={(open) => {
          if (!open) setInfoProject(null);
        }}
        project={infoProject}
        getProjectSessions={getProjectSessions}
        onProjectUpdated={(updatedProject) => {
          // 更新 infoProject 状态以刷新对话框显示
          setInfoProject(updatedProject);
          // 触发项目列表刷新
          onProjectsChange?.();
        }}
      />
    </Sheet>
  );
}
