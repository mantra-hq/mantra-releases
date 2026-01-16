/**
 * ImportComplete Component - 导入完成确认
 * Story 2.9: Task 5
 * Story 2.23: Quick Navigation to Imported Projects
 * Story 2.26: 国际化支持
 * Story 2.29 V2: Empty Project Warning
 *
 * 显示导入完成信息：
 * - 导入统计
 * - 刚导入的项目列表（可快速跳转）
 * - 空项目提示（Story 2.29 V2）
 * - 查看项目按钮
 * - 继续导入按钮
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2, AlertTriangle, FolderKanban, FileCheck, FileX, ChevronRight, MessageSquare, RefreshCw, ChevronDown, Loader2, Info, Plus, GitMerge } from "lucide-react";
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
  /** Whether skipped (e.g. empty) */
  skipped?: boolean;
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
  const successCount = results.filter((r) => r.success && !r.skipped).length;
  const failureCount = results.filter((r) => !r.success).length;
  const skippedCount = results.filter((r) => r.skipped).length;
  const projectIds = new Set(
    results.filter((r) => r.success && r.projectId && !r.skipped).map((r) => r.projectId)
  );
  const projectCount = projectIds.size;

  // Story 2.29 V2: 计算空项目
  const emptyProjects = importedProjects.filter((p) => p.isEmpty);
  const hasEmptyProjects = emptyProjects.length > 0;

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
    <div data-testid="import-complete" className="flex flex-col h-full">
      {/* 可滚动的内容区域 */}
      <div className="flex-1 overflow-y-auto space-y-6 text-center min-h-0">
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
          {skippedCount > 0 && (
            <StatCard
              testId="skipped-stat"
              icon={FileCheck} // Recycled icon, or maybe AlertCircle? Using FileCheck for now but with gray/muted color
              value={skippedCount}
              label={t("import.skippedEmpty")}
              colorClass="text-muted-foreground"
            />
          )}
        </div>

        {/* Story 2.23: 刚导入的项目列表 - 优先展示成功项目 */}
        {importedProjects.length > 0 && onNavigateToProject && (
          <div className="text-left">
            <h4 className="text-sm font-medium text-muted-foreground mb-2">
              {t("import.justImported")}
            </h4>
            <ScrollArea className="max-h-[180px] overflow-hidden">
              <div className="space-y-1.5">
                {importedProjects.map((project) => (
                  <button
                    key={project.id}
                    onClick={() => onNavigateToProject(project.firstNonEmptySessionId ?? project.firstSessionId)}
                    className={cn(
                      "w-full px-3 py-2.5 rounded-md",
                      "bg-muted/50 hover:bg-muted transition-colors cursor-pointer",
                      "text-left group"
                    )}
                    data-testid={`project-${project.id}`}
                  >
                    {/* 第一行：项目图标 + 项目名 + 会话数 + 箭头 */}
                    <div className="flex items-center gap-2">
                      <FolderKanban className="w-4 h-4 text-primary flex-shrink-0" />
                      <span className="text-sm text-foreground truncate flex-1">
                        {project.name}
                      </span>
                      <div className="flex items-center gap-2 text-muted-foreground flex-shrink-0">
                        <div className="flex items-center gap-1">
                          <MessageSquare className="w-3 h-3" />
                          <span className="text-xs">{project.sessionCount}</span>
                        </div>
                        <ChevronRight className="w-4 h-4 opacity-0 group-hover:opacity-100 transition-opacity" />
                      </div>
                    </div>
                    {/* 第二行：新建/合并状态提示 */}
                    <div className="flex items-center gap-1.5 mt-1.5 ml-6 text-xs">
                      {project.isNewProject ? (
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
                    </div>
                  </button>
                ))}
              </div>
            </ScrollArea>
          </div>
        )}

        {/* Story 2.29 V2: 空项目警告 - 优化：添加高度限制和滚动 */}
        {hasEmptyProjects && (
          <div
            className="text-left p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/30"
            data-testid="empty-projects-warning"
          >
            <div className="flex items-start gap-2">
              <Info className="w-4 h-4 text-yellow-500 mt-0.5 flex-shrink-0" />
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium text-yellow-500">
                  {t("import.emptyProjectsWarning")}
                </p>
                <p className="text-xs text-muted-foreground mt-1">
                  {t("import.emptyProjectsWillBeHidden")}
                </p>
                <ScrollArea className="max-h-[100px] overflow-hidden mt-2">
                  <div className="space-y-1 pr-2">
                    {emptyProjects.map((project) => (
                      <div
                        key={project.id}
                        className="flex items-center gap-2 text-xs text-muted-foreground"
                      >
                        <FolderKanban className="w-3 h-3 text-yellow-500/70 flex-shrink-0" />
                        <span className="truncate">{project.name}</span>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              </div>
            </div>
          </div>
        )}

        {/* Story 2.23: 失败文件列表和重试按钮 - 优化：稳健布局 + 滚动提示 */}
        {hasFailures && (
          <div className="text-left rounded-lg border border-red-500/20 bg-red-500/5 overflow-hidden">
            {/* 可折叠的标题栏 */}
            <button
              onClick={() => setErrorsExpanded(!errorsExpanded)}
              className={cn(
                "w-full flex items-center justify-between px-3 py-2.5",
                "text-sm font-medium text-red-500 hover:bg-red-500/10 transition-colors",
                errorsExpanded && "border-b border-red-500/20"
              )}
              data-testid="toggle-errors"
            >
              <div className="flex items-center gap-2">
                <FileX className="w-4 h-4" />
                <span>{t("import.failedFiles")} ({failureCount})</span>
              </div>
              <ChevronDown
                className={cn(
                  "w-4 h-4 transition-transform duration-200",
                  !errorsExpanded && "-rotate-90"
                )}
              />
            </button>

            {/* 展开的内容区域 - 关键：overflow-hidden 确保不会撑破布局 */}
            {errorsExpanded && (
              <div className="relative">
                <ScrollArea className="max-h-[200px] overflow-hidden">
                  <div className="p-2 space-y-1.5">
                    {failedResults.map((result) => (
                      <div
                        key={result.filePath}
                        className="px-3 py-2 rounded-md bg-background/50 border border-red-500/10 group"
                      >
                        <div
                          className="text-sm font-mono text-foreground truncate"
                          title={result.filePath}
                        >
                          {getFileName(result.filePath)}
                        </div>
                        {result.error && (
                          <div
                            className="text-xs text-red-400 mt-1 line-clamp-2"
                            title={result.error}
                          >
                            {result.error}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </ScrollArea>
                {/* 底部渐变遮罩 - 提示用户可以滚动 */}
                {failureCount > 3 && (
                  <div className="absolute bottom-0 left-0 right-2 h-6 bg-gradient-to-t from-red-500/10 to-transparent pointer-events-none" />
                )}
              </div>
            )}

            {/* 重试按钮 */}
            {onRetryFailed && (
              <div className={cn("p-2", errorsExpanded && "border-t border-red-500/20")}>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleRetry}
                  disabled={isRetrying}
                  className="w-full gap-2 text-red-500 hover:text-red-400 border-red-500/30 hover:border-red-500/50 hover:bg-red-500/10"
                  data-testid="retry-failed-button"
                >
                  {isRetrying ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <RefreshCw className="w-4 h-4" />
                  )}
                  {isRetrying ? t("common.retrying") : `${t("import.retryFailed")} (${failureCount})`}
                </Button>
              </div>
            )}
          </div>
        )}
      </div>

      {/* 操作按钮 - 固定在底部 */}
      <div className="flex justify-center gap-3 pt-4 border-t border-border mt-4 flex-shrink-0">
        <Button variant="outline" onClick={onContinueImport}>
          {t("import.continueImport")}
        </Button>
        <Button onClick={onViewProjects}>{t("import.viewProject")}</Button>
      </div>
    </div>
  );
}
