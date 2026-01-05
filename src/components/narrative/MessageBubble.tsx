/**
 * MessageBubble - 消息气泡组件
 * Story 2.3: Task 3, Story 2.4: Task 5
 * Story 2.26: 国际化支持
 *
 * 支持 User 和 AI 两种消息样式变体
 * 使用 ContentBlockRenderer 渲染所有内容块类型
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import type { NarrativeMessage } from "@/types/message";
import { ContentBlockRenderer } from "./ContentBlockRenderer";
import { CopyButton } from "@/components/common/CopyButton";
import { getMessageCopyContent } from "@/lib/copy-utils";

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
function formatTimestamp(timestamp: string, locale: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString(locale, {
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
    const { i18n } = useTranslation();
    const isUser = message.role === "user";
    const timestamp = formatTimestamp(message.timestamp, i18n.language);

    // 获取消息可复制内容 (AC1)
    const copyContent = React.useMemo(
      () => getMessageCopyContent(message),
      [message]
    );

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
            "relative px-4 py-3 text-sm leading-relaxed",
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
          {/* Story 2.15: 启用 ToolCallCard 支持详情面板交互 */}
          <div className="space-y-1">
            {message.content.map((block, blockIndex) => (
              <ContentBlockRenderer
                key={`${message.id}-block-${blockIndex}`}
                block={block}
                useNewToolCard={true}
              />
            ))}
          </div>

          {/* Story 2.22: 消息级复制按钮 - 悬浮时可见 */}
          {/* M4 fix: 使用 -top-1 -right-1 避免遮挡短消息内容 */}
          {copyContent && (
            <div className="absolute -right-1 -top-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100">
              <CopyButton
                content={copyContent}
                size="sm"
                ariaLabel="复制消息"
                tooltip="复制消息"
                className="bg-background/80 backdrop-blur-sm"
              />
            </div>
          )}

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

