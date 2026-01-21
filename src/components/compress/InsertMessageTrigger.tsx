/**
 * InsertMessageTrigger - 消息插入触发器组件
 * Story 10.5: Task 1
 *
 * AC1: 显示虚线插入热区 + Plus 图标
 * 在两条消息之间显示，点击触发插入操作
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Plus, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

/**
 * InsertMessageTrigger 组件 Props
 */
export interface InsertMessageTriggerProps {
  /** 插入位置 (在该 index 之后插入) */
  afterIndex: number;
  /** 是否已有插入内容 */
  hasInsertion: boolean;
  /** 点击触发回调 */
  onClick: () => void;
  /** 删除插入回调 (hasInsertion 为 true 时使用) */
  onRemoveInsertion?: () => void;
  /** 自定义 className */
  className?: string;
}

/**
 * InsertMessageTrigger - 插入热区触发器
 *
 * AC1: 悬停时显示虚线热区 + Plus 图标
 * - 默认状态: 透明占位 (高度约 8px)
 * - 悬停状态: 扩展为 24px，显示虚线边框
 * - 已插入状态: 显示删除按钮
 */
export function InsertMessageTrigger({
  afterIndex,
  hasInsertion,
  onClick,
  onRemoveInsertion,
  className,
}: InsertMessageTriggerProps) {
  const { t } = useTranslation();
  const [isHovered, setIsHovered] = React.useState(false);

  // 已有插入的样式
  if (hasInsertion) {
    return (
      <div
        data-testid="insert-message-trigger"
        data-after-index={afterIndex}
        data-has-insertion="true"
        className={cn(
          // 已插入状态样式
          "h-6 bg-green-500/10 border border-green-500/30 rounded",
          "flex items-center justify-between px-2",
          "text-green-600 text-xs",
          className
        )}
      >
        <span>{t("compress.insertTrigger.hasInsertion")}</span>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="size-5 text-green-600 hover:text-red-500 hover:bg-red-500/10"
              onClick={onRemoveInsertion}
              data-testid="remove-insertion-button"
            >
              <X className="size-3.5" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="top" className="text-xs">
            {t("compress.insertTrigger.removeTooltip")}
          </TooltipContent>
        </Tooltip>
      </div>
    );
  }

  // 默认/悬停状态
  // AC1: 默认状态透明占位，悬停时才显示虚线边框和图标
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div
          data-testid="insert-message-trigger"
          data-after-index={afterIndex}
          data-has-insertion="false"
          role="button"
          tabIndex={0}
          onClick={onClick}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              onClick();
            }
          }}
          onMouseEnter={() => setIsHovered(true)}
          onMouseLeave={() => setIsHovered(false)}
          onFocus={() => setIsHovered(true)}
          onBlur={() => setIsHovered(false)}
          className={cn(
            // 基础样式
            "w-full cursor-pointer rounded",
            "transition-all duration-150",
            // 默认状态: 透明占位 (高度约 8px)
            !isHovered && "h-2",
            // 悬停状态: 扩展为 24px，显示虚线边框 + 背景色变化
            isHovered && [
              "h-6 border-2 border-dashed border-muted-foreground/40",
              "bg-muted/30",
              "flex items-center justify-center gap-1",
              "text-muted-foreground text-xs",
            ],
            className
          )}
        >
          {isHovered && (
            <>
              <Plus className="size-3.5" />
              <span>{t("compress.insertTrigger.tooltip")}</span>
            </>
          )}
        </div>
      </TooltipTrigger>
      {!isHovered && (
        <TooltipContent side="top" className="text-xs">
          {t("compress.insertTrigger.tooltip")}
        </TooltipContent>
      )}
    </Tooltip>
  );
}

export default InsertMessageTrigger;
