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
});
