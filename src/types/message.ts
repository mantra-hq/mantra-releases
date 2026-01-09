/**
 * Message Types - 消息类型定义
 * Story 2.3: Task 2
 *
 * 定义 NarrativeStream 对话流中使用的消息类型
 */

/**
 * 消息角色
 * - user: 用户消息
 * - assistant: AI 助手消息
 */
export type MessageRole = "user" | "assistant";

/**
 * 内容块类型
 * - text: 纯文本内容
 * - thinking: AI 思考过程 (Chain of Thought)
 * - tool_use: 工具调用请求
 * - tool_result: 工具执行结果
 * - code_diff: 代码差异
 * - image: 图片内容
 * - reference: 代码引用
 * - code_suggestion: 代码建议 (Story 8.10)
 */
export type ContentBlockType =
  | "text"
  | "thinking"
  | "tool_use"
  | "tool_result"
  | "code_diff"
  | "image"
  | "reference"
  | "code_suggestion";

/**
 * 会话来源常量
 */
export const SessionSources = {
  CLAUDE: "claude",
  GEMINI: "gemini",
  CURSOR: "cursor",
  COPILOT: "copilot",
  AIDER: "aider",
  CODEX: "codex",
  UNKNOWN: "unknown",
} as const;

export type SessionSource = (typeof SessionSources)[keyof typeof SessionSources] | string;

/**
 * 内容块接口
 * 消息由一个或多个内容块组成
 */
export interface ContentBlock {
  /** 内容块类型 */
  type: ContentBlockType;
  /** 内容文本 */
  content: string;
  /** 工具名称 (tool_use 专用) */
  toolName?: string;
  /** 工具输入参数 (tool_use 专用) */
  toolInput?: Record<string, unknown>;
  /** 工具调用 ID (tool_use / tool_result 共用，用于配对) */
  toolUseId?: string;
  /** 统一配对 ID (用于关联 tool_use 和 tool_result) */
  correlationId?: string;
  /** 是否为错误结果 (tool_result 专用) */
  isError?: boolean;
  /** 关联的文件路径 (tool_result 专用，从对应的 tool_use 继承) */
  associatedFilePath?: string;
  /** 关联的工具名称 (tool_result 专用，从对应的 tool_use 继承) */
  associatedToolName?: string;
  /** 文件路径 (code_diff / reference / code_suggestion 专用) */
  filePath?: string;
  /** 代码差异内容 (code_diff 专用) */
  diff?: string;
  /** 编程语言 (code_diff / code_suggestion 专用) */
  language?: string;
  /** 图片源 (image 专用) */
  source?: string;
  /** 媒体类型 (image 专用) */
  mediaType?: string;
  /** 起始行 (reference 专用) */
  startLine?: number;
  /** 结束行 (reference 专用) */
  endLine?: number;
  /** 符号名称 (reference 专用) */
  symbol?: string;

  // === Story 8.10: 新增字段 ===

  /** 标准化工具类型 (tool_use 专用) */
  standardTool?: StandardTool;
  /** 工具显示名称 (tool_use 专用, Gemini) */
  displayName?: string;
  /** 工具描述 (tool_use 专用, Gemini) */
  description?: string;
  /** 结构化工具结果 (tool_result 专用) */
  structuredResult?: ToolResultData;
  /** 显示内容 (tool_result 专用, Gemini resultDisplay) */
  displayContent?: string;
  /** 是否渲染为 Markdown (tool_result 专用, Gemini) */
  renderAsMarkdown?: boolean;
  /** 用户决策 (tool_result 专用, Cursor approved/rejected) */
  userDecision?: string;
  /** 思考主题 (thinking 专用, Gemini) */
  subject?: string;
  /**
   * 思考时间戳 (thinking 专用, Gemini)
   * 注意: 后端序列化字段名为 "timestamp"，前端使用 "thinkingTimestamp" 避免与消息级别 timestamp 冲突。
   * 数据转换层需要处理 timestamp -> thinkingTimestamp 的映射。
   */
  thinkingTimestamp?: string;
  /** 代码内容 (code_suggestion 专用, Cursor) */
  code?: string;
}

/**
 * 对话消息接口
 * 表示对话流中的单条消息
 */
export interface NarrativeMessage {
  /** 消息唯一标识 */
  id: string;
  /** 消息角色 */
  role: MessageRole;
  /** 消息时间戳 (ISO 8601 格式) */
  timestamp: string;
  /** 消息内容块列表 */
  content: ContentBlock[];
  // === Story 8.10: 新增字段 ===
  /** 消息唯一 ID (用于消息树结构) */
  messageId?: string;
  /** 父消息 ID (用于消息树结构) */
  parentId?: string;
  /** 是否为分支对话 */
  isSidechain?: boolean;
  /** 来源特定元数据透传 */
  sourceMetadata?: Record<string, unknown>;
}

// === Story 8.10: 新增类型定义 ===

/**
 * Git 仓库信息
 * Story 8.10: 对应后端 GitInfo 结构
 */
export interface GitInfo {
  /** Git 分支名 */
  branch?: string;
  /** Git commit hash */
  commit?: string;
  /** Git 仓库 URL */
  repositoryUrl?: string;
}

/**
 * Token 使用统计
 * Story 8.10: 对应后端 TokensBreakdown 结构
 *
 * 注意: 后端使用 Rust u64 类型，前端使用 JavaScript number 类型。
 * JavaScript number 是 64 位浮点数，对于大于 2^53-1 (9,007,199,254,740,991) 的整数会丢失精度。
 * 实际使用中 token 数量不会达到此限制，因此使用 number 类型是安全的。
 */
export interface TokensBreakdown {
  /** 输入 tokens (后端: u64) */
  input?: number;
  /** 输出 tokens (后端: u64) */
  output?: number;
  /** 缓存 tokens (后端: u64) */
  cached?: number;
  /** 思考过程 tokens (后端: u64) */
  thoughts?: number;
  /** 工具调用 tokens (后端: u64) */
  tool?: number;
}

/**
 * 标准化工具类型 - 文件读取
 * Story 8.10: 对应后端 StandardTool::FileRead
 */
export interface StandardToolFileRead {
  type: "file_read";
  path: string;
  startLine?: number;
  endLine?: number;
}

/**
 * 标准化工具类型 - 文件写入
 * Story 8.10: 对应后端 StandardTool::FileWrite
 */
export interface StandardToolFileWrite {
  type: "file_write";
  path: string;
  content: string;
}

/**
 * 标准化工具类型 - 文件编辑
 * Story 8.10: 对应后端 StandardTool::FileEdit
 */
export interface StandardToolFileEdit {
  type: "file_edit";
  path: string;
  oldString?: string;
  newString?: string;
}

/**
 * 标准化工具类型 - Shell 命令执行
 * Story 8.10: 对应后端 StandardTool::ShellExec
 */
export interface StandardToolShellExec {
  type: "shell_exec";
  command: string;
  cwd?: string;
}

/**
 * 标准化工具类型 - 文件搜索
 * Story 8.10: 对应后端 StandardTool::FileSearch
 */
export interface StandardToolFileSearch {
  type: "file_search";
  pattern: string;
  path?: string;
}

/**
 * 标准化工具类型 - 内容搜索
 * Story 8.10: 对应后端 StandardTool::ContentSearch
 */
export interface StandardToolContentSearch {
  type: "content_search";
  pattern: string;
  path?: string;
}

/**
 * 标准化工具类型 - 其他工具
 * Story 8.10: 对应后端 StandardTool::Other
 * Story 8.13: 重命名为 Unknown，应趋近于零
 * @deprecated 使用 StandardToolUnknown 代替
 */
export interface StandardToolOther {
  type: "other";
  name: string;
  input: Record<string, unknown>;
}

/**
 * 标准化工具类型 - 未知工具
 * Story 8.13: 对应后端 StandardTool::Unknown (原 Other)
 * 如果生产环境中出现大量 Unknown，说明需要扩展 StandardTool
 */
export interface StandardToolUnknown {
  type: "unknown";
  name: string;
  input: Record<string, unknown>;
}

// === Story 8.13: 新增工具类型 ===

/**
 * 标准化工具类型 - Web 获取
 * Story 8.13: 对应后端 StandardTool::WebFetch
 */
export interface StandardToolWebFetch {
  type: "web_fetch";
  url: string;
  prompt?: string;
}

/**
 * 标准化工具类型 - Web 搜索
 * Story 8.13: 对应后端 StandardTool::WebSearch
 */
export interface StandardToolWebSearch {
  type: "web_search";
  query: string;
}

/**
 * 标准化工具类型 - 知识查询
 * Story 8.13: 对应后端 StandardTool::KnowledgeQuery
 */
export interface StandardToolKnowledgeQuery {
  type: "knowledge_query";
  repo?: string;
  question: string;
}

/**
 * 标准化工具类型 - 代码执行
 * Story 8.13: 对应后端 StandardTool::CodeExec
 */
export interface StandardToolCodeExec {
  type: "code_exec";
  code: string;
  language?: string;
}

/**
 * 标准化工具类型 - 诊断
 * Story 8.13: 对应后端 StandardTool::Diagnostic
 */
export interface StandardToolDiagnostic {
  type: "diagnostic";
  uri?: string;
}

/**
 * 标准化工具类型 - 笔记本编辑
 * Story 8.13: 对应后端 StandardTool::NotebookEdit
 */
export interface StandardToolNotebookEdit {
  type: "notebook_edit";
  notebookPath: string;
  cellId?: string;
  newSource: string;
}

/**
 * 标准化工具类型 - 任务管理
 * Story 8.13: 对应后端 StandardTool::TodoManage
 */
export interface StandardToolTodoManage {
  type: "todo_manage";
  todos: Record<string, unknown>;
}

/**
 * 标准化工具类型 - 子任务/代理
 * Story 8.13: 对应后端 StandardTool::SubTask
 */
export interface StandardToolSubTask {
  type: "sub_task";
  prompt: string;
  agentType?: string;
}

/**
 * 标准化工具类型 - 用户问询
 * Story 8.13: 对应后端 StandardTool::UserPrompt
 */
export interface StandardToolUserPrompt {
  type: "user_prompt";
  question?: string;
  options?: Record<string, unknown>;
}

/**
 * 标准化工具类型 - 计划模式
 * Story 8.13: 对应后端 StandardTool::PlanMode
 */
export interface StandardToolPlanMode {
  type: "plan_mode";
  entering: boolean;
}

/**
 * 标准化工具类型 - 技能调用
 * Story 8.13: 对应后端 StandardTool::SkillInvoke
 */
export interface StandardToolSkillInvoke {
  type: "skill_invoke";
  skill: string;
  args?: string;
}

/**
 * 标准化工具类型 (Discriminated Union)
 * Story 8.10: 对应后端 StandardTool 枚举
 * Story 8.13: 扩展完整应用级概念，添加 Unknown
 */
export type StandardTool =
  | StandardToolFileRead
  | StandardToolFileWrite
  | StandardToolFileEdit
  | StandardToolShellExec
  | StandardToolFileSearch
  | StandardToolContentSearch
  | StandardToolWebFetch
  | StandardToolWebSearch
  | StandardToolKnowledgeQuery
  | StandardToolCodeExec
  | StandardToolDiagnostic
  | StandardToolNotebookEdit
  | StandardToolTodoManage
  | StandardToolSubTask
  | StandardToolUserPrompt
  | StandardToolPlanMode
  | StandardToolSkillInvoke
  | StandardToolUnknown
  | StandardToolOther; // 保留向后兼容

/**
 * 工具结果数据 - 文件读取
 * Story 8.10: 对应后端 ToolResultData::FileRead
 */
export interface ToolResultDataFileRead {
  type: "file_read";
  filePath: string;
  startLine?: number;
  numLines?: number;
  totalLines?: number;
}

/**
 * 工具结果数据 - 文件写入
 * Story 8.10: 对应后端 ToolResultData::FileWrite
 */
export interface ToolResultDataFileWrite {
  type: "file_write";
  filePath: string;
}

/**
 * 工具结果数据 - 文件编辑
 * Story 8.10: 对应后端 ToolResultData::FileEdit
 */
export interface ToolResultDataFileEdit {
  type: "file_edit";
  filePath: string;
  oldString?: string;
  newString?: string;
}

/**
 * 工具结果数据 - Shell 命令执行
 * Story 8.10: 对应后端 ToolResultData::ShellExec
 */
export interface ToolResultDataShellExec {
  type: "shell_exec";
  exitCode?: number;
  stdout?: string;
  stderr?: string;
}

/**
 * 工具结果数据 - 其他结果
 * Story 8.10: 对应后端 ToolResultData::Other
 */
export interface ToolResultDataOther {
  type: "other";
  data: Record<string, unknown>;
}

/**
 * 结构化工具结果 (Discriminated Union)
 * Story 8.10: 对应后端 ToolResultData 枚举
 */
export type ToolResultData =
  | ToolResultDataFileRead
  | ToolResultDataFileWrite
  | ToolResultDataFileEdit
  | ToolResultDataShellExec
  | ToolResultDataOther;

/**
 * 会话元数据
 * Story 8.10: 对应后端 SessionMetadata 结构
 */
export interface SessionMetadata {
  /** 使用的模型名称 */
  model?: string;
  /** 总 token 数 */
  totalTokens?: number;
  /** 会话标题 */
  title?: string;
  /** 原始文件路径 */
  originalPath?: string;
  /** Git 仓库信息 */
  git?: GitInfo;
  /** Token 使用细分 */
  tokensBreakdown?: TokensBreakdown;
  /** 系统指令 (Codex) */
  instructions?: string;
  /** 来源特定元数据透传 */
  sourceMetadata?: Record<string, unknown>;
}

