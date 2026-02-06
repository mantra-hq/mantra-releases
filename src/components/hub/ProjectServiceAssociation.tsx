/**
 * 项目关联配置组件
 * Story 11.6: Task 6 - 项目关联配置 (AC: #7)
 * Story 11.10: Task 4 - 工具策略编辑器集成 (AC: #3)
 * Story 12.4: 迁移使用 ActionSheet + 视图切换（避免 Sheet 套 Sheet）
 *
 * 用于管理 MCP 服务与项目的关联：
 * - 显示服务当前关联的项目列表
 * - 项目选择（多选）
 * - 项目级工具策略编辑（视图切换，非嵌套 Sheet）
 */

import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  ActionSheet,
  ActionSheetContent,
  ActionSheetDescription,
  ActionSheetFooter,
  ActionSheetHeader,
  ActionSheetTitle,
} from "@/components/ui/action-sheet";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Loader2, Folder, Link2, Shield, ArrowLeft } from "lucide-react";
import { feedback } from "@/lib/feedback";
import type { McpService } from "./McpServiceList";
import { ToolPolicyEditor } from "./ToolPolicyEditor";

/**
 * 项目信息
 */
interface Project {
  id: string;
  name: string;
}

interface ProjectServiceAssociationProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  service: McpService | null;
  onSuccess: () => void;
}

/**
 * 视图类型
 */
type ViewType = "list" | "toolPolicy";

export function ProjectServiceAssociation({
  open,
  onOpenChange,
  service,
  onSuccess,
}: ProjectServiceAssociationProps) {
  const { t } = useTranslation();
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  // 项目列表
  const [projects, setProjects] = useState<Project[]>([]);
  // 已关联的项目 ID
  const [linkedProjectIds, setLinkedProjectIds] = useState<Set<string>>(new Set());
  // 选中的项目 ID（临时状态）
  const [selectedProjectIds, setSelectedProjectIds] = useState<Set<string>>(new Set());

  // 视图切换状态（替代嵌套 Sheet）
  const [currentView, setCurrentView] = useState<ViewType>("list");
  const [policyProjectId, setPolicyProjectId] = useState<string | null>(null);
  const [policyProjectName, setPolicyProjectName] = useState<string>("");

  // 加载项目列表和已关联项目
  const loadData = useCallback(async () => {
    if (!service) return;

    setIsLoading(true);
    try {
      // 获取所有项目
      const projectList = await invoke<Project[]>("list_projects");
      setProjects(projectList);

      // 获取服务关联的项目
      const linkedIds = await invoke<string[]>("get_mcp_service_projects", {
        serviceId: service.id,
      });
      setLinkedProjectIds(new Set(linkedIds));
      setSelectedProjectIds(new Set(linkedIds));
    } catch (error) {
      console.error("[ProjectServiceAssociation] Failed to load data:", error);
      feedback.error(t("hub.projectAssociation.loadError"), (error as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [service, t]);

  useEffect(() => {
    if (open && service) {
      loadData();
      // 重置视图状态
      setCurrentView("list");
      setPolicyProjectId(null);
    }
  }, [open, service, loadData]);

  // 切换项目选择
  const handleToggleProject = useCallback((projectId: string) => {
    setSelectedProjectIds((prev) => {
      const next = new Set(prev);
      if (next.has(projectId)) {
        next.delete(projectId);
      } else {
        next.add(projectId);
      }
      return next;
    });
  }, []);

  // 保存关联
  const handleSave = useCallback(async () => {
    if (!service) return;

    setIsSaving(true);
    try {
      // 计算需要添加和移除的项目
      const toAdd = [...selectedProjectIds].filter((id) => !linkedProjectIds.has(id));
      const toRemove = [...linkedProjectIds].filter((id) => !selectedProjectIds.has(id));

      // 添加关联
      for (const projectId of toAdd) {
        await invoke("link_mcp_service_to_project", {
          projectId,
          serviceId: service.id,
          configOverride: null,
        });
      }

      // 移除关联
      for (const projectId of toRemove) {
        await invoke("unlink_mcp_service_from_project", {
          projectId,
          serviceId: service.id,
        });
      }

      feedback.success(t("hub.projectAssociation.saveSuccess"));
      onOpenChange(false);
      onSuccess();
    } catch (error) {
      console.error("[ProjectServiceAssociation] Failed to save:", error);
      feedback.error(t("hub.projectAssociation.saveError"), (error as Error).message);
    } finally {
      setIsSaving(false);
    }
  }, [service, selectedProjectIds, linkedProjectIds, onOpenChange, onSuccess, t]);

  // 打开工具策略视图
  const handleOpenToolPolicy = useCallback((projectId: string, projectName: string) => {
    setPolicyProjectId(projectId);
    setPolicyProjectName(projectName);
    setCurrentView("toolPolicy");
  }, []);

  // 返回列表视图
  const handleBackToList = useCallback(() => {
    setCurrentView("list");
    setPolicyProjectId(null);
    setPolicyProjectName("");
  }, []);

  // 计算变更数量
  const addCount = [...selectedProjectIds].filter((id) => !linkedProjectIds.has(id)).length;
  const removeCount = [...linkedProjectIds].filter((id) => !selectedProjectIds.has(id)).length;
  const hasChanges = addCount > 0 || removeCount > 0;

  return (
    <ActionSheet open={open} onOpenChange={onOpenChange}>
      <ActionSheetContent size="lg" className="overflow-hidden">
        {/* 列表视图 */}
        {currentView === "list" && (
          <>
            <ActionSheetHeader>
              <ActionSheetTitle className="flex items-center gap-2">
                <Link2 className="h-5 w-5" />
                {t("hub.projectAssociation.title")}
              </ActionSheetTitle>
              <ActionSheetDescription>
                {t("hub.projectAssociation.description", { name: service?.name })}
              </ActionSheetDescription>
            </ActionSheetHeader>

            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : projects.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <Folder className="h-12 w-12 mx-auto mb-3 opacity-20" />
                <p className="text-sm">{t("hub.projectAssociation.noProjects")}</p>
                <p className="text-xs mt-1">{t("hub.projectAssociation.noProjectsHint")}</p>
              </div>
            ) : (
              <>
                <ScrollArea className="h-[350px] pr-4">
                  <div className="space-y-2">
                    {projects.map((project) => {
                      const isSelected = selectedProjectIds.has(project.id);
                      const isLinked = linkedProjectIds.has(project.id);
                      return (
                        <div
                          key={project.id}
                          className="flex items-center justify-between p-3 rounded-md border bg-card hover:bg-accent/50 transition-colors cursor-pointer"
                          onClick={() => handleToggleProject(project.id)}
                        >
                          <div className="flex items-center gap-3">
                            <Checkbox
                              checked={isSelected}
                              onCheckedChange={() => handleToggleProject(project.id)}
                              onClick={(e: React.MouseEvent) => e.stopPropagation()}
                              className="border-zinc-400 data-[state=unchecked]:bg-zinc-700/30"
                              data-testid={`project-checkbox-${project.id}`}
                            />
                            <div className="flex items-center gap-2">
                              <Folder className="h-4 w-4 text-muted-foreground" />
                              <span className="text-sm">{project.name}</span>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            {isLinked && (
                              <>
                                <Badge variant="outline" className="text-xs">
                                  {t("hub.projectAssociation.linked")}
                                </Badge>
                                {/* Story 12.5: 文案区分 - 项目工具权限 */}
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => handleOpenToolPolicy(project.id, project.name)}
                                  className="h-7 text-xs gap-1"
                                >
                                  <Shield className="h-3 w-3" />
                                  {t("hub.toolPolicy.projectEntry", "Project Tool Policy")}
                                </Button>
                              </>
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </ScrollArea>

                {/* 变更提示 */}
                {hasChanges && (
                  <div className="text-xs text-muted-foreground text-center py-2">
                    {addCount > 0 && t("hub.projectAssociation.toAdd", { count: addCount })}
                    {addCount > 0 && removeCount > 0 && " • "}
                    {removeCount > 0 && t("hub.projectAssociation.toRemove", { count: removeCount })}
                  </div>
                )}
              </>
            )}

            <ActionSheetFooter>
              <Button
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={isSaving}
              >
                {t("common.cancel")}
              </Button>
              <Button
                onClick={handleSave}
                disabled={isSaving || !hasChanges}
                data-testid="project-association-save"
              >
                {isSaving && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                {t("common.save")}
              </Button>
            </ActionSheetFooter>
          </>
        )}

        {/* 工具策略视图 - Story 11.18 Fix: 与 McpContextSection 布局保持一致 */}
        {currentView === "toolPolicy" && service && policyProjectId && (
          <>
            <ActionSheetHeader>
              <ActionSheetTitle className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 -ml-2"
                  onClick={handleBackToList}
                >
                  <ArrowLeft className="h-4 w-4" />
                </Button>
                <Shield className="h-5 w-5" />
                {t("hub.mcpContext.toolPermissions", "Tool Permissions")}
                <Badge variant="outline" className="ml-2">
                  {service.name}
                </Badge>
              </ActionSheetTitle>
              <ActionSheetDescription>
                {t("hub.toolPolicy.projectScope", "项目: {{project}}", { project: policyProjectName })}
              </ActionSheetDescription>
            </ActionSheetHeader>
            {/* Story 12.5: 添加 projectName 用于上下文提示 */}
            {/* Story 11.18 Fix: 移除包装 div，直接放置 ToolPolicyEditor */}
            <ToolPolicyEditor
              serviceId={service.id}
              projectId={policyProjectId}
              serviceName={service.name}
              projectName={policyProjectName}
              embedded
              onSaved={() => {
                // Story 11.18 Fix: 与项目详情入口保持一致的行为
                handleBackToList();  // 返回列表视图
                loadData();          // 刷新数据（确保策略变更可见）
                feedback.success(t("hub.projectAssociation.overrideSaveSuccess"));
              }}
            />
          </>
        )}
      </ActionSheetContent>
    </ActionSheet>
  );
}

export default ProjectServiceAssociation;
