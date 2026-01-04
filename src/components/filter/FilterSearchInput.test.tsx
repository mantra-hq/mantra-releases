/**
 * FilterSearchInput Tests - 过滤搜索输入组件测试
 * Story 2.16: Task 3.4
 * Story 2.26: 国际化支持
 */

import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { FilterSearchInput } from "./FilterSearchInput";
import { useMessageFilterStore } from "@/stores/useMessageFilterStore";
import { act } from "@testing-library/react";

describe("FilterSearchInput", () => {
    beforeEach(() => {
        vi.useFakeTimers();
        // Reset store before each test
        act(() => {
            useMessageFilterStore.setState({
                selectedTypes: new Set<string>(),
                searchQuery: "",
                isSearchFocused: false,
            });
        });
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it("should render search input with icon", () => {
        render(<FilterSearchInput />);

        expect(screen.getByRole("textbox")).toBeInTheDocument();
        expect(screen.getByLabelText("搜索消息")).toBeInTheDocument();
    });

    it("should show default placeholder text from i18n", () => {
        render(<FilterSearchInput />);

        // 测试默认 placeholder (来自 i18n 的 search.searchMessages)
        expect(screen.getByPlaceholderText("搜索消息")).toBeInTheDocument();
    });

    it("should update local value immediately", async () => {
        render(<FilterSearchInput />);
        const input = screen.getByRole("textbox");

        await act(async () => {
            fireEvent.change(input, { target: { value: "test" } });
        });

        expect(input).toHaveValue("test");
    });

    it("should debounce store updates", async () => {
        render(<FilterSearchInput debounceMs={300} />);
        const input = screen.getByRole("textbox");

        await act(async () => {
            fireEvent.change(input, { target: { value: "test" } });
        });

        // Store should not be updated immediately
        expect(useMessageFilterStore.getState().searchQuery).toBe("");

        // Advance timers
        await act(async () => {
            vi.advanceTimersByTime(300);
        });

        // Now store should be updated
        expect(useMessageFilterStore.getState().searchQuery).toBe("test");
    });

    it("should show clear button when has value", async () => {
        render(<FilterSearchInput />);
        const input = screen.getByRole("textbox");

        // Clear button should not exist initially
        expect(screen.queryByLabelText("清除搜索")).not.toBeInTheDocument();

        await act(async () => {
            fireEvent.change(input, { target: { value: "test" } });
        });

        // Clear button should appear
        expect(screen.getByLabelText("清除搜索")).toBeInTheDocument();
    });

    it("should clear input when clear button is clicked", async () => {
        render(<FilterSearchInput />);
        const input = screen.getByRole("textbox");

        await act(async () => {
            fireEvent.change(input, { target: { value: "test" } });
            vi.advanceTimersByTime(300);
        });

        const clearButton = screen.getByLabelText("清除搜索");
        await act(async () => {
            fireEvent.click(clearButton);
        });

        expect(input).toHaveValue("");
        expect(useMessageFilterStore.getState().searchQuery).toBe("");
    });

    it("should update focus state on focus/blur", async () => {
        render(<FilterSearchInput />);
        const input = screen.getByRole("textbox");

        await act(async () => {
            fireEvent.focus(input);
        });
        expect(useMessageFilterStore.getState().isSearchFocused).toBe(true);

        await act(async () => {
            fireEvent.blur(input);
        });
        expect(useMessageFilterStore.getState().isSearchFocused).toBe(false);
    });

    it("should sync with external store changes", async () => {
        render(<FilterSearchInput />);
        const input = screen.getByRole("textbox");

        await act(async () => {
            useMessageFilterStore.getState().setSearchQuery("external update");
        });

        expect(input).toHaveValue("external update");
    });

    it("should apply custom className", () => {
        render(<FilterSearchInput className="custom-class" />);

        const container = screen.getByRole("textbox").parentElement;
        expect(container).toHaveClass("custom-class");
    });
});
