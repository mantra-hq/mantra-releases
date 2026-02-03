/**
 * SmartTakeoverSheet - 智能接管预览 Sheet
 * Story 11.19: MCP 智能接管合并引擎 - Task 5
 *
 * 功能：
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
} from "lucide-react";
import { SourceIcon } from "@/components/import/SourceIcons";
import { ConfigDiffView } from "./ConfigDiffView";
import {
  previewSmartTakeover,
  executeSmartTakeover,
  previewNeedsDecision,
  previewIsEmpty,
  getPreviewStats,
  type TakeoverPreview,
  type TakeoverDecision,
  type TakeoverDecisionOption,
  type ConflictDetail,
  type SmartTakeoverResult,
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
  const [preview, setPreview] = useState<TakeoverPreview | null>(null);
  const [decisions, setDecisions] = useState<Map<string, TakeoverDecision>>(new Map());
  const [result, setResult] = useState<SmartTakeoverResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 折叠状态
  const [autoCreateOpen, setAutoCreateOpen] = useState(true);
  const [autoSkipOpen, setAutoSkipOpen] = useState(false);
  const [needsDecisionOpen, setNeedsDecisionOpen] = useState(true);

  // 加载预览
  const loadPreview = useCallback(async () => {
    setStep("loading");
    setError(null);
    setDecisions(new Map());
    setResult(null);

    try {
      const previewResult = await previewSmartTakeover(projectId, projectPath);
      setPreview(previewResult);
      setStep("preview");
    } catch (err) {
      console.error("[SmartTakeoverSheet] Failed to load preview:", err);
      setError((err as Error).message || t("hub.smartTakeover.errorLoadPreview", "Failed to load preview"));
      setStep("preview");
    }
  }, [projectId, projectPath, t]);

  // 打开时加载预览
  useEffect(() => {
    if (open) {
      loadPreview();
    }
  }, [open, loadPreview]);

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

  // 检查是否所有冲突都已决策
  const allDecisionsMade = useMemo(() => {
    if (!preview) return true;
    return preview.needs_decision.every((conflict) => decisions.has(conflict.service_name));
  }, [preview, decisions]);

  // 执行接管
  const handleExecute = useCallback(async () => {
    if (!preview) return;

    setStep("executing");
    setError(null);

    try {
      const decisionsList = Array.from(decisions.values());
      const executeResult = await executeSmartTakeover(projectId, preview, decisionsList);
      setResult(executeResult);
      setStep("result");

      // 如果成功，调用回调
      if (executeResult.errors.length === 0) {
        onSuccess?.();
      }
    } catch (err) {
      console.error("[SmartTakeoverSheet] Failed to execute takeover:", err);
      setError((err as Error).message || t("hub.smartTakeover.errorExecute", "Failed to execute takeover"));
      setStep("preview");
    }
  }, [preview, decisions, projectId, onSuccess, t]);

  // 关闭 Sheet
  const handleClose = useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  // 预览统计
  const stats = preview ? getPreviewStats(preview) : null;

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
          {step === "preview" && preview && !error && (
            <div className="space-y-6 py-4">
              {/* 统计摘要 */}
              {stats && (
                <div className="flex gap-4 text-sm">
                  <div className="flex items-center gap-1.5">
                    <Plus className="h-4 w-4 text-green-500" />
                    <span>{t("hub.smartTakeover.statsAutoCreate", "{{count}} new", { count: stats.autoCreateCount })}</span>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <SkipForward className="h-4 w-4 text-blue-500" />
                    <span>{t("hub.smartTakeover.statsAutoSkip", "{{count}} existing", { count: stats.autoSkipCount })}</span>
                  </div>
                  {stats.needsDecisionCount > 0 && (
                    <div className="flex items-center gap-1.5">
                      <HelpCircle className="h-4 w-4 text-amber-500" />
                      <span>{t("hub.smartTakeover.statsNeedsDecision", "{{count}} need decision", { count: stats.needsDecisionCount })}</span>
                    </div>
                  )}
                </div>
              )}

              {/* 空状态 */}
              {previewIsEmpty(preview) && (
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertTitle>{t("hub.smartTakeover.emptyTitle", "No configurations found")}</AlertTitle>
                  <AlertDescription>
                    {t("hub.smartTakeover.emptyDescription", "No MCP configurations were detected in this project.")}
                  </AlertDescription>
                </Alert>
              )}

              {/* 三档分类展示 */}
              <div className="space-y-2">
                {/* 可直接导入 */}
                {preview.auto_create.length > 0 && (
                  <Collapsible open={autoCreateOpen} onOpenChange={setAutoCreateOpen}>
                    <CollapsibleTrigger className="flex items-center gap-2 w-full p-3 rounded-lg border hover:bg-muted/50 transition-colors">
                      {autoCreateOpen ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                      <Plus className="h-4 w-4 text-green-500" />
                      <span className="font-medium">{t("hub.smartTakeover.sectionAutoCreate", "New Services")}</span>
                      <Badge variant="secondary" className="ml-auto">
                        {preview.auto_create.length}
                      </Badge>
                    </CollapsibleTrigger>
                    <CollapsibleContent className="pt-2 pl-4">
                      <div className="space-y-2">
                        {preview.auto_create.map((item) => (
                          <div
                            key={`${item.service_name}-${item.adapter_id}`}
                            className="flex items-center justify-between p-2 rounded-md bg-muted/50"
                          >
                            <div className="flex items-center gap-2">
                              <SourceIcon source={item.adapter_id as "claude" | "cursor" | "codex" | "gemini"} className="h-4 w-4" />
                              <span className="font-medium">{item.service_name}</span>
                            </div>
                            <div className="flex items-center gap-2 text-xs text-muted-foreground">
                              <Badge variant="outline" className="text-xs">
                                {getAdapterLabel(item.adapter_id)}
                              </Badge>
                              <Badge variant="outline" className="text-xs">
                                {getScopeLabel(item.scope, t)}
                              </Badge>
                            </div>
                          </div>
                        ))}
                      </div>
                    </CollapsibleContent>
                  </Collapsible>
                )}

                {/* 已存在（跳过） */}
                {preview.auto_skip.length > 0 && (
                  <Collapsible open={autoSkipOpen} onOpenChange={setAutoSkipOpen}>
                    <CollapsibleTrigger className="flex items-center gap-2 w-full p-3 rounded-lg border hover:bg-muted/50 transition-colors">
                      {autoSkipOpen ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                      <SkipForward className="h-4 w-4 text-blue-500" />
                      <span className="font-medium">{t("hub.smartTakeover.sectionAutoSkip", "Existing Services")}</span>
                      <Badge variant="secondary" className="ml-auto">
                        {preview.auto_skip.length}
                      </Badge>
                    </CollapsibleTrigger>
                    <CollapsibleContent className="pt-2 pl-4">
                      <p className="text-xs text-muted-foreground mb-2">
                        {t("hub.smartTakeover.autoSkipHint", "These services already exist with identical configuration.")}
                      </p>
                      <div className="space-y-2">
                        {preview.auto_skip.map((item) => (
                          <div
                            key={`${item.service_name}-${item.detected_adapter_id}`}
                            className="flex items-center justify-between p-2 rounded-md bg-muted/50"
                          >
                            <div className="flex items-center gap-2">
                              <CheckCircle2 className="h-4 w-4 text-green-500" />
                              <span className="font-medium">{item.service_name}</span>
                            </div>
                            <Badge variant="outline" className="text-xs">
                              {getAdapterLabel(item.detected_adapter_id)}
                            </Badge>
                          </div>
                        ))}
                      </div>
                    </CollapsibleContent>
                  </Collapsible>
                )}

                {/* 需要决策 */}
                {preview.needs_decision.length > 0 && (
                  <Collapsible open={needsDecisionOpen} onOpenChange={setNeedsDecisionOpen}>
                    <CollapsibleTrigger className="flex items-center gap-2 w-full p-3 rounded-lg border border-amber-500/50 hover:bg-muted/50 transition-colors">
                      {needsDecisionOpen ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                      <HelpCircle className="h-4 w-4 text-amber-500" />
                      <span className="font-medium">{t("hub.smartTakeover.sectionNeedsDecision", "Needs Decision")}</span>
                      <Badge variant="secondary" className="ml-auto bg-amber-500/10 text-amber-500">
                        {preview.needs_decision.length}
                      </Badge>
                    </CollapsibleTrigger>
                    <CollapsibleContent className="pt-2 pl-4">
                      <div className="space-y-4">
                        {preview.needs_decision.map((conflict) => (
                          <ConflictDecisionPanel
                            key={conflict.service_name}
                            conflict={conflict}
                            decision={decisions.get(conflict.service_name)}
                            onDecision={setDecision}
                            t={t}
                          />
                        ))}
                      </div>
                    </CollapsibleContent>
                  </Collapsible>
                )}
              </div>

              {/* 环境变量提示 */}
              {preview.env_vars_needed.length > 0 && (
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertTitle>{t("hub.smartTakeover.envVarsTitle", "Environment Variables Required")}</AlertTitle>
                  <AlertDescription>
                    {t("hub.smartTakeover.envVarsDescription", "The following environment variables are needed: {{vars}}", {
                      vars: preview.env_vars_needed.join(", "),
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
              {result.errors.length === 0 ? (
                <Alert className="border-green-500/50 bg-green-500/10">
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                  <AlertTitle className="text-green-500">
                    {t("hub.smartTakeover.successTitle", "Takeover Complete")}
                  </AlertTitle>
                  <AlertDescription>
                    {t("hub.smartTakeover.successDescription", "Successfully imported {{created}} services, skipped {{skipped}}, updated {{updated}}.", {
                      created: result.created_count,
                      skipped: result.skipped_count,
                      updated: result.updated_count,
                    })}
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
                disabled={!preview || previewIsEmpty(preview) || (previewNeedsDecision(preview) && !allDecisionsMade)}
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

// ===== 冲突决策面板 =====

interface ConflictDecisionPanelProps {
  conflict: ConflictDetail;
  decision?: TakeoverDecision;
  onDecision: (serviceName: string, decision: TakeoverDecisionOption, candidateIndex?: number) => void;
  t: (key: string, fallback: string, opts?: Record<string, unknown>) => string;
}

function ConflictDecisionPanel({ conflict, decision, onDecision, t }: ConflictDecisionPanelProps) {
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
      className="flex items-start gap-2 w-full text-left p-2 rounded hover:bg-muted/50 transition-colors"
    >
      {selected ? (
        <CircleDot className="h-4 w-4 mt-0.5 text-primary shrink-0" />
      ) : (
        <Circle className="h-4 w-4 mt-0.5 text-muted-foreground shrink-0" />
      )}
      <div className="grid gap-0.5 min-w-0">
        <span className="text-sm font-medium">{label}</span>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
    </button>
  );

  return (
    <div className="border rounded-lg p-4 space-y-3">
      {/* 服务名称和冲突类型 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 text-amber-500" />
          <span className="font-medium">{conflict.service_name}</span>
        </div>
        <Badge variant="outline" className="text-xs text-amber-500 border-amber-500/50">
          {getConflictTypeLabel(conflict.conflict_type)}
        </Badge>
      </div>

      {/* 配置差异对比 (如果有) */}
      {conflict.existing_service && conflict.candidates.length > 0 && conflict.conflict_type === "config_diff" && (
        <ConfigDiffView
          serviceName={conflict.service_name}
          existing={{
            command: conflict.existing_service.config_summary.command,
            args: conflict.existing_service.config_summary.args,
            env: null,
          }}
          candidates={conflict.candidates.map((c) => ({
            name: conflict.service_name,
            source_file: c.config_path,
            adapter_id: c.adapter_id,
            command: c.config_summary.command,
            args: c.config_summary.args,
            env: null,
          }))}
          getSourceText={getAdapterLabel}
        />
      )}

      {/* 候选项选择 (multi_source) */}
      {conflict.conflict_type === "multi_source" && conflict.candidates.length > 1 && (
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">
            {t("hub.smartTakeover.selectCandidate", "Select configuration source:")}
          </Label>
          <div className="space-y-1">
            {conflict.candidates.map((candidate, idx) => (
              <button
                key={idx}
                type="button"
                onClick={() => onDecision(conflict.service_name, "use_new", idx)}
                className="flex items-center gap-2 w-full text-left p-2 rounded hover:bg-muted/50 transition-colors"
              >
                {decision?.selected_candidate_index === idx ? (
                  <CircleDot className="h-4 w-4 text-primary shrink-0" />
                ) : (
                  <Circle className="h-4 w-4 text-muted-foreground shrink-0" />
                )}
                <SourceIcon source={candidate.adapter_id as "claude" | "cursor" | "codex" | "gemini"} className="h-3.5 w-3.5" />
                <span className="text-sm">
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
