/**
 * useSearchStore Tests - 搜索状态管理测试
 * Story 2.10: Task 8.3
 */

import { describe, it, expect, beforeEach } from "vitest";
import { act } from "@testing-library/react";
import { useSearchStore, type SearchResult, type RecentSession } from "./useSearchStore";

// 测试数据
const mockSearchResult: SearchResult = {
    id: "session-1-msg-1",
    projectId: "project-1",
    projectName: "Test Project",
    sessionId: "session-1",
    sessionName: "Test Session",
    messageId: "msg-1",
    snippet: "This is a test snippet",
    highlightRanges: [[10, 14]],
    timestamp: Date.now(),
};

const mockSearchResult2: SearchResult = {
    id: "session-2-msg-2",
    projectId: "project-2",
    projectName: "Another Project",
    sessionId: "session-2",
    sessionName: "Another Session",
    messageId: "msg-2",
    snippet: "Another test snippet",
    highlightRanges: [[8, 12]],
    timestamp: Date.now(),
};

const mockRecentSession: RecentSession = {
    projectId: "project-1",
    projectName: "Test Project",
    sessionId: "session-1",
    sessionName: "Test Session",
    accessedAt: Date.now(),
};

describe("useSearchStore", () => {
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
    });

    describe("open/close", () => {
        it("should open search modal and reset state", () => {
            act(() => {
                useSearchStore.getState().setQuery("old query");
                useSearchStore.getState().open();
            });

            const state = useSearchStore.getState();
            expect(state.isOpen).toBe(true);
            expect(state.query).toBe("");
            expect(state.results).toEqual([]);
            expect(state.selectedIndex).toBe(0);
        });

        it("should close search modal", () => {
            act(() => {
                useSearchStore.getState().open();
                useSearchStore.getState().close();
            });

            expect(useSearchStore.getState().isOpen).toBe(false);
        });
    });

    describe("setQuery", () => {
        it("should update query and reset selectedIndex", () => {
            act(() => {
                useSearchStore.getState().setResults([mockSearchResult, mockSearchResult2]);
                useSearchStore.setState({ selectedIndex: 1 });
                useSearchStore.getState().setQuery("new query");
            });

            const state = useSearchStore.getState();
            expect(state.query).toBe("new query");
            expect(state.selectedIndex).toBe(0);
        });
    });

    describe("setResults", () => {
        it("should update results and reset loading/selectedIndex", () => {
            act(() => {
                useSearchStore.getState().setLoading(true);
                useSearchStore.setState({ selectedIndex: 1 });
                useSearchStore.getState().setResults([mockSearchResult]);
            });

            const state = useSearchStore.getState();
            expect(state.results).toEqual([mockSearchResult]);
            expect(state.isLoading).toBe(false);
            expect(state.selectedIndex).toBe(0);
        });
    });

    describe("setLoading", () => {
        it("should update loading state", () => {
            act(() => {
                useSearchStore.getState().setLoading(true);
            });
            expect(useSearchStore.getState().isLoading).toBe(true);

            act(() => {
                useSearchStore.getState().setLoading(false);
            });
            expect(useSearchStore.getState().isLoading).toBe(false);
        });
    });

    describe("selectNext/selectPrev", () => {
        beforeEach(() => {
            act(() => {
                useSearchStore.getState().setResults([mockSearchResult, mockSearchResult2]);
            });
        });

        it("should select next result", () => {
            act(() => {
                useSearchStore.getState().selectNext();
            });
            expect(useSearchStore.getState().selectedIndex).toBe(1);
        });

        it("should not exceed results length", () => {
            act(() => {
                useSearchStore.getState().selectNext();
                useSearchStore.getState().selectNext();
                useSearchStore.getState().selectNext();
            });
            expect(useSearchStore.getState().selectedIndex).toBe(1);
        });

        it("should select previous result", () => {
            act(() => {
                useSearchStore.setState({ selectedIndex: 1 });
                useSearchStore.getState().selectPrev();
            });
            expect(useSearchStore.getState().selectedIndex).toBe(0);
        });

        it("should not go below 0", () => {
            act(() => {
                useSearchStore.getState().selectPrev();
                useSearchStore.getState().selectPrev();
            });
            expect(useSearchStore.getState().selectedIndex).toBe(0);
        });
    });

    describe("confirm", () => {
        it("should return selected result", () => {
            act(() => {
                useSearchStore.getState().setResults([mockSearchResult, mockSearchResult2]);
                useSearchStore.setState({ selectedIndex: 1 });
            });

            const result = useSearchStore.getState().confirm();
            expect(result).toEqual(mockSearchResult2);
        });

        it("should return null when no results", () => {
            const result = useSearchStore.getState().confirm();
            expect(result).toBeNull();
        });
    });

    describe("addRecentSession", () => {
        it("should add recent session to beginning", () => {
            act(() => {
                useSearchStore.getState().addRecentSession(mockRecentSession);
            });

            const state = useSearchStore.getState();
            expect(state.recentSessions).toHaveLength(1);
            expect(state.recentSessions[0]).toEqual(mockRecentSession);
        });

        it("should not duplicate sessions", () => {
            act(() => {
                useSearchStore.getState().addRecentSession(mockRecentSession);
                useSearchStore.getState().addRecentSession({
                    ...mockRecentSession,
                    accessedAt: Date.now() + 1000,
                });
            });

            expect(useSearchStore.getState().recentSessions).toHaveLength(1);
        });

        it("should limit to 10 recent sessions", () => {
            act(() => {
                for (let i = 0; i < 15; i++) {
                    useSearchStore.getState().addRecentSession({
                        ...mockRecentSession,
                        sessionId: `session-${i}`,
                        accessedAt: Date.now() + i,
                    });
                }
            });

            expect(useSearchStore.getState().recentSessions).toHaveLength(10);
        });
    });

    describe("reset", () => {
        it("should reset search state but keep recentSessions", () => {
            act(() => {
                useSearchStore.getState().addRecentSession(mockRecentSession);
                useSearchStore.getState().setQuery("test");
                useSearchStore.getState().setResults([mockSearchResult]);
                useSearchStore.getState().setLoading(true);
                useSearchStore.setState({ selectedIndex: 1 });
                useSearchStore.getState().reset();
            });

            const state = useSearchStore.getState();
            expect(state.query).toBe("");
            expect(state.results).toEqual([]);
            expect(state.isLoading).toBe(false);
            expect(state.selectedIndex).toBe(0);
            expect(state.recentSessions).toHaveLength(1); // Preserved
        });
    });
});
