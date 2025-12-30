/**
 * import-ipc 测试文件
 * Story 2.9: Task 6
 *
 * 测试 Tauri IPC 集成函数
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { scanLogDirectory, parseLogFiles, selectLogFiles } from "./import-ipc";
import type { DiscoveredFile } from "@/components/import";

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock @tauri-apps/plugin-dialog
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

// Mock @tauri-apps/api/event
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

describe("import-ipc", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // Task 6.2: scanLogDirectory
  describe("scanLogDirectory", () => {
    it("calls invoke with correct command and source", async () => {
      const mockFiles: DiscoveredFile[] = [
        {
          path: "/path/to/file.json",
          name: "file.json",
          size: 1024,
          modifiedAt: Date.now(),
          projectPath: "/project",
        },
      ];
      vi.mocked(invoke).mockResolvedValue(mockFiles);

      const result = await scanLogDirectory("claude");

      expect(invoke).toHaveBeenCalledWith("scan_log_directory", { source: "claude" });
      expect(result).toEqual(mockFiles);
    });

    it("handles different sources", async () => {
      vi.mocked(invoke).mockResolvedValue([]);

      await scanLogDirectory("gemini");
      expect(invoke).toHaveBeenCalledWith("scan_log_directory", { source: "gemini" });

      await scanLogDirectory("cursor");
      expect(invoke).toHaveBeenCalledWith("scan_log_directory", { source: "cursor" });
    });

    it("propagates errors from invoke", async () => {
      const error = new Error("Failed to scan");
      vi.mocked(invoke).mockRejectedValue(error);

      await expect(scanLogDirectory("claude")).rejects.toThrow("Failed to scan");
    });
  });

  // Task 6.3: parseLogFiles
  describe("parseLogFiles", () => {
    it("calls invoke with correct command and paths", async () => {
      const paths = ["/path/file1.json", "/path/file2.json"];
      const mockResults = [
        { success: true, filePath: paths[0], projectId: "proj-1", sessionId: "sess-1" },
        { success: true, filePath: paths[1], projectId: "proj-1", sessionId: "sess-2" },
      ];
      vi.mocked(invoke).mockResolvedValue(mockResults);

      const onProgress = vi.fn();
      const result = await parseLogFiles(paths, onProgress);

      expect(invoke).toHaveBeenCalledWith("parse_log_files", { paths });
      expect(result).toEqual(mockResults);
    });

    it("propagates errors from invoke", async () => {
      vi.mocked(invoke).mockRejectedValue(new Error("Parse failed"));

      await expect(parseLogFiles(["/path/file.json"], vi.fn())).rejects.toThrow("Parse failed");
    });
  });

  // Task 6.4: selectLogFiles
  describe("selectLogFiles", () => {
    it("calls open with correct options", async () => {
      const mockPaths = ["/path/file1.json", "/path/file2.json"];
      vi.mocked(open).mockResolvedValue(mockPaths);

      const result = await selectLogFiles();

      expect(open).toHaveBeenCalledWith({
        multiple: true,
        filters: [
          {
            name: "JSON Logs",
            extensions: ["json"],
          },
        ],
      });
      expect(result).toEqual(mockPaths);
    });

    it("returns empty array when user cancels", async () => {
      vi.mocked(open).mockResolvedValue(null);

      const result = await selectLogFiles();

      expect(result).toEqual([]);
    });

    it("handles single file selection", async () => {
      vi.mocked(open).mockResolvedValue("/path/single.json" as unknown as string[]);

      const result = await selectLogFiles();

      expect(result).toEqual(["/path/single.json"]);
    });
  });
});
