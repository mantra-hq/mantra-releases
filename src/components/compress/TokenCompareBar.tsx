/**
 * TokenCompareBar - Token 对比进度条组件
 * Story 10.6: Task 2
 *
 * 双行进度条可视化对比原始和压缩后的 Token 数量
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

/**
 * TokenCompareBar 组件 Props
 */
export interface TokenCompareBarProps {
  /** 原始 Token 数 */
  originalTokens: number;
  /** 压缩后 Token 数 */
  compressedTokens: number;
  /** 自定义 className */
  className?: string;
}

/**
 * TokenCompareBar - Token 对比进度条
 *
 * AC2: 双行进度条对比 (原始 vs 压缩后)
 * AC3: 进度条宽度使用 CSS transition 平滑过渡
 */
export function TokenCompareBar({
  originalTokens,
  compressedTokens,
  className,
}: TokenCompareBarProps) {
  const { t } = useTranslation();

  // 计算压缩百分比
  const percentage = originalTokens > 0
    ? (compressedTokens / originalTokens) * 100
    : 100;

  // 计算节省百分比
  const savedPercentage = originalTokens > 0
    ? ((originalTokens - compressedTokens) / originalTokens) * 100
    : 0;

  return (
    <div className={cn("space-y-1.5", className)} data-testid="token-compare-bar">
      {/* 原始 Token 条 */}
      <div className="flex items-center gap-2">
        <span className="text-xs text-muted-foreground w-14 shrink-0 text-right">
          {t("compress.tokenStats.original")}
        </span>
        <div className="flex-1 h-2 bg-muted/30 rounded-full overflow-hidden">
          <div
            className="h-full bg-muted rounded-full transition-all duration-300"
            style={{ width: "100%" }}
            data-testid="original-bar"
          />
        </div>
        <span className="w-10 shrink-0" />
      </div>

      {/* 压缩后 Token 条 */}
      <div className="flex items-center gap-2">
        <span className="text-xs text-muted-foreground w-14 shrink-0 text-right">
          {t("compress.tokenStats.compressed")}
        </span>
        <div className="flex-1 h-2 bg-muted/30 rounded-full overflow-hidden">
          <div
            className={cn(
              "h-full rounded-full transition-all duration-300",
              compressedTokens < originalTokens ? "bg-primary" : "bg-muted"
            )}
            style={{
              width: `${Math.min(100, percentage)}%`
            }}
            data-testid="compressed-bar"
          />
        </div>
        {/* 百分比标签 */}
        <span className={cn(
          "text-xs font-medium w-10 text-right shrink-0",
          savedPercentage > 0 ? "text-green-500" : "text-muted-foreground"
        )}>
          {savedPercentage > 0 ? `-${Math.round(savedPercentage)}%` : "0%"}
        </span>
      </div>
    </div>
  );
}

export default TokenCompareBar;
