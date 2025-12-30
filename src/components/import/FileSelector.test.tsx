/**
 * FileSelector 测试文件
 * Story 2.9: Task 3
 *
 * 测试文件选择组件
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { FileSelector, type DiscoveredFile } from "./FileSelector";

/** 测试用文件数据 */
const mockFiles: DiscoveredFile[] = [
  {
    path: "/home/user/.claude/projects/proj1/conversation_123.json",
    name: "conversation_123.json",
    size: 1024,
    modifiedAt: Date.now() - 3600000, // 1 小时前
    projectPath: "/home/user/myproject",
  },
  {
    path: "/home/user/.claude/projects/proj2/conversation_456.json",
    name: "conversation_456.json",
    size: 2048,
    modifiedAt: Date.now() - 86400000, // 1 天前
    projectPath: "/home/user/another-project",
  },
];

describe("FileSelector", () => {
  // Task 3.2 & 3.3: 操作按钮
  describe("Action Buttons", () => {
    it("shows scan default path button", () => {
      render(
        <FileSelector
          files={[]}
          selectedFiles={new Set()}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      expect(screen.getByText("扫描默认路径")).toBeInTheDocument();
    });

    it("shows select files button", () => {
      render(
        <FileSelector
          files={[]}
          selectedFiles={new Set()}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      expect(screen.getByText("手动选择文件")).toBeInTheDocument();
    });

    it("calls onScan when scan button is clicked", () => {
      const onScan = vi.fn();
      render(
        <FileSelector
          files={[]}
          selectedFiles={new Set()}
          onScan={onScan}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      fireEvent.click(screen.getByText("扫描默认路径"));
      expect(onScan).toHaveBeenCalled();
    });

    it("calls onSelectFiles when select button is clicked", () => {
      const onSelectFiles = vi.fn();
      render(
        <FileSelector
          files={[]}
          selectedFiles={new Set()}
          onScan={vi.fn()}
          onSelectFiles={onSelectFiles}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      fireEvent.click(screen.getByText("手动选择文件"));
      expect(onSelectFiles).toHaveBeenCalled();
    });
  });

  // Task 3.4: 显示文件列表
  describe("File List", () => {
    it("displays discovered files", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      expect(screen.getByText("conversation_123.json")).toBeInTheDocument();
      expect(screen.getByText("conversation_456.json")).toBeInTheDocument();
    });

    it("shows empty state when no files", () => {
      render(
        <FileSelector
          files={[]}
          selectedFiles={new Set()}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      expect(screen.getByText("暂无文件")).toBeInTheDocument();
    });

    // Task 3.5: 文件信息显示
    it("displays file size", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      // 1024 bytes = 1 KB
      expect(screen.getByText(/1.*KB/i)).toBeInTheDocument();
    });

    it("displays project path", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      expect(screen.getByText("myproject")).toBeInTheDocument();
      expect(screen.getByText("another-project")).toBeInTheDocument();
    });
  });

  // Task 3.6: 全选/反选
  describe("Select All", () => {
    it("shows select all checkbox in header", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      expect(screen.getByTestId("select-all-checkbox")).toBeInTheDocument();
    });

    it("calls onToggleAll when select all is clicked", () => {
      const onToggleAll = vi.fn();
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={onToggleAll}
          loading={false}
        />
      );

      fireEvent.click(screen.getByTestId("select-all-checkbox"));
      expect(onToggleAll).toHaveBeenCalled();
    });

    it("shows checked state when all files selected", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      const checkbox = screen.getByTestId("select-all-checkbox");
      expect(checkbox).toHaveAttribute("data-state", "checked");
    });

    it("shows unchecked state when no files selected", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set()}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      const checkbox = screen.getByTestId("select-all-checkbox");
      expect(checkbox).toHaveAttribute("data-state", "unchecked");
    });
  });

  // Task 3.7: 单个文件选择
  describe("Individual File Selection", () => {
    it("shows checkbox for each file", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      expect(screen.getByTestId(`file-checkbox-${mockFiles[0].path}`)).toBeInTheDocument();
      expect(screen.getByTestId(`file-checkbox-${mockFiles[1].path}`)).toBeInTheDocument();
    });

    it("calls onToggleFile when file checkbox is clicked", () => {
      const onToggleFile = vi.fn();
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={onToggleFile}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      fireEvent.click(screen.getByTestId(`file-checkbox-${mockFiles[0].path}`));
      expect(onToggleFile).toHaveBeenCalledWith(mockFiles[0].path);
    });

    it("shows checked state for selected files", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set([mockFiles[0].path])}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      const checkbox1 = screen.getByTestId(`file-checkbox-${mockFiles[0].path}`);
      const checkbox2 = screen.getByTestId(`file-checkbox-${mockFiles[1].path}`);

      expect(checkbox1).toHaveAttribute("data-state", "checked");
      expect(checkbox2).toHaveAttribute("data-state", "unchecked");
    });
  });

  // 加载状态
  describe("Loading State", () => {
    it("shows loading indicator when loading", () => {
      render(
        <FileSelector
          files={[]}
          selectedFiles={new Set()}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={true}
        />
      );

      expect(screen.getByTestId("file-selector-loading")).toBeInTheDocument();
    });

    it("disables buttons when loading", () => {
      render(
        <FileSelector
          files={[]}
          selectedFiles={new Set()}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={true}
        />
      );

      expect(screen.getByText("扫描默认路径").closest("button")).toBeDisabled();
      expect(screen.getByText("手动选择文件").closest("button")).toBeDisabled();
    });
  });

  // 文件计数
  describe("File Count", () => {
    it("shows selected count", () => {
      render(
        <FileSelector
          files={mockFiles}
          selectedFiles={new Set([mockFiles[0].path])}
          onScan={vi.fn()}
          onSelectFiles={vi.fn()}
          onToggleFile={vi.fn()}
          onToggleAll={vi.fn()}
          loading={false}
        />
      );

      expect(screen.getByText(/已选择 1 个/)).toBeInTheDocument();
    });
  });
});
