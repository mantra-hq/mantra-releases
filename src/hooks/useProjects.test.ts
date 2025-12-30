/**
 * useProjects Tests - 项目列表 Hook 测试
 * Story 2.8: Task 4
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useProjects } from "./useProjects";
import type { Project } from "@/types/project";

// Mock Tauri IPC
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock project data
const mockProjects: Project[] = [
  {
    id: "project-1",
    name: "project-alpha",
    path: "/home/user/projects/project-alpha",
    sessions: [
      {
        id: "session-1",
        title: "初始化项目",
        source: "claude",
        messageCount: 30,
        startTime: Date.now() - 86400000, // 1 day ago
        endTime: Date.now() - 82800000,
      },
    ],
    lastActivity: Date.now() - 3600000, // 1 hour ago
  },
  {
    id: "project-2",
    name: "project-beta",
    path: "/home/user/projects/project-beta",
    sessions: [],
    lastActivity: Date.now() - 7200000, // 2 hours ago
  },
  {
    id: "project-3",
    name: "project-gamma",
    path: "/home/user/projects/project-gamma",
    sessions: [
      {
        id: "session-2",
        title: "修复 Bug",
        source: "gemini",
        messageCount: 15,
        startTime: Date.now() - 1800000,
        endTime: Date.now() - 900000,
      },
    ],
    lastActivity: Date.now() - 1800000, // 30 minutes ago (most recent)
  },
];

describe("useProjects", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe("初始状态", () => {
    it("初始应该处于加载状态", () => {
      mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves
      const { result } = renderHook(() => useProjects());

      expect(result.current.isLoading).toBe(true);
      expect(result.current.projects).toEqual([]);
      expect(result.current.error).toBeNull();
    });
  });

  describe("成功加载", () => {
    it("应该正确调用 Tauri IPC", async () => {
      mockInvoke.mockResolvedValueOnce(mockProjects);
      renderHook(() => useProjects());

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("get_projects");
      });
    });

    it("加载成功后应该返回项目列表", async () => {
      mockInvoke.mockResolvedValueOnce(mockProjects);
      const { result } = renderHook(() => useProjects());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.projects).toHaveLength(3);
      expect(result.current.error).toBeNull();
    });

    it("项目应该按最后活动时间降序排列", async () => {
      mockInvoke.mockResolvedValueOnce(mockProjects);
      const { result } = renderHook(() => useProjects());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      // project-gamma (30 min ago) > project-alpha (1 hour ago) > project-beta (2 hours ago)
      expect(result.current.projects[0].name).toBe("project-gamma");
      expect(result.current.projects[1].name).toBe("project-alpha");
      expect(result.current.projects[2].name).toBe("project-beta");
    });
  });

  describe("错误处理", () => {
    it("IPC 失败时应该设置错误", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("IPC 连接失败"));
      const { result } = renderHook(() => useProjects());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.error).toBe("IPC 连接失败");
      expect(result.current.projects).toEqual([]);
    });

    it("未知错误应该返回默认错误消息", async () => {
      mockInvoke.mockRejectedValueOnce("unknown error");
      const { result } = renderHook(() => useProjects());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.error).toBe("获取项目列表失败");
    });
  });

  describe("refetch", () => {
    it("应该支持重新获取项目列表", async () => {
      mockInvoke.mockResolvedValueOnce(mockProjects);
      const { result } = renderHook(() => useProjects());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // 更新 mock 返回新数据
      const updatedProjects = [...mockProjects];
      updatedProjects.push({
        id: "project-4",
        name: "project-delta",
        path: "/home/user/projects/project-delta",
        sessions: [],
        lastActivity: Date.now(),
      });
      mockInvoke.mockResolvedValueOnce(updatedProjects);

      // 调用 refetch
      await act(async () => {
        await result.current.refetch();
      });

      expect(mockInvoke).toHaveBeenCalledTimes(2);
      expect(result.current.projects).toHaveLength(4);
    });

    it("refetch 时应该重新设置加载状态", async () => {
      mockInvoke.mockResolvedValueOnce(mockProjects);
      const { result } = renderHook(() => useProjects());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      mockInvoke.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve(mockProjects), 100))
      );

      act(() => {
        result.current.refetch();
      });

      expect(result.current.isLoading).toBe(true);

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });

    it("refetch 失败应该清除之前的错误并设置新错误", async () => {
      // 第一次成功
      mockInvoke.mockResolvedValueOnce(mockProjects);
      const { result } = renderHook(() => useProjects());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.error).toBeNull();

      // 第二次失败
      mockInvoke.mockRejectedValueOnce(new Error("网络错误"));

      await act(async () => {
        await result.current.refetch();
      });

      expect(result.current.error).toBe("网络错误");
    });
  });

  describe("空项目列表", () => {
    it("应该正确处理空列表", async () => {
      mockInvoke.mockResolvedValueOnce([]);
      const { result } = renderHook(() => useProjects());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.projects).toEqual([]);
      expect(result.current.error).toBeNull();
    });
  });
});

