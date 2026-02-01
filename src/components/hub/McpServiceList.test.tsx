/**
 * McpServiceList 组件测试
 * Story 11.6: Task 9.2 - McpServiceList 组件测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { McpServiceList, type McpService } from "./McpServiceList";

// Mock IPC adapter
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: vi.fn(),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (params?.name) return `${key}: ${params.name}`;
      return key;
    },
  }),
}));

// Mock feedback
vi.mock("@/lib/feedback", () => ({
  feedback: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Import after mocking
import { invoke } from "@/lib/ipc-adapter";
import { feedback } from "@/lib/feedback";

const mockInvokeFn = vi.mocked(invoke);

const mockServices: McpService[] = [
  {
    id: "service-1",
    name: "git-mcp",
    transport_type: "stdio",
    command: "npx",
    args: ["--yes", "@anthropic/mcp-server-git"],
    env: null,
    url: null,
    headers: null,
    source: "imported",
    source_file: "/path/to/.mcp.json",
    enabled: true,
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
  },
  {
    id: "service-2",
    name: "postgres",
    transport_type: "stdio",
    command: "npx",
    args: ["--yes", "@anthropic/mcp-server-postgres"],
    env: { DATABASE_URL: "$DATABASE_URL" },
    url: null,
    headers: null,
    source: "manual",
    source_file: null,
    enabled: false,
    created_at: "2024-01-02T00:00:00Z",
    updated_at: "2024-01-02T00:00:00Z",
  },
];

describe("McpServiceList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvokeFn.mockResolvedValue(mockServices);
  });

  describe("列表渲染", () => {
    it("应该渲染服务列表组件", async () => {
      render(<McpServiceList />);

      // 组件应该渲染
      expect(screen.getByTestId("mcp-service-list")).toBeInTheDocument();
    });

    it("应该调用 list_mcp_services", async () => {
      render(<McpServiceList />);

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("list_mcp_services");
      });
    });

    it("空列表时应该显示提示信息", async () => {
      mockInvokeFn.mockResolvedValue([]);

      render(<McpServiceList />);

      await waitFor(() => {
        expect(screen.getByText("hub.services.empty")).toBeInTheDocument();
      });
    });

    it("应该显示添加服务按钮", async () => {
      render(<McpServiceList />);

      // 添加按钮始终可见
      expect(screen.getByTestId("mcp-service-add-button")).toBeInTheDocument();
    });
  });

  describe("添加服务", () => {
    it("点击添加按钮应该打开表单对话框", async () => {
      const user = userEvent.setup();

      render(<McpServiceList />);

      await user.click(screen.getByTestId("mcp-service-add-button"));

      await waitFor(() => {
        expect(screen.getByText("hub.services.form.addTitle")).toBeInTheDocument();
      });
    });
  });

  describe("错误处理", () => {
    it("加载失败时应该显示错误提示", async () => {
      mockInvokeFn.mockRejectedValue(new Error("Failed to load"));

      render(<McpServiceList />);

      await waitFor(() => {
        expect(feedback.error).toHaveBeenCalled();
      });
    });
  });
});
