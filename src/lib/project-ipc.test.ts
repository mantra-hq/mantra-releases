/**
 * project-ipc.test - 项目 IPC 测试
 * Story 2.11: Task 5
 * Story 9.2: 更新 mock 为 IPC 适配器
 */

import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock IPC 适配器
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@/lib/ipc-adapter";
import {
  getProject,
  getProjectByCwd,
  getRepresentativeFile,
  getFileAtHead,
  detectGitRepo,
  getSnapshotAtTime,
  listProjects,
} from "./project-ipc";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("project-ipc", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("getProject", () => {
    it("calls invoke with correct command and params", async () => {
      const mockProject = {
        id: "test-id",
        name: "test",
        cwd: "/test/path",
        session_count: 5,
        created_at: "2024-01-01T00:00:00Z",
        last_activity: "2024-01-02T00:00:00Z",
        git_repo_path: "/test/path",
        has_git_repo: true,
      };
      mockInvoke.mockResolvedValue(mockProject);

      const result = await getProject("test-id");

      expect(mockInvoke).toHaveBeenCalledWith("get_project", { projectId: "test-id" });
      expect(result).toEqual(mockProject);
    });

    it("returns null when project not found", async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await getProject("nonexistent");

      expect(result).toBeNull();
    });
  });

  describe("getProjectByCwd", () => {
    it("calls invoke with correct command and params", async () => {
      const mockProject = {
        id: "test-id",
        name: "test",
        cwd: "/test/path",
        session_count: 5,
        created_at: "2024-01-01T00:00:00Z",
        last_activity: "2024-01-02T00:00:00Z",
        git_repo_path: "/test/path",
        has_git_repo: true,
      };
      mockInvoke.mockResolvedValue(mockProject);

      const result = await getProjectByCwd("/test/path");

      expect(mockInvoke).toHaveBeenCalledWith("get_project_by_cwd", { cwd: "/test/path" });
      expect(result).toEqual(mockProject);
    });
  });

  describe("getRepresentativeFile", () => {
    it("calls invoke with correct command and params", async () => {
      const mockFile = {
        path: "README.md",
        content: "# Test Project",
        language: "markdown",
      };
      mockInvoke.mockResolvedValue(mockFile);

      const result = await getRepresentativeFile("/test/repo");

      expect(mockInvoke).toHaveBeenCalledWith("get_representative_file", { repoPath: "/test/repo" });
      expect(result).toEqual(mockFile);
    });

    it("returns null when no file found", async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await getRepresentativeFile("/test/repo");

      expect(result).toBeNull();
    });
  });

  describe("getFileAtHead", () => {
    it("calls invoke with correct command and params", async () => {
      const mockSnapshot = {
        content: "file content",
        commit_hash: "abc123",
        commit_message: "Initial commit",
        commit_timestamp: 1704067200,
      };
      mockInvoke.mockResolvedValue(mockSnapshot);

      const result = await getFileAtHead("/test/repo", "src/main.rs");

      expect(mockInvoke).toHaveBeenCalledWith("get_file_at_head", {
        repoPath: "/test/repo",
        filePath: "src/main.rs",
      });
      expect(result).toEqual(mockSnapshot);
    });
  });

  describe("detectGitRepo", () => {
    it("returns repo path when found", async () => {
      mockInvoke.mockResolvedValue("/test/repo");

      const result = await detectGitRepo("/test/repo/subdir");

      expect(mockInvoke).toHaveBeenCalledWith("detect_git_repo", { dirPath: "/test/repo/subdir" });
      expect(result).toBe("/test/repo");
    });

    it("returns null when not a git repo", async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await detectGitRepo("/tmp");

      expect(result).toBeNull();
    });
  });

  describe("getSnapshotAtTime", () => {
    it("calls invoke with correct command and params", async () => {
      const mockSnapshot = {
        content: "old content",
        commit_hash: "def456",
        commit_message: "Previous commit",
        commit_timestamp: 1704000000,
      };
      mockInvoke.mockResolvedValue(mockSnapshot);

      const result = await getSnapshotAtTime("/test/repo", "src/main.rs", 1704000000);

      expect(mockInvoke).toHaveBeenCalledWith("get_snapshot_at_time", {
        repoPath: "/test/repo",
        filePath: "src/main.rs",
        timestamp: 1704000000,
      });
      expect(result).toEqual(mockSnapshot);
    });
  });

  describe("listProjects", () => {
    it("returns projects list", async () => {
      const mockProjects = [
        {
          id: "proj1",
          name: "Project 1",
          cwd: "/path/1",
          session_count: 3,
          created_at: "2024-01-01T00:00:00Z",
          last_activity: "2024-01-02T00:00:00Z",
          git_repo_path: "/path/1",
          has_git_repo: true,
        },
        {
          id: "proj2",
          name: "Project 2",
          cwd: "/path/2",
          session_count: 1,
          created_at: "2024-01-01T00:00:00Z",
          last_activity: "2024-01-01T00:00:00Z",
          git_repo_path: null,
          has_git_repo: false,
        },
      ];
      mockInvoke.mockResolvedValue(mockProjects);

      const result = await listProjects();

      expect(mockInvoke).toHaveBeenCalledWith("list_projects");
      expect(result).toEqual(mockProjects);
    });
  });
});
