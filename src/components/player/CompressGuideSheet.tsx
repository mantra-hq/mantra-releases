/**
 * CompressGuideSheet - 压缩模式首次使用引导面板
 * Story 10.1: AC #2
 * Story 12.3: Dialog → Sheet 改造
 *
 * 首次切换到压缩模式时显示引导提示
 * - 说明功能用途
 * - 提供"知道了，不再提示"选项
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Minimize2, Trash2, Edit3, PlusCircle, BarChart3 } from "lucide-react";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";

export interface CompressGuideSheetProps {
  /** 是否打开面板 */
  open: boolean;
  /** 关闭面板回调（临时隐藏，下次还会显示） */
  onClose: () => void;
  /** 关闭并不再提示回调 */
  onDismissForever: () => void;
}

/**
 * CompressGuideSheet 组件
 * 显示压缩模式功能引导
 */
export function CompressGuideSheet({
  open,
  onClose,
  onDismissForever,
}: CompressGuideSheetProps) {
  const { t } = useTranslation();
  const [dontShowAgain, setDontShowAgain] = React.useState(false);

  const handleClose = React.useCallback(() => {
    if (dontShowAgain) {
      onDismissForever();
    } else {
      onClose();
    }
  }, [dontShowAgain, onClose, onDismissForever]);

  return (
    <Sheet open={open} onOpenChange={(isOpen) => !isOpen && handleClose()}>
      <SheetContent side="right" className="w-full max-w-md">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Minimize2 className="h-5 w-5 text-primary" />
            {t("player.compressGuide.title")}
          </SheetTitle>
          <SheetDescription>
            {t("player.compressGuide.description")}
          </SheetDescription>
        </SheetHeader>

        {/* 功能说明列表 */}
        <div className="space-y-3 py-4 px-4">
          <FeatureItem
            icon={<Trash2 className="h-4 w-4" />}
            text={t("player.compressGuide.feature1")}
          />
          <FeatureItem
            icon={<Edit3 className="h-4 w-4" />}
            text={t("player.compressGuide.feature2")}
          />
          <FeatureItem
            icon={<PlusCircle className="h-4 w-4" />}
            text={t("player.compressGuide.feature3")}
          />
          <FeatureItem
            icon={<BarChart3 className="h-4 w-4" />}
            text={t("player.compressGuide.feature4")}
          />
        </div>

        {/* 好处说明 */}
        <div className="rounded-lg bg-muted/50 p-3 mx-4 text-sm text-muted-foreground">
          <p className="font-medium text-foreground mb-1">
            {t("player.compressGuide.benefitsTitle")}
          </p>
          <ul className="list-disc list-inside space-y-1">
            <li>{t("player.compressGuide.benefit1")}</li>
            <li>{t("player.compressGuide.benefit2")}</li>
            <li>{t("player.compressGuide.benefit3")}</li>
          </ul>
        </div>

        <SheetFooter className="flex-col sm:flex-row gap-3 sm:gap-0">
          {/* 不再提示选项 */}
          <div className="flex items-center space-x-2 mr-auto">
            <Checkbox
              id="dont-show-again"
              checked={dontShowAgain}
              onCheckedChange={(checked) =>
                setDontShowAgain(checked === true)
              }
            />
            <Label
              htmlFor="dont-show-again"
              className="text-sm text-muted-foreground cursor-pointer"
            >
              {t("player.compressGuide.dontShowAgain")}
            </Label>
          </div>

          {/* 开始使用按钮 */}
          <Button onClick={handleClose}>
            {t("player.compressGuide.getStarted")}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
}

/**
 * 功能项组件
 */
interface FeatureItemProps {
  icon: React.ReactNode;
  text: string;
}

function FeatureItem({ icon, text }: FeatureItemProps) {
  return (
    <div className="flex items-start gap-3">
      <div className="flex-shrink-0 mt-0.5 text-primary">{icon}</div>
      <span className="text-sm text-foreground">{text}</span>
    </div>
  );
}

CompressGuideSheet.displayName = "CompressGuideSheet";

// 向后兼容的别名
export const RefineGuideSheet = CompressGuideSheet;
export type RefineGuideSheetProps = CompressGuideSheetProps;

// 旧名称向后兼容（将在未来版本移除）
/** @deprecated 使用 CompressGuideSheet 代替 */
export const CompressGuideDialog = CompressGuideSheet;
/** @deprecated 使用 CompressGuideSheetProps 代替 */
export type CompressGuideDialogProps = CompressGuideSheetProps;
/** @deprecated 使用 RefineGuideSheet 代替 */
export const RefineGuideDialog = CompressGuideSheet;
/** @deprecated 使用 RefineGuideSheetProps 代替 */
export type RefineGuideDialogProps = CompressGuideSheetProps;

export default CompressGuideSheet;
