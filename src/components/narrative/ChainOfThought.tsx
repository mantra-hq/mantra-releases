/**
 * ChainOfThought - 思维链组件
 * Story 2.4: Task 1
 * Story 2.26: 国际化支持
 *
 * 显示 AI 的思考过程 (CoT)，默认折叠，支持展开查看
 * AC: #2, #5, #6, #7
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";

export interface ChainOfThoughtProps {
  /** 思考内容 */
  content: string;
  /** 默认是否展开 */
  defaultOpen?: boolean;
  /** 自定义 className */
  className?: string;
  // === Story 8.11: 新增字段 (AC #7) ===
  /** 思考主题 (Gemini) */
  subject?: string;
  /** 思考时间戳 (Gemini) */
  thinkingTimestamp?: string;
}

/**
 * 格式化思考时间戳为用户友好显示
 * 支持多种格式: ISO 8601, 毫秒时间戳, 或直接显示原始值
 */
function formatThinkingTimestamp(timestamp: string): string {
  // 尝试解析为日期
  const date = new Date(timestamp);
  if (!isNaN(date.getTime())) {
    // 有效日期，显示时:分:秒
    return date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }
  // 无法解析，返回原始值
  return timestamp;
}

/**
 * ChainOfThought 组件
 *
 * 视觉规范:
 * - 容器: 左侧 2px 虚线边框，padding-left 12px
 * - 头部: 💭 图标 + "思考过程" + 展开箭头
 * - 内容: 斜体文字，muted 颜色，13px
 * - 动画: 150ms ease-out
 */
export function ChainOfThought({
  content,
  defaultOpen = false,
  className,
  subject,
  thinkingTimestamp,
}: ChainOfThoughtProps) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = React.useState(defaultOpen);

  // 阻止事件冒泡，避免触发父组件的消息选中逻辑
  const handleClick = React.useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
  }, []);

  return (
    <Collapsible.Root
      open={isOpen}
      onOpenChange={setIsOpen}
      className={cn(
        // 容器样式
        "border-l-2 border-dashed border-muted-foreground/30",
        "pl-3 my-2",
        className
      )}
      onClick={handleClick}
    >
      <Collapsible.Trigger
        className={cn(
          // 头部样式
          "flex items-center gap-2",
          "cursor-pointer select-none",
          "text-muted-foreground text-xs",
          // Hover 效果
          "hover:text-muted-foreground/80",
          // Focus 状态
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
          "focus-visible:ring-offset-2 focus-visible:ring-offset-background",
          "rounded-sm px-1 -mx-1"
        )}
        aria-expanded={isOpen}
      >
        {/* 图标 */}
        <span className="text-sm" role="img" aria-label={t("message.thinking")}>
          💭
        </span>

        {/* Story 8.11: 优先显示 subject，否则显示默认标题 (AC #7) */}
        {subject ? (
          <span className="font-semibold">{subject}</span>
        ) : (
          <span>{t("message.thinkingProcess")}</span>
        )}

        {/* Story 8.11: 显示时间戳 (AC #7) */}
        {thinkingTimestamp && (
          <span className="ml-auto text-[10px] text-muted-foreground/60">
            {formatThinkingTimestamp(thinkingTimestamp)}
          </span>
        )}

        {/* 展开箭头 */}
        <ChevronRight
          className={cn(
            "h-3.5 w-3.5 shrink-0",
            "transition-transform duration-150 ease-out",
            isOpen && "rotate-90"
          )}
        />
      </Collapsible.Trigger>

      <Collapsible.Content
        className={cn(
          // 内容样式
          "overflow-hidden",
          // 动画 (150ms ease-out)
          "data-[state=open]:animate-collapsible-down",
          "data-[state=closed]:animate-collapsible-up"
        )}
      >
        <div
          className={cn(
            // 内容文本样式
            "pt-2 text-[13px] leading-relaxed",
            "italic text-muted-foreground",
            "whitespace-pre-wrap"
          )}
        >
          {content}
        </div>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}

export default ChainOfThought;












