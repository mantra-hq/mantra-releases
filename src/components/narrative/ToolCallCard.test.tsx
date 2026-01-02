/**
 * ToolCallCard 测试
 * Story 2.15: Task 3.5
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

// Mock IntersectionObserver (used by useCollapsible)
vi.stubGlobal("IntersectionObserver", class MockIntersectionObserver {
    callback: IntersectionObserverCallback;
    constructor(callback: IntersectionObserverCallback) {
        this.callback = callback;
    }
    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
});

import { ToolCallCard } from "./ToolCallCard";

describe("ToolCallCard", () => {
    const defaultProps = {
        toolUseId: "test-id-123",
        toolName: "read_file",
        toolInput: { path: "/src/test.ts" },
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("应该渲染工具卡片", () => {
        render(<ToolCallCard {...defaultProps} />);

        const card = screen.getByTestId("tool-call-card");
        expect(card).toBeInTheDocument();
        expect(card).toHaveAttribute("data-tool-use-id", "test-id-123");
    });

    it("应该显示工具名称", () => {
        render(<ToolCallCard {...defaultProps} />);

        expect(screen.getByText("read_file")).toBeInTheDocument();
    });

    it("应该为 read_file 显示智能摘要", () => {
        render(<ToolCallCard {...defaultProps} />);

        // 只显示文件名 (不包含 emoji，因为使用的是 Lucide 图标组件)
        expect(screen.getByText("test.ts")).toBeInTheDocument();
    });

    it("应该为 run_command 显示命令摘要", () => {
        render(
            <ToolCallCard
                toolUseId="cmd-1"
                toolName="run_command"
                toolInput={{ command: "npm install" }}
            />
        );

        expect(screen.getByText(/\$ npm install/)).toBeInTheDocument();
    });

    it("应该为 grep_search 显示搜索摘要", () => {
        render(
            <ToolCallCard
                toolUseId="grep-1"
                toolName="grep_search"
                toolInput={{ query: "function" }}
            />
        );

        // 不包含 emoji，因为使用的是 Lucide 图标组件
        expect(screen.getByText(/"function"/)).toBeInTheDocument();
    });

    it("应该显示耗时", () => {
        render(<ToolCallCard {...defaultProps} duration={1.5} />);

        expect(screen.getByText("1.5s")).toBeInTheDocument();
    });

    it("应该显示毫秒级耗时", () => {
        render(<ToolCallCard {...defaultProps} duration={0.25} />);

        expect(screen.getByText("250ms")).toBeInTheDocument();
    });

    it("成功状态应该显示绿色图标", () => {
        render(<ToolCallCard {...defaultProps} status="success" />);

        const card = screen.getByTestId("tool-call-card");
        expect(card).not.toHaveClass("border-destructive");
    });

    it("错误状态应该显示红色边框", () => {
        render(<ToolCallCard {...defaultProps} status="error" />);

        const card = screen.getByTestId("tool-call-card");
        expect(card).toHaveClass("border-destructive");
    });

    it("高亮状态应该显示 ring", () => {
        render(<ToolCallCard {...defaultProps} isHighlighted={true} />);

        const card = screen.getByTestId("tool-call-card");
        expect(card).toHaveClass("ring-2");
    });

    it("应该显示展开原始内容按钮", () => {
        render(<ToolCallCard {...defaultProps} />);

        // 使用 title 属性查找展开按钮
        expect(screen.getByTitle("展开原始内容")).toBeInTheDocument();
    });

    it("点击展开应该显示 JSON 内容", () => {
        render(<ToolCallCard {...defaultProps} />);

        // 使用 title 属性查找展开按钮
        const expandButton = screen.getByTitle("展开原始内容");
        fireEvent.click(expandButton);

        // 展开后 title 变为 "收起"
        expect(screen.getByTitle("收起")).toBeInTheDocument();
    });

    it("应该调用 onHover 回调", () => {
        const onHover = vi.fn();
        render(<ToolCallCard {...defaultProps} onHover={onHover} />);

        const card = screen.getByTestId("tool-call-card");
        fireEvent.mouseEnter(card);

        expect(onHover).toHaveBeenCalledWith("test-id-123");

        fireEvent.mouseLeave(card);
        expect(onHover).toHaveBeenCalledWith(null);
    });

    it("应该调用 onViewDetail 回调", () => {
        const onViewDetail = vi.fn();
        render(<ToolCallCard {...defaultProps} onViewDetail={onViewDetail} />);

        // 按钮文本是 "详情" 而非 "查看详情 →"
        const button = screen.getByText("详情");
        fireEvent.click(button);

        expect(onViewDetail).toHaveBeenCalledWith("test-id-123");
    });

    it("没有 onViewDetail 时不显示查看详情按钮", () => {
        render(<ToolCallCard {...defaultProps} />);

        // 按钮文本是 "详情"
        expect(screen.queryByText("详情")).not.toBeInTheDocument();
    });
});
