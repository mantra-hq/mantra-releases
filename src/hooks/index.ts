/**
 * Hooks Index
 * 导出所有自定义 Hooks
 */

export { useLayoutPersist } from "./use-layout-persist";
export { useResponsiveLayout, type LayoutMode } from "./use-responsive-layout";
export { useTheme } from "./use-theme";
export { useTimeMachine, type SnapshotResult } from "./useTimeMachine";
export {
    useTimeSync,
    type UseTimeSyncOptions,
    type UseTimeSyncResult,
} from "./useTimeSync";
export {
    useDiffFadeOut,
    type UseDiffFadeOutResult,
} from "./useDiffFadeOut";
export { useProjects, type UseProjectsResult } from "./useProjects";
export { useDebouncedValue } from "./useDebouncedValue";
export { useGlobalShortcut } from "./useGlobalShortcut";

// Story 2.15: 工具配对和可折叠 hooks
export {
    useToolPairing,
    type ToolCallMessage,
    type ToolOutputMessage,
    type ToolPair,
    type ToolPairMap,
    type UseToolPairingResult,
} from "./useToolPairing";
export {
    useCollapsible,
    type UseCollapsibleOptions,
    type UseCollapsibleResult,
} from "./useCollapsible";

// Story 2.17: 当前会话 hook
export {
    useCurrentSession,
    type UseCurrentSessionResult,
} from "./useCurrentSession";

// 脱敏预览 hook
export {
    useSanitizePreview,
    type UseSanitizePreviewResult,
} from "./useSanitizePreview";

// 通知系统初始化 hook
export { useNotificationInit } from "./useNotificationInit";
