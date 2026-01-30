/**
 * MCP 服务列表组件
 * Story 11.6: Task 3 - MCP 服务列表 (AC: #1, #3)
 *
 * 显示 MCP 服务列表：
 * - 表格展示（名称、命令、来源、状态、操作）
 * - 启用/禁用 Switch 组件
 * - 服务状态指示器
 * - 批量操作工具栏
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Plus,
  MoreVertical,
  Pencil,
  Trash2,
  RefreshCw,
  Server,
  Loader2,
  FileCode,
  Hand,
  Link2,
} from "lucide-react";
import { feedback } from "@/lib/feedback";
import { McpServiceForm } from "./McpServiceForm";
import { McpServiceDeleteDialog } from "./McpServiceDeleteDialog";
import { ProjectServiceAssociation } from "./ProjectServiceAssociation";

/**
 * MCP 服务类型
 */
export interface McpService {
  id: string;
  name: string;
  command: string;
  args: string[] | null;
  env: Record<string, string> | null;
  source: "imported" | "manual";
  source_file: string | null;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export function McpServiceList() {
  const { t } = useTranslation();
  const [services, setServices] = useState<McpService[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [togglingIds, setTogglingIds] = useState<Set<string>>(new Set());

  // 表单状态
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editService, setEditService] = useState<McpService | null>(null);

  // 删除确认状态
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [deleteService, setDeleteService] = useState<McpService | null>(null);

  // 项目关联状态
  const [isAssociationOpen, setIsAssociationOpen] = useState(false);
  const [associationService, setAssociationService] = useState<McpService | null>(null);

  // 加载服务列表
  const loadServices = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await invoke<McpService[]>("list_mcp_services");
      setServices(result);
    } catch (error) {
      console.error("[McpServiceList] Failed to load services:", error);
      feedback.error(t("hub.services.loadError"), (error as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [t]);

  // 切换服务启用状态
  const handleToggle = useCallback(async (service: McpService, enabled: boolean) => {
    setTogglingIds((prev) => new Set(prev).add(service.id));
    try {
      const updated = await invoke<McpService>("toggle_mcp_service", {
        id: service.id,
        enabled,
      });
      setServices((prev) =>
        prev.map((s) => (s.id === service.id ? updated : s))
      );
      feedback.success(
        enabled
          ? t("hub.services.enableSuccess", { name: service.name })
          : t("hub.services.disableSuccess", { name: service.name })
      );
    } catch (error) {
      console.error("[McpServiceList] Failed to toggle service:", error);
      feedback.error(t("hub.services.toggleError"), (error as Error).message);
    } finally {
      setTogglingIds((prev) => {
        const next = new Set(prev);
        next.delete(service.id);
        return next;
      });
    }
  }, [t]);

  // 打开添加表单
  const handleAdd = useCallback(() => {
    setEditService(null);
    setIsFormOpen(true);
  }, []);

  // 打开编辑表单
  const handleEdit = useCallback((service: McpService) => {
    setEditService(service);
    setIsFormOpen(true);
  }, []);

  // 打开删除确认
  const handleDelete = useCallback((service: McpService) => {
    setDeleteService(service);
    setIsDeleteDialogOpen(true);
  }, []);

  // 打开项目关联
  const handleLinkProjects = useCallback((service: McpService) => {
    setAssociationService(service);
    setIsAssociationOpen(true);
  }, []);

  // 操作成功后刷新
  const handleSuccess = useCallback(() => {
    loadServices();
  }, [loadServices]);

  // 初始加载
  useEffect(() => {
    loadServices();
  }, [loadServices]);

  // 来源图标
  const getSourceIcon = (source: string) => {
    return source === "imported" ? (
      <FileCode className="h-3.5 w-3.5" />
    ) : (
      <Hand className="h-3.5 w-3.5" />
    );
  };

  // 来源文本
  const getSourceText = (source: string) => {
    return source === "imported"
      ? t("hub.services.imported")
      : t("hub.services.manual");
  };

  return (
    <Card data-testid="mcp-service-list">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-md bg-blue-500/10">
              <Server className="h-5 w-5 text-blue-500" />
            </div>
            <div>
              <CardTitle className="text-base">{t("hub.services.title")}</CardTitle>
              <CardDescription>{t("hub.services.description")}</CardDescription>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={loadServices}
              disabled={isLoading}
              title={t("common.refresh")}
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
            </Button>
            <Button
              size="sm"
              onClick={handleAdd}
              data-testid="mcp-service-add-button"
            >
              <Plus className="h-4 w-4 mr-1" />
              {t("hub.services.add")}
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : services.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <Server className="h-12 w-12 mx-auto mb-3 opacity-20" />
            <p className="text-sm">{t("hub.services.empty")}</p>
            <p className="text-xs mt-1">{t("hub.services.emptyHint")}</p>
          </div>
        ) : (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-[180px]">{t("hub.services.name")}</TableHead>
                  <TableHead>{t("hub.services.command")}</TableHead>
                  <TableHead className="w-[100px]">{t("hub.services.source")}</TableHead>
                  <TableHead className="w-[80px] text-center">{t("hub.services.enabled")}</TableHead>
                  <TableHead className="w-[60px]">{t("hub.services.actions")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {services.map((service) => (
                  <TableRow key={service.id} data-testid={`mcp-service-row-${service.id}`}>
                    <TableCell className="font-medium">{service.name}</TableCell>
                    <TableCell>
                      <code className="text-xs bg-muted px-1.5 py-0.5 rounded">
                        {service.command}
                        {service.args && service.args.length > 0 && (
                          <span className="text-muted-foreground ml-1">
                            {service.args.slice(0, 2).join(" ")}
                            {service.args.length > 2 && " ..."}
                          </span>
                        )}
                      </code>
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant="outline"
                        className="gap-1 text-xs"
                      >
                        {getSourceIcon(service.source)}
                        {getSourceText(service.source)}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-center">
                      {togglingIds.has(service.id) ? (
                        <Loader2 className="h-4 w-4 animate-spin mx-auto" />
                      ) : (
                        <Switch
                          checked={service.enabled}
                          onCheckedChange={(checked) => handleToggle(service, checked)}
                          aria-label={t("hub.services.toggleAria", { name: service.name })}
                          data-testid={`mcp-service-toggle-${service.id}`}
                        />
                      )}
                    </TableCell>
                    <TableCell>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8"
                            data-testid={`mcp-service-menu-${service.id}`}
                          >
                            <MoreVertical className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => handleEdit(service)}>
                            <Pencil className="h-4 w-4 mr-2" />
                            {t("hub.services.edit")}
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => handleLinkProjects(service)}>
                            <Link2 className="h-4 w-4 mr-2" />
                            {t("hub.services.linkProjects")}
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            onClick={() => handleDelete(service)}
                            className="text-destructive focus:text-destructive"
                          >
                            <Trash2 className="h-4 w-4 mr-2" />
                            {t("hub.services.delete")}
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

            {/* 统计信息 */}
            <div className="text-xs text-muted-foreground text-center mt-4">
              {t("hub.services.count", { count: services.length })}
              {" • "}
              {t("hub.services.enabledCount", {
                count: services.filter((s) => s.enabled).length,
              })}
            </div>
          </>
        )}
      </CardContent>

      {/* 添加/编辑表单对话框 */}
      <McpServiceForm
        open={isFormOpen}
        onOpenChange={setIsFormOpen}
        editService={editService}
        onSuccess={handleSuccess}
      />

      {/* 删除确认对话框 */}
      <McpServiceDeleteDialog
        open={isDeleteDialogOpen}
        onOpenChange={setIsDeleteDialogOpen}
        service={deleteService}
        onSuccess={handleSuccess}
      />

      {/* 项目关联对话框 */}
      <ProjectServiceAssociation
        open={isAssociationOpen}
        onOpenChange={setIsAssociationOpen}
        service={associationService}
        onSuccess={handleSuccess}
      />
    </Card>
  );
}

export default McpServiceList;
