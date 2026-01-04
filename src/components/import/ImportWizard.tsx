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
import { useImportStore } from "@/stores";
import { scanLogDirectory, selectLogFiles, importSessionsWithProgress, cancelImport } from "@/lib/import-ipc";
import { getImportedProjectPaths } from "@/lib/project-ipc";
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

/** 步骤配置列表 */
const STEPS: StepConfig[] = [
  { id: "source", label: "选择来源", number: 1 },
  { id: "files", label: "选择文件", number: 2 },
  { id: "progress", label: "导入中", number: 3 },
  { id: "complete", label: "完成", number: 4 },
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
}: {
  step: StepConfig;
  state: "pending" | "active" | "completed";
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
        {step.label}
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
    importedPaths,
    importedProjects,
    setStep,
    setSource,
    setDiscoveredFiles,
    toggleFile,
    selectAll,
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
    setImportedPaths,
    selectAllNew,
    addImportedProject,
    mergeRetryResults,
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

  // Story 2.20: 加载已导入项目路径
  React.useEffect(() => {
    if (open) {
      getImportedProjectPaths()
        .then((paths) => {
          setImportedPaths(paths);
        })
        .catch((err) => {
          console.error("Failed to load imported project paths:", err);
        });
    }
  }, [open, setImportedPaths]);

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
    } catch (err) {
      console.error("扫描失败:", err);
    } finally {
      setLoading(false);
    }
  }, [source, setLoading, setDiscoveredFiles]);

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
    const selectedPaths = Array.from(selectedFiles);
    if (selectedPaths.length === 0) return;

    setStep("progress");
    setLoading(true);
    setRecentFiles([]);

    try {
      // 初始化进度
      const initialProgress: ImportProgressData = {
        current: 0,
        total: selectedPaths.length,
        currentFile: "",
        successCount: 0,
        failureCount: 0,
      };
      setProgress(initialProgress);

      // 使用带进度事件的导入函数
      const parseResults = await importSessionsWithProgress(selectedPaths, {
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
          if (event.success && event.projectId && event.sessionId) {
            addImportedProject(event.projectId, event.sessionId, event.filePath);
          }
        },
        onCancelled: () => {
          // 导入被取消，重置状态并跳转到完成页
          setIsCancelling(false);
          setStep("complete");
        },
      });

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
        current: selectedPaths.length,
        total: selectedPaths.length,
        currentFile: "",
        successCount: parseResults.filter((r) => r.success).length,
        failureCount: parseResults.filter((r) => !r.success).length,
      };
      setProgress(finalProgress);

      // 跳转到完成步骤
      setStep("complete");
    } catch (err) {
      console.error("导入失败:", err);
    } finally {
      setLoading(false);
      setIsCancelling(false);
    }
  }, [selectedFiles, setStep, setLoading, setProgress, addResult, addError]);

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
          if (event.success && event.projectId && event.sessionId) {
            addImportedProject(event.projectId, event.sessionId, event.filePath);
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
      feedback.error("重试导入", (err as Error).message);
    } finally {
      setIsRetrying(false);
    }
  }, [addImportedProject, mergeRetryResults]);

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
            onSelectAll={selectAll}
            onClearAll={clearAll}
            onInvertSelection={invertSelection}
            onToggleProject={toggleProject}
            onToggleProjectExpand={toggleProjectExpand}
            onSearchChange={setSearchQuery}
            loading={isLoading}
            importedPaths={importedPaths}
            onSelectAllNew={selectAllNew}
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
        className="sm:max-w-[600px] max-h-[80vh] flex flex-col"
        aria-labelledby="import-wizard-title"
      >
        {/* Header */}
        <DialogHeader>
          <DialogTitle id="import-wizard-title">导入日志</DialogTitle>
          <DialogDescription className="sr-only">
            多步骤导入向导，用于导入 AI 对话日志文件
          </DialogDescription>
        </DialogHeader>

        {/* 步骤指示器 */}
        <div className="flex items-center gap-4 py-4 border-b border-border">
          {STEPS.map((step, index) => (
            <React.Fragment key={step.id}>
              <StepIndicator
                step={step}
                state={getStepState(step.id, currentStep)}
              />
              {index < STEPS.length - 1 && (
                <div className="flex-1 h-px bg-border" />
              )}
            </React.Fragment>
          ))}
        </div>

        {/* 内容区域 */}
        <div className="flex-1 overflow-y-auto py-4">{renderStepContent()}</div>

        {/* Footer */}
        {!isCompleteStep && (
          <div className="flex justify-between pt-4 border-t border-border">
            {/* 返回按钮 */}
            {!isFirstStep && !isProgressStep ? (
              <Button
                variant="outline"
                onClick={handleBack}
                data-testid="back-button"
              >
                返回
              </Button>
            ) : (
              <div />
            )}

            {/* 下一步按钮 */}
            {!isProgressStep && (
              <Button
                onClick={handleNext}
                disabled={!canProceed || isLoading}
                data-testid="next-button"
              >
                {currentStep === "files" ? "开始导入" : "下一步"}
              </Button>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
