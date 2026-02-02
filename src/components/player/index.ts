/**
 * Player Components Index
 * Story 2.21: Task 1
 * Story 10.1: RefineGuideSheet
 * Story 10.11: ModeSwitch 已移至 layout/ModeSwitch.tsx
 * Story 12.3: CompressGuideDialog → CompressGuideSheet
 */

export { PlayerEmptyState } from "./PlayerEmptyState";
export type { PlayerEmptyStateProps } from "./PlayerEmptyState";

// Story 10.1/12.3: 压缩模式引导面板
export { CompressGuideSheet, RefineGuideSheet } from "./CompressGuideSheet";
export type { CompressGuideSheetProps, RefineGuideSheetProps } from "./CompressGuideSheet";
// 向后兼容别名 (deprecated)
export { CompressGuideDialog, RefineGuideDialog } from "./CompressGuideSheet";
export type { CompressGuideDialogProps, RefineGuideDialogProps } from "./CompressGuideSheet";

// Story 10.11: ModeSwitch 已统一到 layout/ModeSwitch.tsx
// 如需使用，请从 @/components/layout/ModeSwitch 导入
