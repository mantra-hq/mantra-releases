/**
 * RemoveProjectDialog Component - 移除项目确认对话框
 * Story 2.19: Task 2
 * Story 2.26: 国际化支持
 *
 * 确认移除项目的对话框，明确说明不会影响原始代码项目
 */

import { useTranslation } from "react-i18next";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogAction,
  AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/**
 * RemoveProjectDialog Props
 */
export interface RemoveProjectDialogProps {
  /** 是否打开 */
  isOpen: boolean;
  /** 打开状态变化 */
  onOpenChange: (open: boolean) => void;
  /** 项目名称（用于显示） */
  projectName: string;
  /** 确认移除回调 */
  onConfirm: () => void;
}

/**
 * RemoveProjectDialog 组件
 * 确认移除项目的对话框
 */
export function RemoveProjectDialog({
  isOpen,
  onOpenChange,
  projectName,
  onConfirm,
}: RemoveProjectDialogProps) {
  const { t } = useTranslation();

  const handleConfirm = () => {
    onConfirm();
    onOpenChange(false);
  };

  return (
    <AlertDialog open={isOpen} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          {/* AC13: 标题包含警告图标 */}
          <AlertDialogTitle>⚠️ {t("project.removeFromMantra")}</AlertDialogTitle>
          <AlertDialogDescription>
            {/* AC14: 明确说明不会影响原始代码项目 */}
            {t("project.removeConfirm", { name: projectName })}
            <br />
            <br />
            {t("project.removeDescription")}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
          {/* AC15: 确认按钮使用 destructive 样式 */}
          <AlertDialogAction
            onClick={handleConfirm}
            className={cn(buttonVariants({ variant: "destructive" }))}
          >
            {t("project.removeProject")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
