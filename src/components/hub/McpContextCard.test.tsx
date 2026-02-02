/**
 * McpContextCard 组件测试
 * Story 11.9: Task 12.4 - 前端组件测试
 *
 * 测试三种状态：已接管、可接管、空状态
 * 以及 Phase 2 的 PolicyBadge 和管理工具入口
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { McpContextCard } from "./McpContextCard";

// Mock IPC adapter
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: vi.fn(),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback: string, opts?: Record<string, unknown>) => {
      if (opts?.count !== undefined) return `${opts.count} ${fallback}`;
      return fallback || key;
    },
  }),
}));

// Mock child components that are heavy/complex
vi.mock("./McpConfigImportDialog", () => ({
  McpConfigImportDialog: ({ open }: { open: boolean }) =>
    open ? <div data-testid="import-dialog">Import Dialog</div> : null,
}));

vi.mock("./ToolPolicyEditor", () => ({
  ToolPolicyEditor: ({
    serviceId,
    onSaved,
  }: {
    serviceId: string;
    onSaved?: () => void;
  }) => (
    <div data-testid={`tool-policy-editor-${serviceId}`}>
      <button onClick={onSaved} data-testid="save-policy">
        Save
      </button>
    </div>
  ),
}));

// Mock SourceIcon
vi.mock("@/components/import/SourceIcons", () => ({
  SourceIcon: ({ source }: { source: string }) => (
    <span data-testid={`source-icon-${source}`}>{source}</span>
  ),
}));

// Import after mocking
import { invoke } from "@/lib/ipc-adapter";
const mockInvokeFn = vi.mocked(invoke);

// ===== Test Data =====

const takenOverStatus = {
  is_taken_over: true,
  associated_services: [
    {
      id: "svc-1",
      name: "git-mcp",
      adapter_id: "claude",
      is_running: true,
      error_message: null,
      tool_policy_mode: null,
      custom_tools_count: null,
    },
    {
      id: "svc-2",
      name: "postgres",
      adapter_id: "cursor",
      is_running: false,
      error_message: null,
      tool_policy_mode: "deny_all",
      custom_tools_count: null,
    },
    {
      id: "svc-3",
      name: "context7",
      adapter_id: "codex",
      is_running: false,
      error_message: "Connection refused",
      tool_policy_mode: "custom",
      custom_tools_count: 3,
    },
  ],
  detectable_configs: [],
};

const detectableStatus = {
  is_taken_over: false,
  associated_services: [],
  detectable_configs: [
    {
      adapter_id: "claude",
      config_path: "/project/.mcp.json",
      scope: "project",
      service_count: 2,
    },
    {
      adapter_id: "cursor",
      config_path: "/project/.cursor/mcp.json",
      scope: "project",
      service_count: 1,
    },
  ],
};

const emptyStatus = {
  is_taken_over: false,
  associated_services: [],
  detectable_configs: [],
};

describe("McpContextCard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ===== 加载状态 =====

  describe("加载状态", () => {
    it("加载中显示 spinner", () => {
      mockInvokeFn.mockReturnValue(new Promise(() => {})); // never resolves

      render(<McpContextCard projectId="proj-1" />);

      expect(
        screen.getByTestId("mcp-context-card-loading")
      ).toBeInTheDocument();
    });

    it("调用 check_project_mcp_status", async () => {
      mockInvokeFn.mockResolvedValue(emptyStatus);

      render(<McpContextCard projectId="proj-1" projectPath="/my/project" />);

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith(
          "check_project_mcp_status",
          {
            projectId: "proj-1",
            projectPath: "/my/project",
          }
        );
      });
    });
  });

  // ===== 已接管状态 (AC1, AC3, AC4) =====

  describe("已接管状态", () => {
    beforeEach(() => {
      mockInvokeFn.mockResolvedValue(takenOverStatus);
    });

    it("渲染 mcp-context-card", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(screen.getByTestId("mcp-context-card")).toBeInTheDocument();
      });
    });

    it("显示服务列表", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(screen.getByTestId("mcp-service-svc-1")).toBeInTheDocument();
        expect(screen.getByTestId("mcp-service-svc-2")).toBeInTheDocument();
        expect(screen.getByTestId("mcp-service-svc-3")).toBeInTheDocument();
      });
    });

    it("显示服务名称", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(screen.getByText("git-mcp")).toBeInTheDocument();
        expect(screen.getByText("postgres")).toBeInTheDocument();
        expect(screen.getByText("context7")).toBeInTheDocument();
      });
    });

    it("显示适配器图标", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("source-icon-claude")
        ).toBeInTheDocument();
        expect(
          screen.getByTestId("source-icon-cursor")
        ).toBeInTheDocument();
        expect(
          screen.getByTestId("source-icon-codex")
        ).toBeInTheDocument();
      });
    });

    it("显示 Manage Services 按钮", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-manage-services-button")
        ).toBeInTheDocument();
      });
    });

    it("点击 Manage Services 调用 onNavigateToHub", async () => {
      const onNavigateToHub = vi.fn();
      const user = userEvent.setup();

      render(
        <McpContextCard
          projectId="proj-1"
          onNavigateToHub={onNavigateToHub}
        />
      );

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-manage-services-button")
        ).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("mcp-manage-services-button"));
      expect(onNavigateToHub).toHaveBeenCalledWith("proj-1");
    });

    // ===== Phase 2: PolicyBadge =====

    it("deny_all 服务显示 PolicyBadge", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-policy-badge-svc-2")
        ).toBeInTheDocument();
      });

      expect(screen.getByTestId("mcp-policy-badge-svc-2")).toHaveTextContent(
        "All Denied"
      );
    });

    it("custom 服务显示 PolicyBadge 含工具数量", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-policy-badge-svc-3")
        ).toBeInTheDocument();
      });
    });

    it("allow_all 或 null 服务不显示 PolicyBadge", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(screen.getByTestId("mcp-context-card")).toBeInTheDocument();
      });

      expect(
        screen.queryByTestId("mcp-policy-badge-svc-1")
      ).not.toBeInTheDocument();
    });

    // ===== Phase 2: 管理工具按钮 =====

    it("每个服务有管理工具按钮", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-manage-tools-svc-1")
        ).toBeInTheDocument();
        expect(
          screen.getByTestId("mcp-manage-tools-svc-2")
        ).toBeInTheDocument();
        expect(
          screen.getByTestId("mcp-manage-tools-svc-3")
        ).toBeInTheDocument();
      });
    });

    it("点击管理工具按钮打开 ToolPolicyEditor Dialog", async () => {
      const user = userEvent.setup();

      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-manage-tools-svc-1")
        ).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("mcp-manage-tools-svc-1"));

      await waitFor(() => {
        expect(
          screen.getByTestId("tool-policy-editor-svc-1")
        ).toBeInTheDocument();
      });
    });
  });

  // ===== 可接管状态 (AC2) =====

  describe("可接管状态", () => {
    beforeEach(() => {
      mockInvokeFn.mockResolvedValue(detectableStatus);
    });

    it("渲染 takeover 卡片", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-context-card-takeover")
        ).toBeInTheDocument();
      });
    });

    it("显示检测到的配置数量", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        // 2 + 1 = 3 configs
        expect(screen.getByText("3")).toBeInTheDocument();
      });
    });

    it("显示适配器图标", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(screen.getByText("Claude")).toBeInTheDocument();
        expect(screen.getByText("Cursor")).toBeInTheDocument();
      });
    });

    it("显示 Import & Takeover 按钮", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-import-takeover-button")
        ).toBeInTheDocument();
      });
    });

    it("点击 Import & Takeover 打开导入对话框", async () => {
      const user = userEvent.setup();

      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-import-takeover-button")
        ).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("mcp-import-takeover-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-dialog")).toBeInTheDocument();
      });
    });
  });

  // ===== 空状态 (AC5) =====

  describe("空状态", () => {
    beforeEach(() => {
      mockInvokeFn.mockResolvedValue(emptyStatus);
    });

    it("渲染 empty 卡片", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-context-card-empty")
        ).toBeInTheDocument();
      });
    });

    it("显示空状态提示文字", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByText("No MCP services configured")
        ).toBeInTheDocument();
      });
    });

    it("显示 Add Services 按钮", async () => {
      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-add-services-button")
        ).toBeInTheDocument();
      });
    });

    it("点击 Add Services 调用 onNavigateToHub", async () => {
      const onNavigateToHub = vi.fn();
      const user = userEvent.setup();

      render(
        <McpContextCard
          projectId="proj-1"
          onNavigateToHub={onNavigateToHub}
        />
      );

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-add-services-button")
        ).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("mcp-add-services-button"));
      expect(onNavigateToHub).toHaveBeenCalledWith("proj-1");
    });
  });

  // ===== 错误处理 =====

  describe("错误处理", () => {
    it("API 调用失败时显示空状态", async () => {
      mockInvokeFn.mockRejectedValue(new Error("Network error"));

      render(<McpContextCard projectId="proj-1" />);

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-context-card-empty")
        ).toBeInTheDocument();
      });
    });
  });
});
