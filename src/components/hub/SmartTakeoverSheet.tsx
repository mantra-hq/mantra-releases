/**
 * SmartTakeoverSheet - 智能接管预览 Sheet
 * Story 11.19: MCP 智能接管合并引擎 - Task 5
 * Story 11.20: 全工具自动接管生成 - Task 6
 *
 * 功能：
 * - 全工具分组展示（按工具类型分组）
 * - 工具勾选/取消勾选功能
 * - 显示工具检测状态（已安装/未安装图标）
 * - 显示各 Scope 的服务数量徽章
 * - 三档分类展示 UI（可直接导入 / 已存在 / 需决策）
 * - 配置冲突 Diff 对比视图
 * - Scope 冲突选择 UI
 * - 执行按钮 + 进度反馈
 * - Gateway 未运行时显示启动提示
 */

import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetFooter,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Loader2,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Plus,
  SkipForward,
  HelpCircle,
  Play,
  RefreshCw,
  Rocket,
  ChevronDown,
  ChevronRight,
  Circle,
  CircleDot,
  CircleSlash,
  User,
  FolderGit2,
} from "lucide-react";
import { SourceIcon } from "@/components/import/SourceIcons";
import { ConfigDiffView } from "./ConfigDiffView";
import {
  previewFullToolTakeover,
  executeFullToolTakeover,
  fullPreviewIsEmpty,
  getFullPreviewStats,
  convertToTakeoverPreview,
  type FullToolTakeoverPreview,
  type ToolTakeoverPreview,
  type ScopeTakeoverPreview,
  type TakeoverDecision,
  type TakeoverDecisionOption,
  type ConflictDetail,
  type FullTakeoverResult,
} from "@/lib/smart-takeover-ipc";

// ===== 类型定义 =====

export interface SmartTakeoverSheetProps {
  /** 是否打开 */
  open: boolean;
  /** 关闭回调 */
  onOpenChange: (open: boolean) => void;
  /** 项目 ID */
  projectId: string;
  /** 项目路径 */
  projectPath: string;
  /** 项目名称 */
  projectName?: string;
  /** 接管成功回调 */
  onSuccess?: () => void;
}

type SheetStep = "loading" | "preview" | "executing" | "result";

// ===== 辅助函数 =====

function getAdapterLabel(adapterId: string): string {
  switch (adapterId) {
    case "claude":
      return "Claude Code";
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

function getScopeLabel(scope: string, t: (key: string, fallback: string) => string): string {
  switch (scope) {
    case "project":
      return t("hub.smartTakeover.scopeProject", "Project");
    case "user":
      return t("hub.smartTakeover.scopeUser", "User");
    default:
      return scope;
  }
}

// ===== 组件 =====

export function SmartTakeoverSheet({
  open,
  onOpenChange,
  projectId,
  projectPath,
  projectName,
  onSuccess,
}: SmartTakeoverSheetProps) {
  const { t } = useTranslation();

  // 状态
  const [step, setStep] = useState<SheetStep>("loading");
  const [fullPreview, setFullPreview] = useState<FullToolTakeoverPreview | null>(null);
  const [selectedTools, setSelectedTools] = useState<Set<string>>(new Set());
  const [decisions, setDecisions] = useState<Map<string, TakeoverDecision>>(new Map());
  const [result, setResult] = useState<FullTakeoverResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 折叠状态
  const [toolsExpandedState, setToolsExpandedState] = useState<Record<string, boolean>>({});

  // 加载全工具预览
  const loadPreview = useCallback(async () => {
    setStep("loading");
    setError(null);
    setDecisions(new Map());
    setResult(null);

    try {
      const previewResult = await previewFullToolTakeover(projectPath);
      setFullPreview(previewResult);

      // 默认选中所有已安装且有配置的工具
      const initialSelected = new Set<string>();
      const initialExpanded: Record<string, boolean> = {};
      for (const tool of previewResult.tools) {
        if (tool.installed && tool.total_service_count > 0) {
          initialSelected.add(tool.adapter_id);
          initialExpanded[tool.adapter_id] = true;
        }
      }
      setSelectedTools(initialSelected);
      setToolsExpandedState(initialExpanded);

      setStep("preview");
    } catch (err) {
      console.error("[SmartTakeoverSheet] Failed to load preview:", err);
      setError((err as Error).message || t("hub.smartTakeover.errorLoadPreview", "Failed to load preview"));
      setStep("preview");
    }
  }, [projectPath, t]);

  // 打开时加载预览
  useEffect(() => {
    if (open) {
      loadPreview();
    }
  }, [open, loadPreview]);

  // 切换工具选择
  const toggleToolSelection = useCallback((adapterId: string) => {
    setSelectedTools((prev) => {
      const next = new Set(prev);
      if (next.has(adapterId)) {
        next.delete(adapterId);
      } else {
        next.add(adapterId);
      }
      return next;
    });
  }, []);

  // 切换工具展开状态
  const toggleToolExpanded = useCallback((adapterId: string) => {
    setToolsExpandedState((prev) => ({
      ...prev,
      [adapterId]: !prev[adapterId],
    }));
  }, []);

  // 设置决策
  const setDecision = useCallback((serviceName: string, decision: TakeoverDecisionOption, candidateIndex?: number) => {
    setDecisions((prev) => {
      const next = new Map(prev);
      next.set(serviceName, {
        service_name: serviceName,
        decision,
        selected_candidate_index: candidateIndex,
      });
      return next;
    });
  }, []);

  // 获取选中工具的冲突列表
  const selectedConflicts = useMemo(() => {
    if (!fullPreview) return [];
    const conflicts: ConflictDetail[] = [];
    for (const tool of fullPreview.tools) {
      if (!selectedTools.has(tool.adapter_id)) continue;
      if (tool.user_scope_preview) {
        conflicts.push(...tool.user_scope_preview.needs_decision);
      }
      if (tool.project_scope_preview) {
        conflicts.push(...tool.project_scope_preview.needs_decision);
      }
    }
    return conflicts;
  }, [fullPreview, selectedTools]);

  // 检查是否所有选中工具的冲突都已决策
  const allDecisionsMade = useMemo(() => {
    return selectedConflicts.every((conflict) => decisions.has(conflict.service_name));
  }, [selectedConflicts, decisions]);

  // 执行接管
  const handleExecute = useCallback(async () => {
    if (!fullPreview) return;

    setStep("executing");
    setError(null);

    try {
      // 转换为标准 TakeoverPreview
      const preview = convertToTakeoverPreview(fullPreview, Array.from(selectedTools));
      const decisionsList = Array.from(decisions.values());

      const executeResult = await executeFullToolTakeover(projectId, preview, decisionsList);
      setResult(executeResult);
      setStep("result");

      // 如果成功，调用回调
      if (executeResult.success && executeResult.errors.length === 0) {
        onSuccess?.();
      }
    } catch (err) {
      console.error("[SmartTakeoverSheet] Failed to execute takeover:", err);
      setError((err as Error).message || t("hub.smartTakeover.errorExecute", "Failed to execute takeover"));
      setStep("preview");
    }
  }, [fullPreview, selectedTools, decisions, projectId, onSuccess, t]);

  // 关闭 Sheet
  const handleClose = useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  // 预览统计
  const stats = fullPreview ? getFullPreviewStats(fullPreview) : null;

  // 选中工具的服务统计
  const selectedStats = useMemo(() => {
    if (!fullPreview) return { serviceCount: 0, conflictCount: 0 };
    let serviceCount = 0;
    let conflictCount = 0;
    for (const tool of fullPreview.tools) {
      if (!selectedTools.has(tool.adapter_id)) continue;
      serviceCount += tool.total_service_count;
      conflictCount += tool.conflict_count;
    }
    return { serviceCount, conflictCount };
  }, [fullPreview, selectedTools]);

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        side="right"
        className="w-[540px] sm:w-[640px] flex flex-col"
        data-testid="smart-takeover-sheet"
      >
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Rocket className="h-5 w-5" />
            {t("hub.smartTakeover.title", "Smart Takeover")}
          </SheetTitle>
          <SheetDescription>
            {projectName
              ? t("hub.smartTakeover.descriptionWithProject", "Import MCP services for {{project}}", { project: projectName })
              : t("hub.smartTakeover.description", "Import MCP services from detected configurations")}
          </SheetDescription>
        </SheetHeader>

        <ScrollArea className="flex-1 -mx-6 px-6">
          {/* Loading 状态 */}
          {step === "loading" && (
            <div className="flex flex-col items-center justify-center py-12 gap-4">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                {t("hub.smartTakeover.scanning", "Scanning configurations...")}
              </p>
            </div>
          )}

          {/* 错误状态 */}
          {error && step === "preview" && (
            <Alert variant="destructive" className="my-4">
              <XCircle className="h-4 w-4" />
              <AlertTitle>{t("hub.smartTakeover.errorTitle", "Error")}</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {/* 预览状态 */}
          {step === "preview" && fullPreview && !error && (
            <div className="space-y-6 py-4">
              {/* 统计摘要 */}
              {stats && (
                <div className="flex gap-4 text-sm flex-wrap">
                  <div className="flex items-center gap-1.5">
                    <CheckCircle2 className="h-4 w-4 text-green-500" />
                    <span>{t("hub.smartTakeover.statsInstalled", "{{count}} tools installed", { count: stats.installedCount })}</span>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <Plus className="h-4 w-4 text-blue-500" />
                    <span>{t("hub.smartTakeover.statsTotalServices", "{{count}} services", { count: stats.totalServiceCount })}</span>
                  </div>
                  {stats.conflictCount > 0 && (
                    <div className="flex items-center gap-1.5">
                      <HelpCircle className="h-4 w-4 text-amber-500" />
                      <span>{t("hub.smartTakeover.statsConflicts", "{{count}} conflicts", { count: stats.conflictCount })}</span>
                    </div>
                  )}
                </div>
              )}

              {/* 空状态 */}
              {fullPreviewIsEmpty(fullPreview) && (
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertTitle>{t("hub.smartTakeover.emptyTitle", "No configurations found")}</AlertTitle>
                  <AlertDescription>
                    {t("hub.smartTakeover.emptyDescription", "No MCP configurations were detected for any installed tools.")}
                  </AlertDescription>
                </Alert>
              )}

              {/* 工具分组展示 */}
              <div className="space-y-3">
                {fullPreview.tools.map((tool) => (
                  <ToolPreviewCard
                    key={tool.adapter_id}
                    tool={tool}
                    selected={selectedTools.has(tool.adapter_id)}
                    expanded={toolsExpandedState[tool.adapter_id] ?? false}
                    onToggleSelect={() => toggleToolSelection(tool.adapter_id)}
                    onToggleExpand={() => toggleToolExpanded(tool.adapter_id)}
                    decisions={decisions}
                    onDecision={setDecision}
                    t={t}
                  />
                ))}
              </div>

              {/* 选中统计 */}
              {selectedTools.size > 0 && (
                <div className="p-3 rounded-lg bg-muted/50 border">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">
                      {t("hub.smartTakeover.selectedSummary", "Selected: {{tools}} tools, {{services}} services", {
                        tools: selectedTools.size,
                        services: selectedStats.serviceCount,
                      })}
                    </span>
                    {selectedStats.conflictCount > 0 && (
                      <Badge variant="outline" className="text-amber-500 border-amber-500/50">
                        {t("hub.smartTakeover.pendingDecisions", "{{count}} decisions needed", { count: selectedStats.conflictCount })}
                      </Badge>
                    )}
                  </div>
                </div>
              )}

              {/* 环境变量提示 */}
              {fullPreview.env_vars_needed.length > 0 && (
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertTitle>{t("hub.smartTakeover.envVarsTitle", "Environment Variables Required")}</AlertTitle>
                  <AlertDescription>
                    {t("hub.smartTakeover.envVarsDescription", "The following environment variables are needed: {{vars}}", {
                      vars: fullPreview.env_vars_needed.join(", "),
                    })}
                  </AlertDescription>
                </Alert>
              )}
            </div>
          )}

          {/* 执行中状态 */}
          {step === "executing" && (
            <div className="flex flex-col items-center justify-center py-12 gap-4">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
              <p className="text-sm text-muted-foreground">
                {t("hub.smartTakeover.executing", "Executing takeover...")}
              </p>
            </div>
          )}

          {/* 结果状态 */}
          {step === "result" && result && (
            <div className="space-y-4 py-4">
              {/* 成功/失败摘要 */}
              {result.success && result.errors.length === 0 ? (
                <Alert className="border-green-500/50 bg-green-500/10">
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                  <AlertTitle className="text-green-500">
                    {t("hub.smartTakeover.successTitle", "Takeover Complete")}
                  </AlertTitle>
                  <AlertDescription>
                    {t("hub.smartTakeover.successDescriptionFull", "Successfully imported {{created}} services ({{skipped}} skipped, {{updated}} updated) from {{tools}} tools.", {
                      created: result.stats.created_count,
                      skipped: result.stats.skipped_count,
                      updated: result.stats.updated_count,
                      tools: result.stats.tool_count,
                    })}
                  </AlertDescription>
                </Alert>
              ) : result.rolled_back ? (
                <Alert variant="destructive">
                  <XCircle className="h-4 w-4" />
                  <AlertTitle>{t("hub.smartTakeover.rolledBackTitle", "Takeover Failed - Rolled Back")}</AlertTitle>
                  <AlertDescription>
                    <p>{t("hub.smartTakeover.rolledBackDescription", "The takeover failed and all changes have been rolled back.")}</p>
                    <ul className="list-disc list-inside mt-2 space-y-1">
                      {result.errors.map((err, idx) => (
                        <li key={idx} className="text-sm">{err}</li>
                      ))}
                    </ul>
                  </AlertDescription>
                </Alert>
              ) : (
                <Alert variant="destructive">
                  <XCircle className="h-4 w-4" />
                  <AlertTitle>{t("hub.smartTakeover.partialErrorTitle", "Completed with Errors")}</AlertTitle>
                  <AlertDescription>
                    <ul className="list-disc list-inside mt-2 space-y-1">
                      {result.errors.map((err, idx) => (
                        <li key={idx} className="text-sm">{err}</li>
                      ))}
                    </ul>
                  </AlertDescription>
                </Alert>
              )}

              {/* 警告信息 */}
              {result.warnings.length > 0 && (
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertTitle>{t("hub.smartTakeover.warningsTitle", "Warnings")}</AlertTitle>
                  <AlertDescription>
                    <ul className="list-disc list-inside mt-2 space-y-1">
                      {result.warnings.map((warn, idx) => (
                        <li key={idx} className="text-sm">{warn}</li>
                      ))}
                    </ul>
                  </AlertDescription>
                </Alert>
              )}

              {/* Gateway 状态提示 */}
              {!result.gateway_running && (
                <Alert>
                  <Play className="h-4 w-4" />
                  <AlertTitle>{t("hub.smartTakeover.gatewayNotRunning", "Gateway Not Running")}</AlertTitle>
                  <AlertDescription>
                    {t("hub.smartTakeover.gatewayNotRunningHint", "The MCP Gateway is not running. Start it to use the imported services.")}
                  </AlertDescription>
                </Alert>
              )}
            </div>
          )}
        </ScrollArea>

        <Separator className="my-4" />

        <SheetFooter className="gap-2">
          {step === "preview" && (
            <>
              <Button variant="ghost" onClick={loadPreview}>
                <RefreshCw className="h-4 w-4 mr-2" />
                {t("hub.smartTakeover.refresh", "Refresh")}
              </Button>
              <Button variant="outline" onClick={handleClose}>
                {t("common.cancel", "Cancel")}
              </Button>
              <Button
                onClick={handleExecute}
                disabled={
                  !fullPreview ||
                  fullPreviewIsEmpty(fullPreview) ||
                  selectedTools.size === 0 ||
                  (selectedStats.conflictCount > 0 && !allDecisionsMade)
                }
              >
                {t("hub.smartTakeover.execute", "Execute Takeover")}
              </Button>
            </>
          )}
          {step === "result" && (
            <Button onClick={handleClose}>
              {t("common.close", "Close")}
            </Button>
          )}
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
}

// ===== 工具预览卡片 =====

interface ToolPreviewCardProps {
  tool: ToolTakeoverPreview;
  selected: boolean;
  expanded: boolean;
  onToggleSelect: () => void;
  onToggleExpand: () => void;
  decisions: Map<string, TakeoverDecision>;
  onDecision: (serviceName: string, decision: TakeoverDecisionOption, candidateIndex?: number) => void;
  t: (key: string, fallback: string, opts?: Record<string, unknown>) => string;
}

function ToolPreviewCard({
  tool,
  selected,
  expanded,
  onToggleSelect,
  onToggleExpand,
  decisions,
  onDecision,
  t,
}: ToolPreviewCardProps) {
  const hasServices = tool.total_service_count > 0;
  const hasConflicts = tool.conflict_count > 0;

  return (
    <div
      className={`border rounded-lg transition-colors ${
        selected ? "border-primary/50 bg-primary/5" : "border-border"
      } ${!tool.installed ? "opacity-60" : ""}`}
    >
      {/* 工具头部 */}
      <div className="flex items-center gap-3 p-3">
        {/* 勾选框 */}
        <Checkbox
          checked={selected}
          onCheckedChange={onToggleSelect}
          disabled={!tool.installed || !hasServices}
          className="shrink-0"
        />

        {/* 工具图标和名称 */}
        <button
          type="button"
          className="flex items-center gap-2 flex-1 text-left"
          onClick={onToggleExpand}
          disabled={!hasServices}
        >
          <SourceIcon
            source={tool.adapter_id as "claude" | "cursor" | "codex" | "gemini"}
            className="h-5 w-5 shrink-0"
          />
          <span className="font-medium">{tool.display_name}</span>

          {/* 安装状态 */}
          {!tool.installed && (
            <Badge variant="outline" className="text-xs text-muted-foreground">
              <CircleSlash className="h-3 w-3 mr-1" />
              {t("hub.smartTakeover.notInstalled", "Not installed")}
            </Badge>
          )}
        </button>

        {/* 统计徽章 */}
        <div className="flex items-center gap-2">
          {hasServices && (
            <Badge variant="secondary" className="text-xs">
              {tool.total_service_count} {t("hub.smartTakeover.services", "services")}
            </Badge>
          )}
          {hasConflicts && (
            <Badge variant="outline" className="text-xs text-amber-500 border-amber-500/50">
              {tool.conflict_count}
            </Badge>
          )}
          {hasServices && (
            <button
              type="button"
              onClick={onToggleExpand}
              className="p-1 hover:bg-muted rounded"
            >
              {expanded ? (
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
              )}
            </button>
          )}
        </div>
      </div>

      {/* 展开内容 */}
      {expanded && hasServices && (
        <div className="px-3 pb-3 pt-0 space-y-3">
          <Separator />

          {/* User Scope */}
          {tool.user_scope_preview && tool.user_scope_preview.service_count > 0 && (
            <ScopePreviewSection
              scope="user"
              preview={tool.user_scope_preview}
              decisions={decisions}
              onDecision={onDecision}
              t={t}
            />
          )}

          {/* Project Scope */}
          {tool.project_scope_preview && tool.project_scope_preview.service_count > 0 && (
            <ScopePreviewSection
              scope="project"
              preview={tool.project_scope_preview}
              decisions={decisions}
              onDecision={onDecision}
              t={t}
            />
          )}
        </div>
      )}
    </div>
  );
}

// ===== Scope 预览区域 =====

interface ScopePreviewSectionProps {
  scope: "user" | "project";
  preview: ScopeTakeoverPreview;
  decisions: Map<string, TakeoverDecision>;
  onDecision: (serviceName: string, decision: TakeoverDecisionOption, candidateIndex?: number) => void;
  t: (key: string, fallback: string, opts?: Record<string, unknown>) => string;
}

function ScopePreviewSection({
  scope,
  preview,
  decisions,
  onDecision,
  t,
}: ScopePreviewSectionProps) {
  const [isOpen, setIsOpen] = useState(true);

  const ScopeIcon = scope === "user" ? User : FolderGit2;
  const scopeLabel = scope === "user"
    ? t("hub.smartTakeover.scopeUser", "User Scope")
    : t("hub.smartTakeover.scopeProject", "Project Scope");

  const hasAutoCreate = preview.auto_create.length > 0;
  const hasAutoSkip = preview.auto_skip.length > 0;
  const hasNeedsDecision = preview.needs_decision.length > 0;

  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen}>
      <CollapsibleTrigger className="flex items-center gap-2 w-full text-left p-2 rounded hover:bg-muted/50 transition-colors">
        {isOpen ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
        <ScopeIcon className="h-4 w-4 text-muted-foreground" />
        <span className="text-sm font-medium">{scopeLabel}</span>
        <Badge variant="outline" className="ml-auto text-xs">
          {preview.service_count}
        </Badge>
      </CollapsibleTrigger>

      <CollapsibleContent className="pl-6 space-y-2 pt-2">
        {/* 可直接导入 */}
        {hasAutoCreate && (
          <div className="space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Plus className="h-3 w-3 text-green-500" />
              <span>{t("hub.smartTakeover.newServices", "New")}</span>
            </div>
            {preview.auto_create.map((item) => (
              <div
                key={`${item.service_name}-${item.adapter_id}`}
                className="flex items-center justify-between p-2 rounded-md bg-muted/30 text-sm"
              >
                <span>{item.service_name}</span>
                <Badge variant="outline" className="text-xs text-green-500 border-green-500/50">
                  {t("hub.smartTakeover.willCreate", "Will create")}
                </Badge>
              </div>
            ))}
          </div>
        )}

        {/* 已存在（跳过） */}
        {hasAutoSkip && (
          <div className="space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <SkipForward className="h-3 w-3 text-blue-500" />
              <span>{t("hub.smartTakeover.existingServices", "Existing")}</span>
            </div>
            {preview.auto_skip.map((item) => (
              <div
                key={`${item.service_name}-${item.detected_adapter_id}`}
                className="flex items-center justify-between p-2 rounded-md bg-muted/30 text-sm"
              >
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="h-3 w-3 text-green-500" />
                  <span>{item.service_name}</span>
                </div>
                <Badge variant="outline" className="text-xs">
                  {t("hub.smartTakeover.willSkip", "Will skip")}
                </Badge>
              </div>
            ))}
          </div>
        )}

        {/* 需要决策 */}
        {hasNeedsDecision && (
          <div className="space-y-2">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <HelpCircle className="h-3 w-3 text-amber-500" />
              <span>{t("hub.smartTakeover.needsDecision", "Needs decision")}</span>
            </div>
            {preview.needs_decision.map((conflict) => (
              <ConflictDecisionPanel
                key={conflict.service_name}
                conflict={conflict}
                decision={decisions.get(conflict.service_name)}
                onDecision={onDecision}
                t={t}
                compact
              />
            ))}
          </div>
        )}
      </CollapsibleContent>
    </Collapsible>
  );
}

// ===== 冲突决策面板 =====

interface ConflictDecisionPanelProps {
  conflict: ConflictDetail;
  decision?: TakeoverDecision;
  onDecision: (serviceName: string, decision: TakeoverDecisionOption, candidateIndex?: number) => void;
  t: (key: string, fallback: string, opts?: Record<string, unknown>) => string;
  compact?: boolean;
}

function ConflictDecisionPanel({ conflict, decision, onDecision, t, compact }: ConflictDecisionPanelProps) {
  const getConflictTypeLabel = (type: string): string => {
    switch (type) {
      case "config_diff":
        return t("hub.smartTakeover.conflictConfigDiff", "Configuration Difference");
      case "scope_conflict":
        return t("hub.smartTakeover.conflictScopeConflict", "Scope Conflict");
      case "multi_source":
        return t("hub.smartTakeover.conflictMultiSource", "Multiple Sources");
      default:
        return type;
    }
  };

  // 根据冲突类型生成决策选项
  const getOptions = (): { value: TakeoverDecisionOption; label: string; description: string }[] => {
    if (conflict.conflict_type === "scope_conflict") {
      return [
        {
          value: "use_project_scope",
          label: t("hub.smartTakeover.decisionUseProjectScope", "Use Project Scope"),
          description: t("hub.smartTakeover.decisionUseProjectScopeDesc", "Use the project-level configuration"),
        },
        {
          value: "use_user_scope",
          label: t("hub.smartTakeover.decisionUseUserScope", "Use User Scope"),
          description: t("hub.smartTakeover.decisionUseUserScopeDesc", "Use the user-level configuration"),
        },
      ];
    }

    if (conflict.existing_service) {
      return [
        {
          value: "keep_existing",
          label: t("hub.smartTakeover.decisionKeepExisting", "Keep Existing"),
          description: t("hub.smartTakeover.decisionKeepExistingDesc", "Skip import and keep the existing service"),
        },
        {
          value: "use_new",
          label: t("hub.smartTakeover.decisionUseNew", "Use New"),
          description: t("hub.smartTakeover.decisionUseNewDesc", "Replace with the detected configuration"),
        },
        {
          value: "keep_both",
          label: t("hub.smartTakeover.decisionKeepBoth", "Keep Both"),
          description: t("hub.smartTakeover.decisionKeepBothDesc", "Import as a new service with a different name"),
        },
      ];
    }

    // multi_source without existing
    return [
      {
        value: "use_new",
        label: t("hub.smartTakeover.decisionImport", "Import"),
        description: t("hub.smartTakeover.decisionImportDesc", "Import the selected configuration"),
      },
    ];
  };

  const options = getOptions();

  // 自定义 Radio 选项组件
  const RadioOption = ({
    label,
    description,
    selected,
    onSelect
  }: {
    value: TakeoverDecisionOption;
    label: string;
    description: string;
    selected: boolean;
    onSelect: () => void;
  }) => (
    <button
      type="button"
      onClick={onSelect}
      className={`flex items-start gap-2 w-full text-left p-2 rounded hover:bg-muted/50 transition-colors ${compact ? "py-1.5" : ""}`}
    >
      {selected ? (
        <CircleDot className="h-4 w-4 mt-0.5 text-primary shrink-0" />
      ) : (
        <Circle className="h-4 w-4 mt-0.5 text-muted-foreground shrink-0" />
      )}
      <div className="grid gap-0.5 min-w-0">
        <span className={`font-medium ${compact ? "text-xs" : "text-sm"}`}>{label}</span>
        {!compact && <p className="text-xs text-muted-foreground">{description}</p>}
      </div>
    </button>
  );

  return (
    <div className={`border rounded-lg space-y-2 ${compact ? "p-2 border-amber-500/30" : "p-4"}`}>
      {/* 服务名称和冲突类型 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <AlertTriangle className={`text-amber-500 ${compact ? "h-3 w-3" : "h-4 w-4"}`} />
          <span className={`font-medium ${compact ? "text-sm" : ""}`}>{conflict.service_name}</span>
        </div>
        <Badge variant="outline" className={`text-amber-500 border-amber-500/50 ${compact ? "text-[10px]" : "text-xs"}`}>
          {getConflictTypeLabel(conflict.conflict_type)}
        </Badge>
      </div>

      {/* 配置差异对比 (如果有) */}
      {!compact && conflict.existing_service && conflict.candidates.length > 0 && conflict.conflict_type === "config_diff" && (
        <ConfigDiffView
          serviceName={conflict.service_name}
          existing={{
            command: conflict.existing_service.config_summary.command ?? "",
            args: conflict.existing_service.config_summary.args ?? null,
            env: null,
          }}
          candidates={conflict.candidates.map((c) => ({
            name: conflict.service_name,
            source_file: c.config_path,
            adapter_id: c.adapter_id,
            command: c.config_summary.command ?? "",
            args: c.config_summary.args ?? null,
            env: null,
          }))}
          getSourceText={getAdapterLabel}
        />
      )}

      {/* 候选项选择 (multi_source) */}
      {conflict.conflict_type === "multi_source" && conflict.candidates.length > 1 && (
        <div className="space-y-1">
          <Label className="text-xs text-muted-foreground">
            {t("hub.smartTakeover.selectCandidate", "Select configuration source:")}
          </Label>
          <div className="space-y-1">
            {conflict.candidates.map((candidate, idx) => (
              <button
                key={idx}
                type="button"
                onClick={() => onDecision(conflict.service_name, "use_new", idx)}
                className={`flex items-center gap-2 w-full text-left p-2 rounded hover:bg-muted/50 transition-colors ${compact ? "py-1" : ""}`}
              >
                {decision?.selected_candidate_index === idx ? (
                  <CircleDot className="h-3 w-3 text-primary shrink-0" />
                ) : (
                  <Circle className="h-3 w-3 text-muted-foreground shrink-0" />
                )}
                <SourceIcon source={candidate.adapter_id as "claude" | "cursor" | "codex" | "gemini"} className="h-3 w-3" />
                <span className={compact ? "text-xs" : "text-sm"}>
                  {getAdapterLabel(candidate.adapter_id)} ({getScopeLabel(candidate.scope, t)})
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* 决策选项 */}
      <div className="space-y-1">
        {options.map((option) => (
          <RadioOption
            key={option.value}
            value={option.value}
            label={option.label}
            description={option.description}
            selected={decision?.decision === option.value}
            onSelect={() => onDecision(conflict.service_name, option.value)}
          />
        ))}
      </div>
    </div>
  );
}

export default SmartTakeoverSheet;
