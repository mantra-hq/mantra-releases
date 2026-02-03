/**
 * McpContextSection 组件测试
 * Story 11.18: AC2 - 项目视角入口
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { McpContextSection } from "./McpContextSection";
import { TooltipProvider } from "@/components/ui/tooltip";

// Mock IPC adapter
const mockInvokeFn = vi.fn();
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: (...args: unknown[]) => mockInvokeFn(...args),
}));

// Mock feedback
vi.mock("@/lib/feedback", () => ({
  feedback: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock ToolPolicyEditor
vi.mock("@/components/hub/ToolPolicyEditor", () => ({
  ToolPolicyEditor: ({ serviceId, onSaved }: { serviceId: string; onSaved?: () => void }) => (
    <div data-testid={`tool-policy-editor-${serviceId}`}>
      <button onClick={onSaved} data-testid="mock-save-button">
        Mock Save
      </button>
    </div>
  ),
}));

// Mock McpConfigImportSheet
vi.mock("@/components/hub/McpConfigImportSheet", () => ({
  McpConfigImportSheet: () => <div data-testid="mock-import-sheet" />,
}));

// Helper to render with providers
function renderWithProviders(ui: React.ReactElement) {
  return render(<TooltipProvider>{ui}</TooltipProvider>);
}

// ===== 测试数据 =====

const emptyStatus = {
  is_taken_over: false,
  associated_services: [],
  detectable_configs: [],
};

const withServicesStatus = {
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
  ],
};

const allServices = [
  { id: "svc-1", name: "git-mcp", source_file: "/home/.claude.json" },
  { id: "svc-2", name: "postgres", source_file: "/home/.cursor/mcp.json" },
  { id: "svc-3", name: "deepwiki", source_file: "/project/.mcp.json" },
];

describe("McpContextSection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ===== 加载状态 =====

  describe("加载状态", () => {
    it("加载中显示 spinner", () => {
      mockInvokeFn.mockReturnValue(new Promise(() => {})); // never resolves

      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      expect(
        screen.getByTestId("mcp-context-section-loading")
      ).toBeInTheDocument();
    });

    it("调用 check_project_mcp_status", async () => {
      mockInvokeFn.mockResolvedValue(emptyStatus);

      renderWithProviders(
        <McpContextSection
          projectId="proj-1"
          projectPath="/my/project"
        />
      );

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

  // ===== 已关联服务 (AC2) =====

  describe("已关联服务", () => {
    beforeEach(() => {
      mockInvokeFn.mockImplementation((cmd: string) => {
        if (cmd === "check_project_mcp_status") {
          return Promise.resolve(withServicesStatus);
        }
        if (cmd === "list_mcp_services") {
          return Promise.resolve(allServices);
        }
        return Promise.resolve(null);
      });
    });

    it("显示已关联服务列表", async () => {
      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        expect(screen.getByTestId("mcp-context-section")).toBeInTheDocument();
      });

      expect(screen.getByText("git-mcp")).toBeInTheDocument();
      expect(screen.getByText("postgres")).toBeInTheDocument();
    });

    it("显示服务数量徽标", async () => {
      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        // 2 个服务中 1 个 running，中文显示 "1 个服务运行中" 或英文 "1 Active"
        const badge = screen.getByText(/1.*Active|1.*运行中|Active.*1|运行中.*1/i);
        expect(badge).toBeInTheDocument();
      });
    });

    it("custom 模式服务显示 PolicyBadge", async () => {
      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        // svc-2 是 custom 模式，custom_tools_count = 3
        // 中文显示 "自定义 3" 或英文 "Custom 3"
        expect(screen.getByText(/Custom.*3|3.*Custom|自定义.*3|3.*自定义/i)).toBeInTheDocument();
      });
    });

    it("每个服务有工具策略管理按钮", async () => {
      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-manage-tools-svc-1")
        ).toBeInTheDocument();
        expect(
          screen.getByTestId("mcp-manage-tools-svc-2")
        ).toBeInTheDocument();
      });
    });

    it("点击工具策略按钮打开 ToolPolicyEditor", async () => {
      const user = userEvent.setup();

      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-manage-tools-svc-1")
        ).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("mcp-manage-tools-svc-1"));

      await waitFor(() => {
        expect(
          screen.getByTestId("tool-policy-sheet")
        ).toBeInTheDocument();
      });
    });
  });

  // ===== [+ 关联更多服务] 功能 =====

  describe("关联更多服务", () => {
    beforeEach(() => {
      mockInvokeFn.mockImplementation((cmd: string) => {
        if (cmd === "check_project_mcp_status") {
          return Promise.resolve(withServicesStatus);
        }
        if (cmd === "list_mcp_services") {
          return Promise.resolve(allServices);
        }
        if (cmd === "link_mcp_service_to_project") {
          return Promise.resolve(null);
        }
        if (cmd === "unlink_mcp_service_from_project") {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });
    });

    it("显示 [+ 关联更多服务] 按钮", async () => {
      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-link-services-trigger")
        ).toBeInTheDocument();
      });
    });

    it("展开后显示所有可用服务", async () => {
      const user = userEvent.setup();

      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-link-services-trigger")
        ).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("mcp-link-services-trigger"));

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("list_mcp_services");
      });

      await waitFor(() => {
        expect(
          screen.getByTestId("service-link-item-svc-3")
        ).toBeInTheDocument();
      });
    });

    it("切换服务选择状态", async () => {
      const user = userEvent.setup();

      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-link-services-trigger")
        ).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("mcp-link-services-trigger"));

      await waitFor(() => {
        expect(
          screen.getByTestId("service-link-item-svc-3")
        ).toBeInTheDocument();
      });

      // 点击新服务进行选择
      await user.click(screen.getByTestId("service-link-item-svc-3"));

      // 保存按钮应该启用（有变更）
      await waitFor(() => {
        const saveButton = screen.getByTestId("mcp-save-links-button");
        expect(saveButton).not.toBeDisabled();
      });
    });
  });

  // ===== 可接管状态 =====

  describe("可接管状态", () => {
    beforeEach(() => {
      mockInvokeFn.mockResolvedValue(detectableStatus);
    });

    it("显示可检测配置提示", async () => {
      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        // 检测到配置文件，中文显示 "检测到：" 或英文 "Detected"
        expect(screen.getByText(/Detected|检测到/i)).toBeInTheDocument();
      });
    });

    it("显示导入按钮", async () => {
      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        expect(
          screen.getByTestId("mcp-import-takeover-button")
        ).toBeInTheDocument();
      });
    });
  });

  // ===== 空状态 =====

  describe("空状态", () => {
    beforeEach(() => {
      mockInvokeFn.mockImplementation((cmd: string) => {
        if (cmd === "check_project_mcp_status") {
          return Promise.resolve(emptyStatus);
        }
        if (cmd === "list_mcp_services") {
          return Promise.resolve([]);
        }
        return Promise.resolve(null);
      });
    });

    it("无服务时显示空状态提示", async () => {
      renderWithProviders(
        <McpContextSection projectId="proj-1" />
      );

      await waitFor(() => {
        // 无服务且无可检测配置时显示空状态
        // 组件可能显示中文 "未配置 MCP 服务" 或英文 "No MCP services configured"
        expect(screen.getByText(/未配置|No MCP|Configure MCP/i)).toBeInTheDocument();
      });
    });
  });
});
