/**
 * OriginalMessageCard - 原始消息卡片组件
 * Story 10.2: Task 2, Task 3
 * Story 10.4: Task 2 - 集成操作按钮和状态样式
 * Story 10.6/10-2 Fix: 修复消息类型识别和内容显示
 *
 * 显示单条消息的卡片，包含角色图标、内容摘要、Token 数量
 * 支持长内容折叠/展开
 * 支持保留/删除/修改操作按钮
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
  Pencil,
  Brain,
  Code,
  FileCode,
  Image as ImageIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { NarrativeMessage, ContentBlock } from "@/types/message";
import { estimateTokenCount, formatTokenCount } from "@/lib/token-counter";
import { 
  getMessageDisplayContent, 
  detectPrimaryContentType,
  hasContentType,
} from "@/lib/message-utils";
import { MessageActionButtons } from "./MessageActionButtons";
import type { OperationType } from "@/hooks/useCompressState";

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
  /** Story 10.4: 当前操作类型 (默认 keep) */
  currentOperation?: OperationType;
  /** Story 10.4: 点击保留按钮回调 */
  onKeepClick?: () => void;
  /** Story 10.4: 点击删除按钮回调 */
  onDeleteClick?: () => void;
  /** Story 10.4: 点击修改按钮回调 */
  onEditClick?: () => void;
  /** Story 10.4: 是否显示操作按钮 (压缩模式下显示) */
  showActionButtons?: boolean;
  /** Story 10.10: 是否获得键盘焦点 (AC3) */
  isFocused?: boolean;
  /** Story 10.10: 点击卡片回调 (用于设置焦点) */
  onClick?: () => void;
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
 * 获取工具名称 (从 tool_use 块中提取)
 */
function getToolName(content: ContentBlock[]): string | undefined {
  const toolUseBlock = content.find((block) => block.type === "tool_use");
  return toolUseBlock?.toolName || toolUseBlock?.displayName;
}

/**
 * 角色图标和样式配置
 * Story 10.6/10-2 Fix: 扩展支持 thinking, code_diff, code_suggestion, image 类型
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
  // Story 10.6/10-2 Fix: 检测主要内容类型
  const primaryType = detectPrimaryContentType(content);

  // 按优先级检查内容类型
  switch (primaryType) {
    case "tool_use":
      return {
        icon: Hammer,
        label: t("compress.messageCard.toolCall"),
        badgeClasses: "bg-amber-500/10 text-amber-500",
        iconColor: "text-amber-500",
      };

    case "tool_result":
      return {
        icon: ClipboardList,
        label: t("compress.messageCard.toolResult"),
        badgeClasses: "bg-purple-500/10 text-purple-500",
        iconColor: "text-purple-500",
      };

    case "thinking":
      return {
        icon: Brain,
        label: t("compress.messageCard.thinking"),
        badgeClasses: "bg-cyan-500/10 text-cyan-500",
        iconColor: "text-cyan-500",
      };

    case "code_diff":
      return {
        icon: Code,
        label: t("compress.messageCard.codeDiff"),
        badgeClasses: "bg-orange-500/10 text-orange-500",
        iconColor: "text-orange-500",
      };

    case "code_suggestion":
      return {
        icon: FileCode,
        label: t("compress.messageCard.codeSuggestion"),
        badgeClasses: "bg-teal-500/10 text-teal-500",
        iconColor: "text-teal-500",
      };

    case "image":
      return {
        icon: ImageIcon,
        label: t("compress.messageCard.image"),
        badgeClasses: "bg-pink-500/10 text-pink-500",
        iconColor: "text-pink-500",
      };

    case "reference":
      return {
        icon: FileCode,
        label: t("compress.messageCard.reference"),
        badgeClasses: "bg-indigo-500/10 text-indigo-500",
        iconColor: "text-indigo-500",
      };
  }

  // 默认按角色判断
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
 * Story 10.4: 操作按钮和状态样式
 */
export const OriginalMessageCard = React.forwardRef<
  HTMLDivElement,
  OriginalMessageCardProps
>(({
  message,
  measureElement,
  index,
  className,
  currentOperation = "keep",
  onKeepClick,
  onDeleteClick,
  onEditClick,
  showActionButtons = false,
  isFocused = false,
  onClick,
}, ref) => {
  const { t } = useTranslation();

  // AC3: 折叠状态
  const [isExpanded, setIsExpanded] = React.useState(false);

  // Story 10.6/10-2 Fix: 获取消息显示内容 (支持所有内容类型)
  const textContent = React.useMemo(
    () => getMessageDisplayContent(message.content),
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
  const toolName = hasContentType(message.content, "tool_use")
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
      data-operation={currentOperation}
      data-focused={isFocused}
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => {
        // 阻止 Enter/Space 触发点击，让全局快捷键处理
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
        }
      }}
      className={cn(
        // 基础卡片样式
        "border rounded-lg p-3 mb-2",
        "bg-card hover:bg-accent/50 transition-colors",
        // Story 10.10: 焦点状态样式 (AC3) - 键盘焦点高亮
        isFocused && [
          "ring-2 ring-primary ring-offset-2 ring-offset-background",
          "bg-accent/30",
        ],
        // Story 10.4: 删除状态样式 (AC2)
        currentOperation === "delete" && [
          "border-2 border-red-500/50",
          "bg-red-500/5 opacity-50",
        ],
        // Story 10.4: 修改状态样式 (AC4)
        currentOperation === "modify" && [
          "border-2 border-yellow-500",
          "bg-yellow-500/5",
        ],
        // Story 10.10: 可点击样式
        "cursor-pointer",
        // 焦点轮廓 (原生 focus 样式移除，使用自定义)
        "outline-none focus:outline-none",
        className
      )}
    >
      {/* 头部: 角色标签 + 操作按钮 + Token 数量 */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
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

          {/* Story 10.4: 修改状态标识 */}
          {currentOperation === "modify" && (
            <div className="inline-flex items-center gap-1 text-xs text-yellow-500">
              <Pencil className="size-3" />
              <span>{t("compress.actions.edited")}</span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-2">
          {/* Story 10.4: 操作按钮 (AC1) */}
          {showActionButtons && onKeepClick && onDeleteClick && onEditClick && (
            <MessageActionButtons
              messageId={message.id}
              currentOperation={currentOperation}
              onKeepClick={onKeepClick}
              onDeleteClick={onDeleteClick}
              onEditClick={onEditClick}
            />
          )}

          {/* Token 徽章 - 使用格式化显示 */}
          <Badge variant="outline" className="text-xs">
            {formatTokenCount(tokenCount)} {t("compress.messageCard.tokens")}
          </Badge>
        </div>
      </div>

      {/* 内容区域 - Story 10.4: 删除状态添加删除线 (AC2) */}
      <div
        className={cn(
          "text-sm text-foreground/90 whitespace-pre-wrap break-words",
          currentOperation === "delete" && "line-through text-muted-foreground"
        )}
      >
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
