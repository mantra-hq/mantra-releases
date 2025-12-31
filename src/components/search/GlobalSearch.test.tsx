/**
 * GlobalSearch Tests - 全局搜索组件测试
 * Story 2.10: Task 8.1
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BrowserRouter } from "react-router-dom";
import { GlobalSearch } from "./GlobalSearch";
import { useSearchStore } from "@/stores/useSearchStore";

// Mock navigate
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
    const actual = await vi.importActual("react-router-dom");
    return {
        ...actual,
        useNavigate: () => mockNavigate,
    };
});

// Mock search-ipc
vi.mock("@/lib/search-ipc", () => ({
    createDebouncedSearch: () => ({
        debouncedSearch: vi.fn(),
        cancel: vi.fn(),
    }),
}));

describe("GlobalSearch", () => {
    beforeEach(() => {
        // Reset store before each test
        act(() => {
            useSearchStore.setState({
                isOpen: false,
                query: "",
                results: [],
                isLoading: false,
                selectedIndex: 0,
                recentSessions: [],
            });
        });
        mockNavigate.mockClear();
    });

    afterEach(() => {
        vi.clearAllMocks();
    });

    const renderGlobalSearch = () => {
        return render(
            <BrowserRouter>
                <GlobalSearch />
            </BrowserRouter>
        );
    };

    describe("Visibility", () => {
        it("should not render when isOpen is false", () => {
            renderGlobalSearch();
            expect(screen.queryByLabelText("全局搜索")).not.toBeInTheDocument();
        });

        it("should render when isOpen is true", () => {
            act(() => {
                useSearchStore.getState().open();
            });
            renderGlobalSearch();
            expect(screen.getByLabelText("全局搜索")).toBeInTheDocument();
        });
    });

    describe("Search Input", () => {
        it("should have search input focused when opened", async () => {
            act(() => {
                useSearchStore.getState().open();
            });
            renderGlobalSearch();

            // Wait for focus
            await vi.waitFor(() => {
                const input = screen.getByPlaceholderText("搜索会话内容...");
                expect(document.activeElement).toBe(input);
            });
        });

        it("should update query on input", async () => {
            act(() => {
                useSearchStore.getState().open();
            });
            renderGlobalSearch();

            const input = screen.getByPlaceholderText("搜索会话内容...");
            await userEvent.type(input, "test query");

            expect(useSearchStore.getState().query).toBe("test query");
        });
    });

    describe("Keyboard Navigation", () => {
        it("should close on Escape key", async () => {
            act(() => {
                useSearchStore.getState().open();
            });
            renderGlobalSearch();

            const dialog = screen.getByLabelText("全局搜索");
            fireEvent.keyDown(dialog, { key: "Escape" });

            expect(useSearchStore.getState().isOpen).toBe(false);
        });

        it("should navigate results with arrow keys", async () => {
            act(() => {
                useSearchStore.setState({
                    isOpen: true,
                    query: "test",
                    results: [
                        {
                            id: "1",
                            projectId: "p1",
                            projectName: "Project 1",
                            sessionId: "s1",
                            sessionName: "Session 1",
                            messageId: "m1",
                            snippet: "test snippet 1",
                            highlightRanges: [],
                            timestamp: Date.now(),
                        },
                        {
                            id: "2",
                            projectId: "p2",
                            projectName: "Project 2",
                            sessionId: "s2",
                            sessionName: "Session 2",
                            messageId: "m2",
                            snippet: "test snippet 2",
                            highlightRanges: [],
                            timestamp: Date.now(),
                        },
                    ],
                    selectedIndex: 0,
                });
            });
            renderGlobalSearch();

            const dialog = screen.getByLabelText("全局搜索");
            fireEvent.keyDown(dialog, { key: "ArrowDown" });

            expect(useSearchStore.getState().selectedIndex).toBe(1);

            fireEvent.keyDown(dialog, { key: "ArrowUp" });
            expect(useSearchStore.getState().selectedIndex).toBe(0);
        });
    });

    describe("Close Button", () => {
        it("should close when clicking close button", async () => {
            act(() => {
                useSearchStore.getState().open();
            });
            renderGlobalSearch();

            const closeButton = screen.getByLabelText("关闭搜索");
            await userEvent.click(closeButton);

            expect(useSearchStore.getState().isOpen).toBe(false);
        });
    });

    describe("Recent Sessions", () => {
        it("should display recent sessions when no query", () => {
            act(() => {
                useSearchStore.setState({
                    isOpen: true,
                    query: "",
                    recentSessions: [
                        {
                            projectId: "p1",
                            projectName: "Recent Project",
                            sessionId: "s1",
                            sessionName: "Recent Session",
                            accessedAt: Date.now(),
                        },
                    ],
                });
            });
            renderGlobalSearch();

            expect(screen.getByText("最近访问")).toBeInTheDocument();
            expect(screen.getByText("Recent Project")).toBeInTheDocument();
        });
    });

    describe("Empty State", () => {
        it("should show empty state when no results", () => {
            act(() => {
                useSearchStore.setState({
                    isOpen: true,
                    query: "no match",
                    results: [],
                    isLoading: false,
                });
            });
            renderGlobalSearch();

            expect(screen.getByText(/未找到包含/)).toBeInTheDocument();
        });
    });
});
