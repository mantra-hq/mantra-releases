/**
 * EmptyCodeState - 代码区空状态组件
 * Story 2.5: Task 4
 * Story 2.26: 国际化支持
 *
 * 当没有选择代码查看时显示友好的空状态提示
 */

import { useTranslation } from "react-i18next";
import { Code2, MousePointerClick } from "lucide-react";
import { cn } from "@/lib/utils";

export interface EmptyCodeStateProps {
  /** 自定义 className */
  className?: string;
}

/**
 * 空状态组件
 *
 * 功能:
 * - 设计空状态 UI (图标 + 标题 + 说明) (Task 4.2)
 * - 提供操作引导 ("点击左侧对话查看对应代码") (Task 4.3)
 */
export function EmptyCodeState({ className }: EmptyCodeStateProps) {
  const { t } = useTranslation();

  return (
    <div
      className={cn(
        "flex h-full flex-col items-center justify-center",
        "p-8 text-center",
        className
      )}
    >
      {/* 图标 */}
      <div className="mb-4 rounded-full bg-muted/50 p-4">
        <Code2 className="size-12 text-muted-foreground/50" />
      </div>

      {/* 标题 */}
      <h3 className="mb-2 text-base font-semibold text-foreground">
        {t("editor.noCode")}
      </h3>

      {/* 说明文字 */}
      <p className="mb-4 max-w-[280px] text-sm text-muted-foreground">
        {t("editor.selectMessage")}
      </p>

      {/* 操作引导 */}
      <div
        className={cn(
          "flex items-center gap-2",
          "rounded-md bg-muted/30 px-3 py-2",
          "text-xs text-muted-foreground"
        )}
      >
        <MousePointerClick className="size-4" />
        <span>{t("editor.clickLeftMessage")}</span>
      </div>
    </div>
  );
}

export default EmptyCodeState;

