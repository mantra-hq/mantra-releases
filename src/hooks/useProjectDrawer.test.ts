/**
 * useProjectDrawer Hook Tests
 * Story 2.18: Task 6.5
 * Story 1.12: Phase 5 - 更新为逻辑项目视图接口
 */

import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useProjectDrawer } from "./useProjectDrawer";

// Mock useLogicalProjectStats hook (Story 1.12)
vi.mock("./useProjects", () => ({
  useLogicalProjectStats: () => ({
    stats: [
      {
        physical_path: "/home/user/test",
        project_count: 1,
        project_ids: ["proj-1"],
        total_sessions: 3,
        last_activity: "2024-01-02T00:00:00Z",
        display_name: "test-project",
        path_type: "local",
        path_exists: true,
        needs_association: false,
        has_git_repo: false,
      },
    ],
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  }),
  getSessionsByPhysicalPath: vi.fn().mockResolvedValue([
    {
      id: "sess-1",
      source: "claude",
      created_at: "2024-01-01T00:00:00Z",
      updated_at: "2024-01-02T00:00:00Z",
      message_count: 10,
      is_empty: false,
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

  it("provides logicalProjects from useLogicalProjectStats", () => {
    const { result } = renderHook(() => useProjectDrawer());
    expect(result.current.logicalProjects).toHaveLength(1);
    expect(result.current.logicalProjects[0].display_name).toBe("test-project");
  });

  it("provides loading state from useLogicalProjectStats", () => {
    const { result } = renderHook(() => useProjectDrawer());
    expect(result.current.isLoading).toBe(false);
  });

  it("provides getLogicalProjectSessions function", () => {
    const { result } = renderHook(() => useProjectDrawer());
    expect(typeof result.current.getLogicalProjectSessions).toBe("function");
  });
});
