/**
 * Editor Components - 代码编辑器相关组件导出
 * Story 2.5: Task 5.1
 * Story 2.13: Task 2
 */

export { CodeSnapshotView, getLanguageFromPath } from "./CodeSnapshotView";
export type { CodeSnapshotViewProps } from "./CodeSnapshotView";

export { CodeSnapshotHeader } from "./CodeSnapshotHeader";
export type { CodeSnapshotHeaderProps } from "./CodeSnapshotHeader";

export { EmptyCodeState } from "./EmptyCodeState";
export type { EmptyCodeStateProps } from "./EmptyCodeState";

export { NoGitWarning } from "./NoGitWarning";
export type { NoGitWarningProps } from "./NoGitWarning";

export { EditorTabs } from "./EditorTabs";
export type { EditorTabsProps } from "./EditorTabs";

export { Breadcrumbs } from "./Breadcrumbs";
export type { BreadcrumbsProps, SiblingItem } from "./Breadcrumbs";

export { FileTree } from "./FileTree";
export type { FileTreeProps, TreeNode } from "./FileTree";

export { QuickOpen } from "./QuickOpen";
export type { QuickOpenProps } from "./QuickOpen";

export { DiffModeToggle } from "./DiffModeToggle";
export type { DiffModeToggleProps } from "./DiffModeToggle";
