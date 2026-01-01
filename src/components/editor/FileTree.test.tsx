/**
 * FileTree - 文件树组件测试
 * Story 2.13: Task 4 验证
 * AC: #8 侧边栏触发, #9 文件树显示, #10 当前文件高亮, 
 *     #11 双击打开, #12 单击预览, #13 目录操作, #14 虚拟化
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { FileTree, type TreeNode } from "./FileTree";
import { useEditorStore } from "@/stores/useEditorStore";

const mockTree: TreeNode[] = [
    {
        name: "src",
        path: "src",
        type: "directory",
        children: [
            {
                name: "components",
                path: "src/components",
                type: "directory",
                children: [
                    { name: "App.tsx", path: "src/components/App.tsx", type: "file" },
                    { name: "Button.tsx", path: "src/components/Button.tsx", type: "file" },
                ],
            },
            { name: "index.ts", path: "src/index.ts", type: "file" },
        ],
    },
    { name: "package.json", path: "package.json", type: "file" },
];

describe("FileTree", () => {
    beforeEach(() => {
        useEditorStore.getState().closeAllTabs();
        // 清除展开状态
        const state = useEditorStore.getState();
        state.expandedFolders.clear();
    });

    describe("渲染 (AC #9)", () => {
        it("应该渲染文件树", () => {
            render(<FileTree tree={mockTree} />);

            expect(screen.getByText("src")).toBeInTheDocument();
            expect(screen.getByText("package.json")).toBeInTheDocument();
        });

        it("应该显示文件/文件夹图标", () => {
            render(<FileTree tree={mockTree} />);

            // 文件夹应该有展开/折叠箭头
            const srcFolder = screen.getByText("src").closest("[data-node]");
            expect(srcFolder).toBeInTheDocument();
        });

        it("折叠状态下不显示子节点", () => {
            render(<FileTree tree={mockTree} />);

            // 初始状态: src 是折叠的，所以看不到 components
            expect(screen.queryByText("components")).not.toBeInTheDocument();
        });
    });

    describe("目录折叠/展开 (AC #13)", () => {
        it("点击目录应展开/折叠", () => {
            render(<FileTree tree={mockTree} />);

            // 点击 src 展开
            fireEvent.click(screen.getByText("src"));
            expect(screen.getByText("index.ts")).toBeInTheDocument();
            expect(screen.getByText("components")).toBeInTheDocument();

            // 再次点击折叠
            fireEvent.click(screen.getByText("src"));
            expect(screen.queryByText("index.ts")).not.toBeInTheDocument();
        });

        it("展开嵌套目录", () => {
            render(<FileTree tree={mockTree} />);

            // 先展开 src
            fireEvent.click(screen.getByText("src"));
            
            // 再展开 components
            fireEvent.click(screen.getByText("components"));
            
            expect(screen.getByText("App.tsx")).toBeInTheDocument();
            expect(screen.getByText("Button.tsx")).toBeInTheDocument();
        });
    });

    describe("文件点击 (AC #11, #12)", () => {
        it("AC #12: 单击文件应调用 onFileClick (预览)", () => {
            const onFileClick = vi.fn();
            
            render(
                <FileTree
                    tree={mockTree}
                    onFileClick={onFileClick}
                />
            );

            // 先展开 src
            fireEvent.click(screen.getByText("src"));
            
            // 单击 index.ts
            fireEvent.click(screen.getByText("index.ts"));
            
            expect(onFileClick).toHaveBeenCalledWith("src/index.ts");
        });

        it("AC #11: 双击文件应调用 onFileDoubleClick (打开)", () => {
            const onFileDoubleClick = vi.fn();
            
            render(
                <FileTree
                    tree={mockTree}
                    onFileDoubleClick={onFileDoubleClick}
                />
            );

            // 先展开 src
            fireEvent.click(screen.getByText("src"));
            
            // 双击 index.ts
            fireEvent.doubleClick(screen.getByText("index.ts"));
            
            expect(onFileDoubleClick).toHaveBeenCalledWith("src/index.ts");
        });

        it("点击目录不应触发 onFileClick", () => {
            const onFileClick = vi.fn();
            
            render(
                <FileTree
                    tree={mockTree}
                    onFileClick={onFileClick}
                />
            );

            // 点击目录
            fireEvent.click(screen.getByText("src"));
            
            expect(onFileClick).not.toHaveBeenCalled();
        });
    });

    describe("当前文件高亮 (AC #10)", () => {
        it("应该高亮当前打开的文件", () => {
            render(
                <FileTree
                    tree={mockTree}
                    activeFilePath="src/index.ts"
                />
            );

            // 先展开 src
            fireEvent.click(screen.getByText("src"));
            
            const activeNode = screen.getByText("index.ts").closest("[data-node]");
            expect(activeNode).toHaveAttribute("data-active", "true");
        });
    });

    describe("虚拟化 (AC #14)", () => {
        it("应该渲染虚拟化容器", () => {
            render(<FileTree tree={mockTree} />);
            
            // 验证虚拟化容器存在
            const container = screen.getByRole("tree");
            expect(container).toBeInTheDocument();
        });
    });
});


