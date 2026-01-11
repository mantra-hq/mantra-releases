/**
 * FileEditDiff Component Tests
 * Story 8.11: Task 9 - FileEdit Diff 视图
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { FileEditDiff } from "./FileEditDiff";

describe("FileEditDiff", () => {
  const testFilePath = "/src/components/App.tsx";

  describe("AC#9: Inline diff 视图", () => {
    it("renders file path breadcrumb", () => {
      render(
        <FileEditDiff
          filePath={testFilePath}
          oldString="const a = 1;"
          newString="const b = 2;"
        />
      );
      expect(screen.getByText(testFilePath)).toBeInTheDocument();
    });

    it("shows diff with added lines marked with green +", () => {
      const { container } = render(
        <FileEditDiff
          filePath={testFilePath}
          oldString="line1"
          newString="line1\nline2"
        />
      );
      // 查找包含 + 的变更标记 span
      const addedMarkers = container.querySelectorAll(".text-green-600");
      expect(addedMarkers.length).toBeGreaterThan(0);
    });

    it("shows diff with removed lines marked with red -", () => {
      const { container } = render(
        <FileEditDiff
          filePath={testFilePath}
          oldString="line1\nline2"
          newString="line1"
        />
      );
      // 查找包含 - 的变更标记 span
      const removedMarkers = container.querySelectorAll(".text-red-600");
      expect(removedMarkers.length).toBeGreaterThan(0);
    });

    it("shows stats badge with additions and deletions", () => {
      const { container } = render(
        <FileEditDiff
          filePath={testFilePath}
          oldString="old line"
          newString="new line"
        />
      );
      // 统计徽章在头部区域，包含 +N 和 -N
      const greenStats = container.querySelector(".text-green-600.dark\\:text-green-400");
      const redStats = container.querySelector(".text-red-600.dark\\:text-red-400");
      expect(greenStats).toBeInTheDocument();
      expect(redStats).toBeInTheDocument();
    });
  });

  describe("AC#9: 只有 newString 时显示完整新内容", () => {
    it("renders all lines as added when no oldString", () => {
      const { container } = render(
        <FileEditDiff
          filePath={testFilePath}
          newString={"line1\nline2\nline3"}
        />
      );
      // 所有行都应该有绿色背景
      const addedRows = container.querySelectorAll("[class*='bg-green-500']");
      expect(addedRows.length).toBe(3);
    });

    it("shows correct stats when only newString (single line)", () => {
      const { container } = render(
        <FileEditDiff
          filePath={testFilePath}
          newString="single line"
        />
      );
      // 应该只显示绿色统计，没有红色统计
      const greenStats = container.querySelector(".text-green-600.dark\\:text-green-400");
      const redStats = container.querySelector(".text-red-600.dark\\:text-red-400");
      expect(greenStats).toBeInTheDocument();
      expect(redStats).not.toBeInTheDocument();
    });
  });

  describe("AC#9: 无内容时显示提示", () => {
    it("shows noContentToShow message when no newString", () => {
      render(<FileEditDiff filePath={testFilePath} />);
      // 使用默认 fallback 文本匹配
      expect(screen.getByText(/无内容可显示|No content to display/i)).toBeInTheDocument();
    });

    it("shows noContentToShow when newString is empty", () => {
      render(<FileEditDiff filePath={testFilePath} newString="" />);
      expect(screen.getByText(/无内容可显示|No content to display/i)).toBeInTheDocument();
    });
  });

  describe("样式和布局", () => {
    it("applies custom className", () => {
      const { container } = render(
        <FileEditDiff
          filePath={testFilePath}
          newString="content"
          className="custom-class"
        />
      );
      expect(container.firstChild).toHaveClass("custom-class");
    });

    it("renders FileCode2 icon in header", () => {
      const { container } = render(
        <FileEditDiff filePath={testFilePath} newString="content" />
      );
      const icon = container.querySelector("svg");
      expect(icon).toBeInTheDocument();
    });

    it("has max height with overflow scroll", () => {
      const { container } = render(
        <FileEditDiff filePath={testFilePath} newString="content" />
      );
      const scrollContainer = container.querySelector("[class*='max-h-']");
      expect(scrollContainer).toBeInTheDocument();
      expect(scrollContainer).toHaveClass("overflow-auto");
    });
  });

  describe("行号显示", () => {
    it("shows line numbers for diff view", () => {
      const { container } = render(
        <FileEditDiff
          filePath={testFilePath}
          oldString="old"
          newString="new"
        />
      );
      // 应该有行号列 (w-8 class)
      const lineNumberCells = container.querySelectorAll(".w-8");
      expect(lineNumberCells.length).toBeGreaterThan(0);
    });

    it("renders correct number of diff rows", () => {
      const { container } = render(
        <FileEditDiff
          filePath={testFilePath}
          newString={"line1\nline2"}
        />
      );
      // 2 行内容
      const rows = container.querySelectorAll("[class*='flex'][class*='font-mono'][class*='text-xs']");
      expect(rows.length).toBe(2);
    });
  });
});
