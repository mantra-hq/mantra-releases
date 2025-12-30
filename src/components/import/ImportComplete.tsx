/**
 * ImportComplete Component - 导入完成确认
 * Story 2.9: Task 5
 *
 * 显示导入完成信息：
 * - 导入统计
 * - 查看项目按钮
 * - 继续导入按钮
 */

import * as React from "react";
import { CheckCircle2, AlertTriangle, FolderKanban, FileCheck, FileX } from "lucide-react";
import { Button } from "@/components/ui";
import { cn } from "@/lib/utils";

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
  /** 查看项目回调 */
  onViewProjects: () => void;
  /** 继续导入回调 */
  onContinueImport: () => void;
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
  onViewProjects,
  onContinueImport,
}: ImportCompleteProps) {
  // 计算统计数据
  const successCount = results.filter((r) => r.success).length;
  const failureCount = results.filter((r) => !r.success).length;
  const projectIds = new Set(
    results.filter((r) => r.success && r.projectId).map((r) => r.projectId)
  );
  const projectCount = projectIds.size;

  const allSuccess = failureCount === 0;
  const hasFailures = failureCount > 0;

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
        <h3 className="text-xl font-semibold text-foreground">导入完成</h3>
        {hasFailures && (
          <p className="text-sm text-muted-foreground mt-1">
            部分文件导入失败，请检查错误信息
          </p>
        )}
      </div>

      {/* 统计数据 */}
      <div className="flex justify-center gap-2 border border-border rounded-lg divide-x divide-border">
        <StatCard
          testId="success-stat"
          icon={FileCheck}
          value={successCount}
          label="成功导入"
          colorClass="text-emerald-500"
        />
        <StatCard
          testId="failure-stat"
          icon={FileX}
          value={failureCount}
          label="导入失败"
          colorClass="text-red-500"
        />
        <StatCard
          testId="project-stat"
          icon={FolderKanban}
          value={projectCount}
          label="项目"
          colorClass="text-primary"
        />
      </div>

      {/* 操作按钮 */}
      <div className="flex justify-center gap-3">
        <Button variant="outline" onClick={onContinueImport}>
          继续导入
        </Button>
        <Button onClick={onViewProjects}>查看项目</Button>
      </div>
    </div>
  );
}
