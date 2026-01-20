/**
 * CompressPreviewList - 压缩预览列表组件
 * Story 10.3: Task 2
 *
 * 右侧预览面板，实时显示压缩操作的结果
 * 使用 @tanstack/react-virtual 实现虚拟化渲染
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Eye, Filter, EyeOff, FileText } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useCompressState, type PreviewMessage } from "@/hooks/useCompressState";
import { PreviewMessageCard } from "./PreviewMessageCard";
import { DeletedPlaceholder } from "./DeletedPlaceholder";
import type { NarrativeMessage } from "@/types/message";

// ===== 预览模式枚举 =====

export enum PreviewMode {
  /** 完整预览 - 显示所有消息 */
  Full = "full",
  /** 仅变更 - 只显示有操作的消息 */
  ChangesOnly = "changes-only",
  /** 隐藏删除 - 隐藏删除占位符 */
  HideDeleted = "hide-deleted",
}

// ===== Props =====

export interface CompressPreviewListProps {
  /** 原始消息列表 */
  messages: NarrativeMessage[];
  /** 自定义 className */
  className?: string;
}

// ===== 估算高度函数 =====

/**
 * 估算预览消息高度
 */
function estimatePreviewSize(previewMessage: PreviewMessage): number {
  // 删除占位符高度固定
  if (previewMessage.operation === "delete") {
    return 56; // 较小的高度
  }

  // 基础高度
  const baseHeight = 72;
  
  // 根据内容估算
  const textContent = previewMessage.message.content
    .filter((block) => block.type === "text")
    .reduce((acc, block) => acc + block.content.length, 0);
  
  const lineEstimate = Math.ceil(textContent / 60);
  const contentHeight = Math.min(lineEstimate * 20, 60);
  
  return Math.min(Math.max(baseHeight + contentHeight, 72), 140);
}

// ===== 空状态组件 =====

function EmptyState() {
  const { t } = useTranslation();
  return (
    <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
      <div className="flex flex-col items-center gap-4 p-8 text-center">
        <div className="rounded-full bg-muted p-4">
          <FileText className="size-8" />
        </div>
        <div className="space-y-2">
          <h3 className="text-lg font-semibold text-foreground">
            {t("compress.previewList.empty")}
          </h3>
          <p className="text-sm max-w-xs">
            {t("compress.previewList.emptyHint")}
          </p>
        </div>
      </div>
    </div>
  );
}

// ===== 主组件 =====

/**
 * CompressPreviewList - 压缩预览列表
 *
 * AC1: 实时预览 (响应时间 < 100ms)
 * AC5: 预览选项 (完整/仅变更/隐藏删除)
 * AC6: 滚动性能 (虚拟化列表)
 */
export function CompressPreviewList({
  messages,
  className,
}: CompressPreviewListProps) {
  const { t } = useTranslation();
  const scrollContainerRef = React.useRef<HTMLDivElement>(null);
  
  // AC5: 预览模式状态
  const [previewMode, setPreviewMode] = React.useState<PreviewMode>(PreviewMode.Full);
  
  // 获取压缩状态
  const { getPreviewMessages } = useCompressState();
  
  // AC1: 计算预览消息列表 (使用 useMemo 优化性能)
  const previewMessages = React.useMemo(() => {
    return getPreviewMessages(messages);
  }, [getPreviewMessages, messages]);
  
  // AC5: 根据预览模式过滤消息
  const filteredMessages = React.useMemo(() => {
    switch (previewMode) {
      case PreviewMode.ChangesOnly:
        return previewMessages.filter((m) => m.operation !== "keep");
      case PreviewMode.HideDeleted:
        return previewMessages.filter((m) => m.operation !== "delete");
      case PreviewMode.Full:
      default:
        return previewMessages;
    }
  }, [previewMessages, previewMode]);
  
  // AC6: 虚拟化配置
  const virtualizer = useVirtualizer({
    count: filteredMessages.length,
    getScrollElement: () => scrollContainerRef.current,
    estimateSize: (index) => estimatePreviewSize(filteredMessages[index]),
    overscan: 5, // 预渲染优化滚动体验
  });
  
  // 空状态: 无变更时的提示
  const hasNoChanges = previewMessages.every((m) => m.operation === "keep");
  
  if (messages.length === 0) {
    return (
      <div className={cn("h-full flex flex-col", className)}>
        <EmptyState />
      </div>
    );
  }
  
  const virtualItems = virtualizer.getVirtualItems();
  
  return (
    <div className={cn("h-full flex flex-col", className)}>
      {/* AC5: 预览模式切换 */}
      <div className="flex items-center justify-between px-3 py-2 border-b">
        <span className="text-sm font-medium text-foreground">
          {t("compress.previewList.title")}
        </span>
        <div className="flex items-center gap-0.5 p-0.5 bg-muted rounded-md">
          <Button
            variant={previewMode === PreviewMode.Full ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setPreviewMode(PreviewMode.Full)}
            aria-label={t("compress.previewList.modeFull")}
            className="h-7 text-xs px-2"
          >
            <Eye className="size-3.5 mr-1" />
            {t("compress.previewList.modeFull")}
          </Button>
          <Button
            variant={previewMode === PreviewMode.ChangesOnly ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setPreviewMode(PreviewMode.ChangesOnly)}
            aria-label={t("compress.previewList.modeChanges")}
            className="h-7 text-xs px-2"
          >
            <Filter className="size-3.5 mr-1" />
            {t("compress.previewList.modeChanges")}
          </Button>
          <Button
            variant={previewMode === PreviewMode.HideDeleted ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setPreviewMode(PreviewMode.HideDeleted)}
            aria-label={t("compress.previewList.modeHideDeleted")}
            className="h-7 text-xs px-2"
          >
            <EyeOff className="size-3.5 mr-1" />
            {t("compress.previewList.modeHideDeleted")}
          </Button>
        </div>
      </div>
      
      {/* 无变更提示 (仅在 ChangesOnly 模式下显示) */}
      {previewMode === PreviewMode.ChangesOnly && hasNoChanges && (
        <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground p-8">
          <FileText className="size-8 mb-4 opacity-50" />
          <p className="text-sm text-center">
            {t("compress.previewList.empty")}
          </p>
          <p className="text-xs text-center mt-1 max-w-xs">
            {t("compress.previewList.emptyHint")}
          </p>
        </div>
      )}
      
      {/* 虚拟化列表 */}
      {!(previewMode === PreviewMode.ChangesOnly && hasNoChanges) && (
        <div
          ref={scrollContainerRef}
          data-testid="compress-preview-list"
          className={cn(
            "flex-1 overflow-y-auto",
            "scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent"
          )}
        >
          <div
            className="relative w-full px-3 py-2"
            style={{
              height: `${virtualizer.getTotalSize()}px`,
            }}
          >
            {virtualItems.map((virtualItem) => {
              const previewMessage = filteredMessages[virtualItem.index];
              
              return (
                <div
                  key={virtualItem.key}
                  data-index={virtualItem.index}
                  className="absolute left-0 top-0 w-full px-3"
                  style={{
                    transform: `translateY(${virtualItem.start}px)`,
                  }}
                >
                  {previewMessage.operation === "delete" ? (
                    <DeletedPlaceholder
                      originalMessage={previewMessage.message}
                      savedTokens={previewMessage.originalTokens ?? 0}
                      measureElement={virtualizer.measureElement}
                      index={virtualItem.index}
                    />
                  ) : (
                    <PreviewMessageCard
                      previewMessage={previewMessage}
                      measureElement={virtualizer.measureElement}
                      index={virtualItem.index}
                    />
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

export default CompressPreviewList;
