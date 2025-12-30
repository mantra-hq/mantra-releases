/**
 * NarrativePanel - 对话流面板
 * Story 2.2: Task 4.1 (基础)
 * Story 2.3: Task 5 (集成 NarrativeStream)
 *
 * 左侧面板，显示 AI 对话的意图和消息流
 */

import * as React from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import {
  NarrativeStream,
  type NarrativeStreamRef,
} from "@/components/narrative";
import type { NarrativeMessage } from "@/types/message";

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

    // 渲染 NarrativeStream (空状态由 NarrativeStream 内部处理)
    return (
      <NarrativeStream
        ref={streamRef}
        messages={messages ?? []}
        selectedMessageId={selectedMessageId}
        onMessageSelect={onMessageSelect}
        className={cn("h-full", className)}
      />
    );
  }
);

NarrativePanel.displayName = "NarrativePanel";

export default NarrativePanel;

