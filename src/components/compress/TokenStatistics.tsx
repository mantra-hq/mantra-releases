/**
 * TokenStatistics - Token 统计栏组件
 * Story 10.6: Task 1
 *
 * 显示压缩模式下的 Token 统计信息
 * 包括原始/压缩后 Token 数、节省量、变更统计
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import {
  ArrowRight,
  Minus,
  Pencil,
  Plus,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { NarrativeMessage } from "@/types/message";
import { estimateTokenCount, formatTokenCount } from "@/lib/token-counter";
import { getMessageDisplayContent } from "@/lib/message-utils";
import { useCompressState, type ChangeStats } from "@/hooks/useCompressState";
import { TokenCompareBar } from "./TokenCompareBar";

/**
 * TokenStatistics 组件 Props
 */
export interface TokenStatisticsProps {
  /** 原始消息列表 */
  messages: NarrativeMessage[];
  /** 自定义 className */
  className?: string;
}

/**
 * Token 统计数据类型
 */
export interface TokenStats {
  /** 原始 Token 总数 */
  originalTotal: number;
  /** 压缩后 Token 总数 */
  compressedTotal: number;
  /** 节省的 Token 数 */
  savedTokens: number;
  /** 节省百分比 (0-100) */
  savedPercentage: number;
  /** 变更统计 */
  changeStats: ChangeStats;
}

/**
 * 计算原始消息的 Token 总数
 */
function calculateOriginalTokens(messages: NarrativeMessage[]): number {
  return messages.reduce((total, message) => {
    const textContent = getMessageDisplayContent(message.content);
    return total + estimateTokenCount(textContent);
  }, 0);
}

/**
 * TokenStatistics - Token 统计栏
 *
 * AC1: 底部统计栏，显示原始/压缩后 Token 数、节省量、操作计数
 * AC2: 可视化对比条
 * AC3: 实时更新 (使用 useMemo 缓存计算结果)
 * AC4: 使用现有 estimateTokenCount 算法
 */
export function TokenStatistics({
  messages,
  className,
}: TokenStatisticsProps) {
  const { t } = useTranslation();
  const { operations, insertions, getChangeStats } = useCompressState();

  // AC4: 使用 useMemo 缓存 Token 统计计算
  const stats = React.useMemo<TokenStats>(() => {
    // 计算原始 Token 总数
    const originalTotal = calculateOriginalTokens(messages);

    // 计算压缩后 Token 总数
    let compressedTotal = 0;

    // 遍历原始消息，考虑删除/修改操作
    messages.forEach((message) => {
      const operation = operations.get(message.id);

      if (!operation || operation.type === "keep") {
        // 保留: 计入原始 token
        const textContent = getMessageDisplayContent(message.content);
        compressedTotal += estimateTokenCount(textContent);
      } else if (operation.type === "modify" && operation.modifiedContent) {
        // 修改: 计入修改后的 token
        compressedTotal += estimateTokenCount(operation.modifiedContent);
      }
      // delete: 不计入
    });

    // 添加插入的消息 token
    insertions.forEach((insertion) => {
      if (insertion.insertedMessage) {
        const textContent = getMessageDisplayContent(insertion.insertedMessage.content);
        compressedTotal += estimateTokenCount(textContent);
      }
    });

    // 计算节省量
    const savedTokens = originalTotal - compressedTotal;
    const savedPercentage = originalTotal > 0
      ? (savedTokens / originalTotal) * 100
      : 0;

    // 获取变更统计
    const changeStats = getChangeStats();

    return {
      originalTotal,
      compressedTotal,
      savedTokens,
      savedPercentage,
      changeStats,
    };
  }, [messages, operations, insertions, getChangeStats]);

  return (
    <div
      className={cn(
        "flex items-center justify-between gap-4 px-4 py-3",
        "border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60",
        className
      )}
      data-testid="token-statistics"
    >
      {/* 左侧: 统计数字 */}
      <div className="flex items-center gap-4 sm:gap-6">
        {/* 原始 Token */}
        <div className="flex flex-col">
          <span className="text-xs text-muted-foreground">
            {t("compress.tokenStats.original")}
          </span>
          <span className="text-sm font-medium tabular-nums transition-all duration-150">
            {formatTokenCount(stats.originalTotal)}
          </span>
        </div>

        {/* 箭头分隔 */}
        <ArrowRight className="size-4 text-muted-foreground shrink-0" />

        {/* 压缩后 Token */}
        <div className="flex flex-col">
          <span className="text-xs text-muted-foreground">
            {t("compress.tokenStats.compressed")}
          </span>
          <span className="text-sm font-medium tabular-nums text-primary transition-all duration-150">
            {formatTokenCount(stats.compressedTotal)}
          </span>
        </div>

        {/* 节省统计 */}
        <div className="flex flex-col">
          <span className="text-xs text-muted-foreground">
            {t("compress.tokenStats.saved")}
          </span>
          <span className={cn(
            "text-sm font-medium tabular-nums transition-all duration-150",
            stats.savedTokens > 0 ? "text-green-500" : "text-muted-foreground"
          )}>
            {stats.savedTokens > 0 ? `-${formatTokenCount(stats.savedTokens)}` : "0"}
            {stats.savedPercentage > 0 && (
              <span className="ml-1 text-xs">({Math.round(stats.savedPercentage)}%)</span>
            )}
          </span>
        </div>
      </div>

      {/* 中间: 对比条 (隐藏在小屏幕) */}
      <TokenCompareBar
        originalTokens={stats.originalTotal}
        compressedTokens={stats.compressedTotal}
        className="flex-1 max-w-xs hidden md:block"
      />

      {/* 右侧: 操作计数 */}
      <div className="flex items-center gap-3 text-xs text-muted-foreground">
        <span className="flex items-center gap-1" title={t("compress.tokenStats.deleted")}>
          <Minus className="size-3 text-red-500" />
          <span className="tabular-nums">{stats.changeStats.deleted}</span>
        </span>
        <span className="flex items-center gap-1" title={t("compress.tokenStats.modified")}>
          <Pencil className="size-3 text-yellow-500" />
          <span className="tabular-nums">{stats.changeStats.modified}</span>
        </span>
        <span className="flex items-center gap-1" title={t("compress.tokenStats.inserted")}>
          <Plus className="size-3 text-green-500" />
          <span className="tabular-nums">{stats.changeStats.inserted}</span>
        </span>
      </div>
    </div>
  );
}

export default TokenStatistics;
