/**
 * FloatingCollapseBar 测试
 * Story 2.15: Task 7.6
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { FloatingCollapseBar } from "./FloatingCollapseBar";

describe("FloatingCollapseBar", () => {
    it("visible=false 时不渲染", () => {
        render(
            <FloatingCollapseBar visible={false} onCollapse={vi.fn()} />
        );

        expect(screen.queryByTestId("floating-collapse-bar")).not.toBeInTheDocument();
    });

    it("visible=true 时渲染浮动栏", () => {
        render(
            <FloatingCollapseBar visible={true} onCollapse={vi.fn()} />
        );

        expect(screen.getByTestId("floating-collapse-bar")).toBeInTheDocument();
    });

    it("应该显示收起按钮", () => {
        render(
            <FloatingCollapseBar visible={true} onCollapse={vi.fn()} />
        );

        expect(screen.getByText("收起")).toBeInTheDocument();
    });

    it("点击收起应该调用 onCollapse", () => {
        const onCollapse = vi.fn();
        render(
            <FloatingCollapseBar visible={true} onCollapse={onCollapse} />
        );

        fireEvent.click(screen.getByText("收起"));
        expect(onCollapse).toHaveBeenCalled();
    });

    it("有 onScrollToTop 时应该显示回到顶部按钮", () => {
        render(
            <FloatingCollapseBar
                visible={true}
                onCollapse={vi.fn()}
                onScrollToTop={vi.fn()}
            />
        );

        expect(screen.getByText("回到顶部")).toBeInTheDocument();
    });

    it("点击回到顶部应该调用 onScrollToTop", () => {
        const onScrollToTop = vi.fn();
        render(
            <FloatingCollapseBar
                visible={true}
                onCollapse={vi.fn()}
                onScrollToTop={onScrollToTop}
            />
        );

        fireEvent.click(screen.getByText("回到顶部"));
        expect(onScrollToTop).toHaveBeenCalled();
    });

    it("没有 onScrollToTop 时不显示回到顶部按钮", () => {
        render(
            <FloatingCollapseBar visible={true} onCollapse={vi.fn()} />
        );

        expect(screen.queryByText("回到顶部")).not.toBeInTheDocument();
    });
});
