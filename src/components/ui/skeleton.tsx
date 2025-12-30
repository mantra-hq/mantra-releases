/**
 * Skeleton Component - 骨架屏组件
 * Story 2.8: Task 8
 *
 * 加载状态的占位组件
 */

import * as React from "react";
import { cn } from "@/lib/utils";

function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("animate-pulse rounded-md bg-muted", className)}
      {...props}
    />
  );
}

export { Skeleton };

