/**
 * useEditorStore - 编辑器状态管理测试
 * Story 2.13: Task 1 验证
 * AC: #1 标签页显示, #2 标签切换, #3 标签关闭, #5 ViewState 保存
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "./useEditorStore";
import type { editor } from "monaco-editor";

describe("useEditorStore", () => {
    // 每个测试前重置 store
    beforeEach(() => {
        useEditorStore.getState().closeAllTabs();
        // 重置侧边栏和文件夹展开状态
        if (useEditorStore.getState().sidebarOpen) {
            useEditorStore.getState().toggleSidebar();
        }
    });

    describe("初始状态", () => {
        it("应该有正确的初始值", () => {
            const state = useEditorStore.getState();

            expect(state.tabs).toEqual([]);
            expect(state.activeTabId).toBeNull();
            expect(state.sidebarOpen).toBe(false);
            expect(state.expandedFolders).toBeInstanceOf(Set);
            expect(state.expandedFolders.size).toBe(0);
        });
    });

    describe("openTab", () => {
        it("AC #1: 应该打开新标签并设为激活 (双击 = 固定模式)", () => {
            useEditorStore.getState().openTab("src/App.tsx");

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(1);
            expect(state.tabs[0].path).toBe("src/App.tsx");
            expect(state.tabs[0].label).toBe("App.tsx");
            expect(state.tabs[0].isPinned).toBe(true);
            expect(state.tabs[0].isPreview).toBe(false);
            expect(state.activeTabId).toBe("src/App.tsx");
        });

        it("AC #1: 应该支持预览模式 (单击 = 斜体)", () => {
            useEditorStore.getState().openTab("src/index.ts", { preview: true });

            const state = useEditorStore.getState();
            expect(state.tabs[0].isPinned).toBe(false);
            expect(state.tabs[0].isPreview).toBe(true);
        });

        it("AC #1: 预览模式下打开新文件应替换现有预览标签", () => {
            useEditorStore.getState().openTab("src/a.ts", { preview: true });
            useEditorStore.getState().openTab("src/b.ts", { preview: true });

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(1);
            expect(state.tabs[0].path).toBe("src/b.ts");
        });

        it("AC #1: 固定模式不应替换预览标签", () => {
            useEditorStore.getState().openTab("src/a.ts", { preview: true });
            useEditorStore.getState().openTab("src/b.ts"); // 固定模式

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(2);
        });

        it("AC #2: 再次打开已存在的标签应切换到该标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().openTab("src/a.ts"); // 再次打开

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(2);
            expect(state.activeTabId).toBe("src/a.ts");
        });

        it("AC #2: 双击预览标签应将其固定", () => {
            useEditorStore.getState().openTab("src/a.ts", { preview: true });
            useEditorStore.getState().openTab("src/a.ts"); // 双击固定

            const state = useEditorStore.getState();
            expect(state.tabs[0].isPinned).toBe(true);
            expect(state.tabs[0].isPreview).toBe(false);
        });

        it("应该支持历史模式 (commitHash)", () => {
            useEditorStore.getState().openTab("src/App.tsx", { commitHash: "abc1234" });

            const state = useEditorStore.getState();
            expect(state.tabs[0].id).toBe("abc1234:src/App.tsx");
            expect(state.tabs[0].commitHash).toBe("abc1234");
        });

        it("同文件不同 commit 应作为不同标签", () => {
            useEditorStore.getState().openTab("src/App.tsx");
            useEditorStore.getState().openTab("src/App.tsx", { commitHash: "abc1234" });

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(2);
            expect(state.tabs[0].id).toBe("src/App.tsx");
            expect(state.tabs[1].id).toBe("abc1234:src/App.tsx");
        });
    });

    describe("closeTab (AC #3)", () => {
        it("应该关闭指定标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().closeTab("src/a.ts");

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(1);
            expect(state.tabs[0].path).toBe("src/b.ts");
        });

        it("关闭当前激活标签后应激活相邻标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().openTab("src/c.ts");
            // 当前激活: c.ts
            useEditorStore.getState().closeTab("src/c.ts");

            const state = useEditorStore.getState();
            expect(state.activeTabId).toBe("src/b.ts");
        });

        it("关闭中间标签应激活下一个标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().openTab("src/c.ts");
            useEditorStore.getState().setActiveTab("src/b.ts");
            useEditorStore.getState().closeTab("src/b.ts");

            const state = useEditorStore.getState();
            expect(state.activeTabId).toBe("src/c.ts");
        });

        it("关闭所有标签后 activeTabId 应为 null", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().closeTab("src/a.ts");

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(0);
            expect(state.activeTabId).toBeNull();
        });
    });

    describe("setActiveTab (AC #2)", () => {
        it("应该设置激活标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().setActiveTab("src/a.ts");

            expect(useEditorStore.getState().activeTabId).toBe("src/a.ts");
        });
    });

    describe("pinTab", () => {
        it("应该将预览标签固定", () => {
            useEditorStore.getState().openTab("src/a.ts", { preview: true });
            useEditorStore.getState().pinTab("src/a.ts");

            const state = useEditorStore.getState();
            expect(state.tabs[0].isPinned).toBe(true);
            expect(state.tabs[0].isPreview).toBe(false);
        });
    });

    describe("updateViewState (AC #5)", () => {
        it("应该保存标签的 ViewState", () => {
            useEditorStore.getState().openTab("src/a.ts");

            const mockViewState = {
                cursorState: [{ position: { lineNumber: 10, column: 5 } }],
                viewState: { scrollTop: 100 },
            } as unknown as editor.ICodeEditorViewState;

            useEditorStore.getState().updateViewState("src/a.ts", mockViewState);

            const state = useEditorStore.getState();
            expect(state.tabs[0].viewState).toEqual(mockViewState);
        });

        it("每个标签应独立保存 ViewState", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");

            const viewStateA = {
                cursorState: [{ position: { lineNumber: 10 } }],
            } as unknown as editor.ICodeEditorViewState;
            const viewStateB = {
                cursorState: [{ position: { lineNumber: 20 } }],
            } as unknown as editor.ICodeEditorViewState;

            useEditorStore.getState().updateViewState("src/a.ts", viewStateA);
            useEditorStore.getState().updateViewState("src/b.ts", viewStateB);

            const state = useEditorStore.getState();
            expect(state.tabs.find((t) => t.id === "src/a.ts")?.viewState).toEqual(viewStateA);
            expect(state.tabs.find((t) => t.id === "src/b.ts")?.viewState).toEqual(viewStateB);
        });
    });

    describe("toggleSidebar", () => {
        it("应该切换侧边栏状态", () => {
            expect(useEditorStore.getState().sidebarOpen).toBe(false);

            useEditorStore.getState().toggleSidebar();
            expect(useEditorStore.getState().sidebarOpen).toBe(true);

            useEditorStore.getState().toggleSidebar();
            expect(useEditorStore.getState().sidebarOpen).toBe(false);
        });
    });

    describe("toggleFolder", () => {
        it("应该展开/折叠文件夹", () => {
            useEditorStore.getState().toggleFolder("src");

            let state = useEditorStore.getState();
            expect(state.expandedFolders.has("src")).toBe(true);

            useEditorStore.getState().toggleFolder("src");

            state = useEditorStore.getState();
            expect(state.expandedFolders.has("src")).toBe(false);
        });

        it("应该支持多个文件夹独立展开", () => {
            useEditorStore.getState().toggleFolder("src");
            useEditorStore.getState().toggleFolder("components");

            const state = useEditorStore.getState();
            expect(state.expandedFolders.has("src")).toBe(true);
            expect(state.expandedFolders.has("components")).toBe(true);
        });
    });

    describe("nextTab / prevTab", () => {
        beforeEach(() => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().openTab("src/c.ts");
        });

        it("nextTab 应该切换到下一个标签", () => {
            useEditorStore.getState().setActiveTab("src/a.ts");
            useEditorStore.getState().nextTab();

            expect(useEditorStore.getState().activeTabId).toBe("src/b.ts");
        });

        it("nextTab 在最后一个标签时应循环到第一个", () => {
            useEditorStore.getState().setActiveTab("src/c.ts");
            useEditorStore.getState().nextTab();

            expect(useEditorStore.getState().activeTabId).toBe("src/a.ts");
        });

        it("prevTab 应该切换到上一个标签", () => {
            useEditorStore.getState().setActiveTab("src/b.ts");
            useEditorStore.getState().prevTab();

            expect(useEditorStore.getState().activeTabId).toBe("src/a.ts");
        });

        it("prevTab 在第一个标签时应循环到最后一个", () => {
            useEditorStore.getState().setActiveTab("src/a.ts");
            useEditorStore.getState().prevTab();

            expect(useEditorStore.getState().activeTabId).toBe("src/c.ts");
        });
    });

    describe("closeCurrentTab", () => {
        it("应该关闭当前激活的标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            // 当前激活: b.ts

            useEditorStore.getState().closeCurrentTab();

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(1);
            expect(state.tabs[0].path).toBe("src/a.ts");
        });

        it("没有标签时不应报错", () => {
            expect(() => useEditorStore.getState().closeCurrentTab()).not.toThrow();
        });
    });

    describe("closeAllTabs", () => {
        it("应该关闭所有标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            useEditorStore.getState().closeAllTabs();

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(0);
            expect(state.activeTabId).toBeNull();
        });
    });

    // Story 2.14: 快照管理测试
    describe("openTab with isSnapshot (Story 2.14)", () => {
        it("应该使用 snapshot:timestamp:path 格式作为 tabId", () => {
            const snapshotTime = Date.now();
            useEditorStore.getState().openTab("src/App.tsx", {
                isSnapshot: true,
                snapshotTime,
            });

            const state = useEditorStore.getState();
            expect(state.tabs[0].id).toBe(`snapshot:${snapshotTime}:src/App.tsx`);
            expect(state.tabs[0].isSnapshot).toBe(true);
            expect(state.tabs[0].snapshotTime).toBe(snapshotTime);
        });

        it("快照标签和实时标签应作为不同标签", () => {
            useEditorStore.getState().openTab("src/App.tsx");
            useEditorStore.getState().openTab("src/App.tsx", {
                isSnapshot: true,
                snapshotTime: Date.now(),
            });

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(2);
        });
    });

    describe("findLiveTab (Story 2.14)", () => {
        it("应该找到实时标签", () => {
            useEditorStore.getState().openTab("src/App.tsx");

            const liveTab = useEditorStore.getState().findLiveTab("src/App.tsx");
            expect(liveTab).toBeDefined();
            expect(liveTab?.path).toBe("src/App.tsx");
        });

        it("不应该返回快照标签", () => {
            useEditorStore.getState().openTab("src/App.tsx", {
                isSnapshot: true,
                snapshotTime: Date.now(),
            });

            const liveTab = useEditorStore.getState().findLiveTab("src/App.tsx");
            expect(liveTab).toBeUndefined();
        });

        it("不应该返回 Git 历史标签", () => {
            useEditorStore.getState().openTab("src/App.tsx", {
                commitHash: "abc1234",
            });

            const liveTab = useEditorStore.getState().findLiveTab("src/App.tsx");
            expect(liveTab).toBeUndefined();
        });
    });

    describe("exitSnapshot (Story 2.14)", () => {
        it("AC #7: 已有实时标签时应关闭快照并切换到实时标签", () => {
            useEditorStore.getState().openTab("src/App.tsx");
            const snapshotTime = Date.now();
            useEditorStore.getState().openTab("src/App.tsx", {
                isSnapshot: true,
                snapshotTime,
            });

            const snapshotTabId = `snapshot:${snapshotTime}:src/App.tsx`;
            useEditorStore.getState().exitSnapshot(snapshotTabId);

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(1);
            expect(state.tabs[0].id).toBe("src/App.tsx");
            expect(state.activeTabId).toBe("src/App.tsx");
        });

        it("AC #8: 无实时标签时应将快照转换为实时标签", () => {
            const snapshotTime = Date.now();
            useEditorStore.getState().openTab("src/App.tsx", {
                isSnapshot: true,
                snapshotTime,
            });

            const snapshotTabId = `snapshot:${snapshotTime}:src/App.tsx`;
            useEditorStore.getState().exitSnapshot(snapshotTabId);

            const state = useEditorStore.getState();
            expect(state.tabs).toHaveLength(1);
            expect(state.tabs[0].id).toBe("src/App.tsx");
            expect(state.tabs[0].isSnapshot).toBe(false);
            expect(state.tabs[0].snapshotTime).toBeUndefined();
            expect(state.activeTabId).toBe("src/App.tsx");
        });

        it("快照不存在时不应报错", () => {
            expect(() => useEditorStore.getState().exitSnapshot("non-existent")).not.toThrow();
        });
    });
});


