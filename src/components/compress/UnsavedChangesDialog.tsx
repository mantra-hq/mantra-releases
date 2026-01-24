/**
 * UnsavedChangesDialog - 未保存更改确认对话框
 * Story 10.9: Task 4
 *
 * 当用户尝试离开有未保存编辑的页面时显示确认对话框
 * 提供三个选项：导出并离开、不保存、取消
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { AlertTriangle } from "lucide-react";

export interface UnsavedChangesDialogProps {
  /** 对话框是否打开 */
  open: boolean;
  /** 打开状态变更回调 */
  onOpenChange: (open: boolean) => void;
  /** 点击「导出并离开」回调 */
  onExportAndLeave: () => void;
  /** 点击「不保存」回调 */
  onDiscardAndLeave: () => void;
  /** 点击「取消」回调 */
  onCancel: () => void;
}

/**
 * 未保存更改确认对话框
 *
 * 使用 shadcn/ui AlertDialog 组件实现
 * 按钮布局：取消 | 不保存 | 导出并离开
 */
export function UnsavedChangesDialog({
  open,
  onOpenChange,
  onExportAndLeave,
  onDiscardAndLeave,
  onCancel,
}: UnsavedChangesDialogProps) {
  const { t } = useTranslation();

  const handleCancel = React.useCallback(() => {
    onCancel();
    onOpenChange(false);
  }, [onCancel, onOpenChange]);

  const handleDiscard = React.useCallback(() => {
    onDiscardAndLeave();
    onOpenChange(false);
  }, [onDiscardAndLeave, onOpenChange]);

  const handleExport = React.useCallback(() => {
    onExportAndLeave();
    // 注意：不在这里关闭对话框，由调用方在导出完成后关闭
  }, [onExportAndLeave]);

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-warning" />
            <AlertDialogTitle>
              {t("compress.unsavedChanges.title", "有未保存的编辑")}
            </AlertDialogTitle>
          </div>
          <AlertDialogDescription>
            {t(
              "compress.unsavedChanges.description",
              "您对消息的编辑尚未导出。离开后这些更改将会丢失。"
            )}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel asChild>
            <Button variant="outline" onClick={handleCancel} data-testid="unsaved-cancel-button">
              {t("compress.unsavedChanges.cancel", "取消")}
            </Button>
          </AlertDialogCancel>
          <Button
            variant="ghost"
            onClick={handleDiscard}
            data-testid="unsaved-discard-button"
          >
            {t("compress.unsavedChanges.discard", "不保存")}
          </Button>
          <AlertDialogAction asChild>
            <Button onClick={handleExport} data-testid="unsaved-export-button">
              {t("compress.unsavedChanges.exportAndLeave", "导出并离开")}
            </Button>
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

export default UnsavedChangesDialog;
