/**
 * QuickOpen - 快速打开命令面板测试
 * Story 2.13: Task 6 验证
 * AC: #15 Quick Open
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { QuickOpen } from "./QuickOpen";

describe("QuickOpen", () => {
    const mockFiles = [
        "src/App.tsx",
        "src/index.ts",
        "src/components/Button.tsx",
        "src/components/Input.tsx",
        "src/utils/helpers.ts",
    ];

    // 使用假计时器处理防抖
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe("渲染", () => {
        it("open=false 时不渲染内容", () => {
            render(
                <QuickOpen
                    open={false}
                    onOpenChange={() => {}}
                    files={mockFiles}
                    onSelect={() => {}}
                />
            );

            expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
        });

        it("open=true 时渲染对话框", () => {
            render(
                <QuickOpen
                    open={true}
                    onOpenChange={() => {}}
                    files={mockFiles}
                    onSelect={() => {}}
                />
            );

            expect(screen.getByRole("dialog")).toBeInTheDocument();
        });

        it("应该显示搜索输入框", () => {
            render(
                <QuickOpen
                    open={true}
                    onOpenChange={() => {}}
                    files={mockFiles}
                    onSelect={() => {}}
                />
            );

            expect(screen.getByPlaceholderText(/搜索/i)).toBeInTheDocument();
        });

        it("应该显示文件列表", () => {
            render(
                <QuickOpen
                    open={true}
                    onOpenChange={() => {}}
                    files={mockFiles}
                    onSelect={() => {}}
                />
            );

            expect(screen.getByText("src/App.tsx")).toBeInTheDocument();
            expect(screen.getByText("src/index.ts")).toBeInTheDocument();
        });
    });

    describe("搜索过滤 (防抖 150ms)", () => {
        it("输入搜索词应过滤文件列表", async () => {
            render(
                <QuickOpen
                    open={true}
                    onOpenChange={() => {}}
                    files={mockFiles}
                    onSelect={() => {}}
                />
            );

            const input = screen.getByPlaceholderText(/搜索/i);
            fireEvent.change(input, { target: { value: "Button" } });

            // 等待防抖延迟
            await act(async () => {
                vi.advanceTimersByTime(200);
            });

            expect(screen.getByText("src/components/Button.tsx")).toBeInTheDocument();
            expect(screen.queryByText("src/App.tsx")).not.toBeInTheDocument();
        });

        it("搜索应不区分大小写", async () => {
            render(
                <QuickOpen
                    open={true}
                    onOpenChange={() => {}}
                    files={mockFiles}
                    onSelect={() => {}}
                />
            );

            const input = screen.getByPlaceholderText(/搜索/i);
            fireEvent.change(input, { target: { value: "button" } });

            await act(async () => {
                vi.advanceTimersByTime(200);
            });

            expect(screen.getByText("src/components/Button.tsx")).toBeInTheDocument();
        });

        it("无匹配结果时显示提示", async () => {
            render(
                <QuickOpen
                    open={true}
                    onOpenChange={() => {}}
                    files={mockFiles}
                    onSelect={() => {}}
                />
            );

            const input = screen.getByPlaceholderText(/搜索/i);
            fireEvent.change(input, { target: { value: "nonexistent" } });

            await act(async () => {
                vi.advanceTimersByTime(200);
            });

            expect(screen.getByText(/未找到/i)).toBeInTheDocument();
        });
    });

    describe("选择交互", () => {
        it("点击文件项应触发 onSelect", () => {
            const onSelect = vi.fn();
            const onOpenChange = vi.fn();

            render(
                <QuickOpen
                    open={true}
                    onOpenChange={onOpenChange}
                    files={mockFiles}
                    onSelect={onSelect}
                />
            );

            fireEvent.click(screen.getByText("src/App.tsx"));

            expect(onSelect).toHaveBeenCalledWith("src/App.tsx");
            expect(onOpenChange).toHaveBeenCalledWith(false);
        });
    });
});

