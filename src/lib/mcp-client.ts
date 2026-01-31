/**
 * MCP JSON-RPC HTTP 客户端
 * Story 11.11: Task 5 - HTTP Client Integration (AC: 3, 4)
 *
 * 封装与 MCP Gateway 的 JSON-RPC 通信：
 * - Bearer Token 认证
 * - JSON-RPC 2.0 协议
 * - 请求超时处理
 * - 错误响应处理
 */

/**
 * JSON-RPC 2.0 请求格式
 */
export interface JsonRpcRequest {
  jsonrpc: "2.0";
  method: string;
  params: Record<string, unknown>;
  id: number | string;
}

/**
 * JSON-RPC 2.0 成功响应格式
 */
export interface JsonRpcSuccessResponse<T = unknown> {
  jsonrpc: "2.0";
  result: T;
  id: number | string;
}

/**
 * JSON-RPC 2.0 错误对象
 */
export interface JsonRpcError {
  code: number;
  message: string;
  data?: unknown;
}

/**
 * JSON-RPC 2.0 错误响应格式
 */
export interface JsonRpcErrorResponse {
  jsonrpc: "2.0";
  error: JsonRpcError;
  id: number | string | null;
}

/**
 * JSON-RPC 响应类型 (成功或错误)
 */
export type JsonRpcResponse<T = unknown> =
  | JsonRpcSuccessResponse<T>
  | JsonRpcErrorResponse;

/**
 * MCP 工具定义 (来自 tools/list)
 */
export interface McpToolDefinition {
  name: string;
  description?: string;
  inputSchema?: {
    type: "object";
    properties?: Record<string, unknown>;
    required?: string[];
  };
}

/**
 * MCP 资源定义 (来自 resources/list)
 */
export interface McpResourceDefinition {
  uri: string;
  name: string;
  description?: string;
  mimeType?: string;
}

/**
 * tools/list 响应
 */
export interface ToolsListResult {
  tools: McpToolDefinition[];
}

/**
 * resources/list 响应
 */
export interface ResourcesListResult {
  resources: McpResourceDefinition[];
}

/**
 * tools/call 响应 - 内容块
 */
export interface ToolResultContent {
  type: "text" | "image" | "resource";
  text?: string;
  data?: string;
  mimeType?: string;
  uri?: string;
}

/**
 * tools/call 响应
 */
export interface ToolCallResult {
  content: ToolResultContent[];
  isError?: boolean;
}

/**
 * resources/read 响应
 */
export interface ResourceReadResult {
  contents: Array<{
    uri: string;
    mimeType?: string;
    text?: string;
    blob?: string;
  }>;
}

/**
 * MCP 客户端配置
 */
export interface McpClientConfig {
  /** Gateway HTTP 端点 URL (e.g., http://127.0.0.1:3333/message) */
  baseUrl: string;
  /** Bearer Token 用于认证 */
  token: string;
  /** 请求超时时间 (毫秒), 默认 30000 */
  timeout?: number;
}

/**
 * MCP 客户端错误
 */
export class McpClientError extends Error {
  constructor(
    message: string,
    public code: number = -1,
    public data?: unknown
  ) {
    super(message);
    this.name = "McpClientError";
  }

  static fromJsonRpcError(error: JsonRpcError): McpClientError {
    return new McpClientError(error.message, error.code, error.data);
  }
}

/**
 * 判断响应是否为错误响应
 */
export function isJsonRpcError(
  response: JsonRpcResponse
): response is JsonRpcErrorResponse {
  return "error" in response;
}

/**
 * MCP JSON-RPC HTTP 客户端
 */
export class McpClient {
  private baseUrl: string;
  private token: string;
  private timeout: number;
  private requestId: number = 0;

  constructor(config: McpClientConfig) {
    this.baseUrl = config.baseUrl;
    this.token = config.token;
    this.timeout = config.timeout ?? 30000;
  }

  /**
   * 发送 JSON-RPC 请求
   */
  async request<T = unknown>(
    method: string,
    params: Record<string, unknown> = {}
  ): Promise<JsonRpcResponse<T>> {
    const request: JsonRpcRequest = {
      jsonrpc: "2.0",
      method,
      params,
      id: ++this.requestId,
    };

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(this.baseUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${this.token}`,
        },
        body: JSON.stringify(request),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new McpClientError(
          `HTTP ${response.status}: ${response.statusText}`,
          response.status
        );
      }

      const data = (await response.json()) as JsonRpcResponse<T>;
      return data;
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof McpClientError) {
        throw error;
      }

      if (error instanceof DOMException && error.name === "AbortError") {
        throw new McpClientError("Request timeout", -32001);
      }

      throw new McpClientError((error as Error).message);
    }
  }

  /**
   * 发送请求并自动处理错误
   */
  async call<T = unknown>(
    method: string,
    params: Record<string, unknown> = {}
  ): Promise<T> {
    const response = await this.request<T>(method, params);

    if (isJsonRpcError(response)) {
      throw McpClientError.fromJsonRpcError(response.error);
    }

    return response.result;
  }

  /**
   * 列出所有工具
   */
  async listTools(): Promise<McpToolDefinition[]> {
    const result = await this.call<ToolsListResult>("tools/list");
    return result.tools || [];
  }

  /**
   * 列出所有资源
   */
  async listResources(): Promise<McpResourceDefinition[]> {
    const result = await this.call<ResourcesListResult>("resources/list");
    return result.resources || [];
  }

  /**
   * 调用工具
   */
  async callTool(
    name: string,
    args: Record<string, unknown> = {}
  ): Promise<ToolCallResult> {
    return this.call<ToolCallResult>("tools/call", {
      name,
      arguments: args,
    });
  }

  /**
   * 读取资源
   */
  async readResource(uri: string): Promise<ResourceReadResult> {
    return this.call<ResourceReadResult>("resources/read", { uri });
  }
}

/**
 * 创建 MCP 客户端实例
 */
export function createMcpClient(config: McpClientConfig): McpClient {
  return new McpClient(config);
}

/**
 * 标准 JSON-RPC 错误码
 */
export const JSON_RPC_ERROR_CODES = {
  PARSE_ERROR: -32700,
  INVALID_REQUEST: -32600,
  METHOD_NOT_FOUND: -32601,
  INVALID_PARAMS: -32602,
  INTERNAL_ERROR: -32603,
  // 自定义错误码
  TIMEOUT: -32001,
  NETWORK_ERROR: -32002,
} as const;
