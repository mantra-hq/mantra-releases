/**
 * ImportComplete Component - 导入完成确认
 * Story 2.9: Task 5
 * Story 2.23: Quick Navigation to Imported Projects
 * Story 2.26: 国际化支持
 *
 * 显示导入完成信息：
 * - 导入统计
 * - 刚导入的项目列表（可快速跳转）
 * - 查看项目按钮
 * - 继续导入按钮
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2, AlertTriangle, FolderKanban, FileCheck, FileX, ChevronRight, MessageSquare, RefreshCw, ChevronDown, Loader2 } from "lucide-react";
import { Button, ScrollArea } from "@/components/ui";
import { cn } from "@/lib/utils";
import type { ImportedProject } from "@/stores/useImportStore";

/**
 * 从文件路径获取文件名
 */
function getFileName(filePath: string): string {
  const parts = filePath.split("/");
  return parts[parts.length - 1] || filePath;
}

/** 导入结果 */
export interface ImportResult {
  /** 是否成功 */
  success: boolean;
  /** 文件路径 */
  filePath: string;
  /** 项目 ID (成功时) */
  projectId?: string;
  /** 会话 ID (成功时) */
  sessionId?: string;
  /** 错误信息 (失败时) */
  error?: string;
}

/** ImportComplete Props */
export interface ImportCompleteProps {
  /** 导入结果列表 */
  results: ImportResult[];
  /** 刚导入的项目列表 (Story 2.23) */
  importedProjects?: ImportedProject[];
  /** 查看项目回调 */
  onViewProjects: () => void;
  /** 继续导入回调 */
  onContinueImport: () => void;
  /** 导航到项目回调 (Story 2.23) */
  onNavigateToProject?: (sessionId: string) => void;
  /** 重试失败项回调 (Story 2.23) */
  onRetryFailed?: (failedPaths: string[]) => void;
  /** 是否正在重试 (Story 2.23) */
  isRetrying?: boolean;
}

/**
 * 统计卡片组件
 */
function StatCard({
  testId,
  icon: Icon,
  value,
  label,
  colorClass,
}: {
  testId: string;
  icon: React.ComponentType<{ className?: string }>;
  value: number;
  label: string;
  colorClass: string;
}) {
  return (
    <div className="flex flex-col items-center p-4">
      <Icon className={cn("w-6 h-6 mb-2", colorClass)} />
      <div
        data-testid={testId}
        className={cn("text-2xl font-bold", colorClass)}
      >
        {value}
      </div>
      <div className="text-xs text-muted-foreground">{label}</div>
    </div>
  );
}

/**
 * ImportComplete 组件
 * 导入完成确认页面
 */
export function ImportComplete({
  results,
  importedProjects = [],
  onViewProjects,
  onContinueImport,
  onNavigateToProject,
  onRetryFailed,
  isRetrying = false,
}: ImportCompleteProps) {
  const { t } = useTranslation();
  // Story 2.24: 默认折叠错误列表，避免界面遮挡
  const [errorsExpanded, setErrorsExpanded] = React.useState(false);

  // 计算统计数据
  const successCount = results.filter((r) => r.success).length;
  const failureCount = results.filter((r) => !r.success).length;
  const projectIds = new Set(
    results.filter((r) => r.success && r.projectId).map((r) => r.projectId)
  );
  const projectCount = projectIds.size;

  // 获取失败的文件
  const failedResults = results.filter((r) => !r.success);

  const allSuccess = failureCount === 0;
  const hasFailures = failureCount > 0;

  // 处理重试
  const handleRetry = React.useCallback(() => {
    if (onRetryFailed && failedResults.length > 0) {
      const failedPaths = failedResults.map((r) => r.filePath);
      onRetryFailed(failedPaths);
    }
  }, [onRetryFailed, failedResults]);

  return (
    <div data-testid="import-complete" className="space-y-6 text-center">
      {/* 状态图标 */}
      <div className="flex justify-center">
        {allSuccess ? (
          <div
            data-testid="success-icon"
            className="w-16 h-16 rounded-full bg-emerald-500/10 flex items-center justify-center"
          >
            <CheckCircle2 className="w-8 h-8 text-emerald-500" />
          </div>
        ) : (
          <div className="w-16 h-16 rounded-full bg-yellow-500/10 flex items-center justify-center">
            <AlertTriangle className="w-8 h-8 text-yellow-500" />
          </div>
        )}
      </div>

      {/* 标题 */}
      <div>
        <h3 className="text-xl font-semibold text-foreground">{t("import.importComplete")}</h3>
        {hasFailures && (
          <p className="text-sm text-muted-foreground mt-1">
            {t("import.partialFailure")}
          </p>
        )}
      </div>

      {/* 统计数据 */}
      <div className="flex justify-center gap-2 border border-border rounded-lg divide-x divide-border">
        <StatCard
          testId="success-stat"
          icon={FileCheck}
          value={successCount}
          label={t("import.importSuccess")}
          colorClass="text-emerald-500"
        />
        <StatCard
          testId="failure-stat"
          icon={FileX}
          value={failureCount}
          label={t("import.importFailed")}
          colorClass="text-red-500"
        />
        <StatCard
          testId="project-stat"
          icon={FolderKanban}
          value={projectCount}
          label={t("import.project")}
          colorClass="text-primary"
        />
      </div>

      {/* Story 2.23: 刚导入的项目列表 - 优先展示成功项目 */}
      {importedProjects.length > 0 && onNavigateToProject && (
        <div className="text-left">
          <h4 className="text-sm font-medium text-muted-foreground mb-2">
            {t("import.justImported")}
          </h4>
          <ScrollArea className="max-h-[200px]">
            <div className="space-y-1">
              {importedProjects.map((project) => (
                <button
                  key={project.id}
                  onClick={() => onNavigateToProject(project.firstSessionId)}
                  className={cn(
                    "w-full flex items-center justify-between px-3 py-2 rounded-md",
                    "bg-muted/50 hover:bg-muted transition-colors cursor-pointer",
                    "text-left group"
                  )}
                  data-testid={`project-${project.id}`}
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <FolderKanban className="w-4 h-4 text-primary flex-shrink-0" />
                    <span className="text-sm text-foreground truncate">
                      {project.name}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 text-muted-foreground">
                    <div className="flex items-center gap-1">
                      <MessageSquare className="w-3 h-3" />
                      <span className="text-xs">{project.sessionCount}</span>
                    </div>
                    <ChevronRight className="w-4 h-4 opacity-0 group-hover:opacity-100 transition-opacity" />
                  </div>
                </button>
              ))}
            </div>
          </ScrollArea>
        </div>
      )}

      {/* Story 2.23: 失败文件列表和重试按钮 - 次要信息放在后面 */}
      {hasFailures && (
        <div className="text-left">
          <button
            onClick={() => setErrorsExpanded(!errorsExpanded)}
            className="w-full flex items-center justify-between text-sm font-medium text-red-500 hover:text-red-400 transition-colors mb-2"
            data-testid="toggle-errors"
          >
            <span>{t("import.failedFiles")} ({failureCount})</span>
            <ChevronDown
              className={cn(
                "w-4 h-4 transition-transform",
                !errorsExpanded && "-rotate-90"
              )}
            />
          </button>

          {errorsExpanded && (
            <ScrollArea className="max-h-[150px] mb-3">
              <div className="space-y-1">
                {failedResults.map((result) => (
                  <div
                    key={result.filePath}
                    className="px-3 py-2 rounded-md bg-red-500/5 border border-red-500/20"
                  >
                    <div className="text-sm font-mono text-foreground truncate">
                      {getFileName(result.filePath)}
                    </div>
                    {result.error && (
                      <div className="text-xs text-red-400 mt-0.5 truncate">
                        {result.error}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </ScrollArea>
          )}

          {onRetryFailed && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleRetry}
              disabled={isRetrying}
              className="w-full gap-2 text-red-500 hover:text-red-400 border-red-500/30 hover:border-red-500/50"
              data-testid="retry-failed-button"
            >
              {isRetrying ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <RefreshCw className="w-4 h-4" />
              )}
              {isRetrying ? t("common.retrying") : `${t("import.retryFailed")} (${failureCount})`}
            </Button>
          )}
        </div>
      )}

      {/* 操作按钮 */}
      <div className="flex justify-center gap-3">
        <Button variant="outline" onClick={onContinueImport}>
          {t("import.continueImport")}
        </Button>
        <Button onClick={onViewProjects}>{t("import.viewProject")}</Button>
      </div>
    </div>
  );
}
