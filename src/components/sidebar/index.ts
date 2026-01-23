/**
 * Sidebar Components
 * Story 2.18 & 2.19
 * Story 1.12: Phase 5 - 逻辑项目视图组件
 *
 * 导出所有侧边栏组件
 */

export { ProjectDrawer } from "./ProjectDrawer";
export type { ProjectDrawerProps } from "./ProjectDrawer";

export { ProjectTreeItem } from "./ProjectTreeItem";
export type { ProjectTreeItemProps } from "./ProjectTreeItem";

// Story 1.12: 逻辑项目树节点组件
export { LogicalProjectTreeItem } from "./LogicalProjectTreeItem";
export type { LogicalProjectTreeItemProps } from "./LogicalProjectTreeItem";

export { SessionTreeItem } from "./SessionTreeItem";
export type { SessionTreeItemProps } from "./SessionTreeItem";

export { DrawerSearch, HighlightText } from "./DrawerSearch";
export type { DrawerSearchProps, HighlightTextProps } from "./DrawerSearch";

// Story 2.19: Project Management Components
export { ProjectContextMenu } from "./ProjectContextMenu";
export type { ProjectContextMenuProps } from "./ProjectContextMenu";

// Story 1.12: 逻辑项目上下文菜单
export { LogicalProjectContextMenu } from "./LogicalProjectContextMenu";
export type { LogicalProjectContextMenuProps } from "./LogicalProjectContextMenu";

export { RemoveProjectDialog } from "./RemoveProjectDialog";
export type { RemoveProjectDialogProps } from "./RemoveProjectDialog";

export { showSyncResult } from "./SyncResultToast";

export { ProjectRenameInput } from "./ProjectRenameInput";
export type { ProjectRenameInputProps } from "./ProjectRenameInput";

export type { SessionSummary } from "./types";
