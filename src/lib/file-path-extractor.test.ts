/**
 * file-path-extractor.test.ts - 文件路径提取工具测试
 * Story 2.12: Task 1.8
 */

import { describe, it, expect } from "vitest";
import type { ContentBlock, NarrativeMessage } from "@/types/message";
import {
  extractFilePathFromToolUse,
  extractFilePathFromToolResult,
  parseCodeBlockAnnotation,
  parseFilePathComment,
  extractFilePathFromText,
  extractFilePathWithPriority,
  toRelativePath,
  isValidFilePath,
} from "./file-path-extractor";

describe("file-path-extractor", () => {
  describe("extractFilePathFromToolUse", () => {
    it("从 Read tool_use 提取 file_path", () => {
      const block: ContentBlock = {
        type: "tool_use",
        content: "",
        toolName: "Read",
        toolInput: { file_path: "/home/user/project/src/main.ts" },
        toolUseId: "test-1",
      };

      expect(extractFilePathFromToolUse(block)).toBe(
        "/home/user/project/src/main.ts"
      );
    });

    it("从 Write tool_use 提取 file_path", () => {
      const block: ContentBlock = {
        type: "tool_use",
        content: "",
        toolName: "Write",
        toolInput: { file_path: "src/components/Button.tsx", content: "..." },
        toolUseId: "test-2",
      };

      expect(extractFilePathFromToolUse(block)).toBe("src/components/Button.tsx");
    });

    it("从 Edit tool_use 提取 file_path", () => {
      const block: ContentBlock = {
        type: "tool_use",
        content: "",
        toolName: "Edit",
        toolInput: {
          file_path: "./lib/utils.ts",
          old_string: "foo",
          new_string: "bar",
        },
        toolUseId: "test-3",
      };

      expect(extractFilePathFromToolUse(block)).toBe("./lib/utils.ts");
    });

    it("从 Glob tool_use 提取 path", () => {
      const block: ContentBlock = {
        type: "tool_use",
        content: "",
        toolName: "Glob",
        toolInput: { path: "src", pattern: "**/*.ts" },
        toolUseId: "test-4",
      };

      // Glob 的 path 可能不包含扩展名，所以不应该被视为有效文件路径
      expect(extractFilePathFromToolUse(block)).toBe(null);
    });

    it("非 tool_use 类型返回 null", () => {
      const block: ContentBlock = {
        type: "text",
        content: "Hello world",
      };

      expect(extractFilePathFromToolUse(block)).toBe(null);
    });

    it("无 toolInput 返回 null", () => {
      const block: ContentBlock = {
        type: "tool_use",
        content: "",
        toolName: "Read",
        toolUseId: "test-5",
      };

      expect(extractFilePathFromToolUse(block)).toBe(null);
    });
  });

  describe("extractFilePathFromToolResult", () => {
    it("提取 associatedFilePath", () => {
      const block: ContentBlock = {
        type: "tool_result",
        content: "file contents...",
        associatedFilePath: "src/app.ts",
      };

      expect(extractFilePathFromToolResult(block)).toBe("src/app.ts");
    });

    it("非 tool_result 类型返回 null", () => {
      const block: ContentBlock = {
        type: "text",
        content: "Hello",
        associatedFilePath: "src/app.ts",
      };

      expect(extractFilePathFromToolResult(block)).toBe(null);
    });

    it("无 associatedFilePath 返回 null", () => {
      const block: ContentBlock = {
        type: "tool_result",
        content: "result",
      };

      expect(extractFilePathFromToolResult(block)).toBe(null);
    });
  });

  describe("parseCodeBlockAnnotation", () => {
    it("解析 ```typescript:path 格式", () => {
      const text = "Here is the code:\n```typescript:src/components/Button.tsx\nexport function Button() {}\n```";

      expect(parseCodeBlockAnnotation(text)).toEqual([
        "src/components/Button.tsx",
      ]);
    });

    it("解析 ```ts:path 格式", () => {
      const text = "```ts:lib/utils.ts\nconst foo = 1;\n```";

      expect(parseCodeBlockAnnotation(text)).toEqual(["lib/utils.ts"]);
    });

    it("解析多个代码块", () => {
      const text = `
\`\`\`typescript:src/a.ts
const a = 1;
\`\`\`

\`\`\`javascript:src/b.js
const b = 2;
\`\`\`
`;

      expect(parseCodeBlockAnnotation(text)).toEqual(["src/a.ts", "src/b.js"]);
    });

    it("无代码块标注返回空数组", () => {
      const text = "```typescript\nconst foo = 1;\n```";

      expect(parseCodeBlockAnnotation(text)).toEqual([]);
    });

    it("忽略无效路径", () => {
      const text = "```ts:http://example.com\ncode\n```";

      expect(parseCodeBlockAnnotation(text)).toEqual([]);
    });
  });

  describe("parseFilePathComment", () => {
    it("解析 // filepath: path 格式", () => {
      const text = "// filepath: src/utils/helper.ts\nfunction helper() {}";

      expect(parseFilePathComment(text)).toEqual(["src/utils/helper.ts"]);
    });

    it("解析 // file: path 格式", () => {
      const text = "// file: lib/config.json\n{}";

      expect(parseFilePathComment(text)).toEqual(["lib/config.json"]);
    });

    it("解析 # filepath: path 格式 (Python/Shell)", () => {
      const text = "# filepath: scripts/deploy.py\ndef deploy(): pass";

      expect(parseFilePathComment(text)).toEqual(["scripts/deploy.py"]);
    });

    it("解析多个注释", () => {
      const text = `
// filepath: src/a.ts
const a = 1;

// filepath: src/b.ts
const b = 2;
`;

      expect(parseFilePathComment(text)).toEqual(["src/a.ts", "src/b.ts"]);
    });

    it("无注释返回空数组", () => {
      const text = "const foo = 1;";

      expect(parseFilePathComment(text)).toEqual([]);
    });
  });

  describe("extractFilePathFromText", () => {
    it("提取相对路径 ./path", () => {
      const text = "Please check ./src/main.ts for details";

      expect(extractFilePathFromText(text)).toContain("./src/main.ts");
    });

    it("提取相对路径 ../path", () => {
      const text = "See ../lib/utils.ts";

      expect(extractFilePathFromText(text)).toContain("../lib/utils.ts");
    });

    it("提取引号包裹的路径", () => {
      const text = 'Open "src/components/Button.tsx" to edit';

      expect(extractFilePathFromText(text)).toContain(
        "src/components/Button.tsx"
      );
    });

    it("提取反引号包裹的路径", () => {
      const text = "Edit `lib/utils.ts` file";

      expect(extractFilePathFromText(text)).toContain("lib/utils.ts");
    });

    it("去重重复路径", () => {
      const text = "Check src/app.ts and then src/app.ts again";

      const paths = extractFilePathFromText(text);
      const unique = [...new Set(paths)];
      expect(paths).toEqual(unique);
    });

    it("忽略 URL", () => {
      const text = "Visit https://example.com/path.js for docs";

      const paths = extractFilePathFromText(text);
      expect(paths.some((p) => p.includes("http"))).toBe(false);
    });

    it("忽略常见词 e.g", () => {
      const text = "e.g. this is an example";

      expect(extractFilePathFromText(text)).toEqual([]);
    });
  });

  describe("isValidFilePath", () => {
    it("有效的相对路径", () => {
      expect(isValidFilePath("src/main.ts")).toBe(true);
      expect(isValidFilePath("./lib/utils.js")).toBe(true);
      expect(isValidFilePath("../config.json")).toBe(true);
    });

    it("有效的绝对路径", () => {
      expect(isValidFilePath("/home/user/project/src/app.ts")).toBe(true);
    });

    it("无扩展名返回 false", () => {
      expect(isValidFilePath("src/components")).toBe(false);
      expect(isValidFilePath("/home/user")).toBe(false);
    });

    it("URL 返回 false", () => {
      expect(isValidFilePath("https://example.com/file.js")).toBe(false);
      expect(isValidFilePath("http://localhost:3000/app.ts")).toBe(false);
    });

    it("包含无效字符返回 false", () => {
      expect(isValidFilePath("src/file<name>.ts")).toBe(false);
      expect(isValidFilePath("src/file|name.ts")).toBe(false);
    });

    it("过短路径返回 false", () => {
      expect(isValidFilePath("a")).toBe(false);
      expect(isValidFilePath("")).toBe(false);
    });

    it("过长路径返回 false", () => {
      const longPath = "a".repeat(501) + ".ts";
      expect(isValidFilePath(longPath)).toBe(false);
    });
  });

  describe("extractFilePathWithPriority", () => {
    it("优先从 tool_use 提取 (最高优先级)", () => {
      const message: NarrativeMessage = {
        id: "1",
        role: "assistant",
        timestamp: new Date().toISOString(),
        content: [
          {
            type: "text",
            content: "Let me read `other/file.ts` for you",
          },
          {
            type: "tool_use",
            content: "",
            toolName: "Read",
            toolInput: { file_path: "src/main.ts" },
            toolUseId: "t1",
          },
        ],
      };

      const result = extractFilePathWithPriority(message);
      expect(result).toEqual({
        path: "src/main.ts",
        source: "tool_use",
        confidence: "high",
      });
    });

    it("从 tool_result 的 associatedFilePath 提取", () => {
      const message: NarrativeMessage = {
        id: "2",
        role: "assistant",
        timestamp: new Date().toISOString(),
        content: [
          {
            type: "tool_result",
            content: "file contents",
            associatedFilePath: "lib/utils.ts",
          },
        ],
      };

      const result = extractFilePathWithPriority(message);
      expect(result).toEqual({
        path: "lib/utils.ts",
        source: "tool_result",
        confidence: "high",
      });
    });

    it("从代码块标注提取", () => {
      const message: NarrativeMessage = {
        id: "3",
        role: "assistant",
        timestamp: new Date().toISOString(),
        content: [
          {
            type: "text",
            content:
              "```typescript:src/component.tsx\nexport function Component() {}\n```",
          },
        ],
      };

      const result = extractFilePathWithPriority(message);
      expect(result).toEqual({
        path: "src/component.tsx",
        source: "code_block",
        confidence: "medium",
      });
    });

    it("从文件路径注释提取", () => {
      const message: NarrativeMessage = {
        id: "4",
        role: "assistant",
        timestamp: new Date().toISOString(),
        content: [
          {
            type: "text",
            content: "// filepath: src/helper.ts\nfunction help() {}",
          },
        ],
      };

      const result = extractFilePathWithPriority(message);
      expect(result).toEqual({
        path: "src/helper.ts",
        source: "comment",
        confidence: "medium",
      });
    });

    it("从文本匹配提取 (最低优先级)", () => {
      const message: NarrativeMessage = {
        id: "5",
        role: "assistant",
        timestamp: new Date().toISOString(),
        content: [
          {
            type: "text",
            content: "Please check `src/app.ts` for the implementation",
          },
        ],
      };

      const result = extractFilePathWithPriority(message);
      expect(result?.path).toBe("src/app.ts");
      expect(result?.source).toBe("text_match");
      expect(result?.confidence).toBe("low");
    });

    it("无文件路径返回 null", () => {
      const message: NarrativeMessage = {
        id: "6",
        role: "user",
        timestamp: new Date().toISOString(),
        content: [
          {
            type: "text",
            content: "Hello, can you help me?",
          },
        ],
      };

      expect(extractFilePathWithPriority(message)).toBe(null);
    });
  });

  describe("toRelativePath", () => {
    it("绝对路径转相对路径", () => {
      expect(
        toRelativePath("/home/user/project/src/main.ts", "/home/user/project")
      ).toBe("src/main.ts");
    });

    it("已是相对路径则保持不变", () => {
      expect(toRelativePath("src/main.ts", "/home/user/project")).toBe(
        "src/main.ts"
      );
    });

    it("处理 ./ 开头的相对路径", () => {
      expect(toRelativePath("./src/main.ts", "/home/user/project")).toBe(
        "./src/main.ts"
      );
    });

    it("处理 Windows 风格路径", () => {
      expect(
        toRelativePath("C:\\Users\\project\\src\\main.ts", "C:\\Users\\project")
      ).toBe("src/main.ts");
    });

    it("仓库路径末尾有 / 时正常处理", () => {
      expect(
        toRelativePath("/home/user/project/src/main.ts", "/home/user/project/")
      ).toBe("src/main.ts");
    });

    it("不匹配的绝对路径返回去掉开头 / 的路径", () => {
      expect(
        toRelativePath("/other/path/file.ts", "/home/user/project")
      ).toBe("other/path/file.ts");
    });
  });
});
