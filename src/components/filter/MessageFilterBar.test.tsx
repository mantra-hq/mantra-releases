/**
 * MessageFilterBar Tests - 消息过滤栏主组件测试
 * Story 2.16: Task 5.4
 */

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MessageFilterBar } from "./MessageFilterBar";
import { useMessageFilterStore } from "@/stores/useMessageFilterStore";
import { act } from "@testing-library/react";

describe("MessageFilterBar", () => {
    beforeEach(() => {
        // Reset store before each test
        act(() => {
            useMessageFilterStore.setState({
                selectedTypes: new Set<string>(),
                searchQuery: "",
                isSearchFocused: false,
            });
        });
    });

    it("should render type chips", () => {
        render(<MessageFilterBar filteredCount={10} totalCount={10} />);

        expect(screen.getByText("对话")).toBeInTheDocument();
        expect(screen.getByText("工具")).toBeInTheDocument();
        expect(screen.getByText("文件")).toBeInTheDocument();
    });

    it("should render search input", () => {
        render(<MessageFilterBar filteredCount={10} totalCount={10} />);

        expect(screen.getByLabelText("搜索消息")).toBeInTheDocument();
    });

    it("should show filter stats when filtered", () => {
        render(<MessageFilterBar filteredCount={5} totalCount={10} />);

        expect(screen.getByText("匹配: 5/10 条")).toBeInTheDocument();
    });

    it("should not show filter stats when all shown", () => {
        render(<MessageFilterBar filteredCount={10} totalCount={10} />);

        expect(screen.queryByText(/匹配:/)).not.toBeInTheDocument();
    });

    it("should not show clear button when no filters active", () => {
        render(<MessageFilterBar filteredCount={10} totalCount={10} />);

        expect(screen.queryByText("清除过滤")).not.toBeInTheDocument();
    });

    it("should show clear button when type filter is active", async () => {
        render(<MessageFilterBar filteredCount={10} totalCount={10} />);

        // Activate a type filter
        act(() => {
            useMessageFilterStore.getState().toggleType("tool");
        });

        expect(await screen.findByText("清除过滤")).toBeInTheDocument();
    });

    it("should show clear button when search query is active", async () => {
        render(<MessageFilterBar filteredCount={10} totalCount={10} />);

        // Set a search query
        act(() => {
            useMessageFilterStore.getState().setSearchQuery("test");
        });

        expect(await screen.findByText("清除过滤")).toBeInTheDocument();
    });

    it("should clear all filters when clear button is clicked", async () => {
        render(<MessageFilterBar filteredCount={10} totalCount={10} />);

        // Activate filters
        act(() => {
            useMessageFilterStore.getState().toggleType("tool");
            useMessageFilterStore.getState().setSearchQuery("test");
        });

        const clearButton = await screen.findByText("清除过滤");
        fireEvent.click(clearButton);

        const state = useMessageFilterStore.getState();
        expect(state.selectedTypes.size).toBe(0);
        expect(state.searchQuery).toBe("");
    });

    it("should apply custom className", () => {
        const { container } = render(
            <MessageFilterBar
                filteredCount={10}
                totalCount={10}
                className="custom-class"
            />
        );

        expect(container.firstChild).toHaveClass("custom-class");
    });

    describe("keyboard shortcuts (AC #12, #13)", () => {
        it("should focus search input on Cmd/Ctrl+F", () => {
            render(<MessageFilterBar filteredCount={10} totalCount={10} />);
            const input = screen.getByLabelText("搜索消息");

            // Simulate Ctrl+F
            fireEvent.keyDown(document, { key: "f", ctrlKey: true });

            expect(document.activeElement).toBe(input);
        });

        it("should clear search and blur on Escape when search has content", async () => {
            // Set search query first
            act(() => {
                useMessageFilterStore.getState().setSearchQuery("test");
            });

            render(<MessageFilterBar filteredCount={10} totalCount={10} />);
            const input = screen.getByLabelText("搜索消息");

            // Focus input first
            act(() => {
                input.focus();
            });
            await waitFor(() => {
                expect(document.activeElement).toBe(input);
            });

            // Simulate Escape
            act(() => {
                fireEvent.keyDown(document, { key: "Escape" });
            });

            await waitFor(() => {
                expect(useMessageFilterStore.getState().searchQuery).toBe("");
            });
        });
    });
});
