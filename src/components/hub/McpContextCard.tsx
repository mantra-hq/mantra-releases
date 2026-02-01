/**
 * MCP Context Card 组件
 * Story 11.9: Task 2 - 项目详情页 MCP 状态卡片 (AC: #1, #2, #3, #4, #5)
 *
 * 显示项目的 MCP 服务状态：
 * - 已关联的服务列表及运行状态
 * - 可检测到的配置文件 (接管入口)
 * - 空状态引导
 * - 管理入口
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Plug,
  Settings2,
  Download,
  Plus,
  Loader2,
} from "lucide-react";
import { McpServiceStatusDot, type ServiceStatus } from "./McpServiceStatusDot";
import { SourceIcon } from "@/components/import/SourceIcons";
import { McpConfigImportDialog } from "./McpConfigImportDialog";

// ===== 类型定义 =====

/** MCP 服务摘要 */
interface McpServiceSummary {
  id: string;
  name: string;
  adapter_id: string;
  is_running: boolean;
  error_message: string | null;
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

// ===== 组件属性 =====

export interface McpContextCardProps {
  /** 项目 ID */
  projectId: string;
  /** 项目路径 (用于扫描配置文件) */
  projectPath?: string;
  /** 状态变更回调 */
  onStatusChange?: () => void;
  /** 导航到 Hub 页面回调 (如果不提供，则使用 window.location) */
  onNavigateToHub?: (projectId: string) => void;
}

/**
 * 获取适配器显示名称
 */
function getAdapterLabel(adapterId: string): string {
  switch (adapterId) {
    case "claude":
      return "Claude";
    case "cursor":
      return "Cursor";
    case "codex":
      return "Codex";
    case "gemini":
      return "Gemini";
    default:
      return adapterId;
  }
}

/**
 * 将服务状态转换为 ServiceStatus 类型
 */
function getServiceStatus(service: McpServiceSummary): ServiceStatus {
  if (service.error_message) return "error";
  if (service.is_running) return "running";
  return "stopped";
}

export function McpContextCard({
  projectId,
  projectPath,
  onStatusChange,
  onNavigateToHub,
}: McpContextCardProps) {
  const { t } = useTranslation();

  const [status, setStatus] = useState<ProjectMcpStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [importDialogOpen, setImportDialogOpen] = useState(false);

  // 加载 MCP 状态
  const loadStatus = useCallback(async () => {
    if (!projectId) {
      setIsLoading(false);
      return;
    }

    try {
      const result = await invoke<ProjectMcpStatus>("check_project_mcp_status", {
        projectId,
        projectPath: projectPath || null,
      });
      setStatus(result);
    } catch (error) {
      console.error("[McpContextCard] Failed to load status:", error);
      setStatus(null);
    } finally {
      setIsLoading(false);
    }
  }, [projectId, projectPath]);

  // 初始加载
  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  // 导入成功后刷新
  const handleImportSuccess = useCallback(() => {
    loadStatus();
    onStatusChange?.();
  }, [loadStatus, onStatusChange]);

  // 跳转到 Hub 管理页面
  const handleManageServices = useCallback(() => {
    if (onNavigateToHub) {
      onNavigateToHub(projectId);
    } else {
      // 使用 window.location 作为回退
      window.location.href = `/hub?project=${projectId}`;
    }
  }, [onNavigateToHub, projectId]);

  // 加载中状态
  if (isLoading) {
    return (
      <Card className="w-full" data-testid="mcp-context-card-loading">
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Plug className="h-4 w-4" />
            {t("hub.mcpContext.title", "MCP Context")}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-4">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        </CardContent>
      </Card>
    );
  }

  // 计算统计数据
  const totalServices = status?.associated_services.length ?? 0;
  const runningServices = status?.associated_services.filter(s => s.is_running).length ?? 0;
  const totalDetectable = status?.detectable_configs.reduce((sum, c) => sum + c.service_count, 0) ?? 0;
  const detectableAdapters = [...new Set(status?.detectable_configs.map(c => c.adapter_id) ?? [])];

  // 已接管状态：显示关联的服务
  if (status?.is_taken_over) {
    return (
      <Card className="w-full" data-testid="mcp-context-card">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Plug className="h-4 w-4" />
              {t("hub.mcpContext.title", "MCP Context")}
            </CardTitle>
            <Badge variant="secondary" className="text-xs">
              {runningServices > 0
                ? t("hub.mcpContext.servicesActive", "{{count}} Services Active", { count: runningServices })
                : t("hub.mcpContext.servicesCount", "{{count}} Services", { count: totalServices })}
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          {/* 服务列表 */}
          <div className="space-y-2">
            {status.associated_services.map((service) => (
              <div
                key={service.id}
                className="flex items-center gap-3 p-2 rounded-md bg-muted/50"
                data-testid={`mcp-service-${service.id}`}
              >
                <McpServiceStatusDot
                  status={getServiceStatus(service)}
                  errorMessage={service.error_message}
                />
                <span className="text-sm font-medium flex-1 truncate">
                  {service.name}
                </span>
                <div className="flex items-center gap-1.5 shrink-0">
                  <SourceIcon source={service.adapter_id} className="h-4 w-4" />
                  <span className="text-xs text-muted-foreground">
                    {getAdapterLabel(service.adapter_id)}
                  </span>
                </div>
              </div>
            ))}
          </div>

          {/* 管理按钮 */}
          <Button
            variant="outline"
            className="w-full"
            onClick={handleManageServices}
            data-testid="mcp-manage-services-button"
          >
            <Settings2 className="h-4 w-4 mr-2" />
            {t("hub.mcpContext.manageServices", "Manage Services")}
          </Button>
        </CardContent>
      </Card>
    );
  }

  // 可接管状态：检测到配置但未接管
  if (totalDetectable > 0) {
    return (
      <>
        <Card className="w-full" data-testid="mcp-context-card-takeover">
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Plug className="h-4 w-4" />
              {t("hub.mcpContext.title", "MCP Context")}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <p className="text-sm text-muted-foreground">
              {t("hub.mcpContext.noServicesLinked", "No services linked to project")}
            </p>

            {/* 检测到的配置信息 */}
            <div className="flex items-center gap-2 text-sm">
              <span className="text-muted-foreground">
                {t("hub.mcpContext.detected", "Detected:")}{" "}
                <span className="font-medium text-foreground">{totalDetectable}</span>{" "}
                {t("hub.mcpContext.configs", "configs")}
              </span>
            </div>

            {/* 来源工具图标 */}
            <div className="flex items-center gap-2">
              {detectableAdapters.map((adapterId) => (
                <div
                  key={adapterId}
                  className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-muted/50"
                  title={getAdapterLabel(adapterId)}
                >
                  <SourceIcon source={adapterId} className="h-4 w-4" />
                  <span className="text-xs">{getAdapterLabel(adapterId)}</span>
                </div>
              ))}
            </div>

            {/* 接管按钮 */}
            <Button
              className="w-full"
              onClick={() => setImportDialogOpen(true)}
              data-testid="mcp-import-takeover-button"
            >
              <Download className="h-4 w-4 mr-2" />
              {t("hub.mcpContext.importTakeover", "Import & Takeover")}
            </Button>
          </CardContent>
        </Card>

        {/* 导入对话框 */}
        <McpConfigImportDialog
          open={importDialogOpen}
          onOpenChange={setImportDialogOpen}
          onSuccess={handleImportSuccess}
          projectPath={projectPath}
          projectId={projectId}
        />
      </>
    );
  }

  // 空状态：无任何 MCP 配置
  return (
    <Card className="w-full" data-testid="mcp-context-card-empty">
      <CardHeader className="pb-3">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Plug className="h-4 w-4" />
          {t("hub.mcpContext.title", "MCP Context")}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="text-center py-2">
          <p className="text-sm text-muted-foreground mb-1">
            {t("hub.mcpContext.noConfigsFound", "No MCP services configured")}
          </p>
          <p className="text-xs text-muted-foreground">
            {t("hub.mcpContext.addServicesHint", "Add MCP tools to enhance your AI coding experience.")}
          </p>
        </div>

        {/* 添加服务按钮 */}
        <Button
          variant="outline"
          className="w-full"
          onClick={handleManageServices}
          data-testid="mcp-add-services-button"
        >
          <Plus className="h-4 w-4 mr-2" />
          {t("hub.mcpContext.addServices", "Add Services")}
        </Button>
      </CardContent>
    </Card>
  );
}

export default McpContextCard;
