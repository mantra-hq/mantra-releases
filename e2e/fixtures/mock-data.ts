/**
 * Mock Data - E2E 测试 Mock 数据
 * Story 9.2: Task 3
 *
 * 定义测试用的模拟数据:
 * - MOCK_PROJECTS: 2-3 个测试项目
 * - MOCK_SESSIONS: 每个项目 2-3 个会话
 * - MOCK_MESSAGES: 每个会话 5-10 条消息
 */

import type { Project, SnapshotResult } from "@/types/project";
import type { MantraSession, MantraMessage, MantraContentBlock } from "@/lib/session-utils";
import type { SessionSummary } from "@/lib/project-ipc";
import type { SanitizationRule } from "@/components/sanitizer/types";
import type { DefaultPaths } from "@/lib/import-ipc";
import type { StandardToolFileEdit, StandardToolFileRead, StandardToolShellExec, ToolResultDataFileRead, ToolResultDataShellExec } from "@/types/message";

// =============================================================================
// Mock Projects (AC #3: 2-3 个项目)
// =============================================================================

export const MOCK_PROJECTS: Project[] = [
  {
    id: "mock-project-alpha",
    name: "Mock Project Alpha",
    cwd: "/mock/projects/alpha",
    session_count: 3,
    non_empty_session_count: 3,
    created_at: "2025-01-01T00:00:00Z",
    last_activity: "2025-01-10T12:00:00Z",
    git_repo_path: "/mock/projects/alpha",
    has_git_repo: true,
    git_remote_url: "https://github.com/mock/alpha",
    is_empty: false,
  },
  {
    id: "mock-project-beta",
    name: "Mock Project Beta",
    cwd: "/mock/projects/beta",
    session_count: 2,
    non_empty_session_count: 2,
    created_at: "2025-01-02T00:00:00Z",
    last_activity: "2025-01-09T10:00:00Z",
    git_repo_path: "/mock/projects/beta",
    has_git_repo: true,
    git_remote_url: "https://github.com/mock/beta",
    is_empty: false,
  },
  {
    id: "mock-project-gamma",
    name: "Mock Project Gamma",
    cwd: "/mock/projects/gamma",
    session_count: 2,
    non_empty_session_count: 1,
    created_at: "2025-01-03T00:00:00Z",
    last_activity: "2025-01-08T08:00:00Z",
    git_repo_path: null,
    has_git_repo: false,
    git_remote_url: null,
    is_empty: false,
  },
];

// =============================================================================
// Mock Sessions (AC #3: 每项目 2-3 个会话)
// =============================================================================

export const MOCK_SESSION_SUMMARIES: SessionSummary[] = [
  // Alpha 项目会话 (3 个)
  {
    id: "mock-session-alpha-1",
    source: "claude",
    created_at: "2025-01-10T10:00:00Z",
    updated_at: "2025-01-10T12:00:00Z",
    message_count: 8,
    is_empty: false,
    title: "实现用户认证模块",
  },
  {
    id: "mock-session-alpha-2",
    source: "claude",
    created_at: "2025-01-09T14:00:00Z",
    updated_at: "2025-01-09T16:00:00Z",
    message_count: 6,
    is_empty: false,
    title: "修复登录页面 Bug",
  },
  {
    id: "mock-session-alpha-3",
    source: "gemini",
    created_at: "2025-01-08T09:00:00Z",
    updated_at: "2025-01-08T11:00:00Z",
    message_count: 5,
    is_empty: false,
    title: "代码审查讨论",
  },
  // Beta 项目会话 (2 个)
  {
    id: "mock-session-beta-1",
    source: "cursor",
    created_at: "2025-01-09T08:00:00Z",
    updated_at: "2025-01-09T10:00:00Z",
    message_count: 10,
    is_empty: false,
    title: "重构数据库模型",
  },
  {
    id: "mock-session-beta-2",
    source: "claude",
    created_at: "2025-01-07T15:00:00Z",
    updated_at: "2025-01-07T17:00:00Z",
    message_count: 7,
    is_empty: false,
    title: "添加单元测试",
  },
  // Story 8.19: Cursor 工具调用测试会话
  {
    id: "mock-session-cursor-tools",
    source: "cursor",
    created_at: "2025-01-15T10:00:00Z",
    updated_at: "2025-01-15T12:00:00Z",
    message_count: 6,
    is_empty: false,
    title: "Cursor 工具调用测试",
  },
  // Gamma 项目会话 (2 个)
  {
    id: "mock-session-gamma-1",
    source: "gemini",
    created_at: "2025-01-08T06:00:00Z",
    updated_at: "2025-01-08T08:00:00Z",
    message_count: 5,
    is_empty: false,
    title: "项目初始化讨论",
  },
  {
    id: "mock-session-gamma-2",
    source: "claude",
    created_at: "2025-01-05T10:00:00Z",
    updated_at: "2025-01-05T10:30:00Z",
    message_count: 0,
    is_empty: true,
    title: undefined,
  },
];

// 项目 ID -> 会话列表映射
export const MOCK_PROJECT_SESSIONS_MAP: Record<string, SessionSummary[]> = {
  "mock-project-alpha": MOCK_SESSION_SUMMARIES.filter((s) => s.id.includes("alpha")),
  "mock-project-beta": MOCK_SESSION_SUMMARIES.filter((s) => s.id.includes("beta")),
  "mock-project-gamma": MOCK_SESSION_SUMMARIES.filter((s) => s.id.includes("gamma")),
};

// =============================================================================
// Mock Messages (AC #3: 每会话 5-10 条消息)
// =============================================================================

/**
 * 生成 Mock 消息
 */
function createMockMessage(
  role: "user" | "assistant",
  content: string,
  timestamp: string,
  blocks?: MantraContentBlock[]
): MantraMessage {
  return {
    role,
    timestamp,
    content_blocks: blocks ?? [{ type: "text", text: content }],
  };
}

/**
 * Mock 会话详情
 */
export const MOCK_SESSIONS: Record<string, MantraSession> = {
  "mock-session-alpha-1": {
    id: "mock-session-alpha-1",
    source: "claude",
    cwd: "/mock/projects/alpha",
    created_at: "2025-01-10T10:00:00Z",
    updated_at: "2025-01-10T12:00:00Z",
    metadata: {
      model: "claude-3-opus",
      total_tokens: 5000,
      title: "实现用户认证模块",
      original_path: "/mock/logs/alpha-1.json",
    },
    messages: [
      createMockMessage("user", "帮我实现一个用户认证模块", "2025-01-10T10:00:00Z"),
      createMockMessage("assistant", "好的，我来帮你设计用户认证模块。首先需要考虑以下几点...", "2025-01-10T10:01:00Z"),
      createMockMessage("user", "请使用 JWT 方式实现", "2025-01-10T10:05:00Z"),
      createMockMessage("assistant", "明白，我将使用 JWT 实现认证。让我先创建认证相关的文件...", "2025-01-10T10:06:00Z", [
        { type: "text", text: "我将创建以下文件:" },
        {
          type: "tool_use",
          id: "tool-1",
          name: "Write",
          input: { file_path: "src/auth/jwt.ts", content: "// JWT implementation" },
          standard_tool: { type: "file_write", path: "src/auth/jwt.ts", content: "// JWT implementation" },
        },
      ]),
      createMockMessage("assistant", "文件已创建。接下来实现登录逻辑...", "2025-01-10T10:10:00Z"),
      createMockMessage("user", "能添加刷新 token 的功能吗？", "2025-01-10T11:00:00Z"),
      createMockMessage("assistant", "当然可以，我来添加 refresh token 机制...", "2025-01-10T11:01:00Z"),
      createMockMessage("user", "太棒了，谢谢！", "2025-01-10T12:00:00Z"),
    ],
  },
  "mock-session-alpha-2": {
    id: "mock-session-alpha-2",
    source: "claude",
    cwd: "/mock/projects/alpha",
    created_at: "2025-01-09T14:00:00Z",
    updated_at: "2025-01-09T16:00:00Z",
    metadata: {
      model: "claude-3-sonnet",
      total_tokens: 3000,
      title: "修复登录页面 Bug",
    },
    messages: [
      createMockMessage("user", "登录页面点击按钮没反应", "2025-01-09T14:00:00Z"),
      createMockMessage("assistant", "让我检查一下登录按钮的事件处理...", "2025-01-09T14:01:00Z", [
        { type: "text", text: "让我读取登录组件的代码" },
        {
          type: "tool_use",
          id: "tool-2",
          name: "Read",
          input: { file_path: "src/components/Login.tsx" },
          standard_tool: { type: "file_read", path: "src/components/Login.tsx" },
        },
      ]),
      createMockMessage("assistant", "我发现问题了，onClick 事件没有正确绑定。", "2025-01-09T14:05:00Z"),
      createMockMessage("user", "能修复一下吗？", "2025-01-09T14:10:00Z"),
      createMockMessage("assistant", "已修复，问题是 async 函数没有 await。", "2025-01-09T15:00:00Z"),
      createMockMessage("user", "现在可以了，感谢！", "2025-01-09T16:00:00Z"),
    ],
  },
  "mock-session-alpha-3": {
    id: "mock-session-alpha-3",
    source: "gemini",
    cwd: "/mock/projects/alpha",
    created_at: "2025-01-08T09:00:00Z",
    updated_at: "2025-01-08T11:00:00Z",
    metadata: {
      model: "gemini-pro",
      total_tokens: 2500,
      title: "代码审查讨论",
    },
    messages: [
      createMockMessage("user", "帮我审查这段代码", "2025-01-08T09:00:00Z"),
      createMockMessage("assistant", "好的，让我仔细看看这段代码...", "2025-01-08T09:01:00Z"),
      createMockMessage("assistant", "我发现了几个可以改进的地方...", "2025-01-08T09:10:00Z"),
      createMockMessage("user", "第一个建议很好，请帮我修改", "2025-01-08T10:00:00Z"),
      createMockMessage("assistant", "好的，已完成修改。", "2025-01-08T11:00:00Z"),
    ],
  },
  "mock-session-beta-1": {
    id: "mock-session-beta-1",
    source: "cursor",
    cwd: "/mock/projects/beta",
    created_at: "2025-01-09T08:00:00Z",
    updated_at: "2025-01-09T10:00:00Z",
    metadata: {
      model: "gpt-4-turbo",
      total_tokens: 8000,
      title: "重构数据库模型",
    },
    messages: [
      createMockMessage("user", "数据库模型需要重构", "2025-01-09T08:00:00Z"),
      createMockMessage("assistant", "了解，让我分析当前的模型结构...", "2025-01-09T08:01:00Z"),
      createMockMessage("user", "主要是用户表和订单表的关系", "2025-01-09T08:10:00Z"),
      createMockMessage("assistant", "我建议使用外键关联...", "2025-01-09T08:15:00Z"),
      createMockMessage("user", "好的，请实现", "2025-01-09T08:30:00Z"),
      createMockMessage("assistant", "正在创建迁移文件...", "2025-01-09T08:31:00Z"),
      createMockMessage("assistant", "迁移文件已创建", "2025-01-09T09:00:00Z"),
      createMockMessage("user", "测试一下", "2025-01-09T09:30:00Z"),
      createMockMessage("assistant", "测试通过！", "2025-01-09T09:45:00Z"),
      createMockMessage("user", "完美！", "2025-01-09T10:00:00Z"),
    ],
  },
  "mock-session-beta-2": {
    id: "mock-session-beta-2",
    source: "claude",
    cwd: "/mock/projects/beta",
    created_at: "2025-01-07T15:00:00Z",
    updated_at: "2025-01-07T17:00:00Z",
    metadata: {
      model: "claude-3-sonnet",
      total_tokens: 4000,
      title: "添加单元测试",
    },
    messages: [
      createMockMessage("user", "给用户服务添加单元测试", "2025-01-07T15:00:00Z"),
      createMockMessage("assistant", "好的，我来为 UserService 编写测试...", "2025-01-07T15:01:00Z"),
      createMockMessage("user", "使用 Jest", "2025-01-07T15:05:00Z"),
      createMockMessage("assistant", "明白，使用 Jest + @testing-library...", "2025-01-07T15:06:00Z"),
      createMockMessage("assistant", "测试文件已创建", "2025-01-07T16:00:00Z"),
      createMockMessage("user", "运行测试", "2025-01-07T16:30:00Z"),
      createMockMessage("assistant", "所有测试通过！覆盖率 85%", "2025-01-07T17:00:00Z"),
    ],
  },
  // Story 8.19 E2E 测试: Cursor structured_result
  "mock-session-cursor-tools": {
    id: "mock-session-cursor-tools",
    source: "cursor",
    cwd: "/mock/projects/beta",
    created_at: "2025-01-15T10:00:00Z",
    updated_at: "2025-01-15T12:00:00Z",
    metadata: {
      model: "gpt-4-turbo",
      total_tokens: 5000,
      title: "Cursor 工具调用测试",
      original_path: "/mock/logs/cursor-tools-test.json",
    },
    messages: [
      createMockMessage("user", "帮我看一下 main.rs 的内容", "2025-01-15T10:00:00Z"),
      createMockMessage("assistant", "好的，我来读取文件内容。", "2025-01-15T10:01:00Z", [
        { type: "text", text: "读取文件：" },
        {
          type: "tool_use",
          id: "cursor-tool-read-1",
          name: "read_file_v2",
          input: {
            file_path: "/src/main.rs",
            start_line: 1,
            end_line: 50,
          },
          standard_tool: {
            type: "file_read",
            path: "/src/main.rs",
            start_line: 1,
            end_line: 50,
          } as StandardToolFileRead,
        },
      ]),
      createMockMessage("assistant", "文件内容如下。", "2025-01-15T10:02:00Z", [
        {
          type: "tool_result",
          tool_use_id: "cursor-tool-read-1",
          content: "fn main() {\n    println!(\"Hello, world!\");\n}\n\nfn helper() {\n    // TODO: implement\n}",
          is_error: false,
          structured_result: {
            type: "file_read",
            file_path: "/src/main.rs",
            start_line: 1,
            num_lines: 7,
            total_lines: undefined,
          } as ToolResultDataFileRead,
        },
        { type: "text", text: "这是一个简单的 Rust 程序入口。" },
      ]),
      createMockMessage("user", "运行 cargo build", "2025-01-15T10:05:00Z"),
      createMockMessage("assistant", "好的，执行编译命令。", "2025-01-15T10:06:00Z", [
        { type: "text", text: "执行命令：" },
        {
          type: "tool_use",
          id: "cursor-tool-shell-1",
          name: "run_terminal_cmd",
          input: {
            command: "cargo build",
            cwd: "/mock/projects/beta",
          },
          standard_tool: {
            type: "shell_exec",
            command: "cargo build",
            cwd: "/mock/projects/beta",
          } as StandardToolShellExec,
        },
      ]),
      createMockMessage("assistant", "编译完成。", "2025-01-15T10:07:00Z", [
        {
          type: "tool_result",
          tool_use_id: "cursor-tool-shell-1",
          content: "   Compiling beta v0.1.0 (/mock/projects/beta)\n    Finished dev [unoptimized + debuginfo] target(s) in 2.35s\nexit code: 0",
          is_error: false,
          structured_result: {
            type: "shell_exec",
            exitCode: 0,
            stdout: "   Compiling beta v0.1.0 (/mock/projects/beta)\n    Finished dev [unoptimized + debuginfo] target(s) in 2.35s",
            stderr: "",
          } as ToolResultDataShellExec,
        },
        { type: "text", text: "编译成功！" },
      ]),
    ],
  },
  "mock-session-gamma-1": {
    id: "mock-session-gamma-1",
    source: "gemini",
    cwd: "/mock/projects/gamma",
    created_at: "2025-01-08T06:00:00Z",
    updated_at: "2025-01-08T08:00:00Z",
    metadata: {
      model: "gemini-pro",
      total_tokens: 2000,
      title: "项目初始化讨论",
    },
    messages: [
      createMockMessage("user", "帮我初始化一个 React 项目", "2025-01-08T06:00:00Z"),
      createMockMessage("assistant", "好的，我推荐使用 Vite + React + TypeScript...", "2025-01-08T06:01:00Z"),
      createMockMessage("user", "好的，继续", "2025-01-08T06:30:00Z"),
      createMockMessage("assistant", "执行初始化命令...", "2025-01-08T07:00:00Z"),
      createMockMessage("assistant", "项目已初始化完成！", "2025-01-08T08:00:00Z"),
    ],
  },
  "mock-session-gamma-2": {
    id: "mock-session-gamma-2",
    source: "claude",
    cwd: "/mock/projects/gamma",
    created_at: "2025-01-05T10:00:00Z",
    updated_at: "2025-01-05T10:30:00Z",
    metadata: {
      model: "claude-3-haiku",
      title: undefined,
    },
    messages: [],
  },
  // Story 8.11 E2E 测试: file_edit 差异视图
  "mock-session-file-edit": {
    id: "mock-session-file-edit",
    source: "claude",
    cwd: "/mock/projects/alpha",
    created_at: "2025-01-11T10:00:00Z",
    updated_at: "2025-01-11T12:00:00Z",
    metadata: {
      model: "claude-3-opus",
      total_tokens: 3000,
      title: "FileEdit Diff 测试会话",
      original_path: "/mock/logs/file-edit-test.json",
    },
    messages: [
      createMockMessage("user", "帮我修复这个函数的 bug", "2025-01-11T10:00:00Z"),
      createMockMessage("assistant", "好的，我来修复这个问题。", "2025-01-11T10:01:00Z", [
        { type: "text", text: "我发现了问题，需要修改代码：" },
        {
          type: "tool_use",
          id: "tool-file-edit-1",
          name: "Edit",
          input: {
            file_path: "src/utils/calculator.ts",
            old_string: "function add(a, b) {\n  return a - b; // Bug: should be +\n}",
            new_string: "function add(a, b) {\n  return a + b; // Fixed\n}",
          },
          // 使用 snake_case 匹配后端真实格式
          standard_tool: {
            type: "file_edit",
            path: "src/utils/calculator.ts",
            oldString: "function add(a, b) {\n  return a - b; // Bug: should be +\n}",
            newString: "function add(a, b) {\n  return a + b; // Fixed\n}",
          } as StandardToolFileEdit,
        },
      ]),
      createMockMessage("assistant", "文件已修改。", "2025-01-11T10:02:00Z", [
        {
          type: "tool_result",
          tool_use_id: "tool-file-edit-1",
          content: "File edited successfully",
          is_error: false,
        },
        { type: "text", text: "修复完成！现在 add 函数可以正确执行加法了。" },
      ]),
      createMockMessage("user", "再帮我添加一个新函数", "2025-01-11T10:05:00Z"),
      createMockMessage("assistant", "好的，我来添加新函数。", "2025-01-11T10:06:00Z", [
        { type: "text", text: "添加一个乘法函数：" },
        {
          type: "tool_use",
          id: "tool-file-edit-2",
          name: "Edit",
          input: {
            file_path: "src/utils/calculator.ts",
            new_string: "function multiply(a, b) {\n  return a * b;\n}",
          },
          // 只有 new_string 没有 old_string 的情况 (使用 snake_case 匹配后端)
          standard_tool: {
            type: "file_edit",
            path: "src/utils/calculator.ts",
            newString: "function multiply(a, b) {\n  return a * b;\n}",
          } as StandardToolFileEdit,
        },
      ]),
      createMockMessage("assistant", "新函数已添加。", "2025-01-11T10:07:00Z", [
        {
          type: "tool_result",
          tool_use_id: "tool-file-edit-2",
          content: "File edited successfully",
          is_error: false,
        },
        { type: "text", text: "乘法函数已添加完成！" },
      ]),
    ],
  },
  // Dart 语法高亮测试会话
  "mock-session-dart-highlight": {
    id: "mock-session-dart-highlight",
    source: "claude",
    cwd: "/mock/projects/alpha",
    created_at: "2025-01-12T10:00:00Z",
    updated_at: "2025-01-12T12:00:00Z",
    metadata: {
      model: "claude-3-opus",
      total_tokens: 2000,
      title: "Dart 语法高亮测试",
      original_path: "/mock/logs/dart-highlight-test.json",
    },
    messages: [
      createMockMessage("user", "帮我写一个 Dart 类", "2025-01-12T10:00:00Z"),
      createMockMessage("assistant", "好的，我来创建一个 Dart 类。", "2025-01-12T10:01:00Z", [
        { type: "text", text: "创建 User 类：" },
        {
          type: "tool_use",
          id: "tool-dart-1",
          name: "Write",
          input: {
            file_path: "lib/models/user.dart",
            content: `class User {
  final String name;
  final int age;

  User({required this.name, required this.age});

  void greet() {
    print('Hello, I am \$name and I am \$age years old.');
  }
}`,
          },
          standard_tool: {
            type: "file_write",
            path: "lib/models/user.dart",
            content: `class User {
  final String name;
  final int age;

  User({required this.name, required this.age});

  void greet() {
    print('Hello, I am \$name and I am \$age years old.');
  }
}`,
          },
        },
      ]),
      createMockMessage("assistant", "Dart 类已创建。", "2025-01-12T10:02:00Z", [
        {
          type: "tool_result",
          tool_use_id: "tool-dart-1",
          // 工具输出内容应该是实际的代码（用于代码面板显示）
          content: `class User {
  final String name;
  final int age;

  User({required this.name, required this.age});

  void greet() {
    print('Hello, I am \$name and I am \$age years old.');
  }
}`,
          is_error: false,
        },
        { type: "text", text: "User 类已创建完成！" },
      ]),
    ],
  },
  // Shell 命令执行测试会话 (Story 8.11 fix)
  "mock-session-shell-exec": {
    id: "mock-session-shell-exec",
    source: "claude",
    cwd: "/mock/projects/alpha",
    created_at: "2025-01-13T10:00:00Z",
    updated_at: "2025-01-13T12:00:00Z",
    metadata: {
      model: "claude-3-opus",
      total_tokens: 1500,
      title: "终端命令执行测试",
      original_path: "/mock/logs/shell-exec-test.json",
    },
    messages: [
      createMockMessage("user", "帮我运行测试", "2025-01-13T10:00:00Z"),
      createMockMessage("assistant", "好的，我来运行测试。", "2025-01-13T10:01:00Z", [
        { type: "text", text: "执行测试命令：" },
        {
          type: "tool_use",
          id: "tool-shell-1",
          name: "Bash",
          input: {
            command: "npm test",
          },
          standard_tool: {
            type: "shell_exec",
            command: "npm test",
            cwd: "/mock/projects/alpha",
          } as StandardToolShellExec,
        },
      ]),
      createMockMessage("assistant", "测试完成。", "2025-01-13T10:02:00Z", [
        {
          type: "tool_result",
          tool_use_id: "tool-shell-1",
          content: "Test Suites: 5 passed, 5 total\nTests: 25 passed, 25 total\nTime: 3.5s",
          is_error: false,
          structured_result: {
            type: "shell_exec",
            exitCode: 0,
            stdout: "Test Suites: 5 passed, 5 total\nTests: 25 passed, 25 total\nTime: 3.5s",
            stderr: "",
          } as ToolResultDataShellExec,
        },
        { type: "text", text: "所有测试都通过了！" },
      ]),
      createMockMessage("user", "再运行一下 lint 检查", "2025-01-13T10:05:00Z"),
      createMockMessage("assistant", "好的，运行 lint 检查。", "2025-01-13T10:06:00Z", [
        { type: "text", text: "执行 lint 命令：" },
        {
          type: "tool_use",
          id: "tool-shell-2",
          name: "Bash",
          input: {
            command: "npm run lint",
          },
          standard_tool: {
            type: "shell_exec",
            command: "npm run lint",
            cwd: "/mock/projects/alpha",
          } as StandardToolShellExec,
        },
      ]),
      createMockMessage("assistant", "Lint 检查有问题。", "2025-01-13T10:07:00Z", [
        {
          type: "tool_result",
          tool_use_id: "tool-shell-2",
          content: "/src/utils.ts:15:1 warning Missing return type",
          is_error: true,
          structured_result: {
            type: "shell_exec",
            exitCode: 1,
            stdout: "",
            stderr: "/src/utils.ts:15:1 warning Missing return type",
          } as ToolResultDataShellExec,
        },
        { type: "text", text: "发现一个 lint 警告，需要添加返回类型。" },
      ]),
    ],
  },
  // Shell 命令执行测试会话 - JSON 格式 content (无 structured_result)
  // 模拟真实后端数据格式
  "mock-session-shell-json": {
    id: "mock-session-shell-json",
    source: "claude",
    cwd: "/mock/projects/alpha",
    created_at: "2025-01-14T10:00:00Z",
    updated_at: "2025-01-14T12:00:00Z",
    metadata: {
      model: "claude-3-opus",
      total_tokens: 1500,
      title: "终端命令 JSON 格式测试",
      original_path: "/mock/logs/shell-json-test.json",
    },
    messages: [
      createMockMessage("user", "列出当前目录", "2025-01-14T10:00:00Z"),
      createMockMessage("assistant", "好的，我来列出目录。", "2025-01-14T10:01:00Z", [
        { type: "text", text: "执行 ls 命令：" },
        {
          type: "tool_use",
          id: "tool-shell-json-1",
          name: "Bash",
          input: {
            command: "ls -la",
          },
          standard_tool: {
            type: "shell_exec",
            command: "ls -la",
            cwd: "/mock/projects/alpha",
          } as StandardToolShellExec,
        },
      ]),
      createMockMessage("assistant", "目录列表如下。", "2025-01-14T10:02:00Z", [
        {
          type: "tool_result",
          tool_use_id: "tool-shell-json-1",
          // 模拟真实后端返回的 JSON 格式 content (无 structured_result)
          content: JSON.stringify({
            output: "总计 48512\ndrwxr-xr-x 16 decker decker 4096 10月 13 13:22 .\ndrwxrwxr-x  7 decker decker 4096 10月 13 08:17 ..\n-rw-r--r--  1 decker decker  236 10月 12 11:00 README.md\ndrwxr-xr-x  3 decker decker 4096 10月 13 11:00 src",
            metadata: { exit_code: 0, duration_seconds: 0.2 },
          }),
          is_error: false,
          // 注意：没有 structured_result，测试 JSON 解析回退逻辑
        },
        { type: "text", text: "以上是目录内容。" },
      ]),
      createMockMessage("user", "运行一个会失败的命令", "2025-01-14T10:05:00Z"),
      createMockMessage("assistant", "好的，执行命令。", "2025-01-14T10:06:00Z", [
        { type: "text", text: "执行命令：" },
        {
          type: "tool_use",
          id: "tool-shell-json-2",
          name: "Bash",
          input: {
            command: "cat nonexistent.txt",
          },
          standard_tool: {
            type: "shell_exec",
            command: "cat nonexistent.txt",
            cwd: "/mock/projects/alpha",
          } as StandardToolShellExec,
        },
      ]),
      createMockMessage("assistant", "命令执行失败。", "2025-01-14T10:07:00Z", [
        {
          type: "tool_result",
          tool_use_id: "tool-shell-json-2",
          // 模拟失败命令的 JSON 格式 content
          content: JSON.stringify({
            output: "cat: nonexistent.txt: No such file or directory",
            metadata: { exit_code: 1, duration_seconds: 0.1 },
          }),
          is_error: true,
        },
        { type: "text", text: "文件不存在。" },
      ]),
    ],
  },
};

// =============================================================================
// Mock Git Data
// =============================================================================

export const MOCK_SNAPSHOT: SnapshotResult = {
  content: `// Mock file content
export function hello() {
  console.log("Hello, World!");
}
`,
  commit_hash: "abc1234def5678",
  commit_message: "feat: add hello function",
  commit_timestamp: 1704067200, // 2024-01-01 00:00:00 UTC
};

// Dart 代码用于语法高亮测试
export const MOCK_DART_CODE = `class User {
  final String name;
  final int age;

  User({required this.name, required this.age});

  void greet() {
    print('Hello, I am \$name and I am \$age years old.');
  }
}`;

// =============================================================================
// Mock Sanitization Data
// =============================================================================

export const MOCK_BUILTIN_RULES: SanitizationRule[] = [
  {
    id: "mock_api_key",
    name: "API Key",
    pattern: "\\b[A-Za-z0-9]{32,}\\b",
    replacement: "[REDACTED:API_KEY]",
    sensitive_type: "api_key",
    severity: "critical",
    enabled: true,
  },
  {
    id: "mock_email",
    name: "Email",
    pattern: "\\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Z|a-z]{2,}\\b",
    replacement: "[REDACTED:EMAIL]",
    sensitive_type: "email",
    severity: "warning",
    enabled: true,
  },
  {
    id: "mock_ip_address",
    name: "IP Address",
    pattern: "\\b\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\b",
    replacement: "[REDACTED:IP]",
    sensitive_type: "ip_address",
    severity: "info",
    enabled: true,
  },
];

// =============================================================================
// Mock Import Data
// =============================================================================

export const MOCK_DEFAULT_PATHS: DefaultPaths = {
  claude: "/mock/home/.claude/logs",
  gemini: "/mock/home/.gemini/logs",
  cursor: "/mock/home/.cursor/logs",
  codex: "/mock/home/.codex/logs",
};

export const MOCK_DISCOVERED_FILES = [
  {
    path: "/mock/logs/session1.json",
    source: "claude" as const,
    size: 1024,
    modified: "2025-01-10T00:00:00Z",
    sessionId: "discovered-session-1",
    isImported: false,
  },
  {
    path: "/mock/logs/session2.json",
    source: "gemini" as const,
    size: 2048,
    modified: "2025-01-09T00:00:00Z",
    sessionId: "discovered-session-2",
    isImported: true,
  },
];

// =============================================================================
// Mock Search Data
// =============================================================================

export const MOCK_SEARCH_RESULTS = [
  {
    id: "search-1",
    session_id: "mock-session-alpha-1",
    project_id: "mock-project-alpha",
    project_name: "Mock Project Alpha",
    session_name: "实现用户认证模块",
    message_id: "msg-1",
    content: "帮我实现一个用户认证模块",
    match_positions: [[4, 10]] as [number, number][],
    timestamp: 1704880800,
  },
];

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * 根据项目 ID 获取会话列表
 */
export function getSessionsByProjectId(projectId: string): SessionSummary[] {
  return MOCK_PROJECT_SESSIONS_MAP[projectId] ?? [];
}

/**
 * 根据会话 ID 获取完整会话
 */
export function getSessionById(sessionId: string): MantraSession | null {
  return MOCK_SESSIONS[sessionId] ?? null;
}

/**
 * 根据会话 ID 获取所属项目
 */
export function getProjectBySessionId(sessionId: string): Project | null {
  if (sessionId.includes("alpha") || sessionId === "mock-session-file-edit" || sessionId === "mock-session-dart-highlight" || sessionId === "mock-session-shell-exec" || sessionId === "mock-session-shell-json") {
    return MOCK_PROJECTS.find((p) => p.id === "mock-project-alpha") ?? null;
  }
  if (sessionId.includes("beta")) {
    return MOCK_PROJECTS.find((p) => p.id === "mock-project-beta") ?? null;
  }
  if (sessionId.includes("gamma")) {
    return MOCK_PROJECTS.find((p) => p.id === "mock-project-gamma") ?? null;
  }
  return null;
}
