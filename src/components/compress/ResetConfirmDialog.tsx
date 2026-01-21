/**
 * ResetConfirmDialog - 重置确认对话框组件
 * Story 10.8: Task 3
 *
 * AC4: 重置前显示确认对话框
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

/**
 * ResetConfirmDialog Props
 */
export interface ResetConfirmDialogProps {
  /** 对话框是否打开 */
  open: boolean;
  /** 打开状态变更回调 */
  onOpenChange: (open: boolean) => void;
  /** 确认重置回调 */
  onConfirm: () => void;
}

/**
 * ResetConfirmDialog - 重置确认对话框
 */
export function ResetConfirmDialog({
  open,
  onOpenChange,
  onConfirm,
}: ResetConfirmDialogProps) {
  const { t } = useTranslation();

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent data-testid="reset-confirm-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle>
            {t("compress.operations.resetConfirm.title")}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {t("compress.operations.resetConfirm.description")}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel data-testid="reset-cancel-button">
            {t("compress.operations.resetConfirm.cancel")}
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            data-testid="reset-confirm-button"
          >
            {t("compress.operations.resetConfirm.confirm")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

export default ResetConfirmDialog;
