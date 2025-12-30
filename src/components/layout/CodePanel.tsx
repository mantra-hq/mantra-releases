/**
 * CodePanel - 代码快照面板 (占位组件)
 * Story 2.2: Task 4.2
 *
 * 右侧面板，显示代码变更和文件快照
 */

import * as React from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Code2 } from "lucide-react";
import { cn } from "@/lib/utils";

export interface CodePanelProps {
  /** 自定义 className */
  className?: string;
  /** 子内容 */
  children?: React.ReactNode;
}

export function CodePanel({ className, children }: CodePanelProps) {
  // 如果有子内容，直接渲染
  if (children) {
    return (
      <ScrollArea className={cn("h-full", className)}>
        {children}
      </ScrollArea>
    );
  }

  // 占位内容
  return (
    <div
      className={cn(
        "h-full flex flex-col items-center justify-center",
        "text-muted-foreground",
        className
      )}
    >
      <div className="flex flex-col items-center gap-4 p-8 text-center">
        <div className="rounded-full bg-muted p-4">
          <Code2 className="size-8" />
        </div>
        <div className="space-y-2">
          <h3 className="text-lg font-semibold text-foreground">代码快照区域</h3>
          <p className="text-sm max-w-xs">
            这里将显示代码变更快照，支持时间旅行和版本对比
          </p>
        </div>
      </div>
    </div>
  );
}

export default CodePanel;

