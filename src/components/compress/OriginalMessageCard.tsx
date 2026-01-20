/**
 * OriginalMessageCard - 原始消息卡片组件
 * Story 10.2: Task 2, Task 3
 *
 * 显示单条消息的卡片，包含角色图标、内容摘要、Token 数量
 * 支持长内容折叠/展开
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import {
  Wrench,
  User,
  Bot,
  Hammer,
  ClipboardList,
  ChevronDown,
  ChevronUp,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { NarrativeMessage, ContentBlock } from "@/types/message";
import { estimateTokenCount, formatTokenCount } from "@/lib/token-counter";

/**
 * OriginalMessageCard 组件 Props
 */
export interface OriginalMessageCardProps {
  /** 消息数据 */
  message: NarrativeMessage;
  /** 测量元素回调 (用于虚拟化) */
  measureElement?: (node: HTMLElement | null) => void;
  /** data-index 属性 (用于虚拟化) */
  index?: number;
  /** 自定义 className */
  className?: string;
}

// 折叠配置常量
const MAX_COLLAPSED_LINES = 3;
const MAX_COLLAPSED_CHARS = 200;

/**
 * 获取消息的文本内容
 */
function getMessageTextContent(content: ContentBlock[]): string {
  return content
    .filter((block) => block.type === "text")
    .map((block) => block.content)
    .join("\n");
}

/**
 * 判断是否需要折叠
 */
function shouldCollapse(content: string): boolean {
  const lines = content.split("\n").length;
  return lines > MAX_COLLAPSED_LINES || content.length > MAX_COLLAPSED_CHARS;
}

/**
 * 获取折叠后的内容
 */
function getCollapsedContent(content: string): string {
  const lines = content.split("\n").slice(0, MAX_COLLAPSED_LINES);
  let result = lines.join("\n");
  if (result.length > MAX_COLLAPSED_CHARS) {
    result = result.slice(0, MAX_COLLAPSED_CHARS);
  }
  return result + "...";
}

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
 * 获取工具名称 (从 tool_use 块中提取)
 */
function getToolName(content: ContentBlock[]): string | undefined {
  const toolUseBlock = content.find((block) => block.type === "tool_use");
  return toolUseBlock?.toolName;
}

/**
 * 角色图标和样式配置
 */
interface RoleConfig {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  badgeClasses: string;
  iconColor: string;
}

function getRoleConfig(
  role: string,
  content: ContentBlock[],
  t: (key: string) => string
): RoleConfig {
  // 检查是否为工具调用或工具结果
  const isToolUse = hasToolUseContent(content);
  const isToolResult = hasToolResultContent(content);

  if (isToolUse) {
    return {
      icon: Hammer,
      label: t("compress.messageCard.toolCall"),
      badgeClasses: "bg-amber-500/10 text-amber-500",
      iconColor: "text-amber-500",
    };
  }

  if (isToolResult) {
    return {
      icon: ClipboardList,
      label: t("compress.messageCard.toolResult"),
      badgeClasses: "bg-purple-500/10 text-purple-500",
      iconColor: "text-purple-500",
    };
  }

  switch (role) {
    case "user":
      return {
        icon: User,
        label: t("compress.messageCard.user"),
        badgeClasses: "bg-blue-500/10 text-blue-500",
        iconColor: "text-blue-500",
      };
    case "assistant":
      return {
        icon: Bot,
        label: t("compress.messageCard.assistant"),
        badgeClasses: "bg-emerald-500/10 text-emerald-500",
        iconColor: "text-emerald-500",
      };
    default:
      // system 或其他
      return {
        icon: Wrench,
        label: t("compress.messageCard.system"),
        badgeClasses: "bg-muted text-muted-foreground",
        iconColor: "text-muted-foreground",
      };
  }
}

/**
 * OriginalMessageCard - 单条消息卡片
 *
 * AC1: 显示角色图标、内容摘要、Token 数量
 * AC2: 消息分类标识 (不同图标)
 * AC3: 长内容折叠
 */
export const OriginalMessageCard = React.forwardRef<
  HTMLDivElement,
  OriginalMessageCardProps
>(({ message, measureElement, index, className }, ref) => {
  const { t } = useTranslation();

  // AC3: 折叠状态
  const [isExpanded, setIsExpanded] = React.useState(false);

  // 获取消息文本内容
  const textContent = React.useMemo(
    () => getMessageTextContent(message.content),
    [message.content]
  );

  // 判断是否需要折叠
  const needsCollapse = React.useMemo(
    () => shouldCollapse(textContent),
    [textContent]
  );

  // 显示的内容
  const displayContent = React.useMemo(() => {
    if (!needsCollapse || isExpanded) {
      return textContent;
    }
    return getCollapsedContent(textContent);
  }, [textContent, needsCollapse, isExpanded]);

  // Token 估算
  const tokenCount = React.useMemo(
    () => estimateTokenCount(textContent),
    [textContent]
  );

  // 角色配置
  const roleConfig = getRoleConfig(message.role, message.content, t);
  const RoleIcon = roleConfig.icon;

  // 工具名称 (用于 tool_use 显示)
  const toolName = hasToolUseContent(message.content)
    ? getToolName(message.content)
    : undefined;

  // 存储元素引用，用于重新测量
  const elementRef = React.useRef<HTMLDivElement | null>(null);

  // 合并 ref 用于虚拟化测量
  const combinedRef = React.useCallback(
    (node: HTMLDivElement | null) => {
      // 保存到 elementRef 供重新测量使用
      elementRef.current = node;
      // Forward ref
      if (typeof ref === "function") {
        ref(node);
      } else if (ref) {
        ref.current = node;
      }
      // Measure element for virtualization
      measureElement?.(node);
    },
    [ref, measureElement]
  );

  // AC3: 折叠/展开时触发虚拟化重新测量
  const handleToggleExpand = React.useCallback(() => {
    setIsExpanded((prev) => !prev);
    // 延迟触发重新测量，确保 DOM 已更新
    requestAnimationFrame(() => {
      if (measureElement && elementRef.current) {
        measureElement(elementRef.current);
      }
    });
  }, [measureElement]);

  return (
    <div
      ref={combinedRef}
      data-testid="original-message-card"
      data-index={index}
      className={cn(
        // 基础卡片样式
        "border rounded-lg p-3 mb-2",
        "bg-card hover:bg-accent/50 transition-colors",
        className
      )}
    >
      {/* 头部: 角色标签 + Token 数量 */}
      <div className="flex items-center justify-between mb-2">
        <div
          className={cn(
            "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium",
            roleConfig.badgeClasses
          )}
        >
          <RoleIcon className={cn("size-3.5", roleConfig.iconColor)} />
          <span>{roleConfig.label}</span>
          {toolName && (
            <span className="text-muted-foreground">· {toolName}</span>
          )}
        </div>

        {/* Token 徽章 - 使用格式化显示 */}
        <Badge variant="outline" className="text-xs">
          {formatTokenCount(tokenCount)} {t("compress.messageCard.tokens")}
        </Badge>
      </div>

      {/* 内容区域 */}
      <div className="text-sm text-foreground/90 whitespace-pre-wrap break-words">
        {displayContent}
      </div>

      {/* AC3: 折叠/展开按钮 */}
      {needsCollapse && (
        <Button
          variant="ghost"
          size="sm"
          onClick={handleToggleExpand}
          className="mt-2 h-7 text-xs text-muted-foreground hover:text-foreground"
        >
          {isExpanded ? (
            <>
              <ChevronUp className="size-3.5 mr-1" />
              {t("compress.messageCard.collapse")}
            </>
          ) : (
            <>
              <ChevronDown className="size-3.5 mr-1" />
              {t("compress.messageCard.expand")}
            </>
          )}
        </Button>
      )}
    </div>
  );
});

OriginalMessageCard.displayName = "OriginalMessageCard";

export default OriginalMessageCard;
