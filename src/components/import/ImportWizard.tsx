/**
 * ImportWizard Component - 导入向导 Modal
 * Story 2.9: Task 1, Task 8
 *
 * 多步骤导入向导，包含：
 * - 步骤 1: 选择导入源
 * - 步骤 2: 选择文件
 * - 步骤 3: 导入进度
 * - 步骤 4: 完成确认
 */

import * as React from "react";
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
import { useImportStore } from "@/stores";
import { scanLogDirectory, selectLogFiles, parseLogFiles } from "@/lib/import-ipc";
import { SourceSelector, type ImportSource } from "./SourceSelector";
import { FileSelector } from "./FileSelector";
import { ImportProgress, type ImportProgressData } from "./ImportProgress";
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
    setStep,
    setSource,
    setDiscoveredFiles,
    toggleFile,
    toggleAll,
    toggleProject,
    toggleProjectExpand,
    setSearchQuery,
    setProgress,
    addResult,
    setLoading,
    addError,
    reset,
  } = useImportStore();

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

  /**
   * 开始导入
   */
  const handleStartImport = React.useCallback(async () => {
    const selectedPaths = Array.from(selectedFiles);
    if (selectedPaths.length === 0) return;

    setStep("progress");
    setLoading(true);

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

      // 模拟逐个文件处理进度
      const parseResults = await parseLogFiles(selectedPaths, (prog) => {
        setProgress(prog);
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
            onToggleAll={toggleAll}
            onToggleProject={toggleProject}
            onToggleProjectExpand={toggleProjectExpand}
            onSearchChange={setSearchQuery}
            loading={isLoading}
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
          />
        );
      case "complete":
        return (
          <ImportComplete
            results={results}
            onViewProjects={handleViewProjects}
            onContinueImport={handleContinueImport}
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
