/**
 * PreviewMessageCard - 预览消息卡片组件
 * Story 10.3: Task 3
 *
 * 显示压缩预览中的消息卡片
 * 根据操作类型 (keep/modify/insert) 显示不同样式
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import {
  Wrench,
  User,
  Bot,
  Hammer,
  ClipboardList,
  Pencil,
  Sparkles,
  ChevronDown,
  ChevronUp,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { PreviewMessage } from "@/hooks/useCompressState";
import type { ContentBlock } from "@/types/message";
import { estimateTokenCount, formatTokenCount } from "@/lib/token-counter";
import { getMessageTextContent } from "@/lib/message-utils";

// ===== Props =====

export interface PreviewMessageCardProps {
  /** 预览消息数据 */
  previewMessage: PreviewMessage;
  /** 测量元素回调 (用于虚拟化) */
  measureElement?: (node: HTMLElement | null) => void;
  /** data-index 属性 (用于虚拟化) */
  index?: number;
  /** 自定义 className */
  className?: string;
}

// ===== 常量 =====

const MAX_COLLAPSED_LINES = 3;
const MAX_COLLAPSED_CHARS = 200;

// ===== 工具函数 =====

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
 * 获取工具名称
 */
function getToolName(content: ContentBlock[]): string | undefined {
  const toolUseBlock = content.find((block) => block.type === "tool_use");
  return toolUseBlock?.toolName;
}

// ===== 角色配置 =====

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
      return {
        icon: Wrench,
        label: t("compress.messageCard.system"),
        badgeClasses: "bg-muted text-muted-foreground",
        iconColor: "text-muted-foreground",
      };
  }
}

// ===== 主组件 =====

/**
 * PreviewMessageCard - 预览消息卡片
 *
 * AC2: 保留消息样式 (默认样式)
 * AC3: 修改消息样式 (黄色边框 + ✏️)
 * AC4: 新增消息样式 (绿色边框 + ✨)
 */
export const PreviewMessageCard = React.forwardRef<
  HTMLDivElement,
  PreviewMessageCardProps
>(({ previewMessage, measureElement, index, className }, ref) => {
  const { t } = useTranslation();
  const { operation, message, tokenDelta } = previewMessage;

  // 折叠状态
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

  // 工具名称
  const toolName = hasToolUseContent(message.content)
    ? getToolName(message.content)
    : undefined;

  // 存储元素引用
  const elementRef = React.useRef<HTMLDivElement | null>(null);

  // 合并 ref
  const combinedRef = React.useCallback(
    (node: HTMLDivElement | null) => {
      elementRef.current = node;
      if (typeof ref === "function") {
        ref(node);
      } else if (ref) {
        ref.current = node;
      }
      measureElement?.(node);
    },
    [ref, measureElement]
  );

  // 折叠/展开时重新测量
  const handleToggleExpand = React.useCallback(() => {
    setIsExpanded((prev) => !prev);
    requestAnimationFrame(() => {
      if (measureElement && elementRef.current) {
        measureElement(elementRef.current);
      }
    });
  }, [measureElement]);

  // 根据操作类型确定样式
  const cardClasses = cn(
    "border rounded-lg p-3 mb-2 transition-colors",
    {
      // AC2: 保留样式 (默认)
      "bg-card hover:bg-accent/50": operation === "keep",
      // AC3: 修改样式 (黄色边框)
      "border-2 border-yellow-500 bg-yellow-500/5": operation === "modify",
      // AC4: 新增样式 (绿色边框)
      "border-2 border-green-500 bg-green-500/5": operation === "insert",
    },
    className
  );

  // 操作图标
  const OperationIcon = operation === "modify" ? Pencil : operation === "insert" ? Sparkles : null;

  return (
    <div
      ref={combinedRef}
      data-testid="preview-message-card"
      data-operation={operation}
      data-index={index}
      className={cardClasses}
    >
      {/* 头部: 角色标签 + 操作图标 + Token 数量 */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {/* 角色标签 */}
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

          {/* AC3/AC4: 操作标识图标 */}
          {OperationIcon && (
            <div
              className={cn(
                "inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs",
                operation === "modify" && "text-yellow-600 bg-yellow-500/10",
                operation === "insert" && "text-green-600 bg-green-500/10"
              )}
            >
              <OperationIcon className="size-3" />
              <span>
                {operation === "modify"
                  ? t("compress.previewCard.modified")
                  : t("compress.previewCard.inserted")}
              </span>
            </div>
          )}
        </div>

        {/* Token 显示 */}
        <div className="flex items-center gap-1.5">
          {/* Token 差异显示 (modify 时) */}
          {operation === "modify" && tokenDelta !== undefined && (
            <Badge
              variant="outline"
              className={cn(
                "text-xs",
                tokenDelta < 0 ? "text-green-600" : "text-orange-600"
              )}
            >
              {tokenDelta > 0 ? "+" : ""}
              {tokenDelta} {t("compress.previewCard.tokenDelta")}
            </Badge>
          )}
          {/* 当前 Token 数 */}
          <Badge variant="outline" className="text-xs">
            {formatTokenCount(tokenCount)} {t("compress.messageCard.tokens")}
          </Badge>
        </div>
      </div>

      {/* 内容区域 */}
      <div className="text-sm text-foreground/90 whitespace-pre-wrap break-words">
        {displayContent}
      </div>

      {/* 折叠/展开按钮 */}
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

PreviewMessageCard.displayName = "PreviewMessageCard";

export default PreviewMessageCard;
