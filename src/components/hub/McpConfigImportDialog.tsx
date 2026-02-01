/**
 * MCP 配置导入对话框
 * Story 11.3: Task 9 - 配置导入前端 UI (AC: #1, #2, #4, #6)
 *
 * 提供完整的配置导入向导：
 * - 扫描检测 MCP 配置文件
 * - 预览将要导入的服务
 * - 解决配置冲突
 * - 设置所需环境变量
 * - 配置影子模式
 */

import { useState, useCallback, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  Loader2,
  FileCode,
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  FolderOpen,
  Key,
  ChevronRight,
  Eye,
  EyeOff,
} from "lucide-react";
import { feedback } from "@/lib/feedback";
import { cn } from "@/lib/utils";
import { ConfigDiffView } from "./ConfigDiffView";
import { ShadowModePreview } from "./ShadowModePreview";
import { ImportStepper } from "./ImportStepper";

// ===== 类型定义 =====

/** 配置作用域 (与后端 Rust 保持一致) */
type ConfigScope = "user" | "project";

/** 检测到的 MCP 服务 */
interface DetectedService {
  name: string;
  command: string;
  args: string[] | null;
  env: Record<string, string> | null;
  source_file: string;
  /** 适配器 ID (Story 11.8: 替代旧的 source_type) */
  adapter_id: string;
  /** 配置作用域 (Story 11.8: 新增) */
  scope?: ConfigScope;
}

/** 检测到的配置文件 */
interface DetectedConfig {
  /** 适配器 ID (Story 11.8: 替代旧的 source) */
  adapter_id: string;
  path: string;
  /** 配置作用域 (Story 11.8: 新增) */
  scope?: ConfigScope;
  services: DetectedService[];
  parse_errors: string[];
}

/** 服务冲突信息 */
interface ServiceConflict {
  name: string;
  existing: McpService | null;
  candidates: DetectedService[];
}

/** MCP 服务（数据库中的） */
interface McpService {
  id: string;
  name: string;
  command: string;
  args: string[] | null;
  env: Record<string, string> | null;
  source: "imported" | "manual";
  source_file: string | null;
  enabled: boolean;
}

/** 扫描结果 */
interface ScanResult {
  configs: DetectedConfig[];
}

/** 导入预览 */
interface ImportPreview {
  configs: DetectedConfig[];
  conflicts: ServiceConflict[];
  new_services: DetectedService[];
  env_vars_needed: string[];
  total_services: number;
}

/** 冲突解决策略 */
type ConflictResolution =
  | { keep: null }
  | { replace: number }
  | { rename: string }
  | { skip: null };

/** 导入请求 */
interface ImportRequest {
  services_to_import: string[];
  conflict_resolutions: Record<string, ConflictResolution>;
  env_var_values: Record<string, string>;
  enable_shadow_mode: boolean;
  gateway_url: string | null;
  /** 网关认证 Token (Story 11.8: 用于 HTTP Transport Authorization Header) */
  gateway_token?: string | null;
}

/** 导入结果 */
interface ImportResult {
  imported_count: number;
  skipped_count: number;
  backup_files: string[];
  shadow_configs: string[];
  errors: string[];
  imported_service_ids: string[];
}

// ===== 步骤枚举 =====
type ImportStep = "scan" | "preview" | "conflicts" | "env" | "confirm" | "execute" | "result";

// ===== 组件属性 =====
interface McpConfigImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
  projectPath?: string;
}

export function McpConfigImportDialog({
  open,
  onOpenChange,
  onSuccess,
  projectPath,
}: McpConfigImportDialogProps) {
  const { t } = useTranslation();

  // 步骤状态
  const [step, setStep] = useState<ImportStep>("scan");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 扫描结果（保留用于后续回退/重试）
  const [_scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [preview, setPreview] = useState<ImportPreview | null>(null);

  // 用户选择
  const [selectedServices, setSelectedServices] = useState<Set<string>>(new Set());
  const [conflictResolutions, setConflictResolutions] = useState<
    Record<string, ConflictResolution>
  >({});
  const [envVarValues, setEnvVarValues] = useState<Record<string, string>>({});
  const [showEnvValues, setShowEnvValues] = useState<Set<string>>(new Set());
  const [enableShadowMode, setEnableShadowMode] = useState(false);
  const [renameInputs, setRenameInputs] = useState<Record<string, string>>({});

  // 导入结果
  const [importResult, setImportResult] = useState<ImportResult | null>(null);

  // 重置状态
  const resetState = useCallback(() => {
    setStep("scan");
    setIsLoading(false);
    setError(null);
    setScanResult(null);
    setPreview(null);
    setSelectedServices(new Set());
    setConflictResolutions({});
    setEnvVarValues({});
    setShowEnvValues(new Set());
    setEnableShadowMode(false);
    setRenameInputs({});
    setImportResult(null);
  }, []);

  // 对话框打开时重置
  useEffect(() => {
    if (open) {
      resetState();
    }
  }, [open, resetState]);

  // 扫描配置文件
  const handleScan = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const result = await invoke<ScanResult>("scan_mcp_configs_cmd", {
        projectPath: projectPath || null,
      });

      setScanResult(result);

      // 检查是否有配置
      const totalServices = result.configs.reduce(
        (sum, c) => sum + c.services.length,
        0
      );

      if (totalServices === 0) {
        setError(t("hub.import.noConfigsFound"));
        return;
      }

      // 生成预览
      const previewResult = await invoke<ImportPreview>("preview_mcp_import", {
        scanResult: result,
      });

      setPreview(previewResult);

      // 默认选中所有新服务
      const selected = new Set<string>();
      previewResult.new_services.forEach((s) => selected.add(s.name));
      previewResult.conflicts.forEach((c) => selected.add(c.name));
      setSelectedServices(selected);

      // 初始化冲突解决策略
      const defaultResolutions: Record<string, ConflictResolution> = {};
      previewResult.conflicts.forEach((c) => {
        // 如果有已存在的服务，默认保留；否则默认选第一个候选
        if (c.existing) {
          defaultResolutions[c.name] = { keep: null };
        } else if (c.candidates.length > 0) {
          defaultResolutions[c.name] = { replace: 0 };
        }
      });
      setConflictResolutions(defaultResolutions);

      // 初始化环境变量值
      const defaultEnvVars: Record<string, string> = {};
      previewResult.env_vars_needed.forEach((name) => {
        defaultEnvVars[name] = "";
      });
      setEnvVarValues(defaultEnvVars);

      // 进入预览步骤
      setStep("preview");
    } catch (err) {
      console.error("[McpConfigImportDialog] Scan failed:", err);
      setError((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [projectPath, t]);

  // 执行导入
  const handleImport = useCallback(async () => {
    if (!preview) return;

    setIsLoading(true);
    setError(null);

    try {
      // 转换冲突解决策略为后端格式
      const resolutions: Record<string, ConflictResolution> = {};
      for (const [name, resolution] of Object.entries(conflictResolutions)) {
        if ("rename" in resolution) {
          // 使用用户输入的重命名值
          const newName = renameInputs[name] || `${name}_imported`;
          resolutions[name] = { rename: newName };
        } else {
          resolutions[name] = resolution;
        }
      }

      const request: ImportRequest = {
        services_to_import: Array.from(selectedServices),
        conflict_resolutions: resolutions,
        env_var_values: envVarValues,
        enable_shadow_mode: enableShadowMode,
        gateway_url: null, // Will be filled by backend if shadow mode is enabled
      };

      const result = await invoke<ImportResult>("execute_mcp_import", {
        preview,
        request,
      });

      setImportResult(result);
      setStep("result");

      // 如果有成功导入，通知父组件刷新
      if (result.imported_count > 0) {
        onSuccess();
      }
    } catch (err) {
      console.error("[McpConfigImportDialog] Import failed:", err);
      setError((err as Error).message);
      feedback.error(t("hub.import.importError"), (err as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [
    preview,
    selectedServices,
    conflictResolutions,
    renameInputs,
    envVarValues,
    enableShadowMode,
    onSuccess,
    t,
  ]);

  // 获取来源显示文本 (Story 11.8: 支持 adapter_id)
  const getSourceText = useCallback(
    (adapterId: string) => {
      switch (adapterId) {
        case "claude":
          return "Claude Code";
        case "claude_desktop":
          return "Claude Desktop";
        case "cursor":
          return "Cursor";
        case "codex":
          return "Codex CLI";
        case "gemini":
          return "Gemini CLI";
        default:
          return adapterId;
      }
    },
    []
  );

  // 判断是否有冲突需要解决
  const hasConflicts = useMemo(
    () => preview && preview.conflicts.length > 0,
    [preview]
  );

  // 判断是否需要设置环境变量
  const needsEnvVars = useMemo(
    () => preview && preview.env_vars_needed.length > 0,
    [preview]
  );

  // 判断是否可以进行下一步
  const canProceed = useMemo(() => {
    if (step === "preview") {
      return selectedServices.size > 0;
    }
    if (step === "conflicts") {
      // 所有冲突都需要有解决方案
      return preview?.conflicts.every((c) => {
        const resolution = conflictResolutions[c.name];
        if (!resolution) return false;
        if ("rename" in resolution) {
          // 重命名需要有有效名称
          const newName = renameInputs[c.name];
          return newName && newName.trim().length > 0;
        }
        return true;
      });
    }
    if (step === "env") {
      // 环境变量值可以为空（用户可能已经有了）
      return true;
    }
    return true;
  }, [step, selectedServices, preview, conflictResolutions, renameInputs]);

  // 处理下一步
  const handleNext = useCallback(() => {
    if (step === "preview") {
      if (hasConflicts) {
        setStep("conflicts");
      } else if (needsEnvVars) {
        setStep("env");
      } else {
        // 无冲突无环境变量，直接进入确认步骤
        setStep("confirm");
      }
    } else if (step === "conflicts") {
      if (needsEnvVars) {
        setStep("env");
      } else {
        setStep("confirm");
      }
    } else if (step === "env") {
      setStep("confirm");
    } else if (step === "confirm") {
      setStep("execute");
      handleImport();
    }
  }, [step, hasConflicts, needsEnvVars, handleImport]);

  // 处理服务选择
  const toggleService = useCallback((name: string, checked: boolean) => {
    setSelectedServices((prev) => {
      const next = new Set(prev);
      if (checked) {
        next.add(name);
      } else {
        next.delete(name);
      }
      return next;
    });
  }, []);

  // 处理冲突解决选择
  const handleConflictResolution = useCallback(
    (name: string, value: string) => {
      if (value === "keep") {
        setConflictResolutions((prev) => ({ ...prev, [name]: { keep: null } }));
      } else if (value === "skip") {
        setConflictResolutions((prev) => ({ ...prev, [name]: { skip: null } }));
      } else if (value === "rename") {
        setConflictResolutions((prev) => ({ ...prev, [name]: { rename: "" } }));
        if (!renameInputs[name]) {
          setRenameInputs((prev) => ({ ...prev, [name]: `${name}_imported` }));
        }
      } else if (value.startsWith("replace_")) {
        const index = parseInt(value.replace("replace_", ""), 10);
        setConflictResolutions((prev) => ({
          ...prev,
          [name]: { replace: index },
        }));
      }
    },
    [renameInputs]
  );

  // 获取当前冲突解决值
  const getResolutionValue = useCallback(
    (name: string): string => {
      const resolution = conflictResolutions[name];
      if (!resolution) return "keep";
      if ("keep" in resolution) return "keep";
      if ("skip" in resolution) return "skip";
      if ("rename" in resolution) return "rename";
      if ("replace" in resolution) return `replace_${resolution.replace}`;
      return "keep";
    },
    [conflictResolutions]
  );

  // 切换环境变量显示
  const toggleEnvValueVisibility = useCallback((name: string) => {
    setShowEnvValues((prev) => {
      const next = new Set(prev);
      if (next.has(name)) {
        next.delete(name);
      } else {
        next.add(name);
      }
      return next;
    });
  }, []);

  // 渲染扫描步骤
  const renderScanStep = () => (
    <div className="flex flex-col items-center justify-center py-8 space-y-4">
      {error ? (
        <>
          <AlertCircle className="h-12 w-12 text-destructive" />
          <p className="text-sm text-destructive text-center">{error}</p>
          <Button onClick={handleScan} disabled={isLoading}>
            {isLoading && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t("common.retry")}
          </Button>
        </>
      ) : isLoading ? (
        <>
          <Loader2 className="h-12 w-12 animate-spin text-muted-foreground" />
          <p className="text-sm text-muted-foreground">
            {t("hub.import.scanning")}
          </p>
        </>
      ) : (
        <>
          <FolderOpen className="h-12 w-12 text-muted-foreground" />
          <div className="text-center space-y-1">
            <p className="text-sm font-medium">{t("hub.import.scanTitle")}</p>
            <p className="text-xs text-muted-foreground">
              {t("hub.import.scanDescription")}
            </p>
          </div>
          <Button onClick={handleScan} data-testid="import-scan-button">
            <FileCode className="h-4 w-4 mr-2" />
            {t("hub.import.startScan")}
          </Button>
        </>
      )}
    </div>
  );

  // 渲染预览步骤
  const renderPreviewStep = () => {
    if (!preview) return null;

    return (
      <div className="space-y-4">
        {/* 扫描摘要 */}
        <Alert>
          <FileCode className="h-4 w-4" />
          <AlertDescription>
            {t("hub.import.foundSummary", {
              configs: preview.configs.length,
              services: preview.total_services,
            })}
          </AlertDescription>
        </Alert>

        {/* 配置文件列表 */}
        <ScrollArea className="h-[300px] lg:h-[400px] xl:h-[500px] pr-4">
          <div className="space-y-2">
            {preview.configs.map((config, configIndex) => (
              <Collapsible key={configIndex} defaultOpen>
                <div className="border rounded-lg">
                  <CollapsibleTrigger className="flex items-center gap-2 w-full p-3 text-left hover:bg-muted/50 transition-colors">
                    <ChevronRight className="h-4 w-4 shrink-0 transition-transform duration-200 [[data-state=open]>&]:rotate-90" />
                    <Badge variant="outline" className="shrink-0">
                      {getSourceText(config.adapter_id)}
                    </Badge>
                    <span className="text-xs text-muted-foreground truncate flex-1">
                      {config.path}
                    </span>
                    <Badge variant="secondary" className="shrink-0">
                      {config.services.length} {t("hub.import.services")}
                    </Badge>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <div className="space-y-2 p-3 pt-0">
                      {config.services.map((service, serviceIndex) => {
                        const isNew = preview.new_services.some(
                          (s) => s.name === service.name
                        );
                        const hasConflict = preview.conflicts.some(
                          (c) => c.name === service.name
                        );
                        const isSelected = selectedServices.has(service.name);

                        // 计算动作状态
                        const actionLabel = !isSelected
                          ? t("hub.import.actionSkip")
                          : hasConflict
                          ? t("hub.import.actionConflict")
                          : t("hub.import.actionAdd");
                        const actionClass = !isSelected
                          ? "bg-muted text-muted-foreground"
                          : hasConflict
                          ? "bg-amber-500/10 text-amber-500 border-amber-500/20"
                          : "bg-green-500/10 text-green-500 border-green-500/20";

                        return (
                          <div
                            key={serviceIndex}
                            className={cn(
                              "flex items-center gap-3 p-2 rounded-md bg-muted/50",
                              !isSelected && "opacity-60"
                            )}
                          >
                            <Checkbox
                              id={`service-${configIndex}-${serviceIndex}`}
                              checked={isSelected}
                              onCheckedChange={(checked) =>
                                toggleService(service.name, checked as boolean)
                              }
                              data-testid={`import-service-checkbox-${service.name}`}
                              className="border-zinc-500 data-[state=unchecked]:bg-zinc-800/50"
                            />
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2">
                                <span className="font-medium text-sm">
                                  {service.name}
                                </span>
                                {isNew && (
                                  <Badge
                                    variant="default"
                                    className="text-xs bg-green-500/10 text-green-500 border-green-500/20"
                                  >
                                    {t("hub.import.new")}
                                  </Badge>
                                )}
                                {hasConflict && (
                                  <Badge
                                    variant="default"
                                    className="text-xs bg-amber-500/10 text-amber-500 border-amber-500/20"
                                  >
                                    {t("hub.import.conflict")}
                                  </Badge>
                                )}
                              </div>
                              <code className="text-xs text-muted-foreground">
                                {service.command}{" "}
                                {service.args?.slice(0, 2).join(" ")}
                                {service.args && service.args.length > 2 && " ..."}
                              </code>
                            </div>
                            {/* 动作标签 */}
                            <Badge
                              variant="outline"
                              className={cn("text-xs shrink-0", actionClass)}
                              data-testid={`import-action-label-${service.name}`}
                            >
                              {actionLabel}
                            </Badge>
                          </div>
                        );
                      })}

                      {/* 解析错误 */}
                      {config.parse_errors.length > 0 && (
                        <Alert variant="destructive" className="mt-2">
                          <AlertCircle className="h-4 w-4" />
                          <AlertDescription>
                            {config.parse_errors.join(", ")}
                          </AlertDescription>
                        </Alert>
                      )}
                    </div>
                  </CollapsibleContent>
                </div>
              </Collapsible>
            ))}
          </div>
        </ScrollArea>

        {/* 选择摘要 */}
        <div className="flex items-center justify-between text-sm text-muted-foreground">
          <span>
            {t("hub.import.selectedCount", { count: selectedServices.size })}
          </span>
          <div className="flex gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                const all = new Set<string>();
                preview.new_services.forEach((s) => all.add(s.name));
                preview.conflicts.forEach((c) => all.add(c.name));
                setSelectedServices(all);
              }}
            >
              {t("hub.import.selectAll")}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setSelectedServices(new Set())}
            >
              {t("hub.import.selectNone")}
            </Button>
          </div>
        </div>
      </div>
    );
  };

  // 渲染冲突解决步骤
  const renderConflictsStep = () => {
    if (!preview || preview.conflicts.length === 0) return null;

    return (
      <div className="space-y-4">
        <Alert>
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            {t("hub.import.conflictsDescription", {
              count: preview.conflicts.length,
            })}
          </AlertDescription>
        </Alert>

        <ScrollArea className="h-[300px] lg:h-[400px] xl:h-[500px] pr-4">
          <div className="space-y-4">
            {preview.conflicts
              .filter((c) => selectedServices.has(c.name))
              .map((conflict) => (
                <div
                  key={conflict.name}
                  className="p-4 border rounded-lg space-y-3"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-medium">{conflict.name}</span>
                    <Badge variant="outline">
                      {conflict.candidates.length} {t("hub.import.sources")}
                    </Badge>
                  </div>

                  {/* 配置差异对比 */}
                  <ConfigDiffView
                    serviceName={conflict.name}
                    existing={conflict.existing}
                    candidates={conflict.candidates}
                    getSourceText={getSourceText}
                  />

                  <div className="space-y-2">
                    <Label>{t("hub.import.resolution")}</Label>
                    <Select
                      value={getResolutionValue(conflict.name)}
                      onValueChange={(v) =>
                        handleConflictResolution(conflict.name, v)
                      }
                    >
                      <SelectTrigger data-testid={`conflict-resolution-${conflict.name}`}>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {conflict.existing && (
                          <SelectItem value="keep">
                            {t("hub.import.keepExisting")}
                          </SelectItem>
                        )}
                        {conflict.candidates.map((candidate, index) => (
                          <SelectItem key={index} value={`replace_${index}`}>
                            {t("hub.import.useFrom", {
                              source: getSourceText(candidate.adapter_id),
                            })}
                          </SelectItem>
                        ))}
                        <SelectItem value="rename">
                          {t("hub.import.renameAndImport")}
                        </SelectItem>
                        <SelectItem value="skip">
                          {t("hub.import.skip")}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {/* 重命名输入 */}
                  {"rename" in (conflictResolutions[conflict.name] || {}) && (
                    <div className="space-y-2">
                      <Label>{t("hub.import.newName")}</Label>
                      <Input
                        value={renameInputs[conflict.name] || ""}
                        onChange={(e) =>
                          setRenameInputs((prev) => ({
                            ...prev,
                            [conflict.name]: e.target.value,
                          }))
                        }
                        placeholder={`${conflict.name}_imported`}
                        data-testid={`conflict-rename-${conflict.name}`}
                      />
                    </div>
                  )}
                </div>
              ))}
          </div>
        </ScrollArea>
      </div>
    );
  };

  // 渲染环境变量设置步骤
  const renderEnvStep = () => {
    if (!preview || preview.env_vars_needed.length === 0) return null;

    return (
      <div className="space-y-4">
        <Alert>
          <Key className="h-4 w-4" />
          <AlertDescription>
            {t("hub.import.envDescription", {
              count: preview.env_vars_needed.length,
            })}
          </AlertDescription>
        </Alert>

        <ScrollArea className="h-[300px] lg:h-[400px] xl:h-[500px] pr-4">
          <div className="space-y-4">
            {preview.env_vars_needed.map((varName) => (
              <div key={varName} className="space-y-2">
                <Label className="flex items-center gap-2">
                  <Key className="h-3 w-3" />
                  {varName}
                </Label>
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <Input
                      type={showEnvValues.has(varName) ? "text" : "password"}
                      value={envVarValues[varName] || ""}
                      onChange={(e) =>
                        setEnvVarValues((prev) => ({
                          ...prev,
                          [varName]: e.target.value,
                        }))
                      }
                      placeholder={t("hub.import.envPlaceholder")}
                      className="pr-10"
                      data-testid={`env-var-input-${varName}`}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="absolute right-1 top-1/2 -translate-y-1/2 h-7 w-7"
                      onClick={() => toggleEnvValueVisibility(varName)}
                    >
                      {showEnvValues.has(varName) ? (
                        <EyeOff className="h-4 w-4" />
                      ) : (
                        <Eye className="h-4 w-4" />
                      )}
                    </Button>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground">
                  {t("hub.import.envHint")}
                </p>
              </div>
            ))}
          </div>
        </ScrollArea>

        {/* 影子模式设置 */}
        <div className="p-4 border rounded-lg space-y-2">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label className="text-sm font-medium">
                {t("hub.import.shadowMode")}
              </Label>
              <p className="text-xs text-muted-foreground">
                {t("hub.import.shadowModeDescription")}
              </p>
            </div>
            <Switch
              checked={enableShadowMode}
              onCheckedChange={setEnableShadowMode}
              data-testid="shadow-mode-switch"
            />
          </div>

          {/* 影子模式变更预览 */}
          <ShadowModePreview
            enabled={enableShadowMode}
            configs={preview.configs}
          />
        </div>
      </div>
    );
  };

  // 计算确认步骤的统计信息
  const getConfirmStats = useCallback(() => {
    if (!preview) return { addCount: 0, conflictCount: 0, renameCount: 0, fileCount: 0, envCount: 0, envNeeded: 0 };

    // 新服务数量（选中且不冲突的）
    const addCount = preview.new_services.filter((s) =>
      selectedServices.has(s.name)
    ).length;

    // 覆盖冲突数量（选中且解决方式为 replace 的）
    const conflictCount = preview.conflicts.filter((c) => {
      if (!selectedServices.has(c.name)) return false;
      const resolution = conflictResolutions[c.name];
      return resolution && "replace" in resolution;
    }).length;

    // 重命名导入数量（选中且解决方式为 rename 的）
    const renameCount = preview.conflicts.filter((c) => {
      if (!selectedServices.has(c.name)) return false;
      const resolution = conflictResolutions[c.name];
      return resolution && "rename" in resolution;
    }).length;

    // 影子模式下将修改的文件数量
    const fileCount = enableShadowMode ? preview.configs.length : 0;

    // 已设置的环境变量数量
    const envCount = Object.values(envVarValues).filter((v) => v && v.trim() !== "").length;

    // 需要设置的环境变量总数
    const envNeeded = preview.env_vars_needed.length;

    return { addCount, conflictCount, renameCount, fileCount, envCount, envNeeded };
  }, [preview, selectedServices, conflictResolutions, enableShadowMode, envVarValues]);

  // 渲染确认步骤
  const renderConfirmStep = () => {
    if (!preview) return null;

    const { addCount, conflictCount, renameCount, fileCount, envNeeded } = getConfirmStats();

    return (
      <div className="space-y-6">
        <div className="text-center py-4">
          <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-blue-500/10 mb-3">
            <FileCode className="h-6 w-6 text-blue-500" />
          </div>
          <p className="text-sm text-muted-foreground">
            {t("hub.import.confirmDescription")}
          </p>
        </div>

        {/* 操作摘要 */}
        <div className="space-y-2" data-testid="confirm-summary">
          {addCount > 0 && (
            <div className="flex items-center gap-3 p-3 border rounded-lg">
              <CheckCircle2 className="h-5 w-5 text-green-500 shrink-0" />
              <span className="text-sm">
                {t("hub.import.confirmSummaryAdd", { count: addCount })}
              </span>
            </div>
          )}

          {conflictCount > 0 && (
            <div className="flex items-center gap-3 p-3 border rounded-lg">
              <AlertTriangle className="h-5 w-5 text-amber-500 shrink-0" />
              <span className="text-sm">
                {t("hub.import.confirmSummaryConflict", { count: conflictCount })}
              </span>
            </div>
          )}

          {renameCount > 0 && (
            <div className="flex items-center gap-3 p-3 border rounded-lg">
              <CheckCircle2 className="h-5 w-5 text-blue-500 shrink-0" />
              <span className="text-sm">
                {t("hub.import.confirmSummaryRename", { count: renameCount })}
              </span>
            </div>
          )}

          {fileCount > 0 && (
            <div className="flex items-center gap-3 p-3 border rounded-lg">
              <FileCode className="h-5 w-5 text-blue-500 shrink-0" />
              <span className="text-sm">
                {t("hub.import.confirmSummaryFiles", { count: fileCount })}
              </span>
            </div>
          )}

          {envNeeded > 0 && (
            <div className="flex items-center gap-3 p-3 border rounded-lg">
              <Key className="h-5 w-5 text-purple-500 shrink-0" />
              <span className="text-sm">
                {t("hub.import.confirmSummaryEnv", { count: envNeeded })}
              </span>
            </div>
          )}
        </div>
      </div>
    );
  };

  // 渲染执行步骤
  const renderExecuteStep = () => (
    <div className="flex flex-col items-center justify-center py-8 space-y-4">
      <Loader2 className="h-12 w-12 animate-spin text-blue-500" />
      <p className="text-sm font-medium">{t("hub.import.importing")}</p>
      <p className="text-xs text-muted-foreground">
        {t("hub.import.importingHint")}
      </p>
    </div>
  );

  // 渲染结果步骤
  const renderResultStep = () => {
    if (!importResult) return null;

    const hasErrors = importResult.errors.length > 0;
    const isPartialSuccess =
      importResult.imported_count > 0 && importResult.skipped_count > 0;

    return (
      <div className="space-y-4">
        {/* 结果图标和标题 */}
        <div className="flex flex-col items-center py-4 space-y-2">
          {hasErrors ? (
            <AlertCircle className="h-12 w-12 text-destructive" />
          ) : isPartialSuccess ? (
            <AlertTriangle className="h-12 w-12 text-amber-500" />
          ) : (
            <CheckCircle2 className="h-12 w-12 text-green-500" />
          )}
          <p className="text-lg font-medium">
            {hasErrors
              ? t("hub.import.resultError")
              : isPartialSuccess
              ? t("hub.import.resultPartial")
              : t("hub.import.resultSuccess")}
          </p>
        </div>

        {/* 结果摘要 */}
        <div className="grid grid-cols-2 gap-4">
          <div className="p-4 border rounded-lg text-center">
            <p className="text-2xl font-bold text-green-500">
              {importResult.imported_count}
            </p>
            <p className="text-xs text-muted-foreground">
              {t("hub.import.imported")}
            </p>
          </div>
          <div className="p-4 border rounded-lg text-center">
            <p className="text-2xl font-bold text-muted-foreground">
              {importResult.skipped_count}
            </p>
            <p className="text-xs text-muted-foreground">
              {t("hub.import.skipped")}
            </p>
          </div>
        </div>

        {/* 备份文件列表 */}
        {importResult.backup_files.length > 0 && (
          <div className="space-y-2">
            <Label className="text-sm">
              {t("hub.import.backupFiles")} ({importResult.backup_files.length})
            </Label>
            <div className="text-xs text-muted-foreground bg-muted p-2 rounded max-h-20 overflow-auto">
              {importResult.backup_files.map((file, i) => (
                <div key={i}>{file}</div>
              ))}
            </div>
          </div>
        )}

        {/* 影子配置文件列表 */}
        {importResult.shadow_configs.length > 0 && (
          <div className="space-y-2">
            <Label className="text-sm">
              {t("hub.import.shadowConfigs")} ({importResult.shadow_configs.length})
            </Label>
            <div className="text-xs text-muted-foreground bg-muted p-2 rounded max-h-20 overflow-auto">
              {importResult.shadow_configs.map((file, i) => (
                <div key={i}>{file}</div>
              ))}
            </div>
          </div>
        )}

        {/* 错误列表 */}
        {hasErrors && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              <ul className="list-disc list-inside">
                {importResult.errors.map((err, i) => (
                  <li key={i}>{err}</li>
                ))}
              </ul>
            </AlertDescription>
          </Alert>
        )}
      </div>
    );
  };

  // 获取当前步骤标题
  const getStepTitle = () => {
    switch (step) {
      case "scan":
        return t("hub.import.title");
      case "preview":
        return t("hub.import.previewTitle");
      case "conflicts":
        return t("hub.import.conflictsTitle");
      case "env":
        return t("hub.import.envTitle");
      case "confirm":
        return t("hub.import.confirmTitle");
      case "execute":
        return t("hub.import.importingTitle");
      case "result":
        return t("hub.import.resultTitle");
      default:
        return t("hub.import.title");
    }
  };

  // 获取当前步骤描述
  const getStepDescription = () => {
    switch (step) {
      case "scan":
        return t("hub.import.description");
      case "preview":
        return t("hub.import.previewDescription");
      case "conflicts":
        return t("hub.import.conflictsStepDescription");
      case "env":
        return t("hub.import.envStepDescription");
      case "confirm":
        return t("hub.import.confirmDescription");
      case "execute":
        return t("hub.import.importingDescription");
      case "result":
        return t("hub.import.resultDescription");
      default:
        return "";
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileCode className="h-5 w-5" />
            {getStepTitle()}
          </DialogTitle>
          <DialogDescription>{getStepDescription()}</DialogDescription>
        </DialogHeader>

        {/* 步骤指示器 */}
        {step !== "scan" && step !== "result" && (
          <ImportStepper
            currentStep={step}
            hasConflicts={hasConflicts}
            needsEnvVars={needsEnvVars}
          />
        )}

        {/* 步骤内容 */}
        <div className="py-4">
          {step === "scan" && renderScanStep()}
          {step === "preview" && renderPreviewStep()}
          {step === "conflicts" && renderConflictsStep()}
          {step === "env" && renderEnvStep()}
          {step === "confirm" && renderConfirmStep()}
          {step === "execute" && renderExecuteStep()}
          {step === "result" && renderResultStep()}
        </div>

        {/* 底部按钮 */}
        <DialogFooter>
          {step === "result" ? (
            <Button onClick={() => onOpenChange(false)}>
              {t("common.close")}
            </Button>
          ) : step !== "scan" && step !== "execute" ? (
            <>
              <Button
                variant="outline"
                onClick={() => {
                  if (step === "preview") {
                    setStep("scan");
                    setScanResult(null);
                    setPreview(null);
                  } else if (step === "conflicts") {
                    setStep("preview");
                  } else if (step === "env") {
                    if (hasConflicts) {
                      setStep("conflicts");
                    } else {
                      setStep("preview");
                    }
                  } else if (step === "confirm") {
                    // 从确认步骤返回
                    if (needsEnvVars) {
                      setStep("env");
                    } else if (hasConflicts) {
                      setStep("conflicts");
                    } else {
                      setStep("preview");
                    }
                  }
                }}
                disabled={isLoading}
              >
                {step === "confirm" ? t("hub.import.confirmBack") : t("common.back")}
              </Button>
              <Button
                onClick={handleNext}
                disabled={!canProceed || isLoading}
                data-testid="import-next-button"
              >
                {isLoading && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                {step === "confirm" ? t("hub.import.confirmImport") : t("common.next")}
              </Button>
            </>
          ) : null}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default McpConfigImportDialog;
