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
 */
export type ContentBlockType = "text" | "thinking" | "tool_use" | "tool_result";

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
  /** 是否为错误结果 (tool_result 专用) */
  isError?: boolean;
  /** 关联的文件路径 (tool_result 专用，从对应的 tool_use 继承) */
  associatedFilePath?: string;
  /** 关联的工具名称 (tool_result 专用，从对应的 tool_use 继承) */
  associatedToolName?: string;
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
}

