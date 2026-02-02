/**
 * shadcn/ui Component Exports
 * Centralized export for all UI primitives
 */

export { Button, buttonVariants } from "./button";
export {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogOverlay,
  AlertDialogPortal,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "./alert-dialog";
export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
} from "./dialog";
export {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "./dropdown-menu";
export { ScrollArea, ScrollBar } from "./scroll-area";
export {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./tooltip";

export {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "./resizable";

export { Input, type InputProps } from "./input";
export { Skeleton } from "./skeleton";
export { Checkbox } from "./checkbox";
export { Progress } from "./progress";

/**
 * ActionSheet - 统一封装的右侧抽屉组件
 *
 * 提供预设尺寸 (sm/md/lg/xl/2xl) 和标准结构，
 * 默认 side="right"，简化 Sheet 的常见使用模式。
 *
 * @example
 * ```tsx
 * import { ActionSheet, ActionSheetContent, ActionSheetHeader, ActionSheetTitle } from "@/components/ui";
 *
 * <ActionSheet open={open} onOpenChange={setOpen}>
 *   <ActionSheetContent size="lg">
 *     <ActionSheetHeader>
 *       <ActionSheetTitle>标题</ActionSheetTitle>
 *     </ActionSheetHeader>
 *     {content}
 *   </ActionSheetContent>
 * </ActionSheet>
 * ```
 */
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
} from "./action-sheet";
