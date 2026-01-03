/**
 * useProjectDrawer Hook Tests
 * Story 2.18: Task 6.5
 */

import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useProjectDrawer } from "./useProjectDrawer";

// Mock useProjects hook
vi.mock("./useProjects", () => ({
  useProjects: () => ({
    projects: [
      {
        id: "proj-1",
        name: "test-project",
        cwd: "/home/user/test",
        session_count: 3,
        created_at: "2024-01-01T00:00:00Z",
        last_activity: "2024-01-02T00:00:00Z",
        git_repo_path: null,
        has_git_repo: false,
      },
    ],
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  }),
}));

// Mock project-ipc
vi.mock("@/lib/project-ipc", () => ({
  getProjectSessions: vi.fn().mockResolvedValue([
    {
      id: "sess-1",
      source: "claude",
      created_at: "2024-01-01T00:00:00Z",
      updated_at: "2024-01-02T00:00:00Z",
      message_count: 10,
    },
  ]),
}));

describe("useProjectDrawer", () => {
  it("initializes with drawer closed", () => {
    const { result } = renderHook(() => useProjectDrawer());
    expect(result.current.isOpen).toBe(false);
  });

  it("opens drawer with openDrawer", () => {
    const { result } = renderHook(() => useProjectDrawer());

    act(() => {
      result.current.openDrawer();
    });

    expect(result.current.isOpen).toBe(true);
  });

  it("closes drawer with closeDrawer", () => {
    const { result } = renderHook(() => useProjectDrawer());

    act(() => {
      result.current.openDrawer();
    });
    expect(result.current.isOpen).toBe(true);

    act(() => {
      result.current.closeDrawer();
    });
    expect(result.current.isOpen).toBe(false);
  });

  it("toggles drawer with toggleDrawer", () => {
    const { result } = renderHook(() => useProjectDrawer());

    act(() => {
      result.current.toggleDrawer();
    });
    expect(result.current.isOpen).toBe(true);

    act(() => {
      result.current.toggleDrawer();
    });
    expect(result.current.isOpen).toBe(false);
  });

  it("sets drawer state with setIsOpen", () => {
    const { result } = renderHook(() => useProjectDrawer());

    act(() => {
      result.current.setIsOpen(true);
    });
    expect(result.current.isOpen).toBe(true);

    act(() => {
      result.current.setIsOpen(false);
    });
    expect(result.current.isOpen).toBe(false);
  });

  it("provides projects from useProjects", () => {
    const { result } = renderHook(() => useProjectDrawer());
    expect(result.current.projects).toHaveLength(1);
    expect(result.current.projects[0].name).toBe("test-project");
  });

  it("provides loading state from useProjects", () => {
    const { result } = renderHook(() => useProjectDrawer());
    expect(result.current.isLoading).toBe(false);
  });

  it("provides getProjectSessions function", () => {
    const { result } = renderHook(() => useProjectDrawer());
    expect(typeof result.current.getProjectSessions).toBe("function");
  });
});
