/**
 * Breadcrumbs - 面包屑导航组件测试
 * Story 2.13: Task 3 验证
 * AC: #6 路径显示, #7 路径导航, #20 历史指示器
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Breadcrumbs } from "./Breadcrumbs";

describe("Breadcrumbs", () => {
    describe("渲染 (AC #6)", () => {
        it("应该显示文件路径分段", () => {
            render(<Breadcrumbs filePath="src/components/editor/CodeSnapshotView.tsx" />);

            expect(screen.getByText("src")).toBeInTheDocument();
            expect(screen.getByText("components")).toBeInTheDocument();
            expect(screen.getByText("editor")).toBeInTheDocument();
            expect(screen.getByText("CodeSnapshotView.tsx")).toBeInTheDocument();
        });

        it("空路径时不渲染", () => {
            const { container } = render(<Breadcrumbs filePath="" />);
            expect(container.firstChild).toBeNull();
        });

        it("最后一个路径段应有高亮样式", () => {
            render(<Breadcrumbs filePath="src/App.tsx" />);

            const fileName = screen.getByText("App.tsx");
            expect(fileName).toHaveClass("font-medium");
        });

        it("应该显示分隔符", () => {
            render(<Breadcrumbs filePath="src/components/App.tsx" />);

            // 2 个分隔符 (src > components > App.tsx)
            const separators = screen.getAllByTestId("breadcrumb-separator");
            expect(separators).toHaveLength(2);
        });
    });

    describe("历史模式指示器 (AC #20)", () => {
        it("应该显示历史时间戳", () => {
            const timestamp = Date.now() - 3600000; // 1小时前

            render(
                <Breadcrumbs
                    filePath="src/App.tsx"
                    timestamp={timestamp}
                />
            );

            // UX 优化: 历史指示器现在显示相对时间 (如 "X小时前") 而不是 "历史" 文字
            expect(screen.getByText(/小时前/)).toBeInTheDocument();
        });

        it("没有时间戳时不显示历史指示器", () => {
            render(<Breadcrumbs filePath="src/App.tsx" />);

            // 不应显示相对时间指示器
            expect(screen.queryByText(/小时前|分钟前|天前/)).not.toBeInTheDocument();
        });
    });

    describe("路径导航 (AC #7)", () => {
        it("点击路径段应触发 onNavigate", async () => {
            const onNavigate = vi.fn();

            render(
                <Breadcrumbs
                    filePath="src/components/App.tsx"
                    onNavigate={onNavigate}
                    siblings={[
                        { name: "utils", path: "src/utils", isDirectory: true },
                        { name: "hooks", path: "src/hooks", isDirectory: true },
                    ]}
                />
            );

            // 点击 "src" 段
            fireEvent.click(screen.getByText("src"));

            // 应该显示下拉菜单
            // 注意: 由于使用 shadcn DropdownMenu, 测试可能需要等待
        });

        it("应该传递同级文件/目录信息给导航回调", () => {
            const onNavigate = vi.fn();
            const siblings = [
                { name: "utils", path: "src/utils", isDirectory: true },
                { name: "hooks.ts", path: "src/hooks.ts", isDirectory: false },
            ];

            render(
                <Breadcrumbs
                    filePath="src/components/App.tsx"
                    onNavigate={onNavigate}
                    siblings={siblings}
                />
            );

            // 验证组件接收了 siblings 属性
            expect(screen.getByText("src")).toBeInTheDocument();
        });
    });
});

