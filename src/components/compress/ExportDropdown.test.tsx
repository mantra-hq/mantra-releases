/**
 * ExportDropdown Component Tests
 * Story 10.7: Task 7.2
 *
 * 测试导出下拉菜单组件
 * - 菜单交互
 * - 导出触发
 * - Toast 显示
 */

import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ExportDropdown } from "./ExportDropdown";
import type { PreviewMessage } from "@/hooks/useCompressState";
import type { TokenStats } from "./TokenStatistics";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.export.button": "Export",
        "compress.export.jsonl": "Export as JSONL",
        "compress.export.markdown": "Export as Markdown",
        "compress.export.copy": "Copy to Clipboard",
        "compress.export.copySuccess": "Copied to clipboard",
        "compress.export.exportSuccess": "Export successful",
        "compress.export.exportFailed": "Export failed",
        "compress.export.saveDialogTitle": "Export Compressed Session",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock sonner toast
const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();
vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => mockToastSuccess(...args),
    error: (...args: unknown[]) => mockToastError(...args),
  },
}));

// Mock Tauri dialog
const mockSave = vi.fn();
vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: (...args: unknown[]) => mockSave(...args),
}));

// Mock Tauri fs
const mockWriteTextFile = vi.fn();
vi.mock("@tauri-apps/plugin-fs", () => ({
  writeTextFile: (...args: unknown[]) => mockWriteTextFile(...args),
}));

// Mock compress-exporter
const mockExportToJsonl = vi.fn();
const mockExportToMarkdown = vi.fn();
const mockGetExportContent = vi.fn();
const mockFormatExportFilename = vi.fn();
vi.mock("@/lib/compress-exporter", () => ({
  exportToJsonl: (...args: unknown[]) => mockExportToJsonl(...args),
  exportToMarkdown: (...args: unknown[]) => mockExportToMarkdown(...args),
  getExportContent: (...args: unknown[]) => mockGetExportContent(...args),
  formatExportFilename: (...args: unknown[]) => mockFormatExportFilename(...args),
}));

// Mock clipboard
const mockClipboardWriteText = vi.fn().mockResolvedValue(undefined);

// 在模块加载时设置 clipboard mock
beforeAll(() => {
  // jsdom 中 navigator.clipboard 可能不存在，需要创建
  const clipboardMock = {
    writeText: mockClipboardWriteText,
    readText: vi.fn(),
  };
  
  Object.defineProperty(global.navigator, "clipboard", {
    value: clipboardMock,
    writable: true,
    configurable: true,
  });
});

// Mock DropdownMenu to simplify testing (avoid Radix Portal issues)
vi.mock("@/components/ui/dropdown-menu", () => ({
  DropdownMenu: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DropdownMenuTrigger: ({ children, asChild }: { children: React.ReactNode; asChild?: boolean }) => (
    <div data-testid="dropdown-trigger-wrapper">{children}</div>
  ),
  DropdownMenuContent: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="dropdown-content">{children}</div>
  ),
  DropdownMenuItem: ({ children, onClick, "data-testid": testId }: { children: React.ReactNode; onClick?: () => void; "data-testid"?: string }) => (
    <button data-testid={testId} onClick={onClick} type="button">
      {children}
    </button>
  ),
  DropdownMenuSeparator: () => <hr />,
}));

import * as React from "react";

// 创建测试数据
function createPreviewMessage(
  id: string,
  role: "user" | "assistant",
  content: string
): PreviewMessage {
  return {
    id,
    operation: "keep",
    message: {
      id,
      role,
      content: [{ type: "text", content }],
      timestamp: new Date().toISOString(),
    },
  };
}

function createTokenStats(): TokenStats {
  return {
    originalTotal: 1000,
    compressedTotal: 800,
    savedTokens: 200,
    savedPercentage: 20,
    changeStats: {
      deleted: 1,
      modified: 1,
      inserted: 0,
    },
  };
}

describe("ExportDropdown", () => {
  const defaultProps = {
    previewMessages: [
      createPreviewMessage("1", "user", "Hello"),
      createPreviewMessage("2", "assistant", "Hi there"),
    ],
    tokenStats: createTokenStats(),
    sessionName: "test-session",
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockExportToJsonl.mockReturnValue('{"role":"user","content":"Hello"}');
    mockExportToMarkdown.mockReturnValue("# Test Session\n\n## User\n\nHello");
    mockGetExportContent.mockReturnValue("## User\n\nHello");
    mockFormatExportFilename.mockReturnValue("test-session-compressed-2026-01-21.jsonl");
    mockClipboardWriteText.mockResolvedValue(undefined);
  });

  describe("基础渲染 (AC #1, #2)", () => {
    it("应渲染导出按钮", () => {
      render(<ExportDropdown {...defaultProps} />);

      expect(screen.getByTestId("export-dropdown-trigger")).toBeInTheDocument();
      expect(screen.getByText("Export")).toBeInTheDocument();
    });

    it("应显示下拉菜单选项", () => {
      render(<ExportDropdown {...defaultProps} />);

      // 由于 mock 了 DropdownMenu，菜单项直接可见
      expect(screen.getByText("Export as JSONL")).toBeInTheDocument();
      expect(screen.getByText("Export as Markdown")).toBeInTheDocument();
      expect(screen.getByText("Copy to Clipboard")).toBeInTheDocument();
    });

    it("菜单项应有正确的 testid", () => {
      render(<ExportDropdown {...defaultProps} />);

      // 检查菜单项存在
      expect(screen.getByTestId("export-jsonl")).toBeInTheDocument();
      expect(screen.getByTestId("export-markdown")).toBeInTheDocument();
      expect(screen.getByTestId("export-copy")).toBeInTheDocument();
    });
  });

  describe("JSONL 导出 (AC #3)", () => {
    it("点击 JSONL 选项应调用保存对话框", async () => {
      const user = userEvent.setup();
      mockSave.mockResolvedValue("/path/to/file.jsonl");
      mockWriteTextFile.mockResolvedValue(undefined);

      render(<ExportDropdown {...defaultProps} />);

      await user.click(screen.getByTestId("export-jsonl"));

      expect(mockSave).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Export Compressed Session",
          filters: [{ name: "JSONL", extensions: ["jsonl"] }],
        })
      );
    });

    it("选择文件后应写入内容", async () => {
      const user = userEvent.setup();
      mockSave.mockResolvedValue("/path/to/file.jsonl");
      mockWriteTextFile.mockResolvedValue(undefined);

      render(<ExportDropdown {...defaultProps} />);

      await user.click(screen.getByTestId("export-jsonl"));

      await waitFor(() => {
        expect(mockWriteTextFile).toHaveBeenCalledWith(
          "/path/to/file.jsonl",
          expect.any(String)
        );
      });
    });

    it("导出成功后应显示 Toast", async () => {
      const user = userEvent.setup();
      mockSave.mockResolvedValue("/path/to/file.jsonl");
      mockWriteTextFile.mockResolvedValue(undefined);

      render(<ExportDropdown {...defaultProps} />);

      await user.click(screen.getByTestId("export-jsonl"));

      await waitFor(() => {
        expect(mockToastSuccess).toHaveBeenCalledWith("Export successful");
      });
    });

    it("用户取消保存时不应显示 Toast", async () => {
      const user = userEvent.setup();
      mockSave.mockResolvedValue(null);

      render(<ExportDropdown {...defaultProps} />);

      await user.click(screen.getByTestId("export-jsonl"));

      await waitFor(() => {
        expect(mockWriteTextFile).not.toHaveBeenCalled();
        expect(mockToastSuccess).not.toHaveBeenCalled();
        expect(mockToastError).not.toHaveBeenCalled();
      });
    });
  });

  describe("Markdown 导出 (AC #4, #5)", () => {
    it("点击 Markdown 选项应调用保存对话框", async () => {
      const user = userEvent.setup();
      mockSave.mockResolvedValue("/path/to/file.md");
      mockWriteTextFile.mockResolvedValue(undefined);
      mockFormatExportFilename.mockReturnValue("test-session-compressed-2026-01-21.md");

      render(<ExportDropdown {...defaultProps} />);

      await user.click(screen.getByTestId("export-markdown"));

      expect(mockSave).toHaveBeenCalledWith(
        expect.objectContaining({
          filters: [{ name: "Markdown", extensions: ["md"] }],
        })
      );
    });

    it("应传递正确的参数给 exportToMarkdown", async () => {
      const user = userEvent.setup();
      mockSave.mockResolvedValue("/path/to/file.md");
      mockWriteTextFile.mockResolvedValue(undefined);

      render(<ExportDropdown {...defaultProps} />);

      await user.click(screen.getByTestId("export-markdown"));

      await waitFor(() => {
        expect(mockExportToMarkdown).toHaveBeenCalledWith(
          defaultProps.previewMessages,
          defaultProps.tokenStats,
          defaultProps.sessionName
        );
      });
    });
  });

  describe("复制到剪贴板 (AC #6)", () => {
    it("点击复制选项应获取内容并显示成功 Toast", async () => {
      const user = userEvent.setup();

      render(<ExportDropdown {...defaultProps} />);

      await user.click(screen.getByTestId("export-copy"));

      // 验证调用了 getExportContent 来获取内容
      await waitFor(() => {
        expect(mockGetExportContent).toHaveBeenCalledWith(defaultProps.previewMessages);
      });

      // 验证显示了成功 Toast (2秒后消失)
      await waitFor(() => {
        expect(mockToastSuccess).toHaveBeenCalledWith(
          "Copied to clipboard",
          expect.objectContaining({ duration: 2000 })
        );
      });
    });
  });

  describe("错误处理", () => {
    it("写入文件失败时应显示错误 Toast", async () => {
      const user = userEvent.setup();
      mockSave.mockResolvedValue("/path/to/file.jsonl");
      mockWriteTextFile.mockRejectedValue(new Error("Write failed"));

      render(<ExportDropdown {...defaultProps} />);

      await user.click(screen.getByTestId("export-jsonl"));

      await waitFor(() => {
        expect(mockToastError).toHaveBeenCalledWith("Export failed");
      });
    });
  });

  describe("自定义 className", () => {
    it("应支持自定义 className", () => {
      render(<ExportDropdown {...defaultProps} className="custom-class" />);

      const trigger = screen.getByTestId("export-dropdown-trigger");
      expect(trigger).toHaveClass("custom-class");
    });
  });

  describe("无会话名称", () => {
    it("无会话名称时应正常工作", async () => {
      const user = userEvent.setup();
      mockSave.mockResolvedValue("/path/to/file.jsonl");
      mockWriteTextFile.mockResolvedValue(undefined);

      render(
        <ExportDropdown
          previewMessages={defaultProps.previewMessages}
          tokenStats={defaultProps.tokenStats}
        />
      );

      await user.click(screen.getByTestId("export-jsonl"));

      await waitFor(() => {
        expect(mockFormatExportFilename).toHaveBeenCalledWith(undefined, "jsonl");
      });
    });
  });
});
