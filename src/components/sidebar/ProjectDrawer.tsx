/**
 * ProjectDrawer Component - 项目抽屉
 * Story 2.18: Task 2
 * Story 2.19: Task 10 - 集成项目管理功能
 * Story 2-26: i18n 国际化
 * Story 1.12: Phase 5 - 完全切换到逻辑项目视图
 *
 * 侧边抽屉，用于浏览和管理所有项目
 * - 从左侧滑入，宽度 320px
 * - 包含标题栏、搜索框、项目树列表和导入按钮
 * - 显示按物理路径聚合的逻辑项目
 * - Task 12: 移除"未分类"分组，虚拟路径作为独立逻辑项目
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
import { LogicalProjectTreeItem } from "./LogicalProjectTreeItem";
import { LogicalProjectContextMenu } from "./LogicalProjectContextMenu";
import { RemoveProjectDialog } from "./RemoveProjectDialog";
import { ProjectInfoDialog } from "./ProjectInfoDialog";
import { showSyncResult } from "./SyncResultToast";
import {
  syncProject,
  removeProject,
} from "@/lib/project-ipc";
import { toast } from "sonner";
import { appLog } from "@/lib/log-actions";
import { useHideEmptyProjects } from "@/hooks/useHideEmptyProjects";
import type { LogicalProjectStats } from "@/types/project";
import type { SessionSummary } from "./types";

/**
 * ProjectDrawer Props
 * Story 1.12: Phase 5 - 改用逻辑项目视图
 */
export interface ProjectDrawerProps {
  /** 抽屉是否打开 */
  isOpen: boolean;
  /** 抽屉开关回调 */
  onOpenChange: (open: boolean) => void;
  /** 逻辑项目列表 (按物理路径聚合) - Story 1.12 */
  logicalProjects: LogicalProjectStats[];
  /** 是否正在加载 */
  isLoading?: boolean;
  /** 当前会话 ID（用于高亮当前选中） */
  currentSessionId?: string;
  /** 当前物理路径（用于检测移除当前项目） - Story 1.12 */
  currentPhysicalPath?: string;
  /** 会话点击回调 - projectId 参数改为物理路径 */
  onSessionSelect: (sessionId: string, physicalPath: string) => void;
  /** 导入按钮点击回调 */
  onImportClick: () => void;
  /** 获取逻辑项目会话列表 (按物理路径) - Story 1.12 */
  getLogicalProjectSessions: (physicalPath: string) => Promise<SessionSummary[]>;
  /** 项目列表变更回调（用于刷新列表） */
  onProjectsChange?: () => void;
  /** 当前项目被移除时的回调（用于导航到空状态） */
  onCurrentProjectRemoved?: () => void;
}

/**
 * ProjectDrawer 组件
 * 项目抽屉，从左侧滑入显示逻辑项目树
 * Story 1.12: Phase 5 - 完全使用逻辑项目视图
 */
export function ProjectDrawer({
  isOpen,
  onOpenChange,
  logicalProjects,
  isLoading = false,
  currentSessionId,
  currentPhysicalPath,
  onSessionSelect,
  onImportClick,
  getLogicalProjectSessions,
  onProjectsChange,
  onCurrentProjectRemoved,
}: ProjectDrawerProps) {
  const { t } = useTranslation();
  // 搜索关键词状态
  const [searchKeyword, setSearchKeyword] = React.useState("");
  // 展开的逻辑项目路径集合 (使用 physical_path 作为 key)
  const [expandedProjects, setExpandedProjects] = React.useState<Set<string>>(
    new Set()
  );
  // 逻辑项目会话缓存 (key: physical_path)
  const [projectSessions, setProjectSessions] = React.useState<
    Record<string, SessionSummary[]>
  >({});
  // 加载中的逻辑项目路径集合
  const [loadingProjects, setLoadingProjects] = React.useState<Set<string>>(
    new Set()
  );

  // Story 2.19: 项目管理状态
  // 当前操作的逻辑项目（用于对话框）
  const [activeLogicalProject, setActiveLogicalProject] = React.useState<LogicalProjectStats | null>(null);
  // 移除对话框打开状态
  const [isRemoveDialogOpen, setIsRemoveDialogOpen] = React.useState(false);
  // Story 2.18 fix: 菜单打开的逻辑项目路径（用于保持按钮可见）
  const [menuOpenPath, setMenuOpenPath] = React.useState<string | null>(null);
  // Story 2.27: 显示详情对话框的逻辑项目
  const [infoLogicalProject, setInfoLogicalProject] = React.useState<LogicalProjectStats | null>(null);
  // Story 2.29: 隐藏空项目偏好
  const [hideEmptyProjects, setHideEmptyProjects] = useHideEmptyProjects();

  // Task 12: 移除未分类会话相关状态（虚拟路径作为独立逻辑项目显示）

  // 一键折叠所有项目
  const handleCollapseAll = React.useCallback(() => {
    setExpandedProjects(new Set());
  }, []);

  // 过滤后的逻辑项目列表
  const filteredProjects = React.useMemo(() => {
    let result = logicalProjects;

    // Story 2.29 V2: 使用 total_sessions 过滤空项目
    if (hideEmptyProjects) {
      result = result.filter((lp) => lp.total_sessions > 0);
    }

    // 搜索过滤
    if (searchKeyword.trim()) {
      const keyword = searchKeyword.toLowerCase();
      result = result.filter((lp) => {
        // 匹配显示名称
        if (lp.display_name.toLowerCase().includes(keyword)) return true;

        // 匹配物理路径
        if (lp.physical_path.toLowerCase().includes(keyword)) return true;

        // 匹配会话（如果已加载）
        const sessions = projectSessions[lp.physical_path];
        if (sessions?.some((s) => s.id.toLowerCase().includes(keyword))) {
          return true;
        }

        return false;
      });
    }

    return result;
  }, [logicalProjects, searchKeyword, projectSessions, hideEmptyProjects]);

  // 切换逻辑项目展开状态
  const handleToggleProject = React.useCallback(
    async (physicalPath: string) => {
      setExpandedProjects((prev) => {
        const next = new Set(prev);
        if (next.has(physicalPath)) {
          next.delete(physicalPath);
        } else {
          next.add(physicalPath);
        }
        return next;
      });

      // 如果展开且尚未加载会话，则加载
      if (
        !expandedProjects.has(physicalPath) &&
        !projectSessions[physicalPath] &&
        !loadingProjects.has(physicalPath)
      ) {
        setLoadingProjects((prev) => new Set(prev).add(physicalPath));
        try {
          const sessions = await getLogicalProjectSessions(physicalPath);
          setProjectSessions((prev) => ({ ...prev, [physicalPath]: sessions }));
        } catch (error) {
          console.error("Failed to load logical project sessions:", error);
        } finally {
          setLoadingProjects((prev) => {
            const next = new Set(prev);
            next.delete(physicalPath);
            return next;
          });
        }
      }
    },
    [expandedProjects, projectSessions, loadingProjects, getLogicalProjectSessions]
  );

  // 处理会话选择
  const handleSessionSelect = React.useCallback(
    (sessionId: string, physicalPath: string) => {
      onSessionSelect(sessionId, physicalPath);
      onOpenChange(false); // AC11: 导航后自动关闭抽屉
    },
    [onSessionSelect, onOpenChange]
  );

  // 处理导入按钮点击
  const handleImportClick = React.useCallback(() => {
    onImportClick();
    onOpenChange(false);
  }, [onImportClick, onOpenChange]);

  // Story 2.19: 处理移除确认（移除逻辑项目关联的所有存储层项目）
  const handleRemoveConfirm = React.useCallback(async () => {
    if (!activeLogicalProject) return;

    const logicalProjectToRemove = activeLogicalProject;
    const isRemovingCurrent = logicalProjectToRemove.physical_path === currentPhysicalPath;
    setIsRemoveDialogOpen(false);

    try {
      // 移除所有关联的存储层项目
      for (const projectId of logicalProjectToRemove.project_ids) {
        await removeProject(projectId);
      }
      onProjectsChange?.();
      // 如果移除的是当前正在查看的项目，导航到空状态
      if (isRemovingCurrent) {
        onCurrentProjectRemoved?.();
      }
      toast.success(t("project.removed", { name: logicalProjectToRemove.display_name }));
      // 记录移除日志
      appLog.projectRemoved(logicalProjectToRemove.display_name);
    } catch (error) {
      toast.error(t("project.removeFailed"), {
        description: (error as Error).message,
      });
    }
  }, [activeLogicalProject, currentPhysicalPath, onProjectsChange, onCurrentProjectRemoved, t]);

  // 空状态
  const isEmpty = !isLoading && logicalProjects.length === 0;

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
            <div data-testid="project-list" className="py-2">
              {/* Story 1.12: 使用 LogicalProjectTreeItem 显示逻辑项目 */}
              {filteredProjects.map((lp) => (
                <LogicalProjectTreeItem
                  key={lp.physical_path}
                  logicalProject={lp}
                  isExpanded={expandedProjects.has(lp.physical_path)}
                  isLoading={loadingProjects.has(lp.physical_path)}
                  sessions={projectSessions[lp.physical_path] || []}
                  currentSessionId={currentSessionId}
                  searchKeyword={searchKeyword}
                  onToggle={() => handleToggleProject(lp.physical_path)}
                  onSessionSelect={(sessionId) =>
                    handleSessionSelect(sessionId, lp.physical_path)
                  }
                  onProjectClick={() => {
                    setInfoLogicalProject(lp);
                  }}
                  // Story 2.18 fix: 菜单打开状态
                  isSettingsMenuOpen={menuOpenPath === lp.physical_path}
                  // Story 2.29 V2: 隐藏空会话
                  hideEmptySessions={hideEmptyProjects}
                  // 设置菜单
                  settingsMenu={
                    <LogicalProjectContextMenu
                      logicalProject={lp}
                      onOpenChange={(open) => {
                        setMenuOpenPath(open ? lp.physical_path : null);
                      }}
                      onSync={async () => {
                        appLog.syncStart(lp.display_name);
                        try {
                          // 同步所有关联的存储层项目
                          let totalNew = 0;
                          let totalUpdated = 0;
                          for (const projectId of lp.project_ids) {
                            const result = await syncProject(projectId);
                            totalNew += result.new_sessions.length;
                            totalUpdated += result.updated_sessions.length;
                          }
                          showSyncResult(lp.display_name, {
                            new_sessions: Array(totalNew).fill(null) as any[],
                            updated_sessions: Array(totalUpdated).fill(null) as any[],
                            unchanged_count: 0,
                          });
                          appLog.syncComplete(lp.display_name, totalNew, totalUpdated);
                          if (totalNew > 0 || totalUpdated > 0) {
                            const sessions = await getLogicalProjectSessions(lp.physical_path);
                            setProjectSessions((prev) => ({ ...prev, [lp.physical_path]: sessions }));
                            onProjectsChange?.();
                          }
                        } catch (error) {
                          showSyncResult(lp.display_name, null, error as Error);
                          appLog.syncError(lp.display_name, (error as Error).message);
                        }
                      }}
                      onRemove={() => {
                        setActiveLogicalProject(lp);
                        setIsRemoveDialogOpen(true);
                      }}
                      onViewInfo={() => {
                        setInfoLogicalProject(lp);
                      }}
                    />
                  }
                />
              ))}

              {/* Task 12: 移除"未分类"分组 - 虚拟路径作为独立逻辑项目显示 */}
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
        projectName={activeLogicalProject?.display_name ?? ""}
        onConfirm={handleRemoveConfirm}
      />

      {/* Story 2.27 + 1.12: 逻辑项目信息对话框 */}
      {/* Task 15: 详情页统一为唯一关联入口 */}
      <ProjectInfoDialog
        isOpen={infoLogicalProject !== null}
        onOpenChange={(open) => {
          if (!open) setInfoLogicalProject(null);
        }}
        logicalProject={infoLogicalProject}
        getLogicalProjectSessions={getLogicalProjectSessions}
        onProjectUpdated={() => {
          // 触发项目列表刷新
          onProjectsChange?.();
        }}
      />

      {/* Task 12: 移除会话绑定对话框 - 虚拟路径作为独立逻辑项目显示 */}
    </Sheet>
  );
}
