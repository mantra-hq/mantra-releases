/**
 * EditMessageSheet - 消息编辑 Sheet
 * Story 10.4: Task 3
 * Story 12.1: Task 3 - Dialog → Sheet 改造
 * Story 12.4: 迁移使用 ActionSheet 统一封装组件
 *
 * AC3: 弹出编辑 Sheet，显示原始内容和可编辑区域
 * AC4: 实时显示 Token 变化，确认后更新状态
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import {
  ActionSheet,
  ActionSheetContent,
  ActionSheetHeader,
  ActionSheetTitle,
  ActionSheetDescription,
  ActionSheetFooter,
} from "@/components/ui/action-sheet";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { NarrativeMessage } from "@/types/message";
import { estimateTokenCount } from "@/lib/token-counter";
import { getMessageDisplayContent } from "@/lib/message-utils";

/**
 * EditMessageSheet 组件 Props
 * Story 12.1: 重命名 Props 接口
 */
export interface EditMessageSheetProps {
  /** Sheet 是否打开 */
  open: boolean;
  /** Sheet 状态变化回调 */
  onOpenChange: (open: boolean) => void;
  /** 要编辑的消息 */
  message: NarrativeMessage | null;
  /** 确认修改回调 */
  onConfirm: (modifiedContent: string) => void;
}

/**
 * EditMessageSheet - 编辑消息 Sheet
 * Story 12.1: Dialog → Sheet 改造
 *
 * 包含:
 * - 原始内容显示 (只读)
 * - 可编辑的修改区域
 * - 实时 Token 变化显示
 * - 取消和确认按钮
 */
export function EditMessageSheet({
  open,
  onOpenChange,
  message,
  onConfirm,
}: EditMessageSheetProps) {
  const { t } = useTranslation();

  // 获取原始内容 (Story 10.6 Fix: 使用 getMessageDisplayContent 支持所有内容类型)
  const originalContent = React.useMemo(() => {
    if (!message) return "";
    return getMessageDisplayContent(message.content);
  }, [message]);

  // 修改后的内容状态
  const [modifiedContent, setModifiedContent] = React.useState("");

  // Token 计数状态
  const [originalTokens, setOriginalTokens] = React.useState(0);
  const [modifiedTokens, setModifiedTokens] = React.useState(0);
  const [isCalculatingTokens, setIsCalculatingTokens] = React.useState(false);

  // 当消息变化时重置状态 (Story 10.6 Fix: 使用 getMessageDisplayContent 支持所有内容类型)
  React.useEffect(() => {
    if (message && open) {
      const content = getMessageDisplayContent(message.content);
      setModifiedContent(content);
      const tokens = estimateTokenCount(content);
      setOriginalTokens(tokens);
      setModifiedTokens(tokens);
    }
  }, [message, open]);

  // 使用 useEffect + setTimeout 实现 debounce Token 计算 (150ms 延迟)
  React.useEffect(() => {
    setIsCalculatingTokens(true);
    const timer = setTimeout(() => {
      setModifiedTokens(estimateTokenCount(modifiedContent));
      setIsCalculatingTokens(false);
    }, 150);
    return () => clearTimeout(timer);
  }, [modifiedContent]);

  // 处理内容变化
  const handleContentChange = React.useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      setModifiedContent(e.target.value);
    },
    []
  );

  // 计算 Token 变化量
  const tokenDelta = modifiedTokens - originalTokens;

  // 判断是否有变化
  const hasChanges = modifiedContent !== originalContent;

  // 处理确认
  const handleConfirm = React.useCallback(() => {
    if (hasChanges) {
      onConfirm(modifiedContent);
      onOpenChange(false);
    }
  }, [hasChanges, modifiedContent, onConfirm, onOpenChange]);

  // 处理取消
  const handleCancel = React.useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  // 处理键盘快捷键
  const handleKeyDown = React.useCallback(
    (e: React.KeyboardEvent) => {
      // Ctrl/Cmd + Enter 确认
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter" && hasChanges) {
        e.preventDefault();
        handleConfirm();
      }
      // Escape 取消 - Sheet 已有内置处理
    },
    [hasChanges, handleConfirm]
  );

  if (!message) return null;

  return (
    <ActionSheet open={open} onOpenChange={onOpenChange}>
      <ActionSheetContent
        size="2xl"
        className="flex flex-col"
        data-testid="edit-message-sheet"
        onKeyDown={handleKeyDown}
      >
        <ActionSheetHeader>
          <ActionSheetTitle>{t("compress.editDialog.title")}</ActionSheetTitle>
          <ActionSheetDescription className="sr-only">
            {t("compress.editDialog.description")}
          </ActionSheetDescription>
        </ActionSheetHeader>

        {/* 原始内容 - 只读 */}
        <div className="flex-shrink-0 px-4">
          <div className="text-sm text-muted-foreground mb-1">
            {t("compress.editDialog.original")}
          </div>
          <ScrollArea className="h-[20vh] rounded-md border bg-muted/50">
            <pre
              className="p-3 whitespace-pre-wrap text-sm text-muted-foreground"
              data-testid="original-content"
            >
              {originalContent}
            </pre>
          </ScrollArea>
        </div>

        {/* 修改内容 - 可编辑 */}
        <div className="flex-1 min-h-0 flex flex-col px-4">
          <div className="flex justify-between items-center mb-1">
            <span className="text-sm text-muted-foreground">
              {t("compress.editDialog.modified")}
            </span>
            <span className="text-xs text-muted-foreground" data-testid="token-display">
              {originalTokens} →{" "}
              <span className={isCalculatingTokens ? "opacity-50" : ""}>
                {modifiedTokens}
              </span>{" "}
              tokens
              {tokenDelta !== 0 && !isCalculatingTokens && (
                <span
                  className={tokenDelta < 0 ? "text-green-500 ml-1" : "text-red-500 ml-1"}
                  data-testid="token-delta"
                >
                  ({tokenDelta > 0 ? "+" : ""}{tokenDelta})
                </span>
              )}
            </span>
          </div>
          <Textarea
            value={modifiedContent}
            onChange={handleContentChange}
            className="flex-1 min-h-[20vh] resize-none font-mono text-sm"
            placeholder={t("compress.editDialog.placeholder")}
            data-testid="modified-content-input"
          />
        </div>

        <ActionSheetFooter className="px-4">
          <Button
            variant="outline"
            onClick={handleCancel}
            data-testid="cancel-button"
          >
            {t("compress.editDialog.cancel")}
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={!hasChanges}
            data-testid="confirm-button"
          >
            {t("compress.editDialog.confirm")}
          </Button>
        </ActionSheetFooter>
      </ActionSheetContent>
    </ActionSheet>
  );
}

export default EditMessageSheet;
