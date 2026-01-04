/**
 * ImportProgress Component - 导入进度展示
 * Story 2.9: Task 4
 * Story 2.23: Real-time Progress Events + Cancel Support
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
import { CheckCircle2, XCircle, ChevronDown, ChevronRight, FileWarning, Loader2, StopCircle } from "lucide-react";
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
          正在处理: <span className="font-mono">{progress.currentFile || "等待中..."}</span>
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
          aria-label={`导入进度 ${percentage}%`}
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
            <div className="text-xs text-muted-foreground">成功</div>
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
            <div className="text-xs text-muted-foreground">失败</div>
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
            最近处理
          </div>
          <div className="divide-y divide-border">
            {recentFiles.map((file, index) => (
              <div
                key={file.path}
                className={cn(
                  "flex items-center gap-2 px-3 py-2 text-sm",
                  index === 0 && "bg-muted/30"
                )}
              >
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
                {/* 错误信息 */}
                {file.error && (
                  <span className="text-xs text-red-400 truncate max-w-[150px]">
                    {file.error}
                  </span>
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
            <span>解析失败的文件 ({errors.length})</span>
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
            {isCancelling ? "正在取消..." : "取消导入"}
          </Button>
        </div>
      )}

      {/* 取消确认对话框 */}
      <AlertDialog open={showCancelConfirm} onOpenChange={setShowCancelConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确定要取消导入吗？</AlertDialogTitle>
            <AlertDialogDescription>
              已导入的内容会保留，后续未处理的文件将不会被导入。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>继续导入</AlertDialogCancel>
            <AlertDialogAction onClick={handleCancelConfirm}>
              确认取消
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
