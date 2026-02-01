/**
 * 导入流程步骤指示器组件
 * Story 11.13: Task 6 - 增强步骤指示器 (AC: #6)
 *
 * 显示步骤名称（而非仅圆点），当前步骤高亮，已完成步骤显示勾选标记。
 */

import { useTranslation } from "react-i18next";
import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

// ===== 类型定义 =====

type ImportStep = "scan" | "preview" | "conflicts" | "env" | "confirm" | "execute" | "result";

interface ImportStepperProps {
  /** 当前步骤 */
  currentStep: ImportStep;
  /** 是否有冲突需要解决 */
  hasConflicts: boolean;
  /** 是否需要设置环境变量 */
  needsEnvVars: boolean;
}

interface StepInfo {
  id: ImportStep;
  labelKey: string;
  /** 是否可选步骤（冲突/环境变量） */
  optional?: "conflicts" | "env";
}

const STEPS: StepInfo[] = [
  { id: "preview", labelKey: "hub.import.stepSelect" },
  { id: "conflicts", labelKey: "hub.import.stepConflicts", optional: "conflicts" },
  { id: "env", labelKey: "hub.import.stepEnv", optional: "env" },
  { id: "confirm", labelKey: "hub.import.stepConfirm" },
  { id: "execute", labelKey: "hub.import.stepExecute" },
];

export function ImportStepper({
  currentStep,
  hasConflicts,
  needsEnvVars,
}: ImportStepperProps) {
  const { t } = useTranslation();

  // 过滤掉不需要的步骤
  const visibleSteps = STEPS.filter((step) => {
    if (step.optional === "conflicts" && !hasConflicts) return false;
    if (step.optional === "env" && !needsEnvVars) return false;
    return true;
  });

  // 获取步骤索引
  const currentIndex = visibleSteps.findIndex((s) => s.id === currentStep);

  return (
    <div className="flex items-center justify-center py-3" data-testid="import-stepper">
      {visibleSteps.map((step, index) => {
        const isActive = step.id === currentStep;
        const isPast = currentIndex > index;
        const isLast = index === visibleSteps.length - 1;

        return (
          <div key={step.id} className="flex items-center">
            {/* 步骤圆点/勾选 */}
            <div className="flex flex-col items-center">
              <div
                className={cn(
                  "flex items-center justify-center w-6 h-6 rounded-full text-xs font-medium transition-colors",
                  isActive && "bg-blue-500 text-white",
                  isPast && "bg-green-500 text-white",
                  !isActive && !isPast && "bg-muted text-muted-foreground"
                )}
              >
                {isPast ? (
                  <Check className="h-3.5 w-3.5" />
                ) : (
                  <span>{index + 1}</span>
                )}
              </div>
              {/* 步骤名称 */}
              <span
                className={cn(
                  "text-[10px] mt-1 whitespace-nowrap",
                  isActive && "text-blue-500 font-medium",
                  isPast && "text-green-500",
                  !isActive && !isPast && "text-muted-foreground"
                )}
              >
                {t(step.labelKey)}
              </span>
            </div>

            {/* 连接线 */}
            {!isLast && (
              <div
                className={cn(
                  "w-8 h-0.5 mx-2",
                  isPast ? "bg-green-500" : "bg-muted"
                )}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}

export default ImportStepper;
