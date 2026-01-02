/**
 * useImportStore 测试文件
 * Story 2.9: Task 7
 *
 * 测试导入状态管理 Store
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useImportStore } from "./useImportStore";
import { act } from "@testing-library/react";
import type { DiscoveredFile, ImportResult, ImportProgressData } from "@/components/import";

/** 测试用文件数据 */
const mockFiles: DiscoveredFile[] = [
  {
    path: "/path/file1.json",
    name: "file1.json",
    size: 1024,
    modifiedAt: Date.now(),
    projectPath: "/project1",
  },
  {
    path: "/path/file2.json",
    name: "file2.json",
    size: 2048,
    modifiedAt: Date.now(),
    projectPath: "/project2",
  },
];

describe("useImportStore", () => {
  beforeEach(() => {
    // 重置 store
    act(() => {
      useImportStore.getState().reset();
    });
  });

  // Task 7.2: 状态
  describe("Initial State", () => {
    it("has correct initial values", () => {
      const state = useImportStore.getState();

      expect(state.isOpen).toBe(false);
      expect(state.step).toBe("source");
      expect(state.source).toBeNull();
      expect(state.discoveredFiles).toEqual([]);
      expect(state.selectedFiles).toBeInstanceOf(Set);
      expect(state.selectedFiles.size).toBe(0);
      expect(state.progress).toBeNull();
      expect(state.results).toEqual([]);
    });
  });

  // Task 7.3: Actions - open/close
  describe("open/close", () => {
    it("opens the modal and resets to source step", () => {
      act(() => {
        useImportStore.getState().open();
      });

      const state = useImportStore.getState();
      expect(state.isOpen).toBe(true);
      expect(state.step).toBe("source");
    });

    it("closes the modal", () => {
      act(() => {
        useImportStore.getState().open();
        useImportStore.getState().close();
      });

      expect(useImportStore.getState().isOpen).toBe(false);
    });
  });

  // Task 7.3: Actions - setStep
  describe("setStep", () => {
    it("updates the current step", () => {
      act(() => {
        useImportStore.getState().setStep("files");
      });

      expect(useImportStore.getState().step).toBe("files");
    });

    it("can set to any valid step", () => {
      const steps = ["source", "files", "progress", "complete"] as const;

      for (const step of steps) {
        act(() => {
          useImportStore.getState().setStep(step);
        });
        expect(useImportStore.getState().step).toBe(step);
      }
    });
  });

  // Task 7.3: Actions - setSource
  describe("setSource", () => {
    it("updates the source", () => {
      act(() => {
        useImportStore.getState().setSource("claude");
      });

      expect(useImportStore.getState().source).toBe("claude");
    });

    it("can set different sources", () => {
      const sources = ["claude", "gemini", "cursor"] as const;

      for (const source of sources) {
        act(() => {
          useImportStore.getState().setSource(source);
        });
        expect(useImportStore.getState().source).toBe(source);
      }
    });
  });

  // Task 7.3: Actions - setDiscoveredFiles
  describe("setDiscoveredFiles", () => {
    it("sets discovered files and selects all by default", () => {
      act(() => {
        useImportStore.getState().setDiscoveredFiles(mockFiles);
      });

      const state = useImportStore.getState();
      expect(state.discoveredFiles).toEqual(mockFiles);
      expect(state.selectedFiles.size).toBe(mockFiles.length);
      expect(state.selectedFiles.has(mockFiles[0].path)).toBe(true);
      expect(state.selectedFiles.has(mockFiles[1].path)).toBe(true);
    });
  });

  // Task 7.3: Actions - toggleFile
  describe("toggleFile", () => {
    beforeEach(() => {
      act(() => {
        useImportStore.getState().setDiscoveredFiles(mockFiles);
      });
    });

    it("deselects a selected file", () => {
      act(() => {
        useImportStore.getState().toggleFile(mockFiles[0].path);
      });

      const state = useImportStore.getState();
      expect(state.selectedFiles.has(mockFiles[0].path)).toBe(false);
      expect(state.selectedFiles.has(mockFiles[1].path)).toBe(true);
    });

    it("selects a deselected file", () => {
      act(() => {
        useImportStore.getState().toggleFile(mockFiles[0].path); // deselect
        useImportStore.getState().toggleFile(mockFiles[0].path); // select
      });

      expect(useImportStore.getState().selectedFiles.has(mockFiles[0].path)).toBe(true);
    });
  });

  // Task 7.3: Actions - selectAll, clearAll, invertSelection
  describe("selectAll", () => {
    beforeEach(() => {
      act(() => {
        useImportStore.getState().setDiscoveredFiles(mockFiles);
      });
    });

    it("selects all files", () => {
      act(() => {
        useImportStore.getState().clearAll(); // clear first
        useImportStore.getState().selectAll();
      });

      expect(useImportStore.getState().selectedFiles.size).toBe(mockFiles.length);
    });
  });

  describe("clearAll", () => {
    beforeEach(() => {
      act(() => {
        useImportStore.getState().setDiscoveredFiles(mockFiles);
      });
    });

    it("clears all selections", () => {
      act(() => {
        useImportStore.getState().clearAll();
      });

      expect(useImportStore.getState().selectedFiles.size).toBe(0);
    });
  });

  describe("invertSelection", () => {
    beforeEach(() => {
      act(() => {
        useImportStore.getState().setDiscoveredFiles(mockFiles);
      });
    });

    it("inverts selection - selected becomes unselected and vice versa", () => {
      act(() => {
        // Start with all selected, deselect first one
        useImportStore.getState().toggleFile(mockFiles[0].path);
        // Now only file 1 and 2 are selected
        useImportStore.getState().invertSelection();
      });

      // After invert: only file 0 should be selected
      const selected = useImportStore.getState().selectedFiles;
      expect(selected.size).toBe(1);
      expect(selected.has(mockFiles[0].path)).toBe(true);
      expect(selected.has(mockFiles[1].path)).toBe(false);
    });
  });

  // Task 7.3: Actions - setProgress
  describe("setProgress", () => {
    it("updates progress", () => {
      const progress: ImportProgressData = {
        current: 5,
        total: 10,
        currentFile: "test.json",
        successCount: 4,
        failureCount: 1,
      };

      act(() => {
        useImportStore.getState().setProgress(progress);
      });

      expect(useImportStore.getState().progress).toEqual(progress);
    });
  });

  // Task 7.3: Actions - addResult
  describe("addResult", () => {
    it("adds result to results array", () => {
      const result: ImportResult = {
        success: true,
        filePath: "/path/file.json",
        projectId: "proj-1",
        sessionId: "sess-1",
      };

      act(() => {
        useImportStore.getState().addResult(result);
      });

      expect(useImportStore.getState().results).toContainEqual(result);
    });

    it("accumulates multiple results", () => {
      const results: ImportResult[] = [
        { success: true, filePath: "/path/file1.json", projectId: "proj-1", sessionId: "sess-1" },
        { success: false, filePath: "/path/file2.json", error: "parse_error" },
      ];

      act(() => {
        results.forEach((r) => useImportStore.getState().addResult(r));
      });

      expect(useImportStore.getState().results).toHaveLength(2);
    });
  });

  // Task 7.3: Actions - reset
  describe("reset", () => {
    it("resets all state to initial values", () => {
      // Set some state
      act(() => {
        useImportStore.getState().open();
        useImportStore.getState().setSource("claude");
        useImportStore.getState().setStep("files");
        useImportStore.getState().setDiscoveredFiles(mockFiles);
        useImportStore.getState().addResult({ success: true, filePath: "/path/file.json" });
      });

      // Reset
      act(() => {
        useImportStore.getState().reset();
      });

      const state = useImportStore.getState();
      expect(state.step).toBe("source");
      expect(state.source).toBeNull();
      expect(state.discoveredFiles).toEqual([]);
      expect(state.selectedFiles.size).toBe(0);
      expect(state.progress).toBeNull();
      expect(state.results).toEqual([]);
      // Note: isOpen is preserved during reset
    });
  });

  // Loading state
  describe("loading state", () => {
    it("has isLoading state", () => {
      expect(useImportStore.getState().isLoading).toBe(false);
    });

    it("can set loading state", () => {
      act(() => {
        useImportStore.getState().setLoading(true);
      });

      expect(useImportStore.getState().isLoading).toBe(true);
    });
  });

  // Errors state
  describe("errors state", () => {
    it("has errors array", () => {
      expect(useImportStore.getState().errors).toEqual([]);
    });

    it("can add errors", () => {
      const error = { filePath: "/path/file.json", error: "parse_error", message: "Failed" };

      act(() => {
        useImportStore.getState().addError(error);
      });

      expect(useImportStore.getState().errors).toContainEqual(error);
    });
  });
});
