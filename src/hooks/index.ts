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
// Story 1.12: View-based Project Aggregation
export {
    useUnassignedSessions,
    useProjectPaths,
    getProjectPaths,
    addProjectPath,
    removeProjectPath,
    setProjectPrimaryPath,
    bindSessionToProject,
    unbindSession,
    getUnassignedSessions,
} from "./useProjects";
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

// 通知系统初始化 hook
export { useNotificationInit } from "./useNotificationInit";

// Story 10.1: 压缩模式状态管理 hook
export {
    useCompressMode,
    useRefineMode, // 向后兼容别名
    type UseCompressModeOptions,
    type UseCompressModeReturn,
    type UseRefineModeOptions,
    type UseRefineModeReturn,
} from "./use-compress-mode";

// Story 10.3: 压缩操作状态管理 hook
export {
    useCompressState,
    CompressStateProvider,
    type OperationType,
    type CompressOperation,
    type PreviewMessage,
    type ChangeStats,
    type CompressStateContextValue,
    type CompressStateProviderProps,
    type StateSnapshot,
} from "./useCompressState";

// Story 10.9: 压缩状态持久化 hook
export { useCompressPersistence } from "./useCompressPersistence";

// Story 10.9: 导航拦截 hook
export {
    useNavigationGuard,
    type UseNavigationGuardOptions,
    type UseNavigationGuardResult,
} from "./useNavigationGuard";

// Story 10.10: 消息焦点管理 hook
export {
    useMessageFocus,
    type UseMessageFocusOptions,
    type UseMessageFocusReturn,
} from "./useMessageFocus";

// Story 10.10: 压缩模式快捷键 hook
export {
    useCompressHotkeys,
    type UseCompressHotkeysOptions,
} from "./useCompressHotkeys";

// Story 10.8: 平台检测 hook
export {
    usePlatform,
    getModifierKey,
    getShiftKey,
    type Platform,
} from "./usePlatform";

// Story 11.4: 环境变量管理 hook
export { useEnvVariables } from "./useEnvVariables";

// Story 14.5: 更新检查 hook
export {
    useUpdateChecker,
    type UpdateStatus,
    type UpdateInfo,
    type UseUpdateCheckerResult,
} from "./useUpdateChecker";