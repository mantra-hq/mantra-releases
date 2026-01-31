/**
 * 项目关联配置组件
 * Story 11.6: Task 6 - 项目关联配置 (AC: #7)
 * Story 11.10: Task 4 - 工具策略编辑器集成 (AC: #3)
 *
 * 用于管理 MCP 服务与项目的关联：
 * - 显示服务当前关联的项目列表
 * - 项目选择对话框（多选）
 * - 项目级 config_override 编辑器
 * - 工具策略编辑器
 */

import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Loader2, Folder, Link2, Shield, Code } from "lucide-react";
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

/**
 * 项目 MCP 服务配置覆盖
 */
interface ProjectServiceOverride {
  project_id: string;
  service_id: string;
  config_override: Record<string, unknown> | null;
}

interface ProjectServiceAssociationProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  service: McpService | null;
  onSuccess: () => void;
}

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
  // 配置覆盖编辑状态
  const [editOverrideProject, setEditOverrideProject] = useState<string | null>(null);
  const [overrideText, setOverrideText] = useState("");
  const [overrideError, setOverrideError] = useState<string | null>(null);

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

  // 打开配置覆盖编辑器
  const handleEditOverride = useCallback(async (projectId: string) => {
    if (!service) return;

    try {
      // 获取当前覆盖配置
      const overrides = await invoke<ProjectServiceOverride[]>("get_project_mcp_services", {
        projectId,
      });
      const current = overrides.find((o) => o.service_id === service.id);
      setOverrideText(
        current?.config_override
          ? JSON.stringify(current.config_override, null, 2)
          : ""
      );
      setOverrideError(null);
      setEditOverrideProject(projectId);
    } catch (error) {
      console.error("[ProjectServiceAssociation] Failed to get override:", error);
      feedback.error(t("hub.projectAssociation.loadOverrideError"), (error as Error).message);
    }
  }, [service, t]);

  // 保存配置覆盖
  const handleSaveOverride = useCallback(async () => {
    if (!service || !editOverrideProject) return;

    // 验证 JSON
    let configOverride: Record<string, unknown> | null = null;
    if (overrideText.trim()) {
      try {
        configOverride = JSON.parse(overrideText);
        if (typeof configOverride !== "object" || Array.isArray(configOverride)) {
          setOverrideError(t("hub.projectAssociation.overrideMustBeObject"));
          return;
        }
      } catch {
        setOverrideError(t("hub.services.form.invalidJson"));
        return;
      }
    }

    try {
      await invoke("update_project_mcp_service_override", {
        projectId: editOverrideProject,
        serviceId: service.id,
        configOverride,
      });
      feedback.success(t("hub.projectAssociation.overrideSaveSuccess"));
      setEditOverrideProject(null);
    } catch (error) {
      console.error("[ProjectServiceAssociation] Failed to save override:", error);
      feedback.error(t("hub.projectAssociation.overrideSaveError"), (error as Error).message);
    }
  }, [service, editOverrideProject, overrideText, t]);

  // 计算变更数量
  const addCount = [...selectedProjectIds].filter((id) => !linkedProjectIds.has(id)).length;
  const removeCount = [...linkedProjectIds].filter((id) => !selectedProjectIds.has(id)).length;
  const hasChanges = addCount > 0 || removeCount > 0;

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Link2 className="h-5 w-5" />
              {t("hub.projectAssociation.title")}
            </DialogTitle>
            <DialogDescription>
              {t("hub.projectAssociation.description", { name: service?.name })}
            </DialogDescription>
          </DialogHeader>

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
              <ScrollArea className="h-[300px] pr-4">
                <div className="space-y-2">
                  {projects.map((project) => {
                    const isSelected = selectedProjectIds.has(project.id);
                    const isLinked = linkedProjectIds.has(project.id);
                    return (
                      <div
                        key={project.id}
                        className="flex items-center justify-between p-3 rounded-md border bg-card hover:bg-accent/50 transition-colors"
                      >
                        <div className="flex items-center gap-3">
                          <Checkbox
                            checked={isSelected}
                            onCheckedChange={() => handleToggleProject(project.id)}
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
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => handleEditOverride(project.id)}
                                className="h-7 text-xs"
                              >
                                {t("hub.projectAssociation.editOverride")}
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
                <div className="text-xs text-muted-foreground text-center">
                  {addCount > 0 && t("hub.projectAssociation.toAdd", { count: addCount })}
                  {addCount > 0 && removeCount > 0 && " • "}
                  {removeCount > 0 && t("hub.projectAssociation.toRemove", { count: removeCount })}
                </div>
              )}
            </>
          )}

          <DialogFooter>
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
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 配置覆盖编辑对话框 - 包含工具策略编辑器 */}
      <Dialog
        open={!!editOverrideProject}
        onOpenChange={(open) => !open && setEditOverrideProject(null)}
      >
        <DialogContent className="sm:max-w-[600px] max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>{t("hub.projectAssociation.overrideTitle")}</DialogTitle>
            <DialogDescription>
              {t("hub.projectAssociation.overrideDescription")}
            </DialogDescription>
          </DialogHeader>

          <Tabs defaultValue="toolPolicy" className="w-full">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="toolPolicy" className="gap-2">
                <Shield className="h-4 w-4" />
                {t("hub.toolPolicy.title")}
              </TabsTrigger>
              <TabsTrigger value="advanced" className="gap-2">
                <Code className="h-4 w-4" />
                {t("hub.projectAssociation.advancedTab", "Advanced")}
              </TabsTrigger>
            </TabsList>

            <TabsContent value="toolPolicy" className="mt-4">
              {service && editOverrideProject && (
                <ToolPolicyEditor
                  serviceId={service.id}
                  projectId={editOverrideProject}
                  serviceName={service.name}
                  onSaved={() => {
                    feedback.success(t("hub.projectAssociation.overrideSaveSuccess"));
                  }}
                />
              )}
            </TabsContent>

            <TabsContent value="advanced" className="mt-4">
              <div className="space-y-2">
                <Label>{t("hub.projectAssociation.overrideLabel")}</Label>
                <Textarea
                  value={overrideText}
                  onChange={(e) => setOverrideText(e.target.value)}
                  placeholder={t("hub.projectAssociation.overridePlaceholder")}
                  className="font-mono text-sm min-h-[150px]"
                  data-testid="config-override-input"
                />
                <p className="text-xs text-muted-foreground">
                  {t("hub.projectAssociation.overrideHint")}
                </p>
                {overrideError && (
                  <p className="text-xs text-destructive">{overrideError}</p>
                )}
              </div>

              <div className="flex justify-end mt-4">
                <Button onClick={handleSaveOverride} data-testid="config-override-save">
                  {t("common.save")}
                </Button>
              </div>
            </TabsContent>
          </Tabs>

          <DialogFooter className="border-t pt-4 mt-2">
            <Button
              variant="outline"
              onClick={() => setEditOverrideProject(null)}
            >
              {t("common.close", "Close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

export default ProjectServiceAssociation;
