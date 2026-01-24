/**
 * DeletedPlaceholder - 删除占位符组件
 * Story 10.3: Task 4
 *
 * 在预览列表中显示被删除的消息占位符
 * AC2: 虚线边框、灰色淡化样式
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { X, User, Bot, Hammer, ClipboardList, Wrench } from "lucide-react";
import { cn } from "@/lib/utils";
import { formatTokenCount } from "@/lib/token-counter";
import type { NarrativeMessage, ContentBlock } from "@/types/message";

// ===== Props =====

export interface DeletedPlaceholderProps {
  /** 原始消息 */
  originalMessage: NarrativeMessage;
  /** 节省的 token 数 */
  savedTokens: number;
  /** 测量元素回调 (用于虚拟化) */
  measureElement?: (node: HTMLElement | null) => void;
  /** data-index 属性 (用于虚拟化) */
  index?: number;
  /** 自定义 className */
  className?: string;
}

// ===== 工具函数 =====

/**
 * 检测消息是否包含工具调用
 */
function hasToolUseContent(content: ContentBlock[]): boolean {
  return content.some((block) => block.type === "tool_use");
}

/**
 * 检测消息是否包含工具结果
 */
function hasToolResultContent(content: ContentBlock[]): boolean {
  return content.some((block) => block.type === "tool_result");
}

/**
 * 获取消息类型标签
 */
function getMessageTypeLabel(
  role: string,
  content: ContentBlock[],
  t: (key: string) => string
): { label: string; Icon: React.ComponentType<{ className?: string }> } {
  const isToolUse = hasToolUseContent(content);
  const isToolResult = hasToolResultContent(content);

  if (isToolUse) {
    return { label: t("compress.messageCard.toolCall"), Icon: Hammer };
  }

  if (isToolResult) {
    return { label: t("compress.messageCard.toolResult"), Icon: ClipboardList };
  }

  switch (role) {
    case "user":
      return { label: t("compress.messageCard.user"), Icon: User };
    case "assistant":
      return { label: t("compress.messageCard.assistant"), Icon: Bot };
    default:
      return { label: t("compress.messageCard.system"), Icon: Wrench };
  }
}

// ===== 主组件 =====

/**
 * DeletedPlaceholder - 删除占位符
 *
 * AC2: 删除标记展示
 * - 虚线边框占位卡片
 * - 显示 "✕ 已删除: {消息类型}" + "-X tokens"
 * - 灰色/淡化样式 (opacity-50)
 */
export const DeletedPlaceholder = React.forwardRef<
  HTMLDivElement,
  DeletedPlaceholderProps
>(({ originalMessage, savedTokens, measureElement, index, className }, ref) => {
  const { t } = useTranslation();

  // 获取消息类型
  const { label: messageType, Icon: TypeIcon } = getMessageTypeLabel(
    originalMessage.role,
    originalMessage.content,
    t
  );

  // 合并 ref
  const combinedRef = React.useCallback(
    (node: HTMLDivElement | null) => {
      if (typeof ref === "function") {
        ref(node);
      } else if (ref) {
        ref.current = node;
      }
      measureElement?.(node);
    },
    [ref, measureElement]
  );

  return (
    <div
      ref={combinedRef}
      data-testid="deleted-placeholder"
      data-index={index}
      className={cn(
        // AC2: 虚线边框 + 淡化样式
        "border border-dashed border-muted rounded-lg p-3 mb-2",
        "bg-muted/30 opacity-50",
        "flex items-center justify-between",
        className
      )}
    >
      {/* 左侧: 删除标识 + 消息类型 */}
      <div className="flex items-center gap-2 text-muted-foreground">
        <X className="size-4 text-destructive/70" />
        <span className="text-sm">
          {t("compress.previewCard.deleted")}:
        </span>
        <div className="flex items-center gap-1">
          <TypeIcon className="size-3.5" />
          <span className="text-sm">{messageType}</span>
        </div>
      </div>

      {/* 右侧: 节省的 token 数 */}
      <div className="flex items-center gap-1 text-sm text-green-600/70">
        <span>-{formatTokenCount(savedTokens)}</span>
        <span className="text-muted-foreground">
          {t("compress.previewCard.savedTokens")}
        </span>
      </div>
    </div>
  );
});

DeletedPlaceholder.displayName = "DeletedPlaceholder";

export default DeletedPlaceholder;
