/**
 * TypeChips Tests - 消息类型过滤 Chips 组件测试
 * Story 2.16: Task 2.5
 */

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TypeChips } from "./TypeChips";
import { useMessageFilterStore, MESSAGE_TYPES } from "@/stores/useMessageFilterStore";
import { act } from "@testing-library/react";

describe("TypeChips", () => {
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

    it("should render all message type chips", () => {
        render(<TypeChips />);

        MESSAGE_TYPES.forEach((type) => {
            expect(screen.getByText(type.label)).toBeInTheDocument();
        });
    });

    it("should show chips as unselected by default", () => {
        render(<TypeChips />);

        const buttons = screen.getAllByRole("button");
        buttons.forEach((button) => {
            expect(button).toHaveAttribute("aria-pressed", "false");
        });
    });

    it("should toggle chip selection on click", () => {
        render(<TypeChips />);

        const toolChip = screen.getByText("工具").closest("button");
        expect(toolChip).toHaveAttribute("aria-pressed", "false");

        fireEvent.click(toolChip!);

        expect(toolChip).toHaveAttribute("aria-pressed", "true");
        expect(useMessageFilterStore.getState().selectedTypes.has("tool")).toBe(true);
    });

    it("should support multiple selections", () => {
        render(<TypeChips />);

        const toolChip = screen.getByText("工具").closest("button");
        const fileChip = screen.getByText("文件").closest("button");
        const terminalChip = screen.getByText("命令").closest("button");

        fireEvent.click(toolChip!);
        fireEvent.click(fileChip!);
        fireEvent.click(terminalChip!);

        expect(useMessageFilterStore.getState().selectedTypes.size).toBe(3);
        expect(toolChip).toHaveAttribute("aria-pressed", "true");
        expect(fileChip).toHaveAttribute("aria-pressed", "true");
        expect(terminalChip).toHaveAttribute("aria-pressed", "true");
    });

    it("should deselect chip on second click", () => {
        render(<TypeChips />);

        const toolChip = screen.getByText("工具").closest("button");

        fireEvent.click(toolChip!);
        expect(toolChip).toHaveAttribute("aria-pressed", "true");

        fireEvent.click(toolChip!);
        expect(toolChip).toHaveAttribute("aria-pressed", "false");
        expect(useMessageFilterStore.getState().selectedTypes.has("tool")).toBe(false);
    });

    it("should display icons for each type", () => {
        render(<TypeChips />);

        MESSAGE_TYPES.forEach((type) => {
            expect(screen.getByText(type.icon)).toBeInTheDocument();
        });
    });

    it("should have proper aria attributes", () => {
        render(<TypeChips />);

        const group = screen.getByRole("group");
        expect(group).toHaveAttribute("aria-label", "消息类型过滤");
    });

    it("should apply custom className", () => {
        render(<TypeChips className="custom-class" />);

        const group = screen.getByRole("group");
        expect(group).toHaveClass("custom-class");
    });

    it("should reflect external store changes", () => {
        render(<TypeChips />);

        // Simulate external store update
        act(() => {
            useMessageFilterStore.getState().toggleType("thinking");
        });

        const thinkingChip = screen.getByText("思考").closest("button");
        expect(thinkingChip).toHaveAttribute("aria-pressed", "true");
    });
});
