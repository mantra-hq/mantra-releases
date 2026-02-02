import * as React from "react";
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "./sheet";
import { cn } from "@/lib/utils";

/**
 * 尺寸映射
 * 基于 Epic 12 已完成改造中观察到的 Sheet 使用模式
 */
const sizeClasses = {
  sm: "max-w-sm", // 384px - 简单表单 (EnvVariableSheet)
  md: "max-w-md", // 448px - 中等表单 (CompressGuideSheet, AddRuleSheet, BindSessionSheet)
  lg: "max-w-lg", // 512px - 复杂表单 (OAuthConfigSheet, McpServiceSheet, ProjectInfoSheet)
  xl: "max-w-xl", // 576px - 带预览的表单 (TakeoverStatusCard)
  "2xl": "max-w-2xl", // 672px - 多步向导 (McpConfigImportSheet)
} as const;

/**
 * ActionSheet 尺寸类型
 */
type ActionSheetSize = keyof typeof sizeClasses;

/**
 * ActionSheet 根组件 Props
 */
type ActionSheetProps = React.ComponentProps<typeof Sheet>;

/**
 * ActionSheetContent 组件 Props
 */
interface ActionSheetContentProps
  extends Omit<React.ComponentProps<typeof SheetContent>, "side"> {
  /**
   * 预设尺寸
   * @default "md"
   */
  size?: ActionSheetSize;
}

/**
 * ActionSheet - 统一封装的右侧抽屉组件
 *
 * 默认从右侧滑出，提供预设尺寸选项，简化常见的 Sheet 使用模式。
 *
 * @example
 * ```tsx
 * <ActionSheet open={open} onOpenChange={setOpen}>
 *   <ActionSheetContent size="lg">
 *     <ActionSheetHeader>
 *       <ActionSheetTitle>标题</ActionSheetTitle>
 *       <ActionSheetDescription>描述</ActionSheetDescription>
 *     </ActionSheetHeader>
 *     {/* 内容 *\/}
 *     <ActionSheetFooter>
 *       <Button>确认</Button>
 *     </ActionSheetFooter>
 *   </ActionSheetContent>
 * </ActionSheet>
 * ```
 */
function ActionSheet({ ...props }: ActionSheetProps) {
  return <Sheet {...props} />;
}

/**
 * ActionSheetContent - 内容容器
 *
 * 固定从右侧滑出 (side="right")，支持预设尺寸。
 * 自动应用 w-full 确保在小屏幕上响应式。
 */
function ActionSheetContent({
  size = "md",
  className,
  children,
  ...props
}: ActionSheetContentProps) {
  return (
    <SheetContent
      side="right"
      className={cn("w-full", sizeClasses[size], className)}
      {...props}
    >
      {children}
    </SheetContent>
  );
}

/**
 * ActionSheetHeader - 头部容器
 * 直接透传 SheetHeader
 */
const ActionSheetHeader = SheetHeader;

/**
 * ActionSheetFooter - 底部容器
 * 直接透传 SheetFooter
 */
const ActionSheetFooter = SheetFooter;

/**
 * ActionSheetTitle - 标题组件
 * 直接透传 SheetTitle
 */
const ActionSheetTitle = SheetTitle;

/**
 * ActionSheetDescription - 描述组件
 * 直接透传 SheetDescription
 */
const ActionSheetDescription = SheetDescription;

/**
 * ActionSheetClose - 关闭按钮
 * 直接透传 SheetClose
 */
const ActionSheetClose = SheetClose;

export {
  ActionSheet,
  ActionSheetContent,
  ActionSheetHeader,
  ActionSheetFooter,
  ActionSheetTitle,
  ActionSheetDescription,
  ActionSheetClose,
  type ActionSheetProps,
  type ActionSheetSize,
  type ActionSheetContentProps,
};
