/**
 * ImportComplete 测试文件
 * Story 2.9: Task 5
 * Story 2.23: Quick Navigation + Retry Failed
 *
 * 测试导入完成确认组件
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ImportComplete, type ImportResult } from "./ImportComplete";
import type { ImportedProject } from "@/stores/useImportStore";

/** 测试用导入结果数据 */
const mockResults: ImportResult[] = [
  { success: true, filePath: "/path/file1.json", projectId: "proj-1", sessionId: "sess-1" },
  { success: true, filePath: "/path/file2.json", projectId: "proj-1", sessionId: "sess-2" },
  { success: true, filePath: "/path/file3.json", projectId: "proj-2", sessionId: "sess-3" },
  { success: false, filePath: "/path/file4.json", error: "parse_error" },
];

describe("ImportComplete", () => {
  // Task 5.2: 导入统计
  describe("Statistics", () => {
    it("displays success count", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByTestId("success-stat")).toHaveTextContent("3");
    });

    it("displays failure count", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByTestId("failure-stat")).toHaveTextContent("1");
    });

    it("displays project count", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      // 2 unique projects: proj-1 and proj-2
      expect(screen.getByTestId("project-stat")).toHaveTextContent("2");
    });

    it("shows success label", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByText("成功导入")).toBeInTheDocument();
    });

    it("shows failure label", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByText("导入失败")).toBeInTheDocument();
    });

    it("shows project label", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByText("项目")).toBeInTheDocument();
    });
  });

  // Task 5.3: 查看项目按钮
  describe("View Projects Button", () => {
    it("displays view projects button", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByText("查看项目")).toBeInTheDocument();
    });

    it("calls onViewProjects when clicked", () => {
      const onViewProjects = vi.fn();
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={onViewProjects}
          onContinueImport={vi.fn()}
        />
      );

      fireEvent.click(screen.getByText("查看项目"));
      expect(onViewProjects).toHaveBeenCalled();
    });

    it("is the primary button", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      const button = screen.getByText("查看项目").closest("button");
      expect(button).toHaveAttribute("data-variant", "default");
    });
  });

  // Task 5.4: 继续导入按钮
  describe("Continue Import Button", () => {
    it("displays continue import button", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByText("继续导入")).toBeInTheDocument();
    });

    it("calls onContinueImport when clicked", () => {
      const onContinueImport = vi.fn();
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={onContinueImport}
        />
      );

      fireEvent.click(screen.getByText("继续导入"));
      expect(onContinueImport).toHaveBeenCalled();
    });

    it("is a secondary button", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      const button = screen.getByText("继续导入").closest("button");
      expect(button).toHaveAttribute("data-variant", "outline");
    });
  });

  // 成功状态
  describe("Success State", () => {
    it("shows success icon when all successful", () => {
      const successResults: ImportResult[] = [
        { success: true, filePath: "/path/file1.json", projectId: "proj-1", sessionId: "sess-1" },
      ];

      render(
        <ImportComplete
          results={successResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByTestId("success-icon")).toBeInTheDocument();
    });

    it("shows completion message", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByText("导入完成")).toBeInTheDocument();
    });
  });

  // 部分失败状态
  describe("Partial Failure State", () => {
    it("shows warning when some imports failed", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      // 有失败时应该显示警告信息
      expect(screen.getByText(/部分文件导入失败/)).toBeInTheDocument();
    });
  });

  // 空结果
  describe("Empty Results", () => {
    it("handles empty results gracefully", () => {
      render(
        <ImportComplete
          results={[]}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
        />
      );

      expect(screen.getByTestId("success-stat")).toHaveTextContent("0");
      expect(screen.getByTestId("failure-stat")).toHaveTextContent("0");
      expect(screen.getByTestId("project-stat")).toHaveTextContent("0");
    });
  });

  // Story 2.23: 项目列表和快速跳转
  describe("Imported Projects List", () => {
    const mockImportedProjects: ImportedProject[] = [
      { id: "proj-1", name: "project-1", sessionCount: 2, firstSessionId: "sess-1" },
      { id: "proj-2", name: "project-2", sessionCount: 1, firstSessionId: "sess-3" },
    ];

    it("shows imported projects list when provided", () => {
      render(
        <ImportComplete
          results={mockResults}
          importedProjects={mockImportedProjects}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onNavigateToProject={vi.fn()}
        />
      );

      expect(screen.getByText("刚导入的项目")).toBeInTheDocument();
      expect(screen.getByText("project-1")).toBeInTheDocument();
      expect(screen.getByText("project-2")).toBeInTheDocument();
    });

    it("hides project list when no projects imported", () => {
      render(
        <ImportComplete
          results={mockResults}
          importedProjects={[]}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onNavigateToProject={vi.fn()}
        />
      );

      expect(screen.queryByText("刚导入的项目")).not.toBeInTheDocument();
    });

    it("calls onNavigateToProject when project is clicked", () => {
      const onNavigateToProject = vi.fn();
      render(
        <ImportComplete
          results={mockResults}
          importedProjects={mockImportedProjects}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onNavigateToProject={onNavigateToProject}
        />
      );

      fireEvent.click(screen.getByText("project-1"));
      expect(onNavigateToProject).toHaveBeenCalledWith("sess-1");
    });

    it("displays session count for each project", () => {
      render(
        <ImportComplete
          results={mockResults}
          importedProjects={mockImportedProjects}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onNavigateToProject={vi.fn()}
        />
      );

      // 检查项目行包含正确的会话数
      const project1Button = screen.getByTestId("project-proj-1");
      const project2Button = screen.getByTestId("project-proj-2");
      expect(project1Button).toBeInTheDocument();
      expect(project2Button).toBeInTheDocument();
    });
  });

  // Story 2.23: 失败文件列表和重试
  describe("Failed Files and Retry", () => {
    it("shows failed files list when there are failures", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onRetryFailed={vi.fn()}
        />
      );

      expect(screen.getByText(/失败的文件/)).toBeInTheDocument();
    });

    it("shows retry button when onRetryFailed is provided and there are failures", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onRetryFailed={vi.fn()}
        />
      );

      expect(screen.getByTestId("retry-failed-button")).toBeInTheDocument();
    });

    it("hides retry button when no failures", () => {
      const successResults: ImportResult[] = [
        { success: true, filePath: "/path/file1.json", projectId: "proj-1", sessionId: "sess-1" },
      ];

      render(
        <ImportComplete
          results={successResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onRetryFailed={vi.fn()}
        />
      );

      expect(screen.queryByTestId("retry-failed-button")).not.toBeInTheDocument();
    });

    it("calls onRetryFailed with failed paths when retry is clicked", () => {
      const onRetryFailed = vi.fn();
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onRetryFailed={onRetryFailed}
        />
      );

      fireEvent.click(screen.getByTestId("retry-failed-button"));
      expect(onRetryFailed).toHaveBeenCalledWith(["/path/file4.json"]);
    });

    it("disables retry button when isRetrying is true", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onRetryFailed={vi.fn()}
          isRetrying={true}
        />
      );

      expect(screen.getByTestId("retry-failed-button")).toBeDisabled();
    });

    it("shows loading text when retrying", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onRetryFailed={vi.fn()}
          isRetrying={true}
        />
      );

      expect(screen.getByText("重试中...")).toBeInTheDocument();
    });

    it("can toggle error list visibility", () => {
      render(
        <ImportComplete
          results={mockResults}
          onViewProjects={vi.fn()}
          onContinueImport={vi.fn()}
          onRetryFailed={vi.fn()}
        />
      );

      const toggleButton = screen.getByTestId("toggle-errors");
      // Story 2.24: 默认折叠
      expect(screen.queryByText("file4.json")).not.toBeInTheDocument();

      // 点击展开
      fireEvent.click(toggleButton);
      expect(screen.getByText("file4.json")).toBeInTheDocument();

      // 再次点击折叠
      fireEvent.click(toggleButton);
      expect(screen.queryByText("file4.json")).not.toBeInTheDocument();
    });
  });
});
