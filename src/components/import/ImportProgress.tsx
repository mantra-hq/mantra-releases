/**
 * ImportProgress Component - 导入进度展示
 * Story 2.9: Task 4
 * Story 2.23: Real-time Progress Events + Cancel Support
 * Story 2.26: 国际化支持
 *
 * 显示导入进度信息：
 * - 总体进度条
 * - 当前处理文件
 * - 成功/失败计数
 * - 最近处理的文件列表
 * - 错误文件列表
 * - 取消导入按钮
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2, XCircle, ChevronDown, ChevronRight, FileWarning, Loader2, StopCircle, Plus, GitMerge, FolderKanban } from "lucide-react";
import {
  Progress,
  Button,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui";
import { cn } from "@/lib/utils";

/** 导入进度数据 */
export interface ImportProgressData {
  /** 当前处理数 */
  current: number;
  /** 总数 */
  total: number;
  /** 当前处理文件名 */
  currentFile: string;
  /** 成功计数 */
  successCount: number;
  /** 失败计数 */
  failureCount: number;
}

/** 导入错误信息 */
export interface ImportError {
  /** 文件路径 */
  filePath: string;
  /** 错误代码 */
  error: string;
  /** 错误消息 */
  message: string;
}

/** 最近处理的文件 (Story 2.23) */
export interface RecentFile {
  /** 文件路径 */
  path: string;
  /** 是否成功 */
  success: boolean;
  /** 错误信息 */
  error?: string;
  /** 项目名称 */
  projectName?: string;
  /** 是否新建项目 */
  isNewProject?: boolean;
}

/** ImportProgress Props */
export interface ImportProgressProps {
  /** 进度数据 */
  progress: ImportProgressData;
  /** 错误列表 */
  errors: ImportError[];
  /** 最近处理的文件列表 (Story 2.23) */
  recentFiles?: RecentFile[];
  /** 取消导入回调 (Story 2.23) */
  onCancel?: () => void;
  /** 是否正在取消 (Story 2.23) */
  isCancelling?: boolean;
}

/**
 * 从文件路径获取文件名
 */
function getFileName(filePath: string): string {
  const parts = filePath.split("/");
  return parts[parts.length - 1] || filePath;
}

/**
 * ImportProgress 组件
 * 显示导入进度
 */
export function ImportProgress({
  progress,
  errors,
  recentFiles = [],
  onCancel,
  isCancelling = false,
}: ImportProgressProps) {
  const { t } = useTranslation();
  const [errorsExpanded, setErrorsExpanded] = React.useState(true);
  const [showCancelConfirm, setShowCancelConfirm] = React.useState(false);

  const percentage = progress.total > 0
    ? Math.round((progress.current / progress.total) * 100)
    : 0;

  const isComplete = progress.current >= progress.total;

  const handleCancelClick = () => {
    setShowCancelConfirm(true);
  };

  const handleCancelConfirm = () => {
    setShowCancelConfirm(false);
    onCancel?.();
  };

  return (
    <div data-testid="import-progress" className="space-y-6">
      {/* 进度信息 */}
      <div className="text-center">
        <div className="text-3xl font-bold text-foreground mb-2">
          {progress.current} / {progress.total}
        </div>
        <div className="text-sm text-muted-foreground">
          {t("import.processing")}: <span className="font-mono">{progress.currentFile || t("common.waiting")}</span>
        </div>
      </div>

      {/* 进度条 */}
      <div className="space-y-2">
        <Progress
          value={percentage}
          className="h-2"
          role="progressbar"
          aria-valuenow={percentage}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={t("import.progressLabel", { percent: percentage })}
        />
        <div className="text-xs text-center text-muted-foreground">
          {percentage}%
        </div>
      </div>

      {/* 统计信息 */}
      <div className="flex justify-center gap-8">
        {/* 成功 */}
        <div className="flex items-center gap-2">
          <CheckCircle2 className="w-5 h-5 text-emerald-500" />
          <div className="text-center">
            <div
              data-testid="success-count"
              className="text-xl font-semibold text-emerald-500"
            >
              {progress.successCount}
            </div>
            <div className="text-xs text-muted-foreground">{t("common.success")}</div>
          </div>
        </div>

        {/* 失败 */}
        <div className="flex items-center gap-2">
          <XCircle className="w-5 h-5 text-red-500" />
          <div className="text-center">
            <div
              data-testid="failure-count"
              className="text-xl font-semibold text-red-500"
            >
              {progress.failureCount}
            </div>
            <div className="text-xs text-muted-foreground">{t("common.failed")}</div>
          </div>
        </div>
      </div>

      {/* Story 2.23: 最近处理的文件列表 */}
      {recentFiles.length > 0 && (
        <div
          data-testid="recent-files"
          className="border border-border rounded-lg overflow-hidden"
        >
          <div className="px-3 py-2 bg-muted/50 text-sm text-muted-foreground">
            {t("import.recentProcessed")}
          </div>
          <div className="divide-y divide-border">
            {recentFiles.map((file, index) => (
              <div
                key={file.path}
                className={cn(
                  "px-3 py-2 text-sm",
                  index === 0 && "bg-muted/30"
                )}
              >
                {/* 第一行：状态图标 + 文件名 */}
                <div className="flex items-center gap-2">
                  {/* 状态图标 */}
                  {index === 0 && progress.current < progress.total ? (
                    <Loader2 className="w-4 h-4 text-primary animate-spin flex-shrink-0" />
                  ) : file.success ? (
                    <CheckCircle2 className="w-4 h-4 text-emerald-500 flex-shrink-0" />
                  ) : (
                    <XCircle className="w-4 h-4 text-red-500 flex-shrink-0" />
                  )}
                  {/* 文件名 */}
                  <span className="font-mono text-foreground truncate flex-1">
                    {getFileName(file.path)}
                  </span>
                </div>
                {/* 第二行：项目信息（成功时显示）*/}
                {file.success && file.projectName && (
                  <div className="flex items-center gap-1.5 mt-1.5 ml-6 text-xs">
                    {file.isNewProject ? (
                      <>
                        <Plus className="w-3 h-3 text-emerald-500" />
                        <span className="text-emerald-500">{t("import.createdNewProject")}</span>
                      </>
                    ) : (
                      <>
                        <GitMerge className="w-3 h-3 text-blue-500" />
                        <span className="text-blue-500">{t("import.mergedToProject")}</span>
                      </>
                    )}
                    <FolderKanban className="w-3 h-3 text-muted-foreground ml-1" />
                    <span className="text-muted-foreground font-medium truncate">
                      {file.projectName}
                    </span>
                  </div>
                )}
                {/* 错误信息 */}
                {file.error && !file.success && (
                  <div className="mt-1 ml-6 text-xs text-red-400 truncate">
                    {file.error}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 错误列表 */}
      {errors.length > 0 && (
        <div
          data-testid="error-list"
          className="border border-red-500/20 rounded-lg overflow-hidden bg-red-500/5"
        >
          {/* 错误列表头部 */}
          <button
            data-testid="error-toggle"
            onClick={() => setErrorsExpanded(!errorsExpanded)}
            className={cn(
              "w-full flex items-center gap-2 px-3 py-2",
              "text-sm text-red-500 hover:bg-red-500/10 transition-colors"
            )}
          >
            {errorsExpanded ? (
              <ChevronDown className="w-4 h-4" />
            ) : (
              <ChevronRight className="w-4 h-4" />
            )}
            <FileWarning className="w-4 h-4" />
            <span>{t("import.parseFailedFiles")} ({errors.length})</span>
          </button>

          {/* 错误列表内容 */}
          {errorsExpanded && (
            <div className="border-t border-red-500/20">
              {errors.map((error, index) => (
                <div
                  key={index}
                  className="px-3 py-2 border-b border-red-500/10 last:border-b-0"
                >
                  <div className="text-sm font-mono text-foreground">
                    {getFileName(error.filePath)}
                  </div>
                  <div className="text-xs text-red-400 mt-0.5">
                    {error.message}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Story 2.23: 取消导入按钮 */}
      {onCancel && !isComplete && (
        <div className="flex justify-center pt-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleCancelClick}
            disabled={isCancelling}
            className="gap-2 text-muted-foreground hover:text-foreground"
            data-testid="cancel-import-button"
          >
            {isCancelling ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <StopCircle className="w-4 h-4" />
            )}
            {isCancelling ? t("import.cancelling") : t("import.cancelImport")}
          </Button>
        </div>
      )}

      {/* 取消确认对话框 */}
      <AlertDialog open={showCancelConfirm} onOpenChange={setShowCancelConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("import.cancelConfirmTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("import.cancelConfirmDesc")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("import.continueImport")}</AlertDialogCancel>
            <AlertDialogAction onClick={handleCancelConfirm}>
              {t("import.confirmCancel")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
