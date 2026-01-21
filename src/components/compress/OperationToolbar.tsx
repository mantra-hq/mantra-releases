/**
 * OperationToolbar - 操作工具栏组件
 * Story 10.8: Task 2
 *
 * 提供撤销/重做/重置操作按钮
 * AC1: Icon-only 按钮 + Tooltip
 * AC5: 根据状态控制禁用
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Undo2, Redo2, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useCompressState } from "@/hooks/useCompressState";
import { ResetConfirmDialog } from "./ResetConfirmDialog";

/**
 * OperationToolbar Props
 */
export interface OperationToolbarProps {
  /** 自定义 className */
  className?: string;
}

/**
 * 检测是否为 Mac 平台
 */
function usePlatform(): "mac" | "other" {
  const [platform, setPlatform] = React.useState<"mac" | "other">("other");

  React.useEffect(() => {
    const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
    setPlatform(isMac ? "mac" : "other");
  }, []);

  return platform;
}

/**
 * OperationToolbar - 撤销/重做/重置工具栏
 */
export function OperationToolbar({ className }: OperationToolbarProps) {
  const { t } = useTranslation();
  const platform = usePlatform();
  const { undo, redo, resetAll, canUndo, canRedo, hasAnyChanges } = useCompressState();

  // 重置确认对话框状态
  const [resetDialogOpen, setResetDialogOpen] = React.useState(false);

  // Task 4: 键盘快捷键 (AC2, AC3)
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // 检查是否在输入框中
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
      const modKey = isMac ? e.metaKey : e.ctrlKey;

      if (modKey && e.key === "z" && !e.shiftKey) {
        e.preventDefault();
        if (canUndo) {
          undo();
        }
      } else if (modKey && e.key === "z" && e.shiftKey) {
        e.preventDefault();
        if (canRedo) {
          redo();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [canUndo, canRedo, undo, redo]);

  // 快捷键显示文本
  const undoShortcut = platform === "mac" ? "⌘Z" : "Ctrl+Z";
  const redoShortcut = platform === "mac" ? "⌘⇧Z" : "Ctrl+Shift+Z";

  // 处理重置确认
  const handleResetConfirm = React.useCallback(() => {
    resetAll();
    setResetDialogOpen(false);
  }, [resetAll]);

  return (
    <>
      <div className={cn("flex items-center gap-1", className)}>
        <TooltipProvider delayDuration={300}>
          {/* 撤销按钮 */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={undo}
                disabled={!canUndo}
                className="size-8"
                data-testid="undo-button"
              >
                <Undo2 className="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>
                {t("compress.operations.undo")} ({undoShortcut})
              </p>
            </TooltipContent>
          </Tooltip>

          {/* 重做按钮 */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={redo}
                disabled={!canRedo}
                className="size-8"
                data-testid="redo-button"
              >
                <Redo2 className="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>
                {t("compress.operations.redo")} ({redoShortcut})
              </p>
            </TooltipContent>
          </Tooltip>

          {/* 重置按钮 */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setResetDialogOpen(true)}
                disabled={!hasAnyChanges}
                className="size-8"
                data-testid="reset-button"
              >
                <RotateCcw className="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>{t("compress.operations.reset")}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>

      {/* 重置确认对话框 */}
      <ResetConfirmDialog
        open={resetDialogOpen}
        onOpenChange={setResetDialogOpen}
        onConfirm={handleResetConfirm}
      />
    </>
  );
}

export default OperationToolbar;
