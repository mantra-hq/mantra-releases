/**
 * CodeSnapshotHeader 组件测试
 * Story 2.5: Task 6 - 验证与测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { CodeSnapshotHeader } from "./CodeSnapshotHeader";

describe("CodeSnapshotHeader", () => {
  beforeEach(() => {
    // Mock clipboard API
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
    });
  });

  describe("AC3 - 文件路径显示", () => {
    it("should display file path", () => {
      render(<CodeSnapshotHeader filePath="src/components/App.tsx" />);

      expect(screen.getByText("src/components/App.tsx")).toBeInTheDocument();
    });

    it("should show '未选择文件' when path is empty", () => {
      render(<CodeSnapshotHeader filePath="" />);

      expect(screen.getByText("未选择文件")).toBeInTheDocument();
    });

    it("should truncate long file paths", () => {
      const longPath =
        "src/components/very/deep/nested/folder/structure/Component.tsx";
      render(<CodeSnapshotHeader filePath={longPath} />);

      const pathElement = screen.getByText(longPath);
      expect(pathElement).toHaveClass("truncate");
    });
  });

  describe("AC7 - 历史状态指示", () => {
    it("should show historical badge when isHistorical is true", () => {
      render(
        <CodeSnapshotHeader
          filePath="src/index.ts"
          isHistorical={true}
          timestamp="2025-12-30T10:30:00Z"
        />
      );

      expect(screen.getByText("历史快照")).toBeInTheDocument();
    });

    it("should format and display timestamp", () => {
      render(
        <CodeSnapshotHeader
          filePath="src/index.ts"
          isHistorical={true}
          timestamp="2025-12-30T10:30:00Z"
        />
      );

      // 时间戳应该被格式化显示
      expect(screen.getByText("历史快照")).toBeInTheDocument();
    });

    it("should display commit hash (truncated to 7 chars)", () => {
      render(
        <CodeSnapshotHeader
          filePath="src/index.ts"
          isHistorical={true}
          commitHash="abc123def456789"
        />
      );

      expect(screen.getByText(/abc123d/)).toBeInTheDocument();
    });

    it("should not show historical badge when isHistorical is false", () => {
      render(<CodeSnapshotHeader filePath="src/index.ts" isHistorical={false} />);

      expect(screen.queryByText("历史快照")).not.toBeInTheDocument();
    });
  });

  describe("Task 3.4 - 复制路径按钮", () => {
    it("should copy file path when copy button is clicked", async () => {
      render(<CodeSnapshotHeader filePath="src/index.ts" />);

      const copyButton = screen.getByTitle("复制路径");
      fireEvent.click(copyButton);

      await waitFor(() => {
        expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
          "src/index.ts"
        );
      });
    });

    it("should not show copy button when filePath is empty", () => {
      render(<CodeSnapshotHeader filePath="" />);

      expect(screen.queryByTitle("复制路径")).not.toBeInTheDocument();
    });
  });
});

