/**
 * session-file-extractor.test.ts - 会话文件提取工具测试
 * Story 2.30: AC2 - 会话日志内容回退
 * Story 8.12: Task 6.5 - 更新测试使用 standardTool
 */

import { describe, it, expect } from "vitest";
import type { NarrativeMessage, ContentBlock, StandardTool } from "@/types/message";
import { extractFileFromSession } from "./session-file-extractor";

// 辅助函数：创建测试消息
function createAssistantMessage(
    id: string,
    timestamp: string,
    content: ContentBlock[]
): NarrativeMessage {
    return {
        id,
        role: "assistant",
        timestamp,
        content,
    };
}

function createUserMessage(
    id: string,
    timestamp: string,
    text: string
): NarrativeMessage {
    return {
        id,
        role: "user",
        timestamp,
        content: [{ type: "text", content: text }],
    };
}

// 辅助函数：创建 file_write standardTool
function createFileWriteTool(path: string, content: string): StandardTool {
    return { type: "file_write", path, content };
}

// 辅助函数：创建 file_edit standardTool
function createFileEditTool(path: string, oldString?: string, newString?: string): StandardTool {
    return { type: "file_edit", path, oldString, newString };
}

// 辅助函数：创建 file_read standardTool
function createFileReadTool(path: string): StandardTool {
    return { type: "file_read", path };
}

describe("extractFileFromSession", () => {
    describe("基本功能 (使用 standardTool)", () => {
        it("从 file_write standardTool 提取文件内容", () => {
            const standardTool = createFileWriteTool(
                "src/components/Button.tsx",
                'export function Button() { return <button>Click</button>; }'
            );
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool,
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "src/components/Button.tsx", 0);

            expect(result).not.toBeNull();
            expect(result?.content).toBe('export function Button() { return <button>Click</button>; }');
            expect(result?.filePath).toBe("src/components/Button.tsx");
            expect(result?.messageIndex).toBe(0);
        });

        it("从 file_edit standardTool 提取文件内容 (使用 newString)", () => {
            const standardTool = createFileEditTool(
                "src/app.ts",
                "old content",
                "new content"
            );
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Edit",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool,
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "src/app.ts", 0);

            expect(result).not.toBeNull();
            expect(result?.content).toBe("new content");
        });
    });

    describe("路径规范化", () => {
        it("忽略前导 ./ 进行匹配", () => {
            const standardTool = createFileWriteTool("./src/app.ts", "const app = {};");
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool,
                    },
                ]),
            ];

            // 查询不带 ./ 的路径
            const result = extractFileFromSession(messages, "src/app.ts", 0);

            expect(result).not.toBeNull();
            expect(result?.content).toBe("const app = {};");
        });

        it("忽略前导 / 进行匹配", () => {
            const standardTool = createFileWriteTool("/src/main.ts", "const main = {};");
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool,
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "src/main.ts", 0);

            expect(result).not.toBeNull();
            expect(result?.content).toBe("const main = {};");
        });

        it("大小写不敏感匹配", () => {
            const standardTool = createFileWriteTool("SRC/Components/App.tsx", "export default App;");
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool,
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "src/components/app.tsx", 0);

            expect(result).not.toBeNull();
            expect(result?.content).toBe("export default App;");
        });
    });

    describe("搜索策略", () => {
        it("从当前消息向前搜索", () => {
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool: createFileWriteTool("file.ts", "version 1"),
                    },
                ]),
                createUserMessage("2", "2026-01-08T10:01:00Z", "Update the file"),
                createAssistantMessage("3", "2026-01-08T10:02:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-2",
                        toolInput: {},
                        standardTool: createFileWriteTool("file.ts", "version 2"),
                    },
                ]),
            ];

            // 从索引 2 (第三条消息) 向前搜索，应该找到索引 2 的版本
            const result = extractFileFromSession(messages, "file.ts", 2);

            expect(result).not.toBeNull();
            expect(result?.content).toBe("version 2");
            expect(result?.messageIndex).toBe(2);
        });

        it("当前消息之前没有时搜索之后的消息", () => {
            const messages: NarrativeMessage[] = [
                createUserMessage("1", "2026-01-08T10:00:00Z", "Create a file"),
                createAssistantMessage("2", "2026-01-08T10:01:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool: createFileWriteTool("new-file.ts", "new content"),
                    },
                ]),
            ];

            // 从索引 0 搜索，文件在索引 1 才被创建
            const result = extractFileFromSession(messages, "new-file.ts", 0);

            expect(result).not.toBeNull();
            expect(result?.content).toBe("new content");
            expect(result?.messageIndex).toBe(1);
        });

        it("优先返回向前搜索的结果", () => {
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool: createFileWriteTool("file.ts", "earlier version"),
                    },
                ]),
                createUserMessage("2", "2026-01-08T10:01:00Z", "Check"),
                createAssistantMessage("3", "2026-01-08T10:02:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-2",
                        toolInput: {},
                        standardTool: createFileWriteTool("file.ts", "later version"),
                    },
                ]),
            ];

            // 从索引 1 搜索，向前应该找到索引 0
            const result = extractFileFromSession(messages, "file.ts", 1);

            expect(result).not.toBeNull();
            expect(result?.content).toBe("earlier version");
            expect(result?.messageIndex).toBe(0);
        });
    });

    describe("边界情况", () => {
        it("空消息列表返回 null", () => {
            const result = extractFileFromSession([], "file.ts", 0);
            expect(result).toBeNull();
        });

        it("空目标路径返回 null", () => {
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool: createFileWriteTool("file.ts", "content"),
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "", 0);
            expect(result).toBeNull();
        });

        it("文件不存在返回 null", () => {
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool: createFileWriteTool("other-file.ts", "content"),
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "nonexistent.ts", 0);
            expect(result).toBeNull();
        });

        it("忽略用户消息 (只搜索 assistant)", () => {
            const messages: NarrativeMessage[] = [
                createUserMessage("1", "2026-01-08T10:00:00Z", "Create file.ts with content: hello"),
            ];

            const result = extractFileFromSession(messages, "file.ts", 0);
            expect(result).toBeNull();
        });

        it("忽略非 Write/Edit 工具 (file_read)", () => {
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Read",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool: createFileReadTool("file.ts"),
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "file.ts", 0);
            expect(result).toBeNull();
        });

        it("file_write 无内容返回 null", () => {
            const standardTool: StandardTool = { type: "file_write", path: "file.ts", content: "" };
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool,
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "file.ts", 0);
            expect(result).toBeNull();
        });

        it("无 standardTool 返回 null", () => {
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {
                            file_path: "file.ts",
                            content: "content without standardTool",
                        },
                        // 无 standardTool
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "file.ts", 0);
            expect(result).toBeNull();
        });
    });

    describe("ToolResult 回退", () => {
        it("从 tool_result 的 content 提取 (通过 associatedFilePath)", () => {
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:00:00Z", [
                    {
                        type: "tool_result",
                        content: "export const module = {};",
                        associatedFilePath: "src/module.ts",
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "src/module.ts", 0);

            expect(result).not.toBeNull();
            expect(result?.content).toBe("export const module = {};");
        });
    });

    describe("时间戳处理", () => {
        it("正确解析 ISO 时间戳", () => {
            const messages: NarrativeMessage[] = [
                createAssistantMessage("1", "2026-01-08T10:30:00.000Z", [
                    {
                        type: "tool_use",
                        content: "",
                        toolName: "Write",
                        toolUseId: "tu-1",
                        toolInput: {},
                        standardTool: createFileWriteTool("file.ts", "content"),
                    },
                ]),
            ];

            const result = extractFileFromSession(messages, "file.ts", 0);

            expect(result).not.toBeNull();
            expect(result?.timestamp).toBe(new Date("2026-01-08T10:30:00.000Z").getTime());
        });
    });
});
