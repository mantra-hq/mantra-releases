/**
 * Session Utils Tests
 * Story 8.19: 测试前端直接使用 snake_case 字段
 *
 * 架构设计：后端 JSON 使用 snake_case，前端类型定义也使用 snake_case
 * 不再需要 camelCase 转换
 */

import { describe, it, expect } from "vitest";
import type { ToolResultData } from "@/types/message";

describe("session-utils", () => {
  describe("ToolResultData 类型", () => {
    // 后端返回的 snake_case 格式，前端直接使用
    const fileReadResult: ToolResultData = {
      type: "file_read",
      file_path: "/src/main.rs",
      start_line: 10,
      num_lines: 50,
      total_lines: 100,
    };

    const shellExecResult: ToolResultData = {
      type: "shell_exec",
      exit_code: 0,
      stdout: "test output",
      stderr: "",
    };

    const fileEditResult: ToolResultData = {
      type: "file_edit",
      file_path: "/src/lib.rs",
      old_string: "fn old()",
      new_string: "fn new()",
    };

    it("file_read 类型应该使用 snake_case 字段", () => {
      if (fileReadResult.type === "file_read") {
        expect(fileReadResult.file_path).toBe("/src/main.rs");
        expect(fileReadResult.start_line).toBe(10);
        expect(fileReadResult.num_lines).toBe(50);
        expect(fileReadResult.total_lines).toBe(100);
      }
    });

    it("shell_exec 类型应该使用 snake_case 字段", () => {
      if (shellExecResult.type === "shell_exec") {
        expect(shellExecResult.exit_code).toBe(0);
        expect(shellExecResult.stdout).toBe("test output");
        expect(shellExecResult.stderr).toBe("");
      }
    });

    it("file_edit 类型应该使用 snake_case 字段", () => {
      if (fileEditResult.type === "file_edit") {
        expect(fileEditResult.file_path).toBe("/src/lib.rs");
        expect(fileEditResult.old_string).toBe("fn old()");
        expect(fileEditResult.new_string).toBe("fn new()");
      }
    });

    it("type 字段应该正确区分不同类型", () => {
      expect(fileReadResult.type).toBe("file_read");
      expect(shellExecResult.type).toBe("shell_exec");
      expect(fileEditResult.type).toBe("file_edit");
    });
  });
});
