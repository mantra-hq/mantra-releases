/**
 * useEditorStore - 编辑器状态管理
 * Story 2.13: Task 1 - AC #1, #2, #3, #5
 *
 * 管理编辑器核心状态:
 * - 文件标签页 (tabs)
 * - 激活标签 (activeTabId)
 * - 侧边栏展开状态 (sidebarOpen)
 * - 文件夹展开状态 (expandedFolders)
 * - Monaco ViewState 缓存
 * - Diff 模式 (inline / side-by-side)
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { editor } from "monaco-editor";

/**
 * Diff 显示模式
 */
export type DiffMode = "inline" | "side-by-side";

/**
 * 文件标签页数据
 */
export interface EditorTab {
    /** 唯一标识 (文件路径 或 commitHash:path 或 snapshot:timestamp:path) */
    id: string;
    /** 文件相对路径 */
    path: string;
    /** 显示标签 (文件名) */
    label: string;
    /** 是否固定 (非预览模式) */
    isPinned: boolean;
    /** 是否预览模式 (单击打开，斜体显示) */
    isPreview: boolean;
    /** Monaco ViewState (光标、滚动位置) */
    viewState: editor.ICodeEditorViewState | null;
    /** 所属 commit (历史模式) */
    commitHash?: string;
    /** 历史时间戳 (用于标签显示) */
    timestamp?: number;
    /** 文件内容 (历史版本缓存) */
    content?: string;
    /** 前一版本内容 (用于 Diff) */
    previousContent?: string;

    // Story 2.14: 历史状态标识
    /** 是否为会话快照 (来自时间旅行) */
    isSnapshot?: boolean;
    /** 快照时间戳 (Unix ms) */
    snapshotTime?: number;
}

/**
 * 编辑器状态
 */
export interface EditorState {
    /** 打开的标签页列表 */
    tabs: EditorTab[];
    /** 当前激活的标签 ID */
    activeTabId: string | null;
    /** 侧边栏是否展开 */
    sidebarOpen: boolean;
    /** 文件树展开状态 */
    expandedFolders: Set<string>;
    /** Diff 显示模式 */
    diffMode: DiffMode;

    // ======== Actions ========

    /** 打开文件 (双击 = pinned, 单击 = preview) */
    openTab: (path: string, options?: {
        preview?: boolean;
        commitHash?: string;
        timestamp?: number;
        content?: string;
        previousContent?: string;
        /** Story 2.14: 是否为会话快照 */
        isSnapshot?: boolean;
        /** Story 2.14: 快照时间戳 */
        snapshotTime?: number;
    }) => void;
    /** 关闭标签 */
    closeTab: (tabId: string) => void;
    /** 设置激活标签 */
    setActiveTab: (tabId: string) => void;
    /** 固定预览标签 */
    pinTab: (tabId: string) => void;
    /** 更新 ViewState */
    updateViewState: (tabId: string, viewState: editor.ICodeEditorViewState) => void;
    /** 更新标签内容 */
    updateTabContent: (tabId: string, content: string, previousContent?: string) => void;
    /** 切换侧边栏 */
    toggleSidebar: () => void;
    /** 展开/折叠文件夹 */
    toggleFolder: (path: string) => void;
    /** 切换 Diff 模式 */
    toggleDiffMode: () => void;
    /** 设置 Diff 模式 */
    setDiffMode: (mode: DiffMode) => void;
    /** 切换到下一个标签 */
    nextTab: () => void;
    /** 切换到上一个标签 */
    prevTab: () => void;
    /** 关闭当前标签 */
    closeCurrentTab: () => void;
    /** 清除所有标签 */
    closeAllTabs: () => void;

    // Story 2.14: 快照管理
    /** 退出快照模式 (AC #7, #8) */
    exitSnapshot: (snapshotTabId: string) => void;
    /** 查找实时标签 */
    findLiveTab: (path: string) => EditorTab | undefined;
}

/**
 * 从路径提取文件名
 */
function getFileName(path: string): string {
    return path.split("/").pop() || path;
}

/**
 * 格式化历史标签显示名称
 */
function formatHistoryLabel(path: string, timestamp?: number): string {
    const fileName = getFileName(path);
    if (timestamp) {
        const date = new Date(timestamp);
        const timeStr = date.toLocaleTimeString("zh-CN", {
            hour: "2-digit",
            minute: "2-digit",
        });
        return `${fileName} @ ${timeStr}`;
    }
    return fileName;
}

/**
 * 编辑器状态 Store
 */
export const useEditorStore = create<EditorState>()(
    persist(
        (set, get) => ({
            tabs: [],
            activeTabId: null,
            sidebarOpen: true,
            expandedFolders: new Set<string>(),
            diffMode: "inline" as DiffMode,

            openTab: (path, options = {}) => {
                const { preview = false, commitHash, timestamp, content, previousContent, isSnapshot, snapshotTime } = options;
                // Story 2.14: 快照模式的 tabId 格式为 snapshot:timestamp:path
                let tabId = path;
                if (isSnapshot && snapshotTime) {
                    tabId = `snapshot:${snapshotTime}:${path}`;
                } else if (commitHash) {
                    tabId = `${commitHash}:${path}`;
                }
                const state = get();

                // 检查是否已打开
                const existingTab = state.tabs.find((t) => t.id === tabId);
                if (existingTab) {
                    // 如果已存在，更新内容（如果提供了新内容）
                    if (content !== undefined || previousContent !== undefined) {
                        set({
                            tabs: state.tabs.map((t) =>
                                t.id === tabId
                                    ? {
                                        ...t,
                                        content: content ?? t.content,
                                        previousContent: previousContent ?? t.previousContent,
                                        isPinned: !preview ? true : t.isPinned,
                                        isPreview: preview ? t.isPreview : false,
                                    }
                                    : t
                            ),
                            activeTabId: tabId,
                        });
                    } else if (existingTab.isPreview && !preview) {
                        // 如果已存在且当前是预览，双击时固定
                        set({
                            tabs: state.tabs.map((t) =>
                                t.id === tabId ? { ...t, isPinned: true, isPreview: false } : t
                            ),
                            activeTabId: tabId,
                        });
                    } else {
                        set({ activeTabId: tabId });
                    }
                    return;
                }

                // 创建新标签
                const isHistorical = !!commitHash;
                const newTab: EditorTab = {
                    id: tabId,
                    path,
                    label: isSnapshot
                        ? formatHistoryLabel(path, snapshotTime)
                        : isHistorical
                            ? formatHistoryLabel(path, timestamp)
                            : getFileName(path),
                    isPinned: !preview,
                    isPreview: preview,
                    viewState: null,
                    commitHash,
                    timestamp,
                    content,
                    previousContent,
                    // Story 2.14: 快照标识
                    isSnapshot,
                    snapshotTime,
                };

                // 如果是预览模式，替换现有预览标签
                const previewIndex = state.tabs.findIndex((t) => t.isPreview);
                if (preview && previewIndex !== -1) {
                    // 替换预览标签
                    const newTabs = [...state.tabs];
                    newTabs[previewIndex] = newTab;
                    set({ tabs: newTabs, activeTabId: tabId });
                } else {
                    // 添加新标签
                    set({
                        tabs: [...state.tabs, newTab],
                        activeTabId: tabId,
                    });
                }
            },

            closeTab: (tabId) => {
                const state = get();
                const tabIndex = state.tabs.findIndex((t) => t.id === tabId);
                if (tabIndex === -1) return;

                const newTabs = state.tabs.filter((t) => t.id !== tabId);

                // 计算新的激活标签
                let newActiveId = state.activeTabId;
                if (state.activeTabId === tabId) {
                    if (newTabs.length === 0) {
                        newActiveId = null;
                    } else if (tabIndex >= newTabs.length) {
                        newActiveId = newTabs[newTabs.length - 1].id;
                    } else {
                        newActiveId = newTabs[tabIndex].id;
                    }
                }

                set({ tabs: newTabs, activeTabId: newActiveId });
            },

            setActiveTab: (tabId) => set({ activeTabId: tabId }),

            pinTab: (tabId) =>
                set((state) => ({
                    tabs: state.tabs.map((t) =>
                        t.id === tabId ? { ...t, isPinned: true, isPreview: false } : t
                    ),
                })),

            updateViewState: (tabId, viewState) =>
                set((state) => ({
                    tabs: state.tabs.map((t) =>
                        t.id === tabId ? { ...t, viewState } : t
                    ),
                })),

            updateTabContent: (tabId, content, previousContent) =>
                set((state) => ({
                    tabs: state.tabs.map((t) =>
                        t.id === tabId
                            ? { ...t, content, previousContent: previousContent ?? t.previousContent }
                            : t
                    ),
                })),

            toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),

            toggleFolder: (path) =>
                set((state) => {
                    const newExpanded = new Set(state.expandedFolders);
                    if (newExpanded.has(path)) {
                        newExpanded.delete(path);
                    } else {
                        newExpanded.add(path);
                    }
                    return { expandedFolders: newExpanded };
                }),

            toggleDiffMode: () =>
                set((state) => ({
                    diffMode: state.diffMode === "inline" ? "side-by-side" : "inline",
                })),

            setDiffMode: (mode) => set({ diffMode: mode }),

            nextTab: () => {
                const state = get();
                if (state.tabs.length === 0) return;
                const currentIndex = state.tabs.findIndex((t) => t.id === state.activeTabId);
                const nextIndex = (currentIndex + 1) % state.tabs.length;
                set({ activeTabId: state.tabs[nextIndex].id });
            },

            prevTab: () => {
                const state = get();
                if (state.tabs.length === 0) return;
                const currentIndex = state.tabs.findIndex((t) => t.id === state.activeTabId);
                const prevIndex = currentIndex <= 0 ? state.tabs.length - 1 : currentIndex - 1;
                set({ activeTabId: state.tabs[prevIndex].id });
            },

            closeCurrentTab: () => {
                const state = get();
                if (state.activeTabId) {
                    state.closeTab(state.activeTabId);
                }
            },

            closeAllTabs: () => set({ tabs: [], activeTabId: null }),

            // Story 2.14: 快照管理
            findLiveTab: (path) => {
                const state = get();
                return state.tabs.find(t =>
                    t.path === path &&
                    !t.isSnapshot &&
                    !t.commitHash
                );
            },

            exitSnapshot: (snapshotTabId) => {
                const state = get();
                const snapshotTab = state.tabs.find(t => t.id === snapshotTabId);
                if (!snapshotTab) return;

                const liveTab = state.tabs.find(t =>
                    t.path === snapshotTab.path &&
                    !t.isSnapshot &&
                    !t.commitHash
                );

                if (liveTab) {
                    // AC #7: 已有实时标签：关闭快照，切换过去
                    set({
                        tabs: state.tabs.filter(t => t.id !== snapshotTabId),
                        activeTabId: liveTab.id,
                    });
                } else {
                    // AC #8: 无实时标签：转换当前标签为实时
                    const newLiveId = snapshotTab.path;
                    set({
                        tabs: state.tabs.map(t =>
                            t.id === snapshotTabId
                                ? {
                                    ...t,
                                    id: newLiveId,
                                    isSnapshot: false,
                                    snapshotTime: undefined,
                                    commitHash: undefined,
                                    label: getFileName(t.path),
                                }
                                : t
                        ),
                        activeTabId: newLiveId,
                    });
                }
            },
        }),
        {
            name: "mantra-editor-store",
            partialize: (state) => ({
                sidebarOpen: state.sidebarOpen,
                diffMode: state.diffMode,
                expandedFolders: state.expandedFolders,
                // 不持久化 tabs 和 viewState (会话级数据)
            }),
            // 自定义序列化/反序列化以处理 Set
            storage: {
                getItem: (name) => {
                    const str = localStorage.getItem(name);
                    if (!str) return null;
                    const parsed = JSON.parse(str);
                    return {
                        ...parsed,
                        state: {
                            ...parsed.state,
                            expandedFolders: new Set(parsed.state.expandedFolders || []),
                            tabs: [], // 不从持久化恢复 tabs
                            activeTabId: null,
                            diffMode: parsed.state.diffMode || "inline",
                        },
                    };
                },
                setItem: (name, value) => {
                    const serialized = {
                        ...value,
                        state: {
                            ...value.state,
                            expandedFolders: Array.from(value.state.expandedFolders || []),
                        },
                    };
                    localStorage.setItem(name, JSON.stringify(serialized));
                },
                removeItem: (name) => localStorage.removeItem(name),
            },
        }
    )
);

export default useEditorStore;











