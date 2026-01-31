/**
 * MCP 服务状态指示灯组件
 * Story 11.9: Task 2.2 - 状态指示灯 (AC: #4)
 *
 * 根据服务状态显示不同颜色的指示灯：
 * - 🟢 绿色: 运行中
 * - ⚪ 灰色: 未运行
 * - 🔴 红色: 错误
 */

import { cn } from "@/lib/utils";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useTranslation } from "react-i18next";

export type ServiceStatus = "running" | "stopped" | "error";

export interface McpServiceStatusDotProps {
  /** 服务状态 */
  status: ServiceStatus;
  /** 错误信息 (当 status 为 error 时显示) */
  errorMessage?: string | null;
  /** 额外的 CSS 类名 */
  className?: string;
}

/**
 * 获取状态对应的样式类
 */
function getStatusStyles(status: ServiceStatus): string {
  switch (status) {
    case "running":
      return "bg-emerald-500 shadow-emerald-500/50";
    case "error":
      return "bg-red-500 shadow-red-500/50";
    case "stopped":
    default:
      return "bg-zinc-400 shadow-zinc-400/50";
  }
}

export function McpServiceStatusDot({
  status,
  errorMessage,
  className,
}: McpServiceStatusDotProps) {
  const { t } = useTranslation();

  const statusText = {
    running: t("hub.mcpContext.statusRunning", "运行中"),
    stopped: t("hub.mcpContext.statusStopped", "未运行"),
    error: t("hub.mcpContext.statusError", "错误"),
  }[status];

  const dot = (
    <span
      className={cn(
        "inline-block h-2 w-2 rounded-full shadow-[0_0_4px]",
        getStatusStyles(status),
        className
      )}
      aria-label={statusText}
    />
  );

  // 如果是错误状态且有错误信息，显示 tooltip
  if (status === "error" && errorMessage) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>{dot}</TooltipTrigger>
          <TooltipContent side="top" className="max-w-xs">
            <p className="text-xs text-red-400">{errorMessage}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  // 普通状态显示状态文本 tooltip
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>{dot}</TooltipTrigger>
        <TooltipContent side="top">
          <p className="text-xs">{statusText}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export default McpServiceStatusDot;
