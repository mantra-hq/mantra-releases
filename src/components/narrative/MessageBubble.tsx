/**
 * MessageBubble - 消息气泡组件
 * Story 2.3: Task 3, Story 2.4: Task 5
 *
 * 支持 User 和 AI 两种消息样式变体
 * 使用 ContentBlockRenderer 渲染所有内容块类型
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import type { NarrativeMessage } from "@/types/message";
import { ContentBlockRenderer } from "./ContentBlockRenderer";

export interface MessageBubbleProps {
  /** 消息数据 */
  message: NarrativeMessage;
  /** 是否选中 */
  isSelected?: boolean;
  /** 点击回调 */
  onClick?: () => void;
  /** 测量元素回调 (用于虚拟化) */
  measureElement?: (node: HTMLElement | null) => void;
  /** data-index 属性 (用于虚拟化) */
  index?: number;
  /** 自定义 className */
  className?: string;
}

/**
 * 格式化时间戳显示
 */
function formatTimestamp(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return "";
  }
}

export const MessageBubble = React.forwardRef<HTMLDivElement, MessageBubbleProps>(
  (
    { message, isSelected = false, onClick, measureElement, index, className },
    ref
  ) => {
    const isUser = message.role === "user";
    const timestamp = formatTimestamp(message.timestamp);

    // 合并 ref 用于虚拟化测量
    const combinedRef = React.useCallback(
      (node: HTMLDivElement | null) => {
        // Forward ref
        if (typeof ref === "function") {
          ref(node);
        } else if (ref) {
          ref.current = node;
        }
        // Measure element
        measureElement?.(node);
      },
      [ref, measureElement]
    );

    // 键盘事件处理
    const handleKeyDown = React.useCallback(
      (event: React.KeyboardEvent<HTMLDivElement>) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick?.();
        }
      },
      [onClick]
    );

    return (
      <div
        ref={combinedRef}
        data-index={index}
        data-selected={isSelected}
        role="button"
        tabIndex={0}
        onClick={onClick}
        onKeyDown={handleKeyDown}
        className={cn(
          // Base styles
          "group relative outline-none",
          // Layout
          "w-full py-1.5",
          // Focus state
          "focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
          // Alignment based on role
          isUser ? "flex justify-end" : "flex justify-start",
          className
        )}
      >
        <div
          className={cn(
            // Base bubble styles
            "px-4 py-3 text-sm leading-relaxed",
            // Transition
            "transition-all duration-150",
            // Selection ring
            isSelected &&
              "ring-2 ring-primary ring-offset-2 ring-offset-background",
            // Cursor
            onClick && "cursor-pointer",
            // User bubble variant
            isUser && [
              "bg-user-bubble",
              "rounded-xl rounded-br-sm",
              "max-w-[85%]",
              "text-foreground",
            ],
            // AI bubble variant
            !isUser && [
              "bg-transparent",
              "border-l-[3px] border-primary",
              "rounded-r-lg",
              "max-w-[95%]",
              "text-foreground",
            ],
            // Hover state (subtle)
            onClick && "hover:opacity-90"
          )}
        >
          {/* 消息内容块 - 使用 ContentBlockRenderer 渲染所有类型 */}
          <div className="space-y-1">
            {message.content.map((block, blockIndex) => (
              <ContentBlockRenderer
                key={`${message.id}-block-${blockIndex}`}
                block={block}
              />
            ))}
          </div>

          {/* 时间戳 */}
          {timestamp && (
            <div
              className={cn(
                "mt-1 text-xs text-muted-foreground",
                "opacity-0 group-hover:opacity-100 transition-opacity duration-150",
                isUser ? "text-right" : "text-left"
              )}
            >
              {timestamp}
            </div>
          )}
        </div>
      </div>
    );
  }
);

MessageBubble.displayName = "MessageBubble";

export default MessageBubble;

