/**
 * NarrativeStream - 虚拟化对话流组件
 * Story 2.3: Task 4
 *
 * 使用 @tanstack/react-virtual 实现大量消息的高性能虚拟化渲染
 */

import * as React from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { MessageSquare } from "lucide-react";
import { cn } from "@/lib/utils";
import { MessageBubble } from "./MessageBubble";
import type { NarrativeMessage } from "@/types/message";

/**
 * NarrativeStream 组件 Props
 */
export interface NarrativeStreamProps {
  /** 消息列表 */
  messages: NarrativeMessage[];
  /** 当前选中的消息 ID */
  selectedMessageId?: string;
  /** 消息选中回调 */
  onMessageSelect?: (messageId: string, message: NarrativeMessage) => void;
  /** 自定义 className */
  className?: string;
}

/**
 * NarrativeStream Ref 暴露的方法
 */
export interface NarrativeStreamRef {
  /** 滚动到指定消息 */
  scrollToMessage: (messageId: string) => void;
  /** 滚动到顶部 */
  scrollToTop: () => void;
  /** 滚动到底部 */
  scrollToBottom: () => void;
}

/**
 * 估算消息高度
 * 用户消息通常较短，AI 消息通常较长
 */
function estimateMessageSize(message: NarrativeMessage): number {
  const textLength = message.content
    .filter((block) => block.type === "text")
    .reduce((acc, block) => acc + block.content.length, 0);

  // 基础高度 + 内容长度估算
  const baseHeight = message.role === "user" ? 60 : 80;
  const lineEstimate = Math.ceil(textLength / 60); // 约 60 字符每行
  const contentHeight = lineEstimate * 20; // 约 20px 每行

  return Math.min(Math.max(baseHeight + contentHeight, 60), 400); // 限制在 60-400px
}

/**
 * 空状态组件
 */
function EmptyState() {
  return (
    <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
      <div className="flex flex-col items-center gap-4 p-8 text-center">
        <div className="rounded-full bg-muted p-4">
          <MessageSquare className="size-8" />
        </div>
        <div className="space-y-2">
          <h3 className="text-lg font-semibold text-foreground">暂无消息</h3>
          <p className="text-sm max-w-xs">
            对话消息将在这里显示
          </p>
        </div>
      </div>
    </div>
  );
}

export const NarrativeStream = React.forwardRef<
  NarrativeStreamRef,
  NarrativeStreamProps
>(({ messages, selectedMessageId, onMessageSelect, className }, ref) => {
  // 滚动容器 ref
  const scrollContainerRef = React.useRef<HTMLDivElement>(null);

  // 当前焦点的消息索引 (用于键盘导航)
  const [focusedIndex, setFocusedIndex] = React.useState<number>(-1);

  // 虚拟化器
  const virtualizer = useVirtualizer({
    count: messages.length,
    getScrollElement: () => scrollContainerRef.current,
    estimateSize: (index) => estimateMessageSize(messages[index]),
    overscan: 5, // 预渲染 5 条消息以优化滚动体验
  });

  // 创建消息 ID 到索引的映射
  const messageIndexMap = React.useMemo(() => {
    const map = new Map<string, number>();
    messages.forEach((msg, index) => {
      map.set(msg.id, index);
    });
    return map;
  }, [messages]);

  // 暴露给父组件的方法
  React.useImperativeHandle(
    ref,
    () => ({
      scrollToMessage: (messageId: string) => {
        const index = messageIndexMap.get(messageId);
        if (index !== undefined) {
          virtualizer.scrollToIndex(index, { align: "center", behavior: "smooth" });
        }
      },
      scrollToTop: () => {
        virtualizer.scrollToIndex(0, { align: "start", behavior: "smooth" });
      },
      scrollToBottom: () => {
        virtualizer.scrollToIndex(messages.length - 1, {
          align: "end",
          behavior: "smooth",
        });
      },
    }),
    [virtualizer, messageIndexMap, messages.length]
  );

  // 处理消息点击
  const handleMessageClick = React.useCallback(
    (message: NarrativeMessage, index: number) => {
      setFocusedIndex(index);
      onMessageSelect?.(message.id, message);
    },
    [onMessageSelect]
  );

  // 处理键盘导航 (ArrowUp/ArrowDown)
  const handleKeyDown = React.useCallback(
    (event: React.KeyboardEvent<HTMLDivElement>) => {
      if (messages.length === 0) return;

      let newIndex = focusedIndex;

      if (event.key === "ArrowUp") {
        event.preventDefault();
        newIndex = focusedIndex <= 0 ? messages.length - 1 : focusedIndex - 1;
      } else if (event.key === "ArrowDown") {
        event.preventDefault();
        newIndex = focusedIndex >= messages.length - 1 ? 0 : focusedIndex + 1;
      } else if (event.key === "Home") {
        event.preventDefault();
        newIndex = 0;
      } else if (event.key === "End") {
        event.preventDefault();
        newIndex = messages.length - 1;
      } else {
        return;
      }

      setFocusedIndex(newIndex);
      virtualizer.scrollToIndex(newIndex, { align: "center", behavior: "smooth" });
      
      // 选中新焦点的消息
      const message = messages[newIndex];
      if (message) {
        onMessageSelect?.(message.id, message);
      }
    },
    [focusedIndex, messages, virtualizer, onMessageSelect]
  );

  // 空状态
  if (messages.length === 0) {
    return (
      <div className={cn("h-full", className)}>
        <EmptyState />
      </div>
    );
  }

  const virtualItems = virtualizer.getVirtualItems();

  return (
    <div
      ref={scrollContainerRef}
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className={cn(
        "h-full overflow-y-auto",
        // 自定义滚动条样式
        "scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent",
        // 焦点样式
        "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset",
        className
      )}
    >
      {/* 虚拟列表容器 */}
      <div
        className="relative w-full px-4"
        style={{
          height: `${virtualizer.getTotalSize()}px`,
        }}
      >
        {/* 渲染可见的虚拟项 */}
        {virtualItems.map((virtualItem) => {
          const message = messages[virtualItem.index];
          const isSelected = message.id === selectedMessageId;

          return (
            <div
              key={virtualItem.key}
              data-index={virtualItem.index}
              className="absolute left-0 top-0 w-full px-4"
              style={{
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              <MessageBubble
                message={message}
                isSelected={isSelected}
                onClick={() => handleMessageClick(message, virtualItem.index)}
                measureElement={virtualizer.measureElement}
                index={virtualItem.index}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
});

NarrativeStream.displayName = "NarrativeStream";

export default NarrativeStream;

