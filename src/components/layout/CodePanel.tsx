/**
 * CodePanel - 代码面板 (集成编辑器组件)
 * Story 2.2: Task 4.2 (初始占位)
 * Story 2.5: Task 5.2 (集成 CodeSnapshotView)
 * Story 2.11: AC6, AC7 (无 Git 仓库警告)
 * Story 2.12: AC5 (文件不存在处理)
 * Story 2.13: Task 9 (集成 EditorTabs, Breadcrumbs, FileTree, QuickOpen)
 * Story 2.26: 国际化支持
 *
 * 右侧面板，完整的代码浏览器:
 * - 文件标签页 (EditorTabs)
 * - 面包屑导航 (Breadcrumbs)
 * - 文件树侧边栏 (FileTree)
 * - 快速打开 (QuickOpen)
 * - 快捷键支持
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import {
    CodeSnapshotView,
    NoGitWarning,
    EditorTabs,
    Breadcrumbs,
    FileTree,
    QuickOpen,
    type TreeNode,
    type SiblingItem,
} from "@/components/editor";
import { useEditorStore } from "@/stores/useEditorStore";
import { useEditorKeyboard } from "@/hooks/useEditorKeyboard";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Files, PanelLeftClose, PanelLeft } from "lucide-react";
import type { editor } from "monaco-editor";
import { StatusBar, type CursorPosition } from "./StatusBar";
import { BranchSelector } from "@/components/git/BranchSelector";
import { SyncStatus } from "@/components/git/SyncStatus";

export interface CodePanelProps {
    /** 自定义 className */
    className?: string;
    /** 代码内容 */
    code?: string;
    /** 文件路径 */
    filePath?: string;
    /** 历史时间戳 (ISO 8601 或 Unix ms) */
    timestamp?: string | number;
    /** Commit Hash (短格式) */
    commitHash?: string;
    /** Commit 消息 (Story 2.7 AC #6) */
    commitMessage?: string;
    /** 前一个代码内容 (用于 Diff 高亮, Story 2.7 AC #5) */
    previousCode?: string | null;
    /** 是否处于历史模式 (Story 2.7 AC #6) */
    isHistoricalMode?: boolean;
    /** 返回当前回调 (Story 2.7 AC #6) */
    onReturnToCurrent?: () => void;
    /** 无 Git 仓库警告 (Story 2.11 AC6) */
    showNoGitWarning?: boolean;
    /** 项目路径 (用于无 Git 警告显示) */
    projectPath?: string;
    /** 仓库路径 (用于文件树) */
    repoPath?: string;
    /** 了解更多回调 (Story 2.11 AC7) */
    onLearnMore?: () => void;
    /** 文件未找到标志 (Story 2.12 AC #5) */
    fileNotFound?: boolean;
    /** 未找到的文件路径 (Story 2.12 AC #5) */
    notFoundPath?: string;
    /** 清除文件不存在状态回调 (Story 2.12 AC #5) */
    onDismissNotFound?: () => void;
    /** 文件内容加载器 (根据路径和 commitHash 加载文件内容) */
    onLoadFileContent?: (path: string, commitHash?: string) => Promise<string>;
}

/**
 * 代码面板组件
 *
 * 功能:
 * - 集成 EditorTabs 文件标签页 (AC #1-5)
 * - 集成 Breadcrumbs 面包屑导航 (AC #6, #7, #20)
 * - 集成 FileTree 文件树侧边栏 (AC #8-14)
 * - 集成 QuickOpen 快速打开 (AC #15)
 * - 快捷键支持 (AC #15-18)
 * - Monaco ViewState 管理 (AC #5)
 * - 历史模式适配 (AC #19, #20)
 */
export function CodePanel({
    className,
    code = "",
    filePath = "",
    timestamp,
    commitHash,
    commitMessage,
    previousCode,
    isHistoricalMode,
    onReturnToCurrent,
    showNoGitWarning = false,
    projectPath,
    repoPath,
    onLearnMore,
    fileNotFound = false,
    notFoundPath,
    onDismissNotFound,
    onLoadFileContent,
}: CodePanelProps) {
    const { t } = useTranslation();

    // 编辑器状态管理 - 使用独立的选择器确保引用稳定
    const tabs = useEditorStore((state) => state.tabs);
    const activeTabId = useEditorStore((state) => state.activeTabId);
    const sidebarOpen = useEditorStore((state) => state.sidebarOpen);
    const openTab = useEditorStore((state) => state.openTab);
    const updateViewState = useEditorStore((state) => state.updateViewState);
    const toggleSidebar = useEditorStore((state) => state.toggleSidebar);
    const exitSnapshot = useEditorStore((state) => state.exitSnapshot);

    // QuickOpen 状态
    const [quickOpenVisible, setQuickOpenVisible] = React.useState(false);

    // 文件树数据
    const [fileTree, setFileTree] = React.useState<TreeNode[]>([]);
    const [fileList, setFileList] = React.useState<string[]>([]);
    const [treeLoading, setTreeLoading] = React.useState(false);

    // 当前文件的同级文件 (用于 Breadcrumbs)
    const [siblings, setSiblings] = React.useState<SiblingItem[]>([]);

    // Story 2.14: 光标位置状态
    const [cursorPosition, setCursorPosition] = React.useState<CursorPosition | undefined>();

    // 当前活动标签的 ViewState
    const activeTab = React.useMemo(
        () => tabs.find((t) => t.id === activeTabId),
        [tabs, activeTabId]
    );

    // Story 2.14: 组合退出快照回调 (同时清理时间旅行和标签页状态)
    const handleReturnToCurrent = React.useCallback(() => {
        // 1. 调用外部回调 (清理 useTimeTravelStore)
        onReturnToCurrent?.();
        // 2. 清理标签页状态 (exitSnapshot 或关闭历史标签)
        if (activeTabId && (activeTab?.isSnapshot || activeTab?.commitHash)) {
            exitSnapshot(activeTabId);
        }
    }, [onReturnToCurrent, activeTabId, activeTab, exitSnapshot]);

    // 用于追踪是否已为初始文件创建过标签
    const initialTabCreatedRef = React.useRef(false);

    // 自动为初始文件创建标签 (修复默认文件无标签问题)
    React.useEffect(() => {
        // 只在 tabs 为空且未创建过时执行一次
        if (filePath && tabs.length === 0 && !initialTabCreatedRef.current) {
            initialTabCreatedRef.current = true;
            openTab(filePath, {
                preview: false,
                commitHash: commitHash,
                timestamp: typeof timestamp === "number" ? timestamp : undefined,
                content: code,
                previousContent: previousCode ?? undefined,
                isSnapshot: isHistoricalMode,
                snapshotTime: typeof timestamp === "number" ? timestamp : undefined,
            });
        }
    }, [filePath, tabs.length, openTab, commitHash, timestamp, code, previousCode, isHistoricalMode]);

    // 确定当前显示的文件路径和内容
    // 统一从标签读取，如果没有标签则使用 props（初始状态）
    const displayFilePath = activeTab?.path ?? filePath;
    const displayCommitHash = activeTab?.commitHash ?? commitHash;
    const displayTimestamp = activeTab?.timestamp ?? (typeof timestamp === "number" ? timestamp : undefined);

    // 文件内容状态 (支持多标签)
    const [tabContents, setTabContents] = React.useState<Record<string, string>>({});

    // 统一从标签读取内容，标签内容优先于 tabContents 缓存
    const displayCode = activeTab?.content ?? (activeTabId ? tabContents[activeTabId] : null) ?? code;

    // Diff 用的前一版本内容
    const displayPreviousCode = activeTab?.previousContent ?? previousCode;

    // 是否有可用的 Diff 数据 (用于 EditorTabs 显示 Diff 模式切换)
    const hasDiffData = !!(displayPreviousCode && displayPreviousCode !== displayCode);

    // 计算时间戳
    const timestampMs = React.useMemo(() => {
        if (displayTimestamp) return displayTimestamp;
        if (typeof timestamp === "number") return timestamp;
        if (typeof timestamp === "string") {
            const parsed = Date.parse(timestamp);
            return isNaN(parsed) ? undefined : parsed;
        }
        return undefined;
    }, [displayTimestamp, timestamp]);

    // 快捷键支持 (AC #15-18)
    useEditorKeyboard({
        onQuickOpen: () => setQuickOpenVisible(true),
        enabled: true,
    });

    // 加载文件树 (AC #9, #19)
    // 注意: Tauri 2.x 前端使用 camelCase，会自动转换为 Rust 的 snake_case
    React.useEffect(() => {
        if (!repoPath) return;

        const loadTree = async () => {
            setTreeLoading(true);
            try {
                const tree = await invoke<TreeNode[]>("list_tree_at_commit", {
                    repoPath: repoPath,
                    commitHash: isHistoricalMode ? displayCommitHash : undefined,
                    subpath: undefined,
                });
                setFileTree(tree);
            } catch (err) {
                console.error("[CodePanel] 加载文件树失败:", err);
                setFileTree([]);
            } finally {
                setTreeLoading(false);
            }
        };

        loadTree();
    }, [repoPath, isHistoricalMode, displayCommitHash]);

    // 加载文件列表 (用于 QuickOpen)
    React.useEffect(() => {
        if (!repoPath) return;

        const loadFiles = async () => {
            try {
                const files = await invoke<string[]>("list_files_at_commit", {
                    repoPath: repoPath,
                    commitHash: isHistoricalMode ? displayCommitHash : undefined,
                });
                setFileList(files);
            } catch (err) {
                console.error("加载文件列表失败:", err);
                setFileList([]);
            }
        };

        loadFiles();
    }, [repoPath, isHistoricalMode, displayCommitHash]);

    // 计算同级文件 (用于 Breadcrumbs 导航)
    React.useEffect(() => {
        if (!displayFilePath || fileTree.length === 0) {
            setSiblings([]);
            return;
        }

        // 扁平化文件树
        const flatList: SiblingItem[] = [];
        const flatten = (nodes: TreeNode[]) => {
            for (const node of nodes) {
                flatList.push({
                    name: node.name,
                    path: node.path,
                    isDirectory: node.type === "directory",
                });
                if (node.children) {
                    flatten(node.children);
                }
            }
        };
        flatten(fileTree);
        setSiblings(flatList);
    }, [displayFilePath, fileTree]);

    // 加载标签页内容 (使用 ref 避免依赖 tabContents 导致无限重渲染)
    const tabContentsRef = React.useRef(tabContents);
    tabContentsRef.current = tabContents;

    const loadTabContent = React.useCallback(
        async (tabId: string, path: string, tabCommitHash?: string) => {
            // 使用 ref 检查避免依赖 tabContents
            if (tabContentsRef.current[tabId]) return; // 已加载

            // 优先使用外部提供的加载器，否则直接调用 Tauri 命令
            if (onLoadFileContent) {
                try {
                    const content = await onLoadFileContent(path, tabCommitHash);
                    setTabContents((prev) => ({ ...prev, [tabId]: content }));
                } catch (err) {
                    console.error("加载文件内容失败:", err);
                }
            } else if (repoPath) {
                // 直接调用 Tauri 命令加载文件内容
                try {
                    const result = await invoke<{ content: string }>("get_file_at_head", {
                        repoPath: repoPath,
                        filePath: path,
                    });
                    setTabContents((prev) => ({ ...prev, [tabId]: result.content }));
                } catch (err) {
                    console.error("加载文件内容失败:", err);
                    setTabContents((prev) => ({ ...prev, [tabId]: `// 无法加载文件: ${path}` }));
                }
            }
        },
        [onLoadFileContent, repoPath]
    );

    // 当活动标签变化时加载内容
    React.useEffect(() => {
        if (activeTab && (onLoadFileContent || repoPath)) {
            loadTabContent(activeTab.id, activeTab.path, activeTab.commitHash);
        }
    }, [activeTab, loadTabContent, onLoadFileContent, repoPath]);

    // 文件树单击 (预览, AC #12)
    const handleFileClick = React.useCallback(
        (path: string) => {
            openTab(path, {
                preview: true,
                commitHash: isHistoricalMode ? displayCommitHash : undefined,
            });
        },
        [openTab, isHistoricalMode, displayCommitHash]
    );

    // 文件树双击 (打开, AC #11)
    const handleFileDoubleClick = React.useCallback(
        (path: string) => {
            openTab(path, {
                preview: false,
                commitHash: isHistoricalMode ? displayCommitHash : undefined,
            });
        },
        [openTab, isHistoricalMode, displayCommitHash]
    );

    // QuickOpen 选择 (AC #15)
    const handleQuickOpenSelect = React.useCallback(
        (path: string) => {
            openTab(path, {
                preview: false,
                commitHash: isHistoricalMode ? displayCommitHash : undefined,
            });
        },
        [openTab, isHistoricalMode, displayCommitHash]
    );

    // Breadcrumbs 导航
    const handleBreadcrumbNavigate = React.useCallback(
        (path: string) => {
            // 检查是否为目录
            const isDir = siblings.find((s) => s.path === path)?.isDirectory;
            if (isDir) {
                // 目录：展开文件树
                useEditorStore.getState().toggleFolder(path);
            } else {
                // 文件：打开
                openTab(path, {
                    preview: false,
                    commitHash: isHistoricalMode ? displayCommitHash : undefined,
                });
            }
        },
        [siblings, openTab, isHistoricalMode, displayCommitHash]
    );

    // ViewState 变更回调 (AC #5)
    const handleViewStateChange = React.useCallback(
        (viewState: editor.ICodeEditorViewState) => {
            if (activeTabId) {
                updateViewState(activeTabId, viewState);
            }
            // Story 2.14: 更新光标位置
            if (viewState.cursorState?.[0]) {
                const cursor = viewState.cursorState[0];
                setCursorPosition({
                    line: cursor.position.lineNumber,
                    column: cursor.position.column,
                });
            }
        },
        [activeTabId, updateViewState]
    );

    // Story 2.11 AC6: 无 Git 仓库时显示警告
    if (showNoGitWarning && !code && tabs.length === 0) {
        return (
            <div className={cn("h-full", className)}>
                <NoGitWarning projectPath={projectPath} onLearnMore={onLearnMore} />
            </div>
        );
    }

    return (
        <div className={cn("h-full flex", className)}>
            {/* 文件树侧边栏 (AC #8-14) */}
            {sidebarOpen && repoPath && (
                <div className="w-60 border-r border-border flex flex-col bg-muted/30 shrink-0">
                    {/* 侧边栏头部 */}
                    <div className="flex items-center justify-between px-3 py-2 border-b border-border">
                        <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                            <Files className="h-4 w-4" />
                            <span>{t("editor.explorer")}</span>
                        </div>
                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-6 w-6"
                            onClick={toggleSidebar}
                            aria-label={t("editor.closeSidebar")}
                        >
                            <PanelLeftClose className="h-4 w-4" />
                        </Button>
                    </div>

                    {/* 文件树 */}
                    {treeLoading ? (
                        <div className="flex-1 flex items-center justify-center text-muted-foreground text-sm">
                            {t("common.loading")}
                        </div>
                    ) : (
                        <FileTree
                            tree={fileTree}
                            activeFilePath={displayFilePath}
                            onFileClick={handleFileClick}
                            onFileDoubleClick={handleFileDoubleClick}
                            className="flex-1"
                        />
                    )}
                </div>
            )}

            {/* 主内容区 */}
            <div className="flex-1 flex flex-col min-w-0">
                {/* 工具栏 (包含侧边栏切换按钮) */}
                <div className="flex items-center border-b border-border bg-muted/30">
                    {/* 侧边栏切换按钮 (AC #8) */}
                    {repoPath && !sidebarOpen && (
                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8 shrink-0"
                            onClick={toggleSidebar}
                            aria-label={t("editor.openSidebar")}
                        >
                            <PanelLeft className="h-4 w-4" />
                        </Button>
                    )}

                    {/* 文件标签页 (AC #1-5) - UX 优化方案 B: 纯标签管理 */}
                    <EditorTabs className="flex-1 border-b-0" />
                </div>

                {/* 面包屑导航 (UX 优化方案 B: 完整路径 + 历史信息 + Diff 切换 + 返回当前) */}
                {displayFilePath && (
                    <Breadcrumbs
                        filePath={displayFilePath}
                        siblings={siblings}
                        historyInfo={
                            (isHistoricalMode || activeTab?.commitHash) && timestampMs
                                ? {
                                    timestamp: timestampMs,
                                    commitHash: displayCommitHash,
                                    commitMessage: commitMessage,
                                }
                                : undefined
                        }
                        hasDiffData={hasDiffData}
                        onReturnToCurrent={isHistoricalMode || activeTab?.commitHash || activeTab?.isSnapshot ? handleReturnToCurrent : undefined}
                        onNavigate={handleBreadcrumbNavigate}
                    />
                )}

                {/* 代码编辑器 */}
                <div className="flex-1 overflow-hidden">
                    <CodeSnapshotView
                        code={displayCode}
                        filePath={displayFilePath}
                        timestamp={timestampMs}
                        commitHash={displayCommitHash}
                        commitMessage={commitMessage}
                        previousCode={displayPreviousCode}
                        isHistoricalMode={isHistoricalMode || !!activeTab?.commitHash}
                        onReturnToCurrent={handleReturnToCurrent}
                        fileNotFound={fileNotFound}
                        notFoundPath={notFoundPath}
                        onDismissNotFound={onDismissNotFound}
                        viewState={activeTab?.viewState}
                        onViewStateChange={handleViewStateChange}
                    />
                </div>

                {/* Story 2.14: 底部状态栏 (AC #9, #10, #11, #12) */}
                <StatusBar
                    cursorPosition={cursorPosition}
                    leftContent={
                        repoPath ? (
                            <>
                                <BranchSelector currentBranch="main" />
                                <SyncStatus status="synced" />
                            </>
                        ) : null
                    }
                />
            </div>

            {/* 快速打开对话框 (AC #15) */}
            <QuickOpen
                open={quickOpenVisible}
                onOpenChange={setQuickOpenVisible}
                files={fileList}
                onSelect={handleQuickOpenSelect}
                loading={treeLoading}
            />
        </div>
    );
}

export default CodePanel;
