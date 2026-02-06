/**
 * McpConfigImportSheet Tests
 * Story 11.3: Task 9 - 配置导入前端 UI 测试
 * Story 12.1: Task 2 - Dialog → Sheet 改造测试
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { McpConfigImportSheet } from "./McpConfigImportSheet";

// Mock IPC adapter
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: vi.fn(),
}));

// Mock feedback
vi.mock("@/lib/feedback", () => ({
  feedback: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        "hub.import.title": "导入配置",
        "hub.import.description": "从其他工具导入 MCP 配置",
        "hub.import.scanTitle": "扫描配置文件",
        "hub.import.scanDescription": "检测本地 MCP 配置",
        "hub.import.startScan": "开始扫描",
        "hub.import.scanning": "正在扫描...",
        "hub.import.noConfigsFound": "未找到配置文件",
        "hub.import.previewTitle": "预览导入",
        "hub.import.previewDescription": "选择要导入的服务",
        "hub.import.foundSummary": `找到 ${params?.configs || 0} 个配置文件，${params?.services || 0} 个服务`,
        "hub.import.services": "个服务",
        "hub.import.new": "新增",
        "hub.import.conflict": "冲突",
        "hub.import.actionSkip": "跳过",
        "hub.import.actionConflict": "冲突",
        "hub.import.actionAdd": "新增",
        "hub.import.selectedCount": `已选 ${params?.count || 0} 项`,
        "hub.import.selectAll": "全选",
        "hub.import.selectNone": "取消全选",
        "common.retry": "重试",
        "common.back": "返回",
        "common.next": "下一步",
        "common.close": "关闭",
        "hub.import.linkTitle": "关联到项目",
        "hub.import.linkDescription": `选择要关联到「${params?.project || ""}」的服务：`,
        "hub.import.linkDescriptionGeneric": "选择要关联到当前项目的服务：",
        "hub.import.allLinkedTitle": "所有服务已关联到当前项目",
        "hub.import.alreadyLinked": "已关联",
        "hub.import.linkHint": "关联后，这些服务的工具将对该项目的 AI 会话可用",
        "hub.import.linkSelectedCount": `已选择 ${params?.count || 0} 个服务`,
        "hub.import.linkToProject": "关联到项目",
        "hub.import.linkDone": "完成",
        "hub.import.linkSuccess": `已关联 ${params?.count || 0} 个服务到项目`,
        "hub.import.linkError": "关联服务失败",
        "hub.import.servicesInHub": `${params?.count || 0} 个服务已在 Hub 中`,
        "hub.import.skip": "跳过",
        "hub.import.resultTitle": "导入结果",
        "hub.import.resultDescription": "查看导入结果",
        "hub.import.resultSuccess": "导入成功",
        "hub.import.imported": "已导入",
        "hub.import.confirmTitle": "确认导入",
        "hub.import.confirmDescription": "确认导入",
        "hub.import.confirmImport": "确认导入",
        "hub.import.confirmBack": "返回修改",
        "hub.import.stepLink": "关联",
      };
      return translations[key] || key;
    },
    i18n: { language: "zh-CN" },
  }),
}));

// Radix UI PointerEvent polyfill
beforeAll(() => {
  class MockPointerEvent extends MouseEvent {
    constructor(type: string, props: PointerEventInit = {}) {
      super(type, props);
      Object.assign(this, {
        pointerId: props.pointerId ?? 0,
        width: props.width ?? 1,
        height: props.height ?? 1,
        pressure: props.pressure ?? 0,
        tangentialPressure: props.tangentialPressure ?? 0,
        tiltX: props.tiltX ?? 0,
        tiltY: props.tiltY ?? 0,
        twist: props.twist ?? 0,
        pointerType: props.pointerType ?? "mouse",
        isPrimary: props.isPrimary ?? true,
      });
    }
  }
  window.PointerEvent = MockPointerEvent as unknown as typeof PointerEvent;
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn();
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const defaultProps = {
  open: true,
  onOpenChange: vi.fn(),
  onSuccess: vi.fn(),
};

describe("McpConfigImportSheet", () => {
  describe("Sheet rendering", () => {
    it("renders sheet when open", async () => {
      render(<McpConfigImportSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByRole("dialog")).toBeInTheDocument();
      });
    });

    it("does not render when closed", () => {
      render(<McpConfigImportSheet {...defaultProps} open={false} />);

      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    // Story 12.1: Sheet 特定测试
    it("renders with data-testid for sheet", async () => {
      render(<McpConfigImportSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("mcp-config-import-sheet")).toBeInTheDocument();
      });
    });
  });

  describe("Initial scan step", () => {
    it("shows scan button initially", async () => {
      render(<McpConfigImportSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("import-scan-button")).toBeInTheDocument();
      });
    });

    it("shows scan title and description", async () => {
      render(<McpConfigImportSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByText("扫描配置文件")).toBeInTheDocument();
        expect(screen.getByText("检测本地 MCP 配置")).toBeInTheDocument();
      });
    });
  });

  describe("Scan functionality", () => {
    it("shows loading state when scanning", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;

      // Make the scan hang
      mockInvoke.mockImplementation(() => new Promise(() => {}));

      const user = userEvent.setup();
      render(<McpConfigImportSheet {...defaultProps} />);

      const scanButton = screen.getByTestId("import-scan-button");
      await user.click(scanButton);

      await waitFor(() => {
        expect(screen.getByText("正在扫描...")).toBeInTheDocument();
      });
    });

    it("shows error when scan fails", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;
      mockInvoke.mockRejectedValueOnce(new Error("Scan failed"));

      const user = userEvent.setup();
      render(<McpConfigImportSheet {...defaultProps} />);

      const scanButton = screen.getByTestId("import-scan-button");
      await user.click(scanButton);

      await waitFor(() => {
        expect(screen.getByText("Scan failed")).toBeInTheDocument();
      });
    });

    it("shows no configs message when none found", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;
      mockInvoke.mockResolvedValueOnce({ configs: [] });

      const user = userEvent.setup();
      render(<McpConfigImportSheet {...defaultProps} />);

      const scanButton = screen.getByTestId("import-scan-button");
      await user.click(scanButton);

      await waitFor(() => {
        expect(screen.getByText("未找到配置文件")).toBeInTheDocument();
      });
    });
  });

  describe("Preview step", () => {
    it("shows preview after successful scan", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;

      // Mock scan result
      mockInvoke.mockResolvedValueOnce({
        configs: [
          {
            adapter_id: "claude",
            path: "/home/user/.claude/mcp.json",
            services: [
              {
                name: "test-service",
                command: "npx",
                args: ["-y", "test-mcp"],
                env: null,
                source_file: "/home/user/.claude/mcp.json",
                adapter_id: "claude",
              },
            ],
            parse_errors: [],
          },
        ],
      });

      // Mock preview result
      mockInvoke.mockResolvedValueOnce({
        configs: [
          {
            adapter_id: "claude",
            path: "/home/user/.claude/mcp.json",
            services: [
              {
                name: "test-service",
                command: "npx",
                args: ["-y", "test-mcp"],
                env: null,
                source_file: "/home/user/.claude/mcp.json",
                adapter_id: "claude",
              },
            ],
            parse_errors: [],
          },
        ],
        conflicts: [],
        new_services: [
          {
            name: "test-service",
            command: "npx",
            args: ["-y", "test-mcp"],
            env: null,
            source_file: "/home/user/.claude/mcp.json",
            adapter_id: "claude",
          },
        ],
        env_vars_needed: [],
        total_services: 1,
      });

      const user = userEvent.setup();
      render(<McpConfigImportSheet {...defaultProps} />);

      const scanButton = screen.getByTestId("import-scan-button");
      await user.click(scanButton);

      await waitFor(() => {
        expect(screen.getByText("预览导入")).toBeInTheDocument();
      });
    });
  });

  describe("Keyboard navigation (AC #8)", () => {
    it("closes on Escape key press", async () => {
      const onOpenChange = vi.fn();
      const user = userEvent.setup();

      render(<McpConfigImportSheet {...defaultProps} onOpenChange={onOpenChange} />);

      await waitFor(() => {
        expect(screen.getByRole("dialog")).toBeInTheDocument();
      });

      await user.keyboard("{Escape}");

      await waitFor(() => {
        expect(onOpenChange).toHaveBeenCalledWith(false);
      });
    });
  });

  describe("7-step wizard flow (AC #2)", () => {
    it("supports back navigation from preview step", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;

      // Mock scan result
      mockInvoke.mockResolvedValueOnce({
        configs: [
          {
            adapter_id: "claude",
            path: "/home/user/.claude/mcp.json",
            services: [
              {
                name: "test-service",
                command: "npx",
                args: ["-y", "test-mcp"],
                env: null,
                source_file: "/home/user/.claude/mcp.json",
                adapter_id: "claude",
              },
            ],
            parse_errors: [],
          },
        ],
      });

      // Mock preview result
      mockInvoke.mockResolvedValueOnce({
        configs: [
          {
            adapter_id: "claude",
            path: "/home/user/.claude/mcp.json",
            services: [
              {
                name: "test-service",
                command: "npx",
                args: ["-y", "test-mcp"],
                env: null,
                source_file: "/home/user/.claude/mcp.json",
                adapter_id: "claude",
              },
            ],
            parse_errors: [],
          },
        ],
        conflicts: [],
        new_services: [
          {
            name: "test-service",
            command: "npx",
            args: ["-y", "test-mcp"],
            env: null,
            source_file: "/home/user/.claude/mcp.json",
            adapter_id: "claude",
          },
        ],
        env_vars_needed: [],
        total_services: 1,
      });

      const user = userEvent.setup();
      render(<McpConfigImportSheet {...defaultProps} />);

      // Go to preview step
      const scanButton = screen.getByTestId("import-scan-button");
      await user.click(scanButton);

      await waitFor(() => {
        expect(screen.getByText("预览导入")).toBeInTheDocument();
      });

      // Click back button
      const backButton = screen.getByText("返回");
      await user.click(backButton);

      await waitFor(() => {
        expect(screen.getByTestId("import-scan-button")).toBeInTheDocument();
      });
    });
  });

  // ===== Story 11.29: 导入后自动引导关联项目 =====

  describe("Story 11.29: Import Auto-Link to Project", () => {
    // 共享的 mock scan+preview+import 流程辅助函数
    const setupFullImportFlow = () => {
      const scanResult = {
        configs: [
          {
            adapter_id: "claude",
            path: "/home/user/.claude/mcp.json",
            services: [
              {
                name: "git-mcp",
                command: "npx",
                args: ["-y", "git-mcp"],
                env: null,
                source_file: "/home/user/.claude/mcp.json",
                adapter_id: "claude",
              },
              {
                name: "postgres",
                command: "npx",
                args: ["-y", "postgres-mcp"],
                env: null,
                source_file: "/home/user/.claude/mcp.json",
                adapter_id: "claude",
              },
            ],
            parse_errors: [],
          },
        ],
      };

      const previewResult = {
        configs: scanResult.configs,
        conflicts: [],
        new_services: scanResult.configs[0].services,
        env_vars_needed: [],
        total_services: 2,
      };

      const importResult = {
        imported_count: 2,
        skipped_count: 0,
        backup_files: [],
        shadow_configs: [],
        errors: [],
        imported_service_ids: ["id-1", "id-2"],
      };

      return { scanResult, previewResult, importResult };
    };

    it("AC5: Hub 页面导入无关联步骤", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;
      const { scanResult, previewResult, importResult } = setupFullImportFlow();

      // Mock scan → preview → gateway status → import
      mockInvoke
        .mockResolvedValueOnce(scanResult)     // scan_mcp_configs_cmd
        .mockResolvedValueOnce(previewResult)  // preview_mcp_import
        .mockResolvedValueOnce({ running: false, port: null, auth_token: "" }) // get_gateway_status
        .mockResolvedValueOnce(importResult);  // execute_mcp_import

      const user = userEvent.setup();
      // 无 projectId = Hub 页面
      render(<McpConfigImportSheet {...defaultProps} />);

      // 完成扫描
      await user.click(screen.getByTestId("import-scan-button"));
      await waitFor(() => {
        expect(screen.getByText("预览导入")).toBeInTheDocument();
      });

      // 下一步到确认
      await user.click(screen.getByTestId("import-next-button"));
      // 等待确认按钮出现（用 testId 避免匹配标题）
      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toHaveTextContent("确认导入");
      });

      // 点击确认导入按钮
      await user.click(screen.getByTestId("import-next-button"));

      // 应该直接显示结果页（不显示关联步骤）
      await waitFor(() => {
        expect(screen.getByText("导入成功")).toBeInTheDocument();
      });

      // 应该有关闭按钮而非关联按钮
      expect(screen.getByText("关闭")).toBeInTheDocument();
      expect(screen.queryByTestId("link-to-project-button")).not.toBeInTheDocument();
    });

    it("AC1+AC3: 项目详情页导入后显示关联步骤并执行关联", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;
      const { scanResult, previewResult, importResult } = setupFullImportFlow();

      // Mock: scan → preview → gateway → import → list_mcp_services → get_project_mcp_services → link x2
      mockInvoke
        .mockResolvedValueOnce(scanResult)     // scan_mcp_configs_cmd
        .mockResolvedValueOnce(previewResult)  // preview_mcp_import
        .mockResolvedValueOnce({ running: true, port: 3000, auth_token: "token" }) // get_gateway_status
        .mockResolvedValueOnce(importResult)   // execute_mcp_import
        .mockResolvedValueOnce([               // list_mcp_services
          { id: "id-1", name: "git-mcp", source_file: "/home/user/.claude/mcp.json" },
          { id: "id-2", name: "postgres", source_file: "/home/user/.claude/mcp.json" },
        ])
        .mockResolvedValueOnce([])             // get_project_mcp_services (none linked yet)
        .mockResolvedValueOnce(undefined)      // link_mcp_service_to_project (id-1)
        .mockResolvedValueOnce(undefined);     // link_mcp_service_to_project (id-2)

      const onSuccess = vi.fn();
      const onOpenChange = vi.fn();
      const user = userEvent.setup();

      render(
        <McpConfigImportSheet
          open={true}
          onOpenChange={onOpenChange}
          onSuccess={onSuccess}
          projectId="proj-1"
          projectName="Mantra"
        />
      );

      // 完成扫描
      await user.click(screen.getByTestId("import-scan-button"));
      await waitFor(() => {
        expect(screen.getByText("预览导入")).toBeInTheDocument();
      });

      // 下一步到确认
      await user.click(screen.getByTestId("import-next-button"));
      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toHaveTextContent("确认导入");
      });

      // 点击确认导入按钮
      await user.click(screen.getByTestId("import-next-button"));

      // 应该显示关联步骤（不是结果页）
      await waitFor(() => {
        expect(screen.getByText(/选择要关联到「Mantra」的服务/)).toBeInTheDocument();
      });

      // 应该有服务列表
      expect(screen.getByText("git-mcp")).toBeInTheDocument();
      expect(screen.getByText("postgres")).toBeInTheDocument();

      // 点击关联到项目按钮
      await user.click(screen.getByTestId("link-to-project-button"));

      // 应该调用 link_mcp_service_to_project
      await waitFor(() => {
        expect(onSuccess).toHaveBeenCalled();
        expect(onOpenChange).toHaveBeenCalledWith(false);
      });
    });

    it("AC4: 跳过关联直接关闭", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;
      const { scanResult, previewResult, importResult } = setupFullImportFlow();

      mockInvoke
        .mockResolvedValueOnce(scanResult)
        .mockResolvedValueOnce(previewResult)
        .mockResolvedValueOnce({ running: true, port: 3000, auth_token: "token" })
        .mockResolvedValueOnce(importResult)
        .mockResolvedValueOnce([
          { id: "id-1", name: "git-mcp", source_file: "/home/user/.claude/mcp.json" },
          { id: "id-2", name: "postgres", source_file: "/home/user/.claude/mcp.json" },
        ])
        .mockResolvedValueOnce([]);  // get_project_mcp_services

      const onSuccess = vi.fn();
      const onOpenChange = vi.fn();
      const user = userEvent.setup();

      render(
        <McpConfigImportSheet
          open={true}
          onOpenChange={onOpenChange}
          onSuccess={onSuccess}
          projectId="proj-1"
        />
      );

      // 完成导入流程到关联步骤
      await user.click(screen.getByTestId("import-scan-button"));
      await waitFor(() => screen.getByTestId("import-next-button"));
      await user.click(screen.getByTestId("import-next-button"));
      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toHaveTextContent("确认导入");
      });
      await user.click(screen.getByTestId("import-next-button"));

      // 等待关联步骤出现
      await waitFor(() => {
        expect(screen.getByTestId("link-skip-button")).toBeInTheDocument();
      });

      // 点击跳过
      await user.click(screen.getByTestId("link-skip-button"));

      // 应该关闭 Sheet
      await waitFor(() => {
        expect(onOpenChange).toHaveBeenCalledWith(false);
      });
    });

    it("AC6: 所有服务已关联时显示完成提示", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;
      const { scanResult, previewResult, importResult } = setupFullImportFlow();

      mockInvoke
        .mockResolvedValueOnce(scanResult)
        .mockResolvedValueOnce(previewResult)
        .mockResolvedValueOnce({ running: true, port: 3000, auth_token: "token" })
        .mockResolvedValueOnce(importResult)
        .mockResolvedValueOnce([
          { id: "id-1", name: "git-mcp", source_file: "/home/user/.claude/mcp.json" },
          { id: "id-2", name: "postgres", source_file: "/home/user/.claude/mcp.json" },
        ])
        .mockResolvedValueOnce([             // get_project_mcp_services - 全部已关联
          { id: "id-1", name: "git-mcp" },
          { id: "id-2", name: "postgres" },
        ]);

      const user = userEvent.setup();

      render(
        <McpConfigImportSheet
          open={true}
          onOpenChange={vi.fn()}
          onSuccess={vi.fn()}
          projectId="proj-1"
          projectName="Mantra"
        />
      );

      // 完成导入流程
      await user.click(screen.getByTestId("import-scan-button"));
      await waitFor(() => screen.getByTestId("import-next-button"));
      await user.click(screen.getByTestId("import-next-button"));
      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toHaveTextContent("确认导入");
      });
      await user.click(screen.getByTestId("import-next-button"));

      // 应该显示"所有服务已关联"提示
      await waitFor(() => {
        expect(screen.getByText("所有服务已关联到当前项目")).toBeInTheDocument();
      });

      // 应该有完成按钮
      expect(screen.getByText("完成")).toBeInTheDocument();
    });

    it("AC7: 全部已存在时隐藏 0 已导入并显示 N 个服务已在 Hub 中", async () => {
      const { invoke } = await import("@/lib/ipc-adapter");
      const mockInvoke = invoke as ReturnType<typeof vi.fn>;

      // 全部 skipped 的导入结果
      const skippedResult = {
        imported_count: 0,
        skipped_count: 2,
        backup_files: [],
        shadow_configs: [],
        errors: [],
        imported_service_ids: [],
      };

      const { scanResult, previewResult } = setupFullImportFlow();

      mockInvoke
        .mockResolvedValueOnce(scanResult)
        .mockResolvedValueOnce(previewResult)
        .mockResolvedValueOnce({ running: false, port: null, auth_token: "" })
        .mockResolvedValueOnce(skippedResult);

      const user = userEvent.setup();
      // 无 projectId → Hub 页面，显示结果页
      render(<McpConfigImportSheet {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));
      await waitFor(() => screen.getByTestId("import-next-button"));
      await user.click(screen.getByTestId("import-next-button"));
      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toHaveTextContent("确认导入");
      });
      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        // "0 已导入" 不应该显示
        expect(screen.queryByText("已导入")).not.toBeInTheDocument();
        // "2 个服务已在 Hub 中" 应该显示
        expect(screen.getByText("2 个服务已在 Hub 中")).toBeInTheDocument();
      });
    });
  });
});
