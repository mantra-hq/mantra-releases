/**
 * EditorTabs - 编辑器标签页组件测试
 * Story 2.13: Task 2 验证
 * AC: #1 标签页显示, #2 标签切换, #3 标签关闭, #4 标签溢出, #5 ViewState
 */

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { EditorTabs } from "./EditorTabs";
import { useEditorStore } from "@/stores/useEditorStore";
import { TooltipProvider } from "@/components/ui/tooltip";

// 包装组件，提供必要的 Provider
function renderWithProviders(ui: React.ReactElement) {
    return render(<TooltipProvider>{ui}</TooltipProvider>);
}

describe("EditorTabs", () => {
    // 每个测试前重置 store
    beforeEach(() => {
        useEditorStore.getState().closeAllTabs();
    });

    describe("渲染", () => {
        it("AC #1: 无标签时不渲染", () => {
            const { container } = renderWithProviders(<EditorTabs />);
            // TooltipProvider 会渲染，但 EditorTabs 返回 null
            expect(container.querySelector('[role="tablist"]')).toBeNull();
        });

        it("AC #1: 应该渲染标签页", () => {
            useEditorStore.getState().openTab("src/App.tsx");
            useEditorStore.getState().openTab("src/index.ts");

            renderWithProviders(<EditorTabs />);

            expect(screen.getByText("App.tsx")).toBeInTheDocument();
            expect(screen.getByText("index.ts")).toBeInTheDocument();
        });

        it("AC #1: 预览模式标签应显示斜体样式", () => {
            useEditorStore.getState().openTab("src/preview.ts", { preview: true });

            renderWithProviders(<EditorTabs />);

            const tab = screen.getByText("preview.ts").closest("[data-tab]");
            expect(tab).toHaveClass("italic");
        });

        it("AC #2: 当前激活标签应有高亮样式", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");

            renderWithProviders(<EditorTabs />);

            const activeTab = screen.getByText("b.ts").closest("[data-tab]");
            expect(activeTab).toHaveAttribute("data-active", "true");
        });

        it("应该显示历史模式指示器", () => {
            useEditorStore.getState().openTab("src/App.tsx", { commitHash: "abc1234" });

            renderWithProviders(<EditorTabs />);

            // 历史模式标签应有 data-historical 属性
            const tab = screen.getByRole("tab");
            expect(tab).toHaveAttribute("data-historical", "true");
            // 历史模式标签应有琥珀色背景样式
            expect(tab).toHaveClass("bg-amber-500/5");
        });
    });

    describe("交互", () => {
        it("AC #2: 点击标签应切换激活状态", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");

            renderWithProviders(<EditorTabs />);

            // 点击第一个标签
            fireEvent.click(screen.getByText("a.ts"));

            expect(useEditorStore.getState().activeTabId).toBe("src/a.ts");
        });

        it("AC #2: 双击预览标签应固定", () => {
            useEditorStore.getState().openTab("src/preview.ts", { preview: true });

            renderWithProviders(<EditorTabs />);

            // 双击
            fireEvent.doubleClick(screen.getByText("preview.ts"));

            const tab = useEditorStore.getState().tabs[0];
            expect(tab.isPinned).toBe(true);
            expect(tab.isPreview).toBe(false);
        });

        it("AC #3: 点击关闭按钮应关闭标签", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");

            renderWithProviders(<EditorTabs />);

            // 找到第一个标签的关闭按钮
            const closeButtons = screen.getAllByRole("button", { name: /关闭/i });
            fireEvent.click(closeButtons[0]);

            expect(useEditorStore.getState().tabs).toHaveLength(1);
            expect(useEditorStore.getState().tabs[0].path).toBe("src/b.ts");
        });

        it("AC #3: 关闭按钮点击不应触发标签切换", () => {
            useEditorStore.getState().openTab("src/a.ts");
            useEditorStore.getState().openTab("src/b.ts");
            // 当前激活 b.ts

            renderWithProviders(<EditorTabs />);

            // 关闭 a.ts
            const closeButtons = screen.getAllByRole("button", { name: /关闭/i });
            fireEvent.click(closeButtons[0]);

            // b.ts 仍然是激活的
            expect(useEditorStore.getState().activeTabId).toBe("src/b.ts");
        });
    });

    describe("标签溢出 (AC #4)", () => {
        it("应该渲染滚动容器", () => {
            // 添加多个标签
            for (let i = 0; i < 10; i++) {
                useEditorStore.getState().openTab(`src/file${i}.ts`);
            }

            renderWithProviders(<EditorTabs />);

            // 验证标签容器存在
            const container = screen.getByRole("tablist");
            expect(container).toBeInTheDocument();
        });
    });
});

