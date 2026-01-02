/**
 * BreadcrumbItem Component - 面包屑项组件
 * Story 2.17: Task 2
 *
 * 支持图标 + 文字 + 可点击，响应式截断
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

/**
 * BreadcrumbItem Props
 */
export interface BreadcrumbItemProps {
  /** 图标 */
  icon?: React.ReactNode;
  /** 标签文字 */
  label?: string;
  /** 点击回调 */
  onClick?: () => void;
  /** 是否截断长文本 (AC13, AC14) */
  truncate?: boolean;
  /** aria-label */
  "aria-label"?: string;
  /** 测试 ID */
  testId?: string;
  /** 额外的类名 */
  className?: string;
  /** 子元素 */
  children?: React.ReactNode;
}

/**
 * BreadcrumbItem 组件
 * 面包屑导航项，支持图标 + 文字 + 点击交互
 */
export function BreadcrumbItem({
  icon,
  label,
  onClick,
  truncate = false,
  "aria-label": ariaLabel,
  testId,
  className,
  children,
}: BreadcrumbItemProps) {
  const content = (
    <>
      {icon && <span className="shrink-0">{icon}</span>}
      {label && (
        <span
          className={cn(
            truncate &&
              "truncate max-w-[120px] md:max-w-[200px] lg:max-w-none"
          )}
        >
          {label}
        </span>
      )}
      {children}
    </>
  );

  if (onClick) {
    return (
      <Button
        variant="ghost"
        size="sm"
        onClick={onClick}
        aria-label={ariaLabel}
        data-testid={testId}
        className={cn(
          "h-8 px-2 gap-1.5",
          "text-muted-foreground hover:text-foreground",
          "transition-colors",
          className
        )}
      >
        {content}
      </Button>
    );
  }

  return (
    <span
      data-testid={testId}
      className={cn(
        "flex items-center gap-1.5 h-8 px-2",
        "text-muted-foreground",
        className
      )}
    >
      {content}
    </span>
  );
}
