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
import { Button } from "./button";
import { Maximize2, Minimize2 } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * 尺寸映射 - 合理的默认宽度
 */
const sizeClasses = {
  sm: "max-w-sm",   // 384px
  md: "max-w-md",   // 448px
  lg: "max-w-lg",   // 512px
  xl: "max-w-xl",   // 576px
  "2xl": "max-w-2xl", // 672px
  "3xl": "max-w-3xl", // 768px
  "4xl": "max-w-4xl", // 896px
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
  /**
   * 是否为全屏模式
   * @default false
   */
  fullscreen?: boolean;
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
 * 固定从右侧滑出 (side="right")，支持预设尺寸和全屏模式。
 */
function ActionSheetContent({
  size = "md",
  fullscreen = false,
  className,
  children,
  ...props
}: ActionSheetContentProps) {
  return (
    <SheetContent
      side="right"
      className={cn(
        "w-full",
        fullscreen ? "!max-w-none" : sizeClasses[size],
        className
      )}
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

/**
 * ActionSheetFullscreenToggle - 全屏切换按钮
 *
 * 放置在 Header 区域右侧，用于切换全屏/普通模式
 *
 * @example
 * ```tsx
 * const [isFullscreen, setIsFullscreen] = useState(false);
 *
 * <ActionSheetContent fullscreen={isFullscreen}>
 *   <ActionSheetHeader>
 *     <ActionSheetTitle>标题</ActionSheetTitle>
 *     <ActionSheetFullscreenToggle
 *       isFullscreen={isFullscreen}
 *       onToggle={() => setIsFullscreen(!isFullscreen)}
 *     />
 *   </ActionSheetHeader>
 * </ActionSheetContent>
 * ```
 */
interface ActionSheetFullscreenToggleProps {
  isFullscreen: boolean;
  onToggle: () => void;
  className?: string;
  enterLabel?: string;
  exitLabel?: string;
}

function ActionSheetFullscreenToggle({
  isFullscreen,
  onToggle,
  className,
  enterLabel = "进入全屏",
  exitLabel = "退出全屏",
}: ActionSheetFullscreenToggleProps) {
  return (
    <Button
      variant="ghost"
      size="icon"
      className={cn("h-8 w-8 shrink-0", className)}
      onClick={onToggle}
      title={isFullscreen ? exitLabel : enterLabel}
    >
      {isFullscreen ? (
        <Minimize2 className="h-4 w-4" />
      ) : (
        <Maximize2 className="h-4 w-4" />
      )}
    </Button>
  );
}

export {
  ActionSheet,
  ActionSheetContent,
  ActionSheetHeader,
  ActionSheetFooter,
  ActionSheetTitle,
  ActionSheetDescription,
  ActionSheetClose,
  ActionSheetFullscreenToggle,
  type ActionSheetProps,
  type ActionSheetSize,
  type ActionSheetContentProps,
  type ActionSheetFullscreenToggleProps,
};
