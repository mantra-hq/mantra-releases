/**
 * useDetailPanelStore - 详情面板状态管理
 * Story 2.15: Task 8.3
 *
 * 管理右侧面板的 Tab 切换和内容状态
 */

import { create } from "zustand";

/** 右侧面板 Tab 类型 */
export type RightPanelTab = "code" | "terminal" | "tool";

/** 详情面板类型 (兼容旧逻辑) */
export type DetailPanelType = "tool" | "file" | "search" | null;

/** 工具详情数据 */
export interface ToolDetail {
    toolUseId: string;
    toolName: string;
    toolInput?: Record<string, unknown>;
    toolOutput?: string;
    isError?: boolean;
    duration?: number;
}

/** 终端内容数据 */
export interface TerminalContent {
    command?: string;
    output: string;
    isError?: boolean;
    exitCode?: number;
}

/** 详情面板状态 */
interface DetailPanelState {
    /** 右侧面板当前激活的 Tab */
    activeRightTab: RightPanelTab;
    /** 面板类型 (兼容) */
    panelType: DetailPanelType;
    /** 工具详情数据 */
    toolDetail: ToolDetail | null;
    /** 终端内容 */
    terminalContent: TerminalContent | null;
    /** 当前高亮的 toolUseId */
    highlightedToolId: string | null;

    /** 切换右侧 Tab */
    setActiveRightTab: (tab: RightPanelTab) => void;
    /** 打开工具详情面板 (兼容旧逻辑) */
    openToolDetail: (detail: ToolDetail) => void;
    /** 打开文件详情 (切换到代码 tab) */
    openFileDetail: (filePath: string) => void;
    /** 打开终端详情 */
    openTerminalDetail: (content: TerminalContent) => void;
    /** 关闭详情面板 */
    closePanel: () => void;
    /** 设置高亮 ID */
    setHighlightedToolId: (id: string | null) => void;
    /** 重置状态 */
    reset: () => void;
}

const initialState = {
    activeRightTab: "code" as RightPanelTab,
    panelType: null as DetailPanelType,
    toolDetail: null as ToolDetail | null,
    terminalContent: null as TerminalContent | null,
    highlightedToolId: null as string | null,
};

/**
 * useDetailPanelStore
 *
 * 管理右侧面板状态：
 * - 代码编辑 / 终端 Tab 切换
 * - 存储当前查看的工具详情
 * - 管理配对高亮状态
 */
export const useDetailPanelStore = create<DetailPanelState>((set) => ({
    ...initialState,

    setActiveRightTab: (tab) =>
        set({
            activeRightTab: tab,
        }),

    openToolDetail: (detail) =>
        set({
            activeRightTab: "tool", // 切换到工具详情 Tab
            panelType: "tool",
            toolDetail: detail,
        }),

    openFileDetail: (_filePath) =>
        set({
            activeRightTab: "code",
            panelType: "file",
        }),

    openTerminalDetail: (content) =>
        set({
            activeRightTab: "terminal",
            terminalContent: content,
            panelType: "tool",
        }),

    closePanel: () =>
        set({
            panelType: null,
            toolDetail: null,
            terminalContent: null,
        }),

    setHighlightedToolId: (id) =>
        set({
            highlightedToolId: id,
        }),

    reset: () => set(initialState),
}));

export default useDetailPanelStore;
