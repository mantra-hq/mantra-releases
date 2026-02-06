/**
 * MCP Context Section 组件
 * Story 11.18: AC2 - 项目视角入口
 *
 * 在项目详情 Sheet 中显示 MCP 服务状态（不使用 Card 包装）：
 * - 已关联的服务列表及运行状态
 * - [+ 关联更多服务] 展开服务选择列表
 * - 工具策略管理入口
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  ActionSheet,
  ActionSheetContent,
  ActionSheetHeader,
  ActionSheetTitle,
} from "@/components/ui/action-sheet";
import {
  Plug,
  Plus,
  Loader2,
  Shield,
  ShieldAlert,
  ChevronDown,
  Check,
} from "lucide-react";
import { McpServiceStatusDot, type ServiceStatus } from "@/components/hub/McpServiceStatusDot";
import { SourceIcon } from "@/components/import/SourceIcons";
import { McpConfigImportSheet } from "@/components/hub/McpConfigImportSheet";
import { ToolPolicyEditor } from "@/components/hub/ToolPolicyEditor";
import { feedback } from "@/lib/feedback";

// ===== 类型定义 =====

/** MCP 服务摘要 */
interface McpServiceSummary {
  id: string;
  name: string;
  adapter_id: string;
  is_running: boolean;
  error_message: string | null;
  tool_policy_mode: string | null;
  custom_tools_count: number | null;
}

/** 可检测的配置 */
interface DetectableConfig {
  adapter_id: string;
  config_path: string;
  scope: string;
  service_count: number;
}

/** 项目 MCP 状态 */
interface ProjectMcpStatus {
  is_taken_over: boolean;
  associated_services: McpServiceSummary[];
  detectable_configs: DetectableConfig[];
}

/** 全局 MCP 服务 */
interface McpService {
  id: string;
  name: string;
  source_file?: string;
}

// ===== 组件属性 =====

export interface McpContextSectionProps {
  projectId: string;
  projectName?: string;
  projectPath?: string;
  onStatusChange?: () => void;
}

function getServiceStatus(service: McpServiceSummary): ServiceStatus {
  if (service.error_message) return "error";
  if (service.is_running) return "running";
  return "stopped";
}

function inferAdapterId(sourceFile?: string): string {
  if (!sourceFile) return "unknown";
  if (sourceFile.includes("claude") || sourceFile.includes(".mcp.json")) return "claude";
  if (sourceFile.includes("cursor")) return "cursor";
  if (sourceFile.includes("codex")) return "codex";
  if (sourceFile.includes("gemini")) return "gemini";
  return "unknown";
}

/**
 * 策略状态徽标
 * Story 11.18: 简化后只有 allow_all 和 custom 两种显示状态
 * - allow_all/inherit: 不显示（默认状态）
 * - custom: 黄色 ShieldAlert "Custom N"
 * 注: deny_all 模式已废弃，不关联服务 = 禁用
 */
function PolicyBadge({ service, t }: { service: McpServiceSummary; t: (key: string, fallback: string, opts?: Record<string, unknown>) => string }) {
  // Story 11.18: 只有 custom 模式需要显示徽标
  // allow_all, inherit, 或未设置都是默认状态，不显示
  if (!service.tool_policy_mode ||
      service.tool_policy_mode === "allow_all" ||
      service.tool_policy_mode === "inherit") {
    return null;
  }

  if (service.tool_policy_mode === "custom") {
    return (
      <Badge variant="outline" className="text-[10px] px-1.5 py-0 h-5 gap-1 border-yellow-500/50 text-yellow-500">
        <ShieldAlert className="h-3 w-3" />
        {t("hub.mcpContext.policyCustom", "Custom {{count}}", { count: service.custom_tools_count ?? 0 })}
      </Badge>
    );
  }

  return null;
}

export function McpContextSection({
  projectId,
  projectName,
  projectPath,
  onStatusChange,
}: McpContextSectionProps) {
  const { t } = useTranslation();

  const [status, setStatus] = useState<ProjectMcpStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [policyDialogService, setPolicyDialogService] = useState<McpServiceSummary | null>(null);

  // Story 11.18: 服务选择状态
  const [isLinkOpen, setIsLinkOpen] = useState(false);
  const [allServices, setAllServices] = useState<McpService[]>([]);
  const [isLoadingServices, setIsLoadingServices] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [selectedServiceIds, setSelectedServiceIds] = useState<Set<string>>(new Set());

  // 加载项目 MCP 状态
  const loadStatus = useCallback(async () => {
    // DEBUG: 打印传入的参数
    console.log("[McpContextSection] loadStatus called with:", {
      projectId,
      projectPath,
      projectName,
    });

    if (!projectId) {
      console.warn("[McpContextSection] No projectId provided, skipping load");
      setIsLoading(false);
      return;
    }

    try {
      const result = await invoke<ProjectMcpStatus>("check_project_mcp_status", {
        projectId,
        projectPath: projectPath || null,
      });
      // DEBUG: 打印返回结果
      console.log("[McpContextSection] check_project_mcp_status result:", {
        is_taken_over: result.is_taken_over,
        associated_services_count: result.associated_services.length,
        associated_services: result.associated_services.map(s => ({ id: s.id, name: s.name })),
        detectable_configs_count: result.detectable_configs.length,
      });
      setStatus(result);
      // 初始化选中状态
      setSelectedServiceIds(new Set(result.associated_services.map(s => s.id)));
    } catch (error) {
      console.error("[McpContextSection] Failed to load status:", error);
      setStatus(null);
    } finally {
      setIsLoading(false);
    }
  }, [projectId, projectPath]);

  // 加载所有可用服务
  const loadAllServices = useCallback(async () => {
    setIsLoadingServices(true);
    try {
      const services = await invoke<McpService[]>("list_mcp_services");
      setAllServices(services);
    } catch (error) {
      console.error("[McpContextSection] Failed to load services:", error);
    } finally {
      setIsLoadingServices(false);
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  // 展开服务选择时加载所有服务
  useEffect(() => {
    if (isLinkOpen && allServices.length === 0) {
      loadAllServices();
    }
  }, [isLinkOpen, allServices.length, loadAllServices]);

  const handleImportSuccess = useCallback(() => {
    loadStatus();
    onStatusChange?.();
  }, [loadStatus, onStatusChange]);

  // 切换服务选择
  const handleToggleService = useCallback((serviceId: string) => {
    setSelectedServiceIds(prev => {
      const next = new Set(prev);
      if (next.has(serviceId)) {
        next.delete(serviceId);
      } else {
        next.add(serviceId);
      }
      return next;
    });
  }, []);

  // 保存服务关联
  const handleSaveLinks = useCallback(async () => {
    if (!status) return;

    setIsSaving(true);
    try {
      const currentLinked = new Set(status.associated_services.map(s => s.id));
      const toAdd = [...selectedServiceIds].filter(id => !currentLinked.has(id));
      const toRemove = [...currentLinked].filter(id => !selectedServiceIds.has(id));

      // 添加关联
      for (const serviceId of toAdd) {
        await invoke("link_mcp_service_to_project", {
          projectId,
          serviceId,
          configOverride: null,
        });
      }

      // 移除关联
      for (const serviceId of toRemove) {
        await invoke("unlink_mcp_service_from_project", {
          projectId,
          serviceId,
        });
      }

      feedback.success(t("hub.mcpContext.linkSaveSuccess", "Service links updated"));
      setIsLinkOpen(false);
      loadStatus();
      onStatusChange?.();
    } catch (error) {
      console.error("[McpContextSection] Failed to save links:", error);
      feedback.error(t("hub.mcpContext.linkSaveError", "Failed to update links"), (error as Error).message);
    } finally {
      setIsSaving(false);
    }
  }, [status, selectedServiceIds, projectId, t, loadStatus, onStatusChange]);

  // 检查是否有变更
  const hasLinkChanges = status ? (() => {
    const currentLinked = new Set(status.associated_services.map(s => s.id));
    if (currentLinked.size !== selectedServiceIds.size) return true;
    for (const id of selectedServiceIds) {
      if (!currentLinked.has(id)) return true;
    }
    return false;
  })() : false;

  // 加载中
  if (isLoading) {
    return (
      <div className="space-y-3" data-testid="mcp-context-section-loading">
        <div className="flex items-center gap-2">
          <Plug className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm font-medium">{t("hub.mcpContext.title", "MCP Context")}</span>
        </div>
        <div className="flex items-center justify-center py-4">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      </div>
    );
  }

  const totalServices = status?.associated_services.length ?? 0;
  const runningServices = status?.associated_services.filter(s => s.is_running).length ?? 0;
  const totalDetectable = status?.detectable_configs.reduce((sum, c) => sum + c.service_count, 0) ?? 0;
  const detectableAdapters = [...new Set(status?.detectable_configs.map(c => c.adapter_id) ?? [])];

  return (
    <>
      <div className="space-y-3" data-testid="mcp-context-section">
        {/* 标题行 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Plug className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium">{t("hub.mcpContext.title", "MCP Context")}</span>
          </div>
          {totalServices > 0 && (
            <Badge variant="secondary" className="text-xs">
              {runningServices > 0
                ? t("hub.mcpContext.servicesActive", "{{count}} Active", { count: runningServices })
                : t("hub.mcpContext.servicesCount", "{{count}} Services", { count: totalServices })}
            </Badge>
          )}
        </div>

        {/* 已关联服务列表 */}
        {totalServices > 0 && (
          <div className="space-y-2">
            {status!.associated_services.map((service) => (
              <div
                key={service.id}
                className="flex items-center gap-2 p-2 rounded-md bg-muted/50"
                data-testid={`mcp-service-${service.id}`}
              >
                <McpServiceStatusDot
                  status={getServiceStatus(service)}
                  errorMessage={service.error_message}
                />
                <span className="text-sm font-medium flex-1 truncate min-w-0">
                  {service.name}
                </span>
                {/* 来源和策略信息 - 响应式布局 */}
                <div className="flex items-center gap-1.5 shrink-0">
                  <SourceIcon source={service.adapter_id} className="h-3.5 w-3.5" />
                  <PolicyBadge service={service} t={t} />
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-6 w-6"
                        onClick={() => setPolicyDialogService(service)}
                        data-testid={`mcp-manage-tools-${service.id}`}
                      >
                        <Shield className="h-3 w-3" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      {t("hub.toolPolicy.projectEntry", "Project Tool Policy")}
                    </TooltipContent>
                  </Tooltip>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* 可接管状态提示 */}
        {totalServices === 0 && totalDetectable > 0 && (
          <div className="space-y-2">
            <p className="text-sm text-muted-foreground">
              {t("hub.mcpContext.noServicesLinked", "No services linked")}
            </p>
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <span>{t("hub.mcpContext.detected", "Detected:")}</span>
              <span className="font-medium text-foreground">{totalDetectable}</span>
              <span>{t("hub.mcpContext.configs", "configs")}</span>
              {detectableAdapters.map((adapterId) => (
                <SourceIcon key={adapterId} source={adapterId} className="h-3.5 w-3.5" />
              ))}
            </div>
            <Button
              size="sm"
              className="w-full"
              onClick={() => setImportDialogOpen(true)}
              data-testid="mcp-import-takeover-button"
            >
              {t("hub.mcpContext.importTakeover", "Import & Takeover")}
            </Button>
          </div>
        )}

        {/* Story 11.18: [+ 关联更多服务] 展开区域 */}
        <Collapsible open={isLinkOpen} onOpenChange={setIsLinkOpen}>
          <CollapsibleTrigger asChild>
            <Button
              variant="outline"
              size="sm"
              className="w-full justify-between"
              data-testid="mcp-link-services-trigger"
            >
              <span className="flex items-center gap-2">
                <Plus className="h-4 w-4" />
                {t("hub.mcpContext.linkMoreServices", "Link More Services")}
              </span>
              <ChevronDown className={`h-4 w-4 transition-transform ${isLinkOpen ? 'rotate-180' : ''}`} />
            </Button>
          </CollapsibleTrigger>
          <CollapsibleContent className="pt-3">
            {isLoadingServices ? (
              <div className="flex items-center justify-center py-4">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
              </div>
            ) : allServices.length === 0 ? (
              <div className="text-center py-4 text-sm text-muted-foreground">
                {t("hub.mcpContext.noServicesAvailable", "No services available. Import MCP configurations first.")}
              </div>
            ) : (
              <div className="space-y-3">
                <ScrollArea className="max-h-[200px] border rounded-md p-2">
                  <div className="space-y-1">
                    {allServices.map((service) => {
                      const isSelected = selectedServiceIds.has(service.id);
                      const adapterId = inferAdapterId(service.source_file);
                      return (
                        <div
                          key={service.id}
                          className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/50 cursor-pointer"
                          onClick={() => handleToggleService(service.id)}
                          data-testid={`service-link-item-${service.id}`}
                        >
                          <Checkbox
                            checked={isSelected}
                            onCheckedChange={() => handleToggleService(service.id)}
                          />
                          <span className="text-sm flex-1 truncate">{service.name}</span>
                          <SourceIcon source={adapterId} className="h-4 w-4" />
                        </div>
                      );
                    })}
                  </div>
                </ScrollArea>
                <Button
                  size="sm"
                  className="w-full"
                  onClick={handleSaveLinks}
                  disabled={isSaving || !hasLinkChanges}
                  data-testid="mcp-save-links-button"
                >
                  {isSaving ? (
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  ) : (
                    <Check className="h-4 w-4 mr-2" />
                  )}
                  {t("hub.mcpContext.saveLinks", "Save")}
                </Button>
              </div>
            )}
          </CollapsibleContent>
        </Collapsible>

        {/* 空状态 */}
        {totalServices === 0 && totalDetectable === 0 && !isLinkOpen && (
          <p className="text-sm text-muted-foreground text-center py-2">
            {t("hub.mcpContext.noConfigsFound", "No MCP services configured")}
          </p>
        )}
      </div>

      {/* 工具策略编辑 Sheet */}
      <ActionSheet
        open={!!policyDialogService}
        onOpenChange={(open) => {
          if (!open) setPolicyDialogService(null);
        }}
      >
        <ActionSheetContent size="lg" className="overflow-hidden" data-testid="tool-policy-sheet">
          <ActionSheetHeader>
            <ActionSheetTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              {t("hub.mcpContext.toolPermissions", "Tool Permissions")}
              {policyDialogService && (
                <Badge variant="outline" className="ml-2">
                  {policyDialogService.name}
                </Badge>
              )}
            </ActionSheetTitle>
          </ActionSheetHeader>
          {policyDialogService && (
            <ToolPolicyEditor
              projectId={projectId}
              projectName={projectName}
              serviceId={policyDialogService.id}
              serviceName={policyDialogService.name}
              embedded
              onSaved={() => {
                setPolicyDialogService(null);
                loadStatus();
                onStatusChange?.();
              }}
            />
          )}
        </ActionSheetContent>
      </ActionSheet>

      {/* 导入 Sheet */}
      <McpConfigImportSheet
        open={importDialogOpen}
        onOpenChange={setImportDialogOpen}
        onSuccess={handleImportSuccess}
        projectPath={projectPath}
        projectId={projectId}
        projectName={projectName}
      />
    </>
  );
}

export default McpContextSection;
