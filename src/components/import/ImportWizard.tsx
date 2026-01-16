/**
 * ImportWizard Component - 导入向导 Modal
 * Story 2.9: Task 1, Task 8
 * Story 2.20: Import Status Enhancement
 * Story 2.23: Import Progress Events
 *
 * 多步骤导入向导，包含：
 * - 步骤 1: 选择导入源
 * - 步骤 2: 选择文件
 * - 步骤 3: 导入进度
 * - 步骤 4: 完成确认
 */

import * as React from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { CheckIcon } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Button,
} from "@/components/ui";
import { cn } from "@/lib/utils";
import { feedback } from "@/lib/feedback";
import { appLog } from "@/lib/log-actions";
import { useImportStore } from "@/stores";
import { scanLogDirectory, selectLogFiles, importSessionsWithProgress, cancelImport } from "@/lib/import-ipc";
import { getImportedSessionIds, getProject, getProjectSessions } from "@/lib/project-ipc";
import { SourceSelector, type ImportSource } from "./SourceSelector";
import { FileSelector } from "./FileSelector";
import { ImportProgress, type ImportProgressData, type RecentFile } from "./ImportProgress";
import { ImportComplete } from "./ImportComplete";

/** 导入步骤类型 */
export type ImportStep = "source" | "files" | "progress" | "complete";

/** 步骤配置 */
interface StepConfig {
  id: ImportStep;
  label: string;
  number: number;
}

/** 步骤配置列表 (标签通过 i18n 动态获取) */
const STEPS: StepConfig[] = [
  { id: "source", label: "import.steps.source", number: 1 },
  { id: "files", label: "import.steps.files", number: 2 },
  { id: "progress", label: "import.steps.progress", number: 3 },
  { id: "complete", label: "import.steps.complete", number: 4 },
];

/** ImportWizard Props */
export interface ImportWizardProps {
  /** 是否打开 */
  open: boolean;
  /** 打开状态变更回调 */
  onOpenChange: (open: boolean) => void;
  /** 初始步骤 (用于测试) */
  initialStep?: ImportStep;
  /** 导入完成回调 */
  onComplete?: () => void;
}

/**
 * 获取步骤状态
 */
function getStepState(
  stepId: ImportStep,
  currentStep: ImportStep
): "pending" | "active" | "completed" {
  const currentIndex = STEPS.findIndex((s) => s.id === currentStep);
  const stepIndex = STEPS.findIndex((s) => s.id === stepId);

  if (stepIndex < currentIndex) return "completed";
  if (stepIndex === currentIndex) return "active";
  return "pending";
}

/**
 * 步骤指示器组件
 */
function StepIndicator({
  step,
  state,
  t,
}: {
  step: StepConfig;
  state: "pending" | "active" | "completed";
  t: (key: string) => string;
}) {
  return (
    <div
      data-testid={`step-${step.id}`}
      data-state={state}
      aria-current={state === "active" ? "step" : undefined}
      className="flex items-center gap-2"
    >
      {/* 步骤数字/图标 */}
      <div
        className={cn(
          "w-6 h-6 rounded-full flex items-center justify-center text-xs font-semibold transition-colors",
          state === "pending" && "bg-muted text-muted-foreground",
          state === "active" && "bg-primary text-primary-foreground",
          state === "completed" && "bg-emerald-500 text-white"
        )}
      >
        {state === "completed" ? (
          <CheckIcon className="w-3.5 h-3.5" />
        ) : (
          step.number
        )}
      </div>

      {/* 步骤标签 */}
      <span
        className={cn(
          "text-sm transition-colors",
          state === "pending" && "text-muted-foreground",
          state === "active" && "text-primary font-medium",
          state === "completed" && "text-emerald-500"
        )}
      >
        {t(step.label)}
      </span>
    </div>
  );
}

/**
 * ImportWizard 组件
 * 多步骤导入向导 Modal
 */
export function ImportWizard({
  open,
  onOpenChange,
  initialStep = "source",
  onComplete,
}: ImportWizardProps) {
  const navigate = useNavigate();
  const { t } = useTranslation();

  // Store 状态
  const {
    step: currentStep,
    source,
    discoveredFiles,
    selectedFiles,
    expandedProjects,
    searchQuery,
    progress,
    results,
    isLoading,
    errors,
    importedSessionIds,
    importedProjects,
    setStep,
    setSource,
    setDiscoveredFiles,
    toggleFile,
    clearAll,
    invertSelection,
    toggleProject,
    toggleProjectExpand,
    setSearchQuery,
    setProgress,
    addResult,
    setLoading,
    addError,
    reset,
    setImportedSessionIds,
    selectAllNew,
    addImportedProject,
    mergeRetryResults,
    lastScannedSource,
    setLastScannedSource,
    clearDiscoveredFiles,
    updateImportedProjectsIsEmpty,
  } = useImportStore();

  // Story 2.23: 重试状态
  const [isRetrying, setIsRetrying] = React.useState(false);

  // 当 initialStep 变化时同步 (用于测试)
  React.useEffect(() => {
    setStep(initialStep);
  }, [initialStep, setStep]);

  // 重置状态当 dialog 关闭时
  React.useEffect(() => {
    if (!open) {
      reset();
    }
  }, [open, reset]);

  // Story 2.20 改进: 加载已导入会话 ID
  React.useEffect(() => {
    if (open) {
      getImportedSessionIds()
        .then((ids) => {
          setImportedSessionIds(ids);
        })
        .catch((err) => {
          console.error("Failed to load imported session IDs:", err);
        });
    }
  }, [open, setImportedSessionIds]);

  /**
   * 处理来源选择
   */
  const handleSourceSelect = React.useCallback(
    (newSource: ImportSource) => {
      setSource(newSource);
    },
    [setSource]
  );

  /**
   * 扫描默认路径
   */
  const handleScan = React.useCallback(async () => {
    if (!source) return;

    setLoading(true);
    try {
      const files = await scanLogDirectory(source);
      setDiscoveredFiles(files);
      // Story 2.24: 记录当前扫描的源
      setLastScannedSource(source);
    } catch (err) {
      console.error("扫描失败:", err);
    } finally {
      setLoading(false);
    }
  }, [source, setLoading, setDiscoveredFiles, setLastScannedSource]);

  /**
   * 手动选择目录
   */
  const handleSelectFiles = React.useCallback(async () => {
    setLoading(true);
    try {
      // selectLogFiles 现在返回 DiscoveredFile[] (扫描用户选择的目录)
      const files = await selectLogFiles();
      if (files.length > 0) {
        setDiscoveredFiles(files);
      }
    } catch (err) {
      console.error("选择目录失败:", err);
    } finally {
      setLoading(false);
    }
  }, [setLoading, setDiscoveredFiles]);

  // Story 2.23: 自动扫描 - 进入文件选择步骤时自动开始扫描
  // 使用 ref 跟踪是否已扫描，防止重复扫描。当用户返回 source 步骤时会重置此标记。
  // 注意：如果用户快速切换步骤（source → files → source → files），第二次进入 files 步骤
  // 会触发新的扫描，这是预期行为（用户可能更换了 source）。
  const hasAutoScannedRef = React.useRef(false);
  React.useEffect(() => {
    // 当进入 files 步骤且有 source 且没有文件且未扫描过时，自动扫描
    if (
      currentStep === "files" &&
      source &&
      discoveredFiles.length === 0 &&
      !isLoading &&
      !hasAutoScannedRef.current
    ) {
      hasAutoScannedRef.current = true;
      // 延迟 100ms 启动扫描，让 UI 有时间渲染
      const timer = setTimeout(() => {
        handleScan();
      }, 100);
      return () => clearTimeout(timer);
    }
  }, [currentStep, source, discoveredFiles.length, isLoading, handleScan]);

  // Story 2.24: 检测源变化时清除文件并重置扫描标记
  React.useEffect(() => {
    if (source && lastScannedSource && source !== lastScannedSource) {
      // 源已变化，清除发现的文件
      clearDiscoveredFiles();
      hasAutoScannedRef.current = false;
    }
  }, [source, lastScannedSource, clearDiscoveredFiles]);

  // 重置自动扫描标记当返回到 source 步骤（允许用户更换 source 后重新扫描）
  React.useEffect(() => {
    if (currentStep === "source") {
      hasAutoScannedRef.current = false;
    }
  }, [currentStep]);

  // Story 2.23: 最近处理的文件列表状态
  const [recentFiles, setRecentFiles] = React.useState<RecentFile[]>([]);

  // Story 2.23: 取消导入状态
  const [isCancelling, setIsCancelling] = React.useState(false);

  /**
   * 处理取消导入 (Story 2.23)
   */
  const handleCancelImport = React.useCallback(async () => {
    setIsCancelling(true);
    try {
      await cancelImport();
      // 取消事件会通过 onCancelled 回调处理跳转
    } catch (err) {
      console.error("取消导入失败:", err);
    }
    // 不在这里重置 isCancelling，让 onCancelled 回调处理
  }, []);

  /**
   * 开始导入 (使用进度事件)
   */
  const handleStartImport = React.useCallback(async () => {
    const pathsToImport = Array.from(selectedFiles);
    if (pathsToImport.length === 0) return;

    setStep("progress");
    setLoading(true);
    setRecentFiles([]);

    try {
      // 初始化进度
      const initialProgress: ImportProgressData = {
        current: 0,
        total: pathsToImport.length,
        currentFile: "",
        successCount: 0,
        failureCount: 0,
      };
      setProgress(initialProgress);

      // Story 2.28: 记录导入开始日志
      appLog.importStart(source || "unknown", pathsToImport.length);

      // Story 2.29 V2: 不再跳过空会话，全部导入并标记 is_empty
      const parseResults = await importSessionsWithProgress(pathsToImport, {
        onProgress: (event) => {
          setProgress({
            current: event.current,
            total: event.total,
            currentFile: event.currentFile,
            successCount: event.successCount,
            failureCount: event.failureCount,
          });
        },
        onFileDone: (event) => {
          // 更新最近处理的文件列表（最多保留 5 个）
          setRecentFiles((prev) => {
            const newFile: RecentFile = {
              path: event.filePath,
              success: event.success,
              error: event.error,
            };
            const updated = [newFile, ...prev].slice(0, 5);
            return updated;
          });

          // Story 2.23: 收集导入成功的项目信息
          if (event.success && event.projectId && event.sessionId && event.projectName) {
            addImportedProject(event.projectId, event.sessionId, event.projectName);
            // Story 2.28: 记录导入成功日志
            appLog.importFileSuccess(event.filePath, event.projectName);
          } else if (!event.success && event.error) {
            // Story 2.28: 记录导入失败日志
            appLog.importFileError(event.filePath, event.error);
          }
        },
        onCancelled: () => {
          // 导入被取消，重置状态并跳转到完成页
          setIsCancelling(false);
          setStep("complete");
        },
      }, false); // Story 2.29: 不再跳过空会话

      // 处理结果
      for (const result of parseResults) {
        addResult(result);
        if (!result.success && result.error) {
          addError({
            filePath: result.filePath,
            error: result.error,
            message: result.error,
          });
        }
      }

      // 更新最终进度
      const finalProgress: ImportProgressData = {
        current: pathsToImport.length,
        total: pathsToImport.length,
        currentFile: "",
        successCount: parseResults.filter((r) => r.success).length,
        failureCount: parseResults.filter((r) => !r.success).length,
      };
      setProgress(finalProgress);

      // Story 2.28: 记录导入完成日志
      appLog.importComplete(finalProgress.successCount, finalProgress.failureCount);

      // Story 2.29 V2, Story 2.34: 获取导入项目的 is_empty 状态和第一个非空会话ID
      // 在后台更新，不阻塞 UI
      const projectIds = Array.from(
        new Set(parseResults.filter(r => r.success && r.projectId).map(r => r.projectId!))
      );
      Promise.all(projectIds.map(async (id) => {
        const [project, sessions] = await Promise.all([
          getProject(id),
          getProjectSessions(id),
        ]);
        return { project, sessions };
      }))
        .then(results => {
          const projectIsEmptyMap: Record<string, boolean> = {};
          const firstNonEmptySessionMap: Record<string, string> = {};
          for (const { project, sessions } of results) {
            if (project) {
              projectIsEmptyMap[project.id] = project.is_empty ?? false;
              // Story 2.34: 找到第一个非空会话（按更新时间降序，已排好序）
              const firstNonEmpty = sessions.find(s => !s.is_empty);
              if (firstNonEmpty) {
                firstNonEmptySessionMap[project.id] = firstNonEmpty.id;
              }
            }
          }
          updateImportedProjectsIsEmpty(projectIsEmptyMap, firstNonEmptySessionMap);
        })
        .catch(err => {
          console.error("Failed to fetch project is_empty status:", err);
        });

      // 跳转到完成步骤
      setStep("complete");
    } catch (err) {
      console.error("导入失败:", err);
    } finally {
      setLoading(false);
      setIsCancelling(false);
    }
  }, [selectedFiles, setStep, setLoading, setProgress, addResult, addError, updateImportedProjectsIsEmpty]);

  /**
   * 处理下一步
   */
  const handleNext = React.useCallback(() => {
    if (currentStep === "source" && source) {
      setStep("files");
    } else if (currentStep === "files" && selectedFiles.size > 0) {
      handleStartImport();
    }
  }, [currentStep, source, selectedFiles.size, setStep, handleStartImport]);

  /**
   * 处理上一步
   */
  const handleBack = React.useCallback(() => {
    const currentIndex = STEPS.findIndex((s) => s.id === currentStep);
    if (currentIndex > 0) {
      setStep(STEPS[currentIndex - 1].id);
    }
  }, [currentStep, setStep]);

  /**
   * 查看项目
   */
  const handleViewProjects = React.useCallback(() => {
    onOpenChange(false);
    onComplete?.();
  }, [onOpenChange, onComplete]);

  /**
   * 继续导入
   */
  const handleContinueImport = React.useCallback(() => {
    reset();
    setStep("source");
  }, [reset, setStep]);

  /**
   * Story 2.23: 导航到项目的第一个会话
   */
  const handleNavigateToProject = React.useCallback((sessionId: string) => {
    // 关闭导入向导
    onOpenChange(false);
    // 导航到会话
    navigate(`/session/${sessionId}`);
    // 触发完成回调
    onComplete?.();
  }, [onOpenChange, navigate, onComplete]);

  /**
   * Story 2.23: 重试失败的导入项
   */
  const handleRetryFailed = React.useCallback(async (failedPaths: string[]) => {
    if (failedPaths.length === 0) return;

    setIsRetrying(true);

    try {
      // 使用带进度事件的导入函数重试
      const retryResults = await importSessionsWithProgress(failedPaths, {
        onFileDone: (event) => {
          // 收集重试成功的项目信息
          if (event.success && event.projectId && event.sessionId && event.projectName) {
            addImportedProject(event.projectId, event.sessionId, event.projectName);
          }
        },
      });

      // 合并重试结果到现有结果
      mergeRetryResults(retryResults);

      // 显示重试结果反馈
      const successCount = retryResults.filter((r) => r.success).length;
      const failedCount = retryResults.filter((r) => !r.success).length;
      feedback.retryResult(successCount, failedCount);
    } catch (err) {
      console.error("重试导入失败:", err);
      feedback.error(t("import.retryImport"), (err as Error).message);
    } finally {
      setIsRetrying(false);
    }
  }, [addImportedProject, mergeRetryResults, t]);

  /**
   * 渲染当前步骤内容
   */
  const renderStepContent = () => {
    switch (currentStep) {
      case "source":
        return (
          <SourceSelector value={source} onChange={handleSourceSelect} />
        );
      case "files":
        return (
          <FileSelector
            files={discoveredFiles}
            selectedFiles={selectedFiles}
            expandedProjects={expandedProjects}
            searchQuery={searchQuery}
            onScan={handleScan}
            onSelectFiles={handleSelectFiles}
            onToggleFile={toggleFile}
            onToggleProject={toggleProject}
            onToggleProjectExpand={toggleProjectExpand}
            onSearchChange={setSearchQuery}
            loading={isLoading}
            importedSessionIds={importedSessionIds}
          />
        );
      case "progress":
        return (
          <ImportProgress
            progress={
              progress || {
                current: 0,
                total: 0,
                currentFile: "",
                successCount: 0,
                failureCount: 0,
              }
            }
            errors={errors}
            recentFiles={recentFiles}
            onCancel={handleCancelImport}
            isCancelling={isCancelling}
          />
        );
      case "complete":
        return (
          <ImportComplete
            results={results}
            importedProjects={importedProjects}
            onViewProjects={handleViewProjects}
            onContinueImport={handleContinueImport}
            onNavigateToProject={handleNavigateToProject}
            onRetryFailed={handleRetryFailed}
            isRetrying={isRetrying}
          />
        );
    }
  };

  const isFirstStep = currentStep === "source";
  const isProgressStep = currentStep === "progress";
  const isCompleteStep = currentStep === "complete";
  const canProceed =
    (currentStep === "source" && source !== null) ||
    (currentStep === "files" && selectedFiles.size > 0);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        data-testid="import-wizard"
        className="sm:max-w-[600px] max-h-[80vh] flex flex-col"
        aria-labelledby="import-wizard-title"
      >
        {/* Header */}
        <DialogHeader>
          <DialogTitle id="import-wizard-title">{t("import.title")}</DialogTitle>
          <DialogDescription className="sr-only">
            {t("import.description")}
          </DialogDescription>
        </DialogHeader>

        {/* 步骤指示器 */}
        <div className="flex items-center gap-4 py-4 border-b border-border">
          {STEPS.map((step, index) => (
            <React.Fragment key={step.id}>
              <StepIndicator
                step={step}
                state={getStepState(step.id, currentStep)}
                t={t}
              />
              {index < STEPS.length - 1 && (
                <div className="flex-1 h-px bg-border" />
              )}
            </React.Fragment>
          ))}
        </div>

        {/* 内容区域 */}
        <div className="flex-1 overflow-hidden py-4 min-h-0">{renderStepContent()}</div>

        {/* Footer */}
        {!isCompleteStep && (
          <div className="flex justify-between pt-4 border-t border-border">
            {/* 左侧：返回按钮或批量操作按钮 */}
            {currentStep === "files" ? (
              // 文件选择步骤：显示批量操作按钮
              <div className="flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={selectAllNew}
                  className="text-xs h-7 px-2"
                  data-testid="select-all-new-button"
                >
                  {t("import.selectAllNew")}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={clearAll}
                  disabled={selectedFiles.size === 0}
                  className="text-xs h-7 px-2"
                >
                  {t("import.clearSelection")}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={invertSelection}
                  disabled={discoveredFiles.length === 0}
                  className="text-xs h-7 px-2"
                >
                  {t("import.invertSelection")}
                </Button>
              </div>
            ) : !isFirstStep && !isProgressStep ? (
              // 其他步骤：显示返回按钮
              <Button
                variant="outline"
                onClick={handleBack}
                data-testid="back-button"
              >
                {t("common.back")}
              </Button>
            ) : (
              <div />
            )}

            {/* 右侧：导航按钮 */}
            <div className="flex items-center gap-2">
              {/* 文件选择步骤：显示返回按钮 */}
              {currentStep === "files" && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleBack}
                  data-testid="back-button"
                >
                  {t("common.back")}
                </Button>
              )}
              {/* 下一步/开始导入按钮 */}
              {!isProgressStep && (
                <Button
                  onClick={handleNext}
                  disabled={!canProceed || isLoading}
                  data-testid="next-button"
                >
                  {currentStep === "files" ? t("import.startImport") : t("common.next")}
                </Button>
              )}
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
