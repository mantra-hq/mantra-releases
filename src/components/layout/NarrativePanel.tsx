/**
 * NarrativePanel - 对话流面板
 * Story 2.2: Task 4.1 (基础)
 * Story 2.3: Task 5 (集成 NarrativeStream)
 * Story 2.15: 集成 ToolPairingProvider (配对功能)
 * Story 2.16: Task 9 (集成消息过滤)
 *
 * 左侧面板，显示 AI 对话的意图和消息流
 * 支持按类型过滤和关键词搜索消息
 */

import * as React from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import {
  NarrativeStream,
  type NarrativeStreamRef,
} from "@/components/narrative";
import type { NarrativeMessage } from "@/types/message";
import { useMessageFilterStore } from "@/stores/useMessageFilterStore";
import { filterWithPairedResults } from "@/lib/message-filter";
import {
  MessageFilterBar,
  EmptyFilterResult,
} from "@/components/filter";
import { ToolPairingProvider, useToolPairingContext } from "@/contexts/ToolPairingContext";

export interface NarrativePanelProps {
  /** 自定义 className */
  className?: string;
  /** 消息列表 (如果提供，将渲染 NarrativeStream) */
  messages?: NarrativeMessage[];
  /** 当前选中的消息 ID */
  selectedMessageId?: string;
  /** 消息选中回调 */
  onMessageSelect?: (messageId: string, message: NarrativeMessage) => void;
  /** 子内容 (向后兼容) */
  children?: React.ReactNode;
}

/**
 * NarrativePanel Ref 暴露的方法
 */
export interface NarrativePanelRef {
  /** 滚动到指定消息 */
  scrollToMessage: (messageId: string) => void;
  /** 滚动到顶部 */
  scrollToTop: () => void;
  /** 滚动到底部 */
  scrollToBottom: () => void;
}

export const NarrativePanel = React.forwardRef<
  NarrativePanelRef,
  NarrativePanelProps
>(
  (
    { className, messages, selectedMessageId, onMessageSelect, children },
    ref
  ) => {
    // NarrativeStream ref
    const streamRef = React.useRef<NarrativeStreamRef>(null);

    // 过滤状态 (Story 2.16)
    const { selectedTypes, searchQuery } = useMessageFilterStore();

    // 计算过滤后的消息 (Story 2.16)
    const filterResult = React.useMemo(() => {
      return filterWithPairedResults(
        messages ?? [],
        selectedTypes,
        searchQuery
      );
    }, [messages, selectedTypes, searchQuery]);

    // 暴露给父组件的方法
    React.useImperativeHandle(
      ref,
      () => ({
        scrollToMessage: (messageId: string) => {
          streamRef.current?.scrollToMessage(messageId);
        },
        scrollToTop: () => {
          streamRef.current?.scrollToTop();
        },
        scrollToBottom: () => {
          streamRef.current?.scrollToBottom();
        },
      }),
      []
    );

    // 如果有子内容，直接渲染 (向后兼容)
    if (children) {
      return (
        <ScrollArea className={cn("h-full", className)}>{children}</ScrollArea>
      );
    }

    // 判断是否有活动过滤
    const hasActiveFilters = selectedTypes.size > 0 || searchQuery.length > 0;
    const showEmptyFilterResult = hasActiveFilters && filterResult.filteredCount === 0;

    // 渲染 NarrativeStream (空状态由 NarrativeStream 内部处理)
    // Story 2.15: 使用原始 messages 构建配对，确保跨过滤的配对关系正确
    return (
      <ToolPairingProvider messages={messages ?? []}>
        <NarrativePanelContent
          className={className}
          streamRef={streamRef}
          filterResult={filterResult}
          hasActiveFilters={hasActiveFilters}
          showEmptyFilterResult={showEmptyFilterResult}
          selectedMessageId={selectedMessageId}
          onMessageSelect={onMessageSelect}
        />
      </ToolPairingProvider>
    );
  }
);

/** 内部内容组件 - 用于在 ToolPairingProvider 内部注册滚动回调 */
interface NarrativePanelContentProps {
  className?: string;
  streamRef: React.RefObject<NarrativeStreamRef | null>;
  filterResult: ReturnType<typeof filterWithPairedResults>;
  hasActiveFilters: boolean;
  showEmptyFilterResult: boolean;
  selectedMessageId?: string;
  onMessageSelect?: (messageId: string, message: NarrativeMessage) => void;
}

function NarrativePanelContent({
  className,
  streamRef,
  filterResult,
  showEmptyFilterResult,
  selectedMessageId,
  onMessageSelect,
}: NarrativePanelContentProps) {
  // Story 2.15: 注册滚动回调到配对上下文
  const pairingContext = useToolPairingContext();

  React.useEffect(() => {
    if (pairingContext) {
      pairingContext.registerScrollCallback((messageId: string) => {
        streamRef.current?.scrollToMessage(messageId);
      });
    }
  }, [pairingContext, streamRef]);

  return (
    <div className={cn("h-full flex flex-col", className)}>
      {/* 过滤栏 (Story 2.16) */}
      <MessageFilterBar
        filteredCount={filterResult.filteredCount}
        totalCount={filterResult.totalCount}
      />

      {/* 消息流或空状态 */}
      <div className="flex-1 min-h-0">
        {showEmptyFilterResult ? (
          <EmptyFilterResult className="h-full" />
        ) : (
          <NarrativeStream
            ref={streamRef}
            messages={filterResult.messages}
            selectedMessageId={selectedMessageId}
            onMessageSelect={onMessageSelect}
            className="h-full"
          />
        )}
      </div>
    </div>
  );
}

NarrativePanel.displayName = "NarrativePanel";

export default NarrativePanel;
