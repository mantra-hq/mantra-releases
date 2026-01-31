/**
 * McpConfigImportDialog 组件测试
 * Story 11.3: Task 9.9 - 配置导入对话框测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { McpConfigImportDialog } from "./McpConfigImportDialog";

// Mock IPC adapter
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: vi.fn(),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        "hub.import.title": "导入配置",
        "hub.import.description": "从现有工具导入 MCP 服务配置",
        "hub.import.scanTitle": "扫描配置",
        "hub.import.startScan": "开始扫描",
        "hub.import.scanning": "正在扫描配置文件...",
        "hub.import.noConfigsFound": "未检测到任何 MCP 配置文件",
        "hub.import.previewTitle": "选择服务",
        "hub.import.previewDescription": "选择要导入的 MCP 服务",
        "hub.import.foundSummary": `检测到 ${params?.configs || 0} 个配置文件，共 ${params?.services || 0} 个服务`,
        "hub.import.services": "个服务",
        "hub.import.new": "新服务",
        "hub.import.conflict": "冲突",
        "hub.import.selectedCount": `已选择 ${params?.count || 0} 个服务`,
        "hub.import.selectAll": "全选",
        "hub.import.selectNone": "取消全选",
        "hub.import.conflictsTitle": "解决冲突",
        "hub.import.conflictsDescription": `以下 ${params?.count || 0} 个服务存在配置冲突`,
        "hub.import.resolution": "解决方式",
        "hub.import.keepExisting": "保留现有配置",
        "hub.import.useFrom": `使用来自 ${params?.source || ""} 的配置`,
        "hub.import.renameAndImport": "重命名后导入",
        "hub.import.skip": "跳过此服务",
        "hub.import.newName": "新名称",
        "hub.import.envTitle": "环境变量",
        "hub.import.envDescription": `设置 ${params?.count || 0} 个服务所需的环境变量`,
        "hub.import.envPlaceholder": "输入变量值（可为空）",
        "hub.import.envHint": "值将被加密存储，留空则不设置",
        "hub.import.shadowMode": "影子模式",
        "hub.import.shadowModeDescription": "修改原配置文件，将所有服务请求路由到 Mantra Gateway",
        "hub.import.importingTitle": "正在导入",
        "hub.import.importing": "正在导入...",
        "hub.import.startImport": "开始导入",
        "hub.import.resultTitle": "导入结果",
        "hub.import.resultSuccess": "导入成功",
        "hub.import.resultPartial": "部分导入成功",
        "hub.import.resultError": "导入失败",
        "hub.import.imported": "已导入",
        "hub.import.skipped": "已跳过",
        "hub.import.backupFiles": "备份文件",
        "hub.import.shadowConfigs": "影子配置文件",
        "common.close": "关闭",
        "common.back": "返回",
        "common.next": "下一步",
        "common.retry": "重试",
      };
      return translations[key] || key;
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

// 测试数据 (Story 11.8: 更新为使用 adapter_id)
const mockScanResult = {
  configs: [
    {
      adapter_id: "claude",
      path: "/project/.claude/config.json",
      scope: "project" as const,
      services: [
        {
          name: "git-mcp",
          command: "npx",
          args: ["--yes", "@anthropic/mcp-server-git"],
          env: null,
          source_file: "/project/.claude/config.json",
          adapter_id: "claude",
          scope: "project" as const,
        },
        {
          name: "postgres",
          command: "npx",
          args: ["--yes", "@anthropic/mcp-server-postgres"],
          env: { DATABASE_URL: "$DATABASE_URL" },
          source_file: "/project/.claude/config.json",
          adapter_id: "claude",
          scope: "project" as const,
        },
      ],
      parse_errors: [],
    },
  ],
};

const mockPreview = {
  configs: mockScanResult.configs,
  conflicts: [],
  new_services: mockScanResult.configs[0].services,
  env_vars_needed: ["DATABASE_URL"],
  total_services: 2,
};

const mockPreviewWithConflict = {
  ...mockPreview,
  conflicts: [
    {
      name: "git-mcp",
      existing: {
        id: "existing-1",
        name: "git-mcp",
        command: "npx",
        args: ["--yes", "@anthropic/mcp-server-git"],
        env: null,
        source: "manual" as const,
        source_file: null,
        enabled: true,
      },
      candidates: [mockScanResult.configs[0].services[0]],
    },
  ],
  new_services: [mockScanResult.configs[0].services[1]],
};

const mockImportResult = {
  imported_count: 2,
  skipped_count: 0,
  backup_files: ["/project/.claude/config.json.mantra-backup"],
  shadow_configs: [],
  errors: [],
  imported_service_ids: ["new-1", "new-2"],
};

describe("McpConfigImportDialog", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    onSuccess: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("初始状态", () => {
    it("应该渲染扫描步骤", () => {
      render(<McpConfigImportDialog {...defaultProps} />);

      expect(screen.getByText("导入配置")).toBeInTheDocument();
      expect(screen.getByTestId("import-scan-button")).toBeInTheDocument();
    });

    it("关闭时不应该渲染", () => {
      render(<McpConfigImportDialog {...defaultProps} open={false} />);

      expect(screen.queryByText("导入配置")).not.toBeInTheDocument();
    });
  });

  describe("扫描步骤", () => {
    it("点击扫描按钮应该调用 scan_mcp_configs_cmd", async () => {
      const user = userEvent.setup();
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult) // scan_mcp_configs_cmd
        .mockResolvedValueOnce(mockPreview); // preview_mcp_import

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("scan_mcp_configs_cmd", {
          projectPath: null,
        });
      });
    });

    it("扫描成功后应该进入预览步骤", async () => {
      const user = userEvent.setup();
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview);

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByText("选择服务")).toBeInTheDocument();
      });
    });

    it("扫描无结果时应该显示错误", async () => {
      const user = userEvent.setup();
      mockInvokeFn.mockResolvedValueOnce({ configs: [] });

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByText("未检测到任何 MCP 配置文件")).toBeInTheDocument();
      });
    });

    it("扫描失败时应该显示错误", async () => {
      const user = userEvent.setup();
      mockInvokeFn.mockRejectedValueOnce(new Error("Scan failed"));

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByText("Scan failed")).toBeInTheDocument();
      });
    });
  });

  describe("预览步骤", () => {
    beforeEach(() => {
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview);
    });

    it("应该显示检测到的服务", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByText("git-mcp")).toBeInTheDocument();
        expect(screen.getByText("postgres")).toBeInTheDocument();
      });
    });

    it("应该默认选中所有服务", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        const gitCheckbox = screen.getByTestId("import-service-checkbox-git-mcp");
        const postgresCheckbox = screen.getByTestId("import-service-checkbox-postgres");
        expect(gitCheckbox).toBeChecked();
        expect(postgresCheckbox).toBeChecked();
      });
    });

    it("点击全选/取消全选应该切换选择状态", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByText("取消全选")).toBeInTheDocument();
      });

      await user.click(screen.getByText("取消全选"));

      await waitFor(() => {
        const gitCheckbox = screen.getByTestId("import-service-checkbox-git-mcp");
        expect(gitCheckbox).not.toBeChecked();
      });
    });
  });

  describe("冲突解决步骤", () => {
    beforeEach(() => {
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreviewWithConflict);
    });

    it("有冲突时应该显示冲突解决步骤", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("解决冲突")).toBeInTheDocument();
      });
    });

    it("应该显示冲突解决选项", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByTestId("conflict-resolution-git-mcp")).toBeInTheDocument();
      });
    });
  });

  describe("环境变量步骤", () => {
    beforeEach(() => {
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview);
    });

    it("需要环境变量时应该显示环境变量步骤", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("环境变量")).toBeInTheDocument();
        expect(screen.getByTestId("env-var-input-DATABASE_URL")).toBeInTheDocument();
      });
    });

    it("应该显示影子模式开关", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByTestId("shadow-mode-switch")).toBeInTheDocument();
      });
    });
  });

  describe("导入执行", () => {
    beforeEach(() => {
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview)
        .mockResolvedValueOnce(mockImportResult);
    });

    it("导入成功后应该显示结果", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("开始导入")).toBeInTheDocument();
      });

      await user.click(screen.getByText("开始导入"));

      await waitFor(() => {
        expect(screen.getByText("导入成功")).toBeInTheDocument();
      });
    });

    it("导入成功后应该调用 onSuccess", async () => {
      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("开始导入")).toBeInTheDocument();
      });

      await user.click(screen.getByText("开始导入"));

      await waitFor(() => {
        expect(defaultProps.onSuccess).toHaveBeenCalled();
      });
    });

    it("导入失败时应该显示错误", async () => {
      mockInvokeFn
        .mockReset()
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview)
        .mockRejectedValueOnce(new Error("Import failed"));

      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("开始导入")).toBeInTheDocument();
      });

      await user.click(screen.getByText("开始导入"));

      await waitFor(() => {
        expect(feedback.error).toHaveBeenCalled();
      });
    });
  });

  describe("结果步骤", () => {
    it("应该显示导入统计", async () => {
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview)
        .mockResolvedValueOnce(mockImportResult);

      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("开始导入")).toBeInTheDocument();
      });

      await user.click(screen.getByText("开始导入"));

      await waitFor(() => {
        expect(screen.getByText("2")).toBeInTheDocument(); // imported_count
        expect(screen.getByText("0")).toBeInTheDocument(); // skipped_count
      });
    });

    it("应该显示备份文件列表", async () => {
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview)
        .mockResolvedValueOnce(mockImportResult);

      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("开始导入")).toBeInTheDocument();
      });

      await user.click(screen.getByText("开始导入"));

      await waitFor(() => {
        // 使用函数匹配器，因为 Label 组件会把文本分割成多个节点
        expect(screen.getByText((content) => content.includes("备份文件"))).toBeInTheDocument();
        expect(screen.getByText("/project/.claude/config.json.mantra-backup")).toBeInTheDocument();
      });
    });

    it("点击关闭按钮应该关闭对话框", async () => {
      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview)
        .mockResolvedValueOnce(mockImportResult);

      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("开始导入")).toBeInTheDocument();
      });

      await user.click(screen.getByText("开始导入"));

      await waitFor(() => {
        expect(screen.getByText("关闭")).toBeInTheDocument();
      });

      await user.click(screen.getByText("关闭"));

      expect(defaultProps.onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe("部分导入结果", () => {
    it("部分成功时应该显示警告图标", async () => {
      const partialResult = {
        ...mockImportResult,
        imported_count: 1,
        skipped_count: 1,
      };

      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview)
        .mockResolvedValueOnce(partialResult);

      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("开始导入")).toBeInTheDocument();
      });

      await user.click(screen.getByText("开始导入"));

      await waitFor(() => {
        expect(screen.getByText("部分导入成功")).toBeInTheDocument();
      });
    });

    it("有错误时应该显示错误列表", async () => {
      const errorResult = {
        ...mockImportResult,
        imported_count: 0,
        errors: ["Failed to import git-mcp: Name conflict"],
      };

      mockInvokeFn
        .mockResolvedValueOnce(mockScanResult)
        .mockResolvedValueOnce(mockPreview)
        .mockResolvedValueOnce(errorResult);

      const user = userEvent.setup();

      render(<McpConfigImportDialog {...defaultProps} />);

      await user.click(screen.getByTestId("import-scan-button"));

      await waitFor(() => {
        expect(screen.getByTestId("import-next-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("import-next-button"));

      await waitFor(() => {
        expect(screen.getByText("开始导入")).toBeInTheDocument();
      });

      await user.click(screen.getByText("开始导入"));

      await waitFor(() => {
        expect(screen.getByText("Failed to import git-mcp: Name conflict")).toBeInTheDocument();
      });
    });
  });
});
