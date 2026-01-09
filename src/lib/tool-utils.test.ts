/**
 * Tool Utils Tests - 工具类型判断工具函数测试
 * Story 8.12: Task 1.4
 * Story 8.13: 扩展新工具类型支持测试
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
  isUnknownTool,
  isWebFetchTool,
  isWebSearchTool,
  isWebTool,
  isKnowledgeQueryTool,
  isCodeExecTool,
  isDiagnosticTool,
  isNotebookEditTool,
  isTodoManageTool,
  isSubTaskTool,
  isUserPromptTool,
  isPlanModeTool,
  isSkillInvokeTool,
  isAgentTool,
  isInteractiveTool,
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

  // Story 8.13: New tool type fixtures
  const unknownTool: StandardTool = {
    type: "unknown",
    name: "unknown_tool",
    input: { data: "test" },
  };

  const webFetchTool: StandardTool = {
    type: "web_fetch",
    url: "https://example.com",
    prompt: "summarize",
  };

  const webSearchTool: StandardTool = {
    type: "web_search",
    query: "rust programming",
  };

  const knowledgeQueryTool: StandardTool = {
    type: "knowledge_query",
    repo: "facebook/react",
    question: "How does React work?",
  };

  const codeExecTool: StandardTool = {
    type: "code_exec",
    code: "console.log('hello')",
    language: "javascript",
  };

  const diagnosticTool: StandardTool = {
    type: "diagnostic",
    uri: "file:///path/to/file.ts",
  };

  const notebookEditTool: StandardTool = {
    type: "notebook_edit",
    notebookPath: "/notebook.ipynb",
    cellId: "cell-1",
    newSource: "print('hello')",
  };

  const todoManageTool: StandardTool = {
    type: "todo_manage",
    todos: { items: [] },
  };

  const subTaskTool: StandardTool = {
    type: "sub_task",
    prompt: "explore the codebase",
    agentType: "Explore",
  };

  const userPromptTool: StandardTool = {
    type: "user_prompt",
    question: "Which option?",
    options: { a: "Option A" },
  };

  const planModeTool: StandardTool = {
    type: "plan_mode",
    entering: true,
  };

  const skillInvokeTool: StandardTool = {
    type: "skill_invoke",
    skill: "commit",
    args: "-m 'fix'",
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

    it("should have all Story 8.13 new types", () => {
      expect(StandardToolTypes.WEB_FETCH).toBe("web_fetch");
      expect(StandardToolTypes.WEB_SEARCH).toBe("web_search");
      expect(StandardToolTypes.KNOWLEDGE_QUERY).toBe("knowledge_query");
      expect(StandardToolTypes.CODE_EXEC).toBe("code_exec");
      expect(StandardToolTypes.DIAGNOSTIC).toBe("diagnostic");
      expect(StandardToolTypes.NOTEBOOK_EDIT).toBe("notebook_edit");
      expect(StandardToolTypes.TODO_MANAGE).toBe("todo_manage");
      expect(StandardToolTypes.SUB_TASK).toBe("sub_task");
      expect(StandardToolTypes.USER_PROMPT).toBe("user_prompt");
      expect(StandardToolTypes.PLAN_MODE).toBe("plan_mode");
      expect(StandardToolTypes.SKILL_INVOKE).toBe("skill_invoke");
      expect(StandardToolTypes.UNKNOWN).toBe("unknown");
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

    it("should return true for unknown type tools (backward compatible)", () => {
      expect(isOtherTool(unknownTool)).toBe(true);
    });

    it("should return false for known tool types", () => {
      expect(isOtherTool(fileReadTool)).toBe(false);
      expect(isOtherTool(shellExecTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isOtherTool(undefined)).toBe(false);
    });
  });

  // === Story 8.13: New tool type tests ===

  describe("isUnknownTool", () => {
    it("should return true for unknown type tools", () => {
      expect(isUnknownTool(unknownTool)).toBe(true);
    });

    it("should return true for other type tools (backward compatible)", () => {
      expect(isUnknownTool(otherTool)).toBe(true);
    });

    it("should return false for known tool types", () => {
      expect(isUnknownTool(fileReadTool)).toBe(false);
      expect(isUnknownTool(webFetchTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isUnknownTool(undefined)).toBe(false);
    });
  });

  describe("isWebFetchTool", () => {
    it("should return true for web_fetch tools", () => {
      expect(isWebFetchTool(webFetchTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isWebFetchTool(webSearchTool)).toBe(false);
      expect(isWebFetchTool(fileReadTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isWebFetchTool(undefined)).toBe(false);
    });
  });

  describe("isWebSearchTool", () => {
    it("should return true for web_search tools", () => {
      expect(isWebSearchTool(webSearchTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isWebSearchTool(webFetchTool)).toBe(false);
      expect(isWebSearchTool(contentSearchTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isWebSearchTool(undefined)).toBe(false);
    });
  });

  describe("isWebTool", () => {
    it("should return true for all web tools", () => {
      expect(isWebTool(webFetchTool)).toBe(true);
      expect(isWebTool(webSearchTool)).toBe(true);
    });

    it("should return false for non-web tools", () => {
      expect(isWebTool(fileReadTool)).toBe(false);
      expect(isWebTool(contentSearchTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isWebTool(undefined)).toBe(false);
    });
  });

  describe("isKnowledgeQueryTool", () => {
    it("should return true for knowledge_query tools", () => {
      expect(isKnowledgeQueryTool(knowledgeQueryTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isKnowledgeQueryTool(webSearchTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isKnowledgeQueryTool(undefined)).toBe(false);
    });
  });

  describe("isCodeExecTool", () => {
    it("should return true for code_exec tools", () => {
      expect(isCodeExecTool(codeExecTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isCodeExecTool(shellExecTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isCodeExecTool(undefined)).toBe(false);
    });
  });

  describe("isDiagnosticTool", () => {
    it("should return true for diagnostic tools", () => {
      expect(isDiagnosticTool(diagnosticTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isDiagnosticTool(fileReadTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isDiagnosticTool(undefined)).toBe(false);
    });
  });

  describe("isNotebookEditTool", () => {
    it("should return true for notebook_edit tools", () => {
      expect(isNotebookEditTool(notebookEditTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isNotebookEditTool(fileEditTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isNotebookEditTool(undefined)).toBe(false);
    });
  });

  describe("isTodoManageTool", () => {
    it("should return true for todo_manage tools", () => {
      expect(isTodoManageTool(todoManageTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isTodoManageTool(subTaskTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isTodoManageTool(undefined)).toBe(false);
    });
  });

  describe("isSubTaskTool", () => {
    it("should return true for sub_task tools", () => {
      expect(isSubTaskTool(subTaskTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isSubTaskTool(shellExecTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isSubTaskTool(undefined)).toBe(false);
    });
  });

  describe("isUserPromptTool", () => {
    it("should return true for user_prompt tools", () => {
      expect(isUserPromptTool(userPromptTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isUserPromptTool(todoManageTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isUserPromptTool(undefined)).toBe(false);
    });
  });

  describe("isPlanModeTool", () => {
    it("should return true for plan_mode tools", () => {
      expect(isPlanModeTool(planModeTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isPlanModeTool(skillInvokeTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isPlanModeTool(undefined)).toBe(false);
    });
  });

  describe("isSkillInvokeTool", () => {
    it("should return true for skill_invoke tools", () => {
      expect(isSkillInvokeTool(skillInvokeTool)).toBe(true);
    });

    it("should return false for other tools", () => {
      expect(isSkillInvokeTool(subTaskTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isSkillInvokeTool(undefined)).toBe(false);
    });
  });

  describe("isAgentTool", () => {
    it("should return true for agent-related tools", () => {
      expect(isAgentTool(subTaskTool)).toBe(true);
      expect(isAgentTool(planModeTool)).toBe(true);
      expect(isAgentTool(skillInvokeTool)).toBe(true);
    });

    it("should return false for non-agent tools", () => {
      expect(isAgentTool(shellExecTool)).toBe(false);
      expect(isAgentTool(userPromptTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isAgentTool(undefined)).toBe(false);
    });
  });

  describe("isInteractiveTool", () => {
    it("should return true for interactive tools", () => {
      expect(isInteractiveTool(userPromptTool)).toBe(true);
      expect(isInteractiveTool(todoManageTool)).toBe(true);
    });

    it("should return false for non-interactive tools", () => {
      expect(isInteractiveTool(shellExecTool)).toBe(false);
      expect(isInteractiveTool(subTaskTool)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isInteractiveTool(undefined)).toBe(false);
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
