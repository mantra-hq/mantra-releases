/**
 * SearchResultList Tests - 搜索结果列表测试
 * Story 2.10: Task 8.2
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SearchResultList } from "./SearchResultList";
import type { SearchResult } from "@/stores/useSearchStore";

// Mock virtualizer
vi.mock("@tanstack/react-virtual", () => ({
    useVirtualizer: vi.fn().mockImplementation(({ count, getScrollElement: _getScrollElement }) => ({
        getVirtualItems: () =>
            Array.from({ length: count }, (_, i) => ({
                index: i,
                key: `item-${i}`,
                start: i * 72,
                size: 72,
            })),
        getTotalSize: () => count * 72,
        scrollToIndex: vi.fn(),
    })),
}));

const mockResults: SearchResult[] = [
    {
        id: "1",
        projectId: "p1",
        projectName: "Project Alpha",
        sessionId: "s1",
        sessionName: "Session One",
        messageId: "m1",
        snippet: "This is a test snippet for search",
        highlightRanges: [[10, 14]],
        timestamp: Date.now() - 1000 * 60 * 30, // 30 minutes ago
    },
    {
        id: "2",
        projectId: "p2",
        projectName: "Project Beta",
        sessionId: "s2",
        sessionName: "Session Two",
        messageId: "m2",
        snippet: "Another search result here",
        highlightRanges: [[8, 14]],
        timestamp: Date.now() - 1000 * 60 * 60 * 24, // 1 day ago
    },
];

describe("SearchResultList", () => {
    it("should not render anything when results are empty", () => {
        const { container } = render(
            <SearchResultList
                results={[]}
                selectedIndex={0}
                onSelect={vi.fn()}
            />
        );
        expect(container.firstChild).toBeNull();
    });

    it("should render results list", () => {
        render(
            <SearchResultList
                results={mockResults}
                selectedIndex={0}
                onSelect={vi.fn()}
            />
        );

        expect(screen.getByText("Project Alpha")).toBeInTheDocument();
        expect(screen.getByText("Session One")).toBeInTheDocument();
        expect(screen.getByText("Project Beta")).toBeInTheDocument();
        expect(screen.getByText("Session Two")).toBeInTheDocument();
    });

    it("should call onSelect when clicking a result", async () => {
        const onSelect = vi.fn();
        render(
            <SearchResultList
                results={mockResults}
                selectedIndex={0}
                onSelect={onSelect}
            />
        );

        const firstResult = screen.getByText("Project Alpha").closest('[role="option"]');
        if (firstResult) {
            await userEvent.click(firstResult);
        }

        expect(onSelect).toHaveBeenCalledWith(mockResults[0]);
    });

    it("should call onHover when hovering over a result", async () => {
        const onHover = vi.fn();
        render(
            <SearchResultList
                results={mockResults}
                selectedIndex={0}
                onSelect={vi.fn()}
                onHover={onHover}
            />
        );

        const secondResult = screen.getByText("Project Beta").closest('[role="option"]');
        if (secondResult) {
            await userEvent.hover(secondResult);
        }

        expect(onHover).toHaveBeenCalledWith(1);
    });

    it("should mark selected item with aria-selected", () => {
        render(
            <SearchResultList
                results={mockResults}
                selectedIndex={1}
                onSelect={vi.fn()}
            />
        );

        const options = screen.getAllByRole("option");
        expect(options[0]).toHaveAttribute("aria-selected", "false");
        expect(options[1]).toHaveAttribute("aria-selected", "true");
    });
});
