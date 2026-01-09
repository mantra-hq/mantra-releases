/**
 * Tool Utils Tests - 工具类型判断工具函数测试
 * Story 8.12: Task 1.4
 */

import { describe, it, expect } from "vitest";
import {
  StandardToolTypes,
  isFileTool,
  isFileReadTool,
  isFileWriteTool,
  isFileEditTool,
  isTerminalTool,
  isSearchTool,
  isFileSearchTool,
  isContentSearchTool,
  isOtherTool,
  getToolPath,
  getToolCommand,
  getToolContent,
  getToolPattern,
  getOtherToolName,
} from "./tool-utils";
import type { StandardTool } from "@/types/message";

describe("tool-utils", () => {
  // Test fixtures
  const fileReadTool: StandardTool = {
    type: "file_read",
    path: "/src/index.ts",
    startLine: 1,
    endLine: 10,
  };

  const fileWriteTool: StandardTool = {
    type: "file_write",
    path: "/src/new-file.ts",
    content: 'console.log("hello");',
  };

  const fileEditTool: StandardTool = {
    type: "file_edit",
    path: "/src/edit.ts",
    oldString: "old",
    newString: "new",
  };

  const shellExecTool: StandardTool = {
    type: "shell_exec",
    command: "npm install",
    cwd: "/project",
  };

  const fileSearchTool: StandardTool = {
    type: "file_search",
    pattern: "*.ts",
    path: "/src",
  };

  const contentSearchTool: StandardTool = {
    type: "content_search",
    pattern: "TODO",
    path: "/src",
  };

  const otherTool: StandardTool = {
    type: "other",
    name: "custom_tool",
    input: { key: "value" },
  };

  describe("StandardToolTypes", () => {
    it("should have all expected types", () => {
      expect(StandardToolTypes.FILE_READ).toBe("file_read");
      expect(StandardToolTypes.FILE_WRITE).toBe("file_write");
      expect(StandardToolTypes.FILE_EDIT).toBe("file_edit");
      expect(StandardToolTypes.SHELL_EXEC).toBe("shell_exec");
      expect(StandardToolTypes.FILE_SEARCH).toBe("file_search");
      expect(StandardToolTypes.CONTENT_SEARCH).toBe("content_search");
      expect(StandardToolTypes.OTHER).toBe("other");
    });
  });

  describe("isFileReadTool", () => {
    it("should return true for file_read tools", () => {
      expect(isFileReadTool(fileReadTool)).toBe(true);
    });

    it("should return false for other tool types", () => {
      expect(isFileReadTool(fileWriteTool)).toBe(false);
      expect(isFileReadTool(shellExecTool)).toBe(false);
      expect(isFileReadTool(otherTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isFileReadTool(undefined)).toBe(false);
    });
  });

  describe("isFileWriteTool", () => {
    it("should return true for file_write tools", () => {
      expect(isFileWriteTool(fileWriteTool)).toBe(true);
    });

    it("should return false for other tool types", () => {
      expect(isFileWriteTool(fileReadTool)).toBe(false);
      expect(isFileWriteTool(fileEditTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isFileWriteTool(undefined)).toBe(false);
    });
  });

  describe("isFileEditTool", () => {
    it("should return true for file_edit tools", () => {
      expect(isFileEditTool(fileEditTool)).toBe(true);
    });

    it("should return false for other tool types", () => {
      expect(isFileEditTool(fileReadTool)).toBe(false);
      expect(isFileEditTool(fileWriteTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isFileEditTool(undefined)).toBe(false);
    });
  });

  describe("isFileTool", () => {
    it("should return true for all file-related tools", () => {
      expect(isFileTool(fileReadTool)).toBe(true);
      expect(isFileTool(fileWriteTool)).toBe(true);
      expect(isFileTool(fileEditTool)).toBe(true);
    });

    it("should return false for non-file tools", () => {
      expect(isFileTool(shellExecTool)).toBe(false);
      expect(isFileTool(fileSearchTool)).toBe(false);
      expect(isFileTool(contentSearchTool)).toBe(false);
      expect(isFileTool(otherTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isFileTool(undefined)).toBe(false);
    });
  });

  describe("isTerminalTool", () => {
    it("should return true for shell_exec tools", () => {
      expect(isTerminalTool(shellExecTool)).toBe(true);
    });

    it("should return false for non-terminal tools", () => {
      expect(isTerminalTool(fileReadTool)).toBe(false);
      expect(isTerminalTool(fileWriteTool)).toBe(false);
      expect(isTerminalTool(otherTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isTerminalTool(undefined)).toBe(false);
    });
  });

  describe("isFileSearchTool", () => {
    it("should return true for file_search tools", () => {
      expect(isFileSearchTool(fileSearchTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isFileSearchTool(contentSearchTool)).toBe(false);
      expect(isFileSearchTool(fileReadTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isFileSearchTool(undefined)).toBe(false);
    });
  });

  describe("isContentSearchTool", () => {
    it("should return true for content_search tools", () => {
      expect(isContentSearchTool(contentSearchTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isContentSearchTool(fileSearchTool)).toBe(false);
      expect(isContentSearchTool(fileReadTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isContentSearchTool(undefined)).toBe(false);
    });
  });

  describe("isSearchTool", () => {
    it("should return true for all search tools", () => {
      expect(isSearchTool(fileSearchTool)).toBe(true);
      expect(isSearchTool(contentSearchTool)).toBe(true);
    });

    it("should return false for non-search tools", () => {
      expect(isSearchTool(fileReadTool)).toBe(false);
      expect(isSearchTool(shellExecTool)).toBe(false);
      expect(isSearchTool(otherTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isSearchTool(undefined)).toBe(false);
    });
  });

  describe("isOtherTool", () => {
    it("should return true for other type tools", () => {
      expect(isOtherTool(otherTool)).toBe(true);
    });

    it("should return false for known tool types", () => {
      expect(isOtherTool(fileReadTool)).toBe(false);
      expect(isOtherTool(shellExecTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isOtherTool(undefined)).toBe(false);
    });
  });

  describe("getToolPath", () => {
    it("should return path for file tools", () => {
      expect(getToolPath(fileReadTool)).toBe("/src/index.ts");
      expect(getToolPath(fileWriteTool)).toBe("/src/new-file.ts");
      expect(getToolPath(fileEditTool)).toBe("/src/edit.ts");
    });

    it("should return path for search tools", () => {
      expect(getToolPath(fileSearchTool)).toBe("/src");
      expect(getToolPath(contentSearchTool)).toBe("/src");
    });

    it("should return undefined for tools without path", () => {
      expect(getToolPath(shellExecTool)).toBeUndefined();
      expect(getToolPath(otherTool)).toBeUndefined();
    });

    it("should return undefined for undefined input", () => {
      expect(getToolPath(undefined)).toBeUndefined();
    });
  });

  describe("getToolCommand", () => {
    it("should return command for shell_exec tools", () => {
      expect(getToolCommand(shellExecTool)).toBe("npm install");
    });

    it("should return undefined for non-shell tools", () => {
      expect(getToolCommand(fileReadTool)).toBeUndefined();
      expect(getToolCommand(otherTool)).toBeUndefined();
    });

    it("should return undefined for undefined input", () => {
      expect(getToolCommand(undefined)).toBeUndefined();
    });
  });

  describe("getToolContent", () => {
    it("should return content for file_write tools", () => {
      expect(getToolContent(fileWriteTool)).toBe('console.log("hello");');
    });

    it("should return undefined for non-file_write tools", () => {
      expect(getToolContent(fileReadTool)).toBeUndefined();
      expect(getToolContent(fileEditTool)).toBeUndefined();
      expect(getToolContent(shellExecTool)).toBeUndefined();
    });

    it("should return undefined for undefined input", () => {
      expect(getToolContent(undefined)).toBeUndefined();
    });
  });

  describe("getToolPattern", () => {
    it("should return pattern for search tools", () => {
      expect(getToolPattern(fileSearchTool)).toBe("*.ts");
      expect(getToolPattern(contentSearchTool)).toBe("TODO");
    });

    it("should return undefined for non-search tools", () => {
      expect(getToolPattern(fileReadTool)).toBeUndefined();
      expect(getToolPattern(shellExecTool)).toBeUndefined();
    });

    it("should return undefined for undefined input", () => {
      expect(getToolPattern(undefined)).toBeUndefined();
    });
  });

  describe("getOtherToolName", () => {
    it("should return name for other type tools", () => {
      expect(getOtherToolName(otherTool)).toBe("custom_tool");
    });

    it("should return undefined for known tool types", () => {
      expect(getOtherToolName(fileReadTool)).toBeUndefined();
      expect(getOtherToolName(shellExecTool)).toBeUndefined();
    });

    it("should return undefined for undefined input", () => {
      expect(getOtherToolName(undefined)).toBeUndefined();
    });
  });
});
