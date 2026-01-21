/**
 * InsertedMessageCard - 已插入消息卡片组件
 * Story 10.5: Task 3
 *
 * AC3: 显示绿色边框 + ✨ 图标标识
 * AC4: 右上角显示删除按钮
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Sparkles, X, User, Bot, ChevronDown, ChevronUp } from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { NarrativeMessage } from "@/types/message";
import { estimateTokenCount, formatTokenCount } from "@/lib/token-counter";
import { getMessageTextContent } from "@/lib/message-utils";

/**
 * InsertedMessageCard 组件 Props
 */
export interface InsertedMessageCardProps {
  /** 插入的消息 */
  message: NarrativeMessage;
  /** 删除回调 */
  onRemove: () => void;
  /** 自定义 className */
  className?: string;
}

// 折叠配置常量
const MAX_COLLAPSED_LINES = 3;
const MAX_COLLAPSED_CHARS = 200;

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
 * InsertedMessageCard - 已插入消息卡片
 *
 * AC3: 绿色边框 + ✨ 图标
 * AC4: 删除按钮
 */
export function InsertedMessageCard({
  message,
  onRemove,
  className,
}: InsertedMessageCardProps) {
  const { t } = useTranslation();

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
  const RoleIcon = message.role === "user" ? User : Bot;
  const roleLabel =
    message.role === "user"
      ? t("compress.messageCard.user")
      : t("compress.messageCard.assistant");
  const roleClasses =
    message.role === "user"
      ? "bg-blue-500/10 text-blue-500"
      : "bg-emerald-500/10 text-emerald-500";
  const roleIconColor =
    message.role === "user" ? "text-blue-500" : "text-emerald-500";

  // 折叠/展开
  const handleToggleExpand = React.useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  return (
    <div
      data-testid="inserted-message-card"
      data-message-id={message.id}
      className={cn(
        // 绿色边框 + 背景
        "border-2 border-green-500 rounded-lg p-3 mb-2",
        "bg-green-500/5",
        className
      )}
    >
      {/* 头部: 角色标签 + 插入标识 + 删除按钮 + Token 数量 */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {/* 角色标签 */}
          <div
            className={cn(
              "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium",
              roleClasses
            )}
          >
            <RoleIcon className={cn("size-3.5", roleIconColor)} />
            <span>{roleLabel}</span>
          </div>

          {/* 插入标识徽章 */}
          <div className="inline-flex items-center gap-1 text-xs text-green-500">
            <Sparkles className="size-3" />
            <span>{t("compress.insertedCard.inserted")}</span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* 删除按钮 */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="size-6 text-muted-foreground hover:text-destructive"
                onClick={onRemove}
                data-testid="remove-inserted-button"
              >
                <X className="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="top" className="text-xs">
              {t("compress.insertedCard.removeTooltip")}
            </TooltipContent>
          </Tooltip>

          {/* Token 徽章 */}
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
}

export default InsertedMessageCard;
