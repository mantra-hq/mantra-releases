/**
 * TakeoverToolManager - 工具接管管理器
 * Story 11.20: 全工具自动接管生成 - Task 7
 *
 * 功能：
 * - 显示每个工具的接管状态（已接管/未接管）
 * - 按工具分组显示已接管的配置
 * - 单独取消某个工具的接管
 * - 单独添加某个工具的接管
 */

import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
} from "@/components/ui/collapsible";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import {
  Loader2,
  ChevronDown,
  ChevronRight,
  CheckCircle2,
  CircleSlash,
  RotateCcw,
  Plus,
  Settings2,
  User,
  FolderGit2,
} from "lucide-react";
import { SourceIcon } from "@/components/import/SourceIcons";
import { feedback } from "@/lib/feedback";
import {
  detectInstalledTools,
  previewFullToolTakeover,
  executeFullToolTakeover,
  convertToTakeoverPreview,
  type ToolType,
  type ToolDetectionResult,
  type FullToolTakeoverPreview,
  type ToolTakeoverPreview,
} from "@/lib/smart-takeover-ipc";

// ===== 类型定义 =====

/**
 * 接管备份记录
 */
interface TakeoverBackup {
  id: string;
  toolType: ToolType;
  scope: "user" | "project";
  projectPath: string | null;
  originalPath: string;
  backupPath: string;
  takenOverAt: string;
  restoredAt: string | null;
  status: "active" | "restored";
}

export interface TakeoverToolManagerProps {
  /** 项目 ID */
  projectId: string;
  /** 项目路径 */
  projectPath: string;
  /** 接管变更回调 */
  onTakeoverChange?: () => void;
}

// ===== 辅助函数 =====

function getToolLabel(toolType: ToolType): string {
  switch (toolType) {
    case "claude_code":
      return "Claude Code";
    case "cursor":
      return "Cursor";
    case "codex":
      return "Codex";
    case "gemini_cli":
      return "Gemini CLI";
    default:
      return toolType;
  }
}

function toolTypeToAdapterId(toolType: ToolType): string {
  switch (toolType) {
    case "claude_code":
      return "claude";
    case "gemini_cli":
      return "gemini";
    default:
      return toolType;
  }
}

function adapterIdToToolType(adapterId: string): ToolType {
  switch (adapterId) {
    case "claude":
      return "claude_code";
    case "gemini":
      return "gemini_cli";
    default:
      return adapterId as ToolType;
  }
}

// ===== 组件 =====

export function TakeoverToolManager({
  projectId,
  projectPath,
  onTakeoverChange,
}: TakeoverToolManagerProps) {
  const { t } = useTranslation();

  // 状态
  const [isLoading, setIsLoading] = useState(true);
  const [installedTools, setInstalledTools] = useState<ToolDetectionResult[]>([]);
  const [backups, setBackups] = useState<TakeoverBackup[]>([]);
  const [fullPreview, setFullPreview] = useState<FullToolTakeoverPreview | null>(null);
  const [expandedTools, setExpandedTools] = useState<Set<string>>(new Set());
  const [actionInProgress, setActionInProgress] = useState<string | null>(null);

  // 按工具分组的接管状态
  const toolTakeoverStatus = useMemo(() => {
    const status = new Map<ToolType, {
      installed: boolean;
      takenOver: boolean;
      userScopeBackup: TakeoverBackup | null;
      projectScopeBackup: TakeoverBackup | null;
      preview: ToolTakeoverPreview | null;
    }>();

    // 初始化所有支持的工具
    const allToolTypes: ToolType[] = ["claude_code", "cursor", "codex", "gemini_cli"];
    for (const toolType of allToolTypes) {
      const installed = installedTools.some(
        (t) => t.tool_type === toolType && t.installed
      );
      const userBackup = backups.find(
        (b) => b.toolType === toolType && b.scope === "user"
      ) || null;
      const projectBackup = backups.find(
        (b) => b.toolType === toolType && b.scope === "project"
      ) || null;
      const preview = fullPreview?.tools.find(
        (t) => adapterIdToToolType(t.adapter_id) === toolType
      ) || null;

      status.set(toolType, {
        installed,
        takenOver: userBackup !== null || projectBackup !== null,
        userScopeBackup: userBackup,
        projectScopeBackup: projectBackup,
        preview,
      });
    }

    return status;
  }, [installedTools, backups, fullPreview]);

  // 加载数据
  const loadData = useCallback(async () => {
    setIsLoading(true);
    try {
      const [toolsResult, backupsResult, previewResult] = await Promise.all([
        detectInstalledTools(),
        invoke<TakeoverBackup[]>("list_active_takeovers"),
        previewFullToolTakeover(projectPath).catch(() => null),
      ]);

      setInstalledTools(toolsResult.tools);
      setBackups(backupsResult);
      setFullPreview(previewResult);
    } catch (error) {
      console.error("[TakeoverToolManager] Failed to load data:", error);
    } finally {
      setIsLoading(false);
    }
  }, [projectPath]);

  // 初始加载
  useEffect(() => {
    loadData();
  }, [loadData]);

  // 切换工具展开状态
  const toggleToolExpanded = useCallback((toolType: ToolType) => {
    setExpandedTools((prev) => {
      const next = new Set(prev);
      if (next.has(toolType)) {
        next.delete(toolType);
      } else {
        next.add(toolType);
      }
      return next;
    });
  }, []);

  // 取消单个工具的接管（恢复所有该工具的接管）
  // 注意：恢复接管只是还原外部工具的配置文件，不需要重启 Gateway
  // Gateway 本身的 MCP 服务配置没有变化
  const handleCancelTakeover = useCallback(async (toolType: ToolType) => {
    setActionInProgress(toolType);

    try {
      const toolBackups = backups.filter((b) => b.toolType === toolType);
      for (const backup of toolBackups) {
        await invoke("restore_takeover", { backupId: backup.id });
      }

      feedback.success(
        t("hub.toolManager.cancelSuccess", { tool: getToolLabel(toolType) })
      );
      await loadData();
      onTakeoverChange?.();
    } catch (error) {
      console.error("[TakeoverToolManager] Failed to cancel takeover:", error);
      feedback.error(
        t("hub.toolManager.cancelError"),
        (error as Error).message
      );
    } finally {
      setActionInProgress(null);
    }
  }, [backups, t, loadData, onTakeoverChange]);

  // 添加单个工具的接管
  const handleAddTakeover = useCallback(async (toolType: ToolType) => {
    if (!fullPreview) return;

    setActionInProgress(toolType);

    try {
      const adapterId = toolTypeToAdapterId(toolType);
      const preview = convertToTakeoverPreview(fullPreview, [adapterId]);

      if (preview.auto_create.length === 0 && preview.needs_decision.length === 0) {
        feedback.success(t("hub.toolManager.noServicesToTakeover"));
        return;
      }

      const result = await executeFullToolTakeover(projectId, preview, []);

      if (result.success) {
        feedback.success(
          t("hub.toolManager.addSuccess", {
            tool: getToolLabel(toolType),
            count: result.stats.created_count,
          })
        );
      } else {
        feedback.error(
          t("hub.toolManager.addPartialError"),
          result.errors.join(", ")
        );
      }

      await loadData();
      onTakeoverChange?.();
    } catch (error) {
      console.error("[TakeoverToolManager] Failed to add takeover:", error);
      feedback.error(
        t("hub.toolManager.addError"),
        (error as Error).message
      );
    } finally {
      setActionInProgress(null);
    }
  }, [fullPreview, projectId, t, loadData, onTakeoverChange]);

  // 渲染工具行
  const renderToolRow = (toolType: ToolType) => {
    const status = toolTakeoverStatus.get(toolType);
    if (!status) return null;

    const isExpanded = expandedTools.has(toolType);
    const hasBackups = status.userScopeBackup || status.projectScopeBackup;
    const canTakeover = status.installed && !status.takenOver && status.preview && status.preview.total_service_count > 0;
    const isProcessing = actionInProgress === toolType;

    return (
      <div key={toolType} className="border rounded-lg">
        {/* 工具行 */}
        <div className="flex items-center gap-3 p-3">
          {/* 工具图标和名称 */}
          <button
            type="button"
            className="flex items-center gap-2 flex-1 text-left"
            onClick={() => toggleToolExpanded(toolType)}
            disabled={!hasBackups}
          >
            <SourceIcon
              source={toolTypeToAdapterId(toolType)}
              className="h-5 w-5 shrink-0"
            />
            <span className="font-medium">{getToolLabel(toolType)}</span>
          </button>

          {/* 状态徽章 */}
          <div className="flex items-center gap-2">
            {!status.installed ? (
              <Badge variant="outline" className="text-xs text-muted-foreground">
                <CircleSlash className="h-3 w-3 mr-1" />
                {t("hub.toolManager.notInstalled")}
              </Badge>
            ) : status.takenOver ? (
              <Badge variant="secondary" className="text-xs text-green-500 bg-green-500/10">
                <CheckCircle2 className="h-3 w-3 mr-1" />
                {t("hub.toolManager.takenOver")}
              </Badge>
            ) : (
              <Badge variant="outline" className="text-xs text-muted-foreground">
                {t("hub.toolManager.notTakenOver")}
              </Badge>
            )}
          </div>

          {/* 操作按钮 */}
          <div className="flex items-center gap-2">
            {status.takenOver ? (
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={isProcessing}
                  >
                    {isProcessing ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <>
                        <RotateCcw className="h-4 w-4 mr-1" />
                        {t("hub.toolManager.cancel")}
                      </>
                    )}
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>
                      {t("hub.toolManager.cancelConfirmTitle")}
                    </AlertDialogTitle>
                    <AlertDialogDescription>
                      {t("hub.toolManager.cancelConfirmDescription", {
                        tool: getToolLabel(toolType),
                      })}
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
                    <AlertDialogAction onClick={() => handleCancelTakeover(toolType)}>
                      {t("hub.toolManager.cancelConfirm")}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            ) : canTakeover ? (
              <Button
                variant="default"
                size="sm"
                onClick={() => handleAddTakeover(toolType)}
                disabled={isProcessing}
              >
                {isProcessing ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <>
                    <Plus className="h-4 w-4 mr-1" />
                    {t("hub.toolManager.add")}
                  </>
                )}
              </Button>
            ) : null}

            {/* 展开按钮 */}
            {hasBackups && (
              <button
                type="button"
                onClick={() => toggleToolExpanded(toolType)}
                className="p-1 hover:bg-muted rounded"
              >
                {isExpanded ? (
                  <ChevronDown className="h-4 w-4 text-muted-foreground" />
                ) : (
                  <ChevronRight className="h-4 w-4 text-muted-foreground" />
                )}
              </button>
            )}
          </div>
        </div>

        {/* 展开详情 */}
        {isExpanded && hasBackups && (
          <Collapsible open={isExpanded}>
            <CollapsibleContent className="px-3 pb-3 space-y-2">
              {/* User Scope */}
              {status.userScopeBackup && (
                <div className="flex items-center gap-2 p-2 rounded-md bg-muted/30 text-sm">
                  <User className="h-4 w-4 text-blue-500" />
                  <span className="font-medium">
                    {t("hub.toolManager.userScope")}
                  </span>
                  <code className="text-xs text-muted-foreground ml-auto truncate max-w-xs">
                    {status.userScopeBackup.originalPath}
                  </code>
                </div>
              )}

              {/* Project Scope */}
              {status.projectScopeBackup && (
                <div className="flex items-center gap-2 p-2 rounded-md bg-muted/30 text-sm">
                  <FolderGit2 className="h-4 w-4 text-amber-500" />
                  <span className="font-medium">
                    {t("hub.toolManager.projectScope")}
                  </span>
                  <code className="text-xs text-muted-foreground ml-auto truncate max-w-xs">
                    {status.projectScopeBackup.originalPath}
                  </code>
                </div>
              )}
            </CollapsibleContent>
          </Collapsible>
        )}
      </div>
    );
  };

  return (
    <Card data-testid="takeover-tool-manager">
      <CardHeader className="pb-3">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-md bg-primary/10">
            <Settings2 className="h-5 w-5 text-primary" />
          </div>
          <div>
            <CardTitle className="text-base">
              {t("hub.toolManager.title")}
            </CardTitle>
            <CardDescription>
              {t("hub.toolManager.description")}
            </CardDescription>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="space-y-2">
            {(["claude_code", "cursor", "codex", "gemini_cli"] as ToolType[]).map(
              renderToolRow
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default TakeoverToolManager;
