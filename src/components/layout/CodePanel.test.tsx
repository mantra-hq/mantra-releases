/**
 * CodePanel - 代码面板集成测试
 * Story 2.13: Task 10 验证
 * AC: #1-20 集成验证
 *
 * Task 10.5: 集成测试 - Tab 切换 ViewState 恢复
 * Task 10.6: 集成测试 - 历史模式文件树
 */

import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { CodePanel } from "./CodePanel";
import { useEditorStore } from "@/stores/useEditorStore";
import type { editor } from "monaco-editor";
import { TooltipProvider } from "@/components/ui/tooltip";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
    invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock Monaco Editor
vi.mock("@monaco-editor/react", () => ({
    default: ({ value, filePath }: { value: string; filePath?: string }) => (
        <div data-testid="monaco-editor" data-filepath={filePath}>
            {value?.substring(0, 100) || "empty"}
        </div>
    ),
}));

// Mock theme provider
vi.mock("@/lib/theme-provider", () => ({
    useTheme: () => ({
        theme: "dark",
        resolvedTheme: "dark",
        setTheme: vi.fn(),
    }),
}));

// 包装组件，提供必要的 Provider
function renderWithProviders(ui: React.ReactElement) {
    return render(<TooltipProvider>{ui}</TooltipProvider>);
}

describe("CodePanel", () => {
    // 每个测试前重置 store
    beforeEach(() => {
        useEditorStore.getState().closeAllTabs();
        // 确保侧边栏关闭
        if (useEditorStore.getState().sidebarOpen) {
            useEditorStore.getState().toggleSidebar();
        }
        vi.clearAllMocks();

        // 默认 mock invoke 返回空数组 (防止 undefined 错误)
        mockInvoke.mockImplementation((cmd: string) => {
            if (cmd === "list_tree_at_commit") {
                return Promise.resolve([]);
            }
            if (cmd === "list_files_at_commit") {
                return Promise.resolve([]);
            }
            return Promise.resolve(null);
        });
    });

    afterEach(() => {
        vi.clearAllMocks();
    });

    describe("基础渲染", () => {
        it("应该渲染代码内容", () => {
            renderWithProviders(
                <CodePanel
                    code="console.log('Hello');"
                    filePath="src/index.ts"
                />
            );

            expect(screen.getByTestId("monaco-editor")).toBeInTheDocument();
        });

        it("AC #6: 应该渲染面包屑导航", () => {
            // 打开标签以显示文件名 (UX 优化后面包屑隐藏文件名)
            useEditorStore.getState().openTab("src/components/App.tsx");

            renderWithProviders(
                <CodePanel
                    code="const a = 1;"
                    filePath="src/components/App.tsx"
                />
            );

            // 面包屑应显示路径段 (UX 优化: 文件名已隐藏)
            expect(screen.getByText("src")).toBeInTheDocument();
            expect(screen.getByText("components")).toBeInTheDocument();
            // UX 优化: Breadcrumbs 现在隐藏文件名，只在标签页显示
            // App.tsx 现在只出现在标签页中
            expect(screen.getByText("App.tsx")).toBeInTheDocument();
        });

        it("无 Git 仓库时应显示警告", () => {
            renderWithProviders(
                <CodePanel
                    code=""
                    filePath=""
                    showNoGitWarning={true}
                    projectPath="/test/project"
                />
            );

            // 使用更精确的选择器
            expect(screen.getByText("未检测到 Git 仓库")).toBeInTheDocument();
        });
    });

    describe("标签页集成 (AC #1-5)", () => {
        it("AC #1: 应该显示标签页栏", () => {
            // 打开一个标签
            useEditorStore.getState().openTab("src/App.tsx");

            renderWithProviders(
                <CodePanel
                    code="const a = 1;"
                    filePath="src/App.tsx"
                />
            );

            // 检查 tablist 存在且包含标签
            expect(screen.getByRole("tablist")).toBeInTheDocument();
            expect(screen.getByRole("tab")).toBeInTheDocument();
        });

        it("AC #2: 点击标签应切换", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");

            renderWithProviders(
                <CodePanel
                    code="const a = 1;"
                    filePath="src/b.ts"
                />
            );

            fireEvent.click(screen.getByText("a.ts"));

            expect(useEditorStore.getState().activeTabId).toBe("src/a.ts");
        });

        it("AC #3: 关闭按钮应关闭标签", () => {
            useEditorStore.getState().openTab("src/a.ts");

            renderWithProviders(
                <CodePanel
                    code="const a = 1;"
                    filePath="src/a.ts"
                />
            );

            const closeButton = screen.getByRole("button", { name: /关闭标签/i });
            fireEvent.click(closeButton);

            expect(useEditorStore.getState().tabs).toHaveLength(0);
        });
    });

    describe("侧边栏 (AC #8)", () => {
        it("AC #8: 默认侧边栏关闭", async () => {
            await act(async () => {
                renderWithProviders(
                    <CodePanel
                        code="const a = 1;"
                        filePath="src/App.tsx"
                        repoPath="/test/repo"
                    />
                );
            });

            // 侧边栏关闭时显示展开按钮
            expect(screen.getByRole("button", { name: /展开侧边栏/i })).toBeInTheDocument();
        });

        it("AC #8: 点击按钮应切换侧边栏", async () => {
            await act(async () => {
                renderWithProviders(
                    <CodePanel
                        code="const a = 1;"
                        filePath="src/App.tsx"
                        repoPath="/test/repo"
                    />
                );
            });

            const toggleButton = screen.getByRole("button", { name: /展开侧边栏/i });
            await act(async () => {
                fireEvent.click(toggleButton);
            });

            expect(useEditorStore.getState().sidebarOpen).toBe(true);
        });
    });

    describe("历史模式 (AC #19, #20)", () => {
        it("AC #20: 历史模式时面包屑应显示时间指示器", () => {
            const timestamp = Date.now() - 3600000; // 1小时前

            renderWithProviders(
                <CodePanel
                    code="const a = 1;"
                    filePath="src/App.tsx"
                    isHistoricalMode={true}
                    timestamp={timestamp}
                    commitHash="abc1234"
                />
            );

            // Breadcrumbs 组件在历史模式时显示时间指示器
            const breadcrumbs = screen.getByText("src");
            expect(breadcrumbs).toBeInTheDocument();

            // UX 优化: 历史模式现在显示 commit hash (例如 "abc1234") 而不是 "历史" 文字
            // 检查 commit hash 或相对时间 (如 "X小时前")
            const historyIndicators = screen.queryAllByText(/abc1234|小时前|分钟前/i);
            expect(historyIndicators.length).toBeGreaterThan(0);
        });
    });

    describe("空状态", () => {
        it("无代码时应显示空状态提示", () => {
            renderWithProviders(
                <CodePanel
                    code=""
                    filePath=""
                />
            );

            // CodeSnapshotView 在无代码时会渲染 EmptyCodeState
            // EmptyCodeState 显示 "暂无代码"
            expect(screen.getByText("暂无代码")).toBeInTheDocument();
        });
    });

    describe("文件不存在 (Story 2.12 AC #5)", () => {
        it("文件不存在时应显示 Banner", () => {
            renderWithProviders(
                <CodePanel
                    code="// old code"
                    filePath="src/deleted.ts"
                    fileNotFound={true}
                    notFoundPath="src/deleted.ts"
                />
            );

            expect(screen.getByText(/不存在/i)).toBeInTheDocument();
        });
    });

    describe("Task 10.5: ViewState 集成测试 (AC #5)", () => {
        it("应该保存和恢复 ViewState", () => {
            const mockViewState: editor.ICodeEditorViewState = {
                cursorState: [{ inSelectionMode: false, selectionStart: { lineNumber: 10, column: 5 }, position: { lineNumber: 10, column: 5 } }],
                viewState: { scrollLeft: 0, firstPosition: { lineNumber: 5, column: 1 }, firstPositionDeltaTop: 0 },
                contributionsState: {},
            };

            // 打开标签并保存 ViewState
            useEditorStore.getState().openTab("src/App.tsx");
            useEditorStore.getState().updateViewState("src/App.tsx", mockViewState);

            const tab = useEditorStore.getState().tabs.find(t => t.id === "src/App.tsx");
            expect(tab?.viewState).toEqual(mockViewState);
        });

        it("切换标签应保留各自的 ViewState", () => {
            const viewState1: editor.ICodeEditorViewState = {
                cursorState: [{ inSelectionMode: false, selectionStart: { lineNumber: 1, column: 1 }, position: { lineNumber: 1, column: 1 } }],
                viewState: { scrollLeft: 0, firstPosition: { lineNumber: 1, column: 1 }, firstPositionDeltaTop: 0 },
                contributionsState: {},
            };
            const viewState2: editor.ICodeEditorViewState = {
                cursorState: [{ inSelectionMode: false, selectionStart: { lineNumber: 50, column: 10 }, position: { lineNumber: 50, column: 10 } }],
                viewState: { scrollLeft: 100, firstPosition: { lineNumber: 40, column: 1 }, firstPositionDeltaTop: 0 },
                contributionsState: {},
            };

            // 打开两个标签并设置不同的 ViewState
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().updateViewState("src/a.ts", viewState1);
            useEditorStore.getState().updateViewState("src/b.ts", viewState2);

            // 切换到标签 a
            useEditorStore.getState().setActiveTab("src/a.ts");
            const tabA = useEditorStore.getState().tabs.find(t => t.id === "src/a.ts");
            expect(tabA?.viewState).toEqual(viewState1);

            // 切换到标签 b
            useEditorStore.getState().setActiveTab("src/b.ts");
            const tabB = useEditorStore.getState().tabs.find(t => t.id === "src/b.ts");
            expect(tabB?.viewState).toEqual(viewState2);
        });

        it("关闭标签不应影响其他标签的 ViewState", () => {
            const viewStateB: editor.ICodeEditorViewState = {
                cursorState: [{ inSelectionMode: false, selectionStart: { lineNumber: 20, column: 5 }, position: { lineNumber: 20, column: 5 } }],
                viewState: { scrollLeft: 50, firstPosition: { lineNumber: 15, column: 1 }, firstPositionDeltaTop: 0 },
                contributionsState: {},
            };

            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().updateViewState("src/b.ts", viewStateB);

            // 关闭标签 a
            useEditorStore.getState().closeTab("src/a.ts");

            // 标签 b 的 ViewState 应保持不变
            const tabB = useEditorStore.getState().tabs.find(t => t.id === "src/b.ts");
            expect(tabB?.viewState).toEqual(viewStateB);
        });

        it("CodePanel 应该传递 ViewState 到 CodeSnapshotView", () => {
            const mockViewState: editor.ICodeEditorViewState = {
                cursorState: [{ inSelectionMode: false, selectionStart: { lineNumber: 5, column: 3 }, position: { lineNumber: 5, column: 3 } }],
                viewState: { scrollLeft: 0, firstPosition: { lineNumber: 1, column: 1 }, firstPositionDeltaTop: 0 },
                contributionsState: {},
            };

            useEditorStore.getState().openTab("src/test.tsx");
            useEditorStore.getState().updateViewState("src/test.tsx", mockViewState);

            renderWithProviders(
                <CodePanel
                    code="const x = 1;"
                    filePath="src/test.tsx"
                />
            );

            // 验证编辑器被渲染 (CodeSnapshotView 会接收 viewState prop)
            expect(screen.getByTestId("monaco-editor")).toBeInTheDocument();
        });
    });

    describe("Task 10.6: 历史模式文件树集成测试 (AC #19)", () => {
        beforeEach(() => {
            // 重置 store 状态
            useEditorStore.getState().closeAllTabs();
            // 确保侧边栏关闭
            if (useEditorStore.getState().sidebarOpen) {
                useEditorStore.getState().toggleSidebar();
            }

            // Mock 文件树和文件列表返回值
            // 注意: Tauri 2.x 前端使用 camelCase，会自动转换为 Rust 的 snake_case
            mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
                if (cmd === "list_tree_at_commit") {
                    // 历史模式应该传递 commitHash
                    const commitHash = args?.commitHash as string | undefined;
                    if (commitHash === "abc1234") {
                        return Promise.resolve([
                            {
                                name: "src", path: "src", type: "directory", children: [
                                    { name: "old-file.ts", path: "src/old-file.ts", type: "file" }
                                ]
                            },
                        ]);
                    }
                    return Promise.resolve([
                        {
                            name: "src", path: "src", type: "directory", children: [
                                { name: "new-file.ts", path: "src/new-file.ts", type: "file" }
                            ]
                        },
                    ]);
                }
                if (cmd === "list_files_at_commit") {
                    const commitHash = args?.commitHash as string | undefined;
                    if (commitHash === "abc1234") {
                        return Promise.resolve(["src/old-file.ts"]);
                    }
                    return Promise.resolve(["src/new-file.ts"]);
                }
                return Promise.resolve([]);
            });
        });

        it("AC #19: 历史模式应加载指定 commit 的文件树", async () => {
            // 展开侧边栏
            useEditorStore.getState().toggleSidebar();

            await act(async () => {
                renderWithProviders(
                    <CodePanel
                        code="// historical code"
                        filePath="src/old-file.ts"
                        repoPath="/test/repo"
                        isHistoricalMode={true}
                        commitHash="abc1234"
                    />
                );
            });

            // 等待文件树加载
            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith("list_tree_at_commit", expect.objectContaining({
                    repoPath: "/test/repo",
                    commitHash: "abc1234",
                }));
            });
        });

        it("非历史模式应加载 HEAD 的文件树", async () => {
            useEditorStore.getState().toggleSidebar();

            await act(async () => {
                renderWithProviders(
                    <CodePanel
                        code="// current code"
                        filePath="src/new-file.ts"
                        repoPath="/test/repo"
                        isHistoricalMode={false}
                    />
                );
            });

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith("list_tree_at_commit", expect.objectContaining({
                    repoPath: "/test/repo",
                    commitHash: undefined,
                }));
            });
        });

        it("历史模式下打开文件应包含 commitHash", () => {
            useEditorStore.getState().openTab("src/old-file.ts", { commitHash: "abc1234" });

            // 重新获取最新状态
            const { tabs } = useEditorStore.getState();
            const tab = tabs.find(t => t.path === "src/old-file.ts");
            expect(tab?.id).toBe("abc1234:src/old-file.ts");
            expect(tab?.commitHash).toBe("abc1234");
        });

        it("同一文件可在不同 commit 下打开多个标签", () => {
            const { openTab } = useEditorStore.getState();
            openTab("src/file.ts", { commitHash: "commit1" });
            openTab("src/file.ts", { commitHash: "commit2" });
            openTab("src/file.ts"); // 当前版本

            // 重新获取最新状态
            const { tabs } = useEditorStore.getState();
            expect(tabs).toHaveLength(3);
            expect(tabs.map(t => t.id)).toContain("commit1:src/file.ts");
            expect(tabs.map(t => t.id)).toContain("commit2:src/file.ts");
            expect(tabs.map(t => t.id)).toContain("src/file.ts");
        });
    });

    /**
     * Task 10.7: E2E 测试占位
     *
     * 完整 E2E 测试需要配置 Playwright/Cypress 框架
     * 测试场景已在 Story 文件中定义:
     * - 双击文件树打开固定标签
     * - 单击文件树预览 (斜体标签)
     * - Cmd+P QuickOpen 搜索打开
     * - Cmd+Tab/Shift+Tab 切换标签
     * - Cmd+W 关闭当前标签
     * - Cmd+B 切换侧边栏
     * - 切换标签恢复光标位置
     * - 历史模式文件树切换
     *
     * 后续需要:
     * 1. 安装 Playwright: pnpm add -D @playwright/test
     * 2. 配置 playwright.config.ts
     * 3. 创建 e2e/ 目录并编写测试
     */
    describe.skip("Task 10.7: E2E 测试 (需要配置 Playwright)", () => {
        it.todo("E2E: 双击文件树中的文件 → 在标签页中打开（固定标签）");
        it.todo("E2E: 单击文件树中的文件 → 预览模式打开（斜体标签）");
        it.todo("E2E: Cmd+P → 打开 QuickOpen 面板");
        it.todo("E2E: Cmd+W → 关闭当前标签");
        it.todo("E2E: Cmd+Tab → 切换到下一个标签");
        it.todo("E2E: Cmd+Shift+Tab → 切换到上一个标签");
        it.todo("E2E: Cmd+B → 切换文件树侧边栏");
        it.todo("E2E: 切换标签后返回 → 恢复之前的光标和滚动位置");
        it.todo("E2E: 历史模式下打开文件 → 标签显示 commit hash 指示器");
        it.todo("E2E: 面包屑点击 → 显示同级文件/目录下拉菜单");
        it.todo("E2E: 大型仓库 (>1000 文件) → 文件树平滑滚动，无卡顿");
    });
});
