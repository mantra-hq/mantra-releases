/**
 * MessageActionButtons - 消息操作按钮组
 * Story 10.4: Task 1
 *
 * 提供保留/删除/修改三态操作按钮
 * AC1: 显示操作按钮组
 * AC2: 删除操作状态切换
 * AC5: 保留操作恢复原始状态
 */

import { useTranslation } from "react-i18next";
import { Check, Trash2, Pencil, Plus } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { OperationType } from "@/hooks/useCompressState";

/**
 * MessageActionButtons 组件 Props
 */
export interface MessageActionButtonsProps {
  /** 消息 ID */
  messageId: string;
  /** 当前操作类型 */
  currentOperation: OperationType;
  /** 点击保留按钮回调 */
  onKeepClick: () => void;
  /** 点击删除按钮回调 */
  onDeleteClick: () => void;
  /** 点击修改按钮回调 */
  onEditClick: () => void;
  /** 点击插入按钮回调 */
  onInsertClick?: () => void;
  /** 是否是最后一条消息 */
  isLastMessage?: boolean;
  /** 该位置是否已有插入 */
  hasInsertion?: boolean;
  /** 自定义 className */
  className?: string;
}

// 按钮样式配置
const buttonBaseClasses = "size-7 p-0";

// 保留按钮 (激活时)
const keepActiveClasses = "bg-green-500/20 text-green-500 hover:bg-green-500/30";

// 删除按钮 (激活时)
const deleteActiveClasses = "bg-red-500/20 text-red-500 hover:bg-red-500/30";

// 修改按钮 (激活时)
const modifyActiveClasses = "bg-yellow-500/20 text-yellow-500 hover:bg-yellow-500/30";

// 插入按钮 (已有插入时)
const insertActiveClasses = "bg-green-500/20 text-green-500 hover:bg-green-500/30";

// 未激活按钮
const inactiveClasses = "text-muted-foreground hover:text-foreground hover:bg-muted";

/**
 * MessageActionButtons - 操作按钮组
 *
 * 包含三个按钮: 保留 (Check) / 删除 (Trash2) / 修改 (Pencil)
 * 根据当前操作状态高亮对应按钮
 */
export function MessageActionButtons({
  messageId,
  currentOperation,
  onKeepClick,
  onDeleteClick,
  onEditClick,
  onInsertClick,
  isLastMessage = false,
  hasInsertion = false,
  className,
}: MessageActionButtonsProps) {
  const { t } = useTranslation();

  // 判断当前激活状态
  const isKeepActive = currentOperation === "keep";
  const isDeleteActive = currentOperation === "delete";
  const isModifyActive = currentOperation === "modify";

  // 插入按钮的 Tooltip 文案
  const insertTooltip = hasInsertion
    ? t("compress.actions.editInsertedTooltip")
    : isLastMessage
      ? t("compress.actions.insertAtEndTooltip")
      : t("compress.actions.insertAfterTooltip");

  return (
    <div
      className={cn("flex items-center gap-1", className)}
      data-testid="message-action-buttons"
      data-message-id={messageId}
    >
      {/* 保留按钮 */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            onClick={onKeepClick}
            className={cn(
              buttonBaseClasses,
              isKeepActive ? keepActiveClasses : inactiveClasses
            )}
            data-testid="action-keep"
            aria-pressed={isKeepActive}
            aria-label={t("compress.actions.keep")}
          >
            <Check className="size-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
          <p>{t("compress.actions.keepTooltip")}</p>
        </TooltipContent>
      </Tooltip>

      {/* 删除按钮 */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            onClick={onDeleteClick}
            className={cn(
              buttonBaseClasses,
              isDeleteActive ? deleteActiveClasses : inactiveClasses
            )}
            data-testid="action-delete"
            aria-pressed={isDeleteActive}
            aria-label={t("compress.actions.delete")}
          >
            <Trash2 className="size-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
          <p>{t("compress.actions.deleteTooltip")}</p>
        </TooltipContent>
      </Tooltip>

      {/* 修改按钮 */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            onClick={onEditClick}
            className={cn(
              buttonBaseClasses,
              isModifyActive ? modifyActiveClasses : inactiveClasses
            )}
            data-testid="action-edit"
            aria-pressed={isModifyActive}
            aria-label={t("compress.actions.edit")}
          >
            <Pencil className="size-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
          <p>{t("compress.actions.editTooltip")}</p>
        </TooltipContent>
      </Tooltip>

      {/* 插入按钮 */}
      {onInsertClick && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={onInsertClick}
              className={cn(
                buttonBaseClasses,
                hasInsertion ? insertActiveClasses : inactiveClasses
              )}
              data-testid="action-insert"
              aria-label={t("compress.actions.insert")}
            >
              <Plus className="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{insertTooltip}</p>
          </TooltipContent>
        </Tooltip>
      )}
    </div>
  );
}

export default MessageActionButtons;
