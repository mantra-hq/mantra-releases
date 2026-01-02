/**
 * EmptyFilterResult Tests - 空过滤结果组件测试
 * Story 2.16: Task 7.2 (测试部分)
 */

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { EmptyFilterResult } from "./EmptyFilterResult";
import { useMessageFilterStore } from "@/stores/useMessageFilterStore";
import { act } from "@testing-library/react";

describe("EmptyFilterResult", () => {
    beforeEach(() => {
        // Reset store before each test
        act(() => {
            useMessageFilterStore.setState({
                selectedTypes: new Set<string>(["tool"]),
                searchQuery: "test query",
                isSearchFocused: false,
            });
        });
    });

    it("should render empty state message", () => {
        render(<EmptyFilterResult />);

        expect(screen.getByText("没有找到匹配的消息")).toBeInTheDocument();
        expect(
            screen.getByText("尝试调整过滤条件或清除搜索关键词")
        ).toBeInTheDocument();
    });

    it("should render clear filter button", () => {
        render(<EmptyFilterResult />);

        expect(screen.getByText("清除过滤条件")).toBeInTheDocument();
    });

    it("should clear filters when button is clicked", () => {
        render(<EmptyFilterResult />);

        const clearButton = screen.getByText("清除过滤条件");
        fireEvent.click(clearButton);

        const state = useMessageFilterStore.getState();
        expect(state.selectedTypes.size).toBe(0);
        expect(state.searchQuery).toBe("");
    });

    it("should apply custom className", () => {
        const { container } = render(<EmptyFilterResult className="custom-class" />);

        expect(container.firstChild).toHaveClass("custom-class");
    });
});
