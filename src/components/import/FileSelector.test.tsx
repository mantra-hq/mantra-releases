/**
 * FileSelector 测试文件
 * Story 2.9: Task 3 + UX Redesign
 *
 * 测试文件选择组件（项目分组版本）
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { FileSelector, type DiscoveredFile } from "./FileSelector";

/** 默认 props */
const defaultProps = {
  files: [] as DiscoveredFile[],
  selectedFiles: new Set<string>(),
  expandedProjects: new Set<string>(),
  searchQuery: "",
  onScan: vi.fn(),
  onSelectFiles: vi.fn(),
  onToggleFile: vi.fn(),
  onSelectAll: vi.fn(),
  onClearAll: vi.fn(),
  onInvertSelection: vi.fn(),
  onToggleProject: vi.fn(),
  onToggleProjectExpand: vi.fn(),
  onSearchChange: vi.fn(),
  loading: false,
  skipEmptySessions: true,
  onSkipEmptySessionsChange: vi.fn(),
};

/** 测试用文件数据 - 分属不同项目 */
const mockFiles: DiscoveredFile[] = [
  {
    path: "/home/user/.claude/projects/proj1/conversation_123.json",
    name: "conversation_123.json",
    size: 1024,
    modifiedAt: Date.now() - 3600000, // 1 小时前
    projectPath: "/home/user/myproject",
  },
  {
    path: "/home/user/.claude/projects/proj1/conversation_124.json",
    name: "conversation_124.json",
    size: 512,
    modifiedAt: Date.now() - 7200000, // 2 小时前
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
      render(<FileSelector {...defaultProps} />);
      expect(screen.getByText("扫描默认路径")).toBeInTheDocument();
    });

    it("shows select directory button", () => {
      render(<FileSelector {...defaultProps} />);
      expect(screen.getByText("手动选择目录")).toBeInTheDocument();
    });

    it("calls onScan when scan button is clicked", () => {
      const onScan = vi.fn();
      render(<FileSelector {...defaultProps} onScan={onScan} />);
      fireEvent.click(screen.getByText("扫描默认路径"));
      expect(onScan).toHaveBeenCalled();
    });

    it("calls onSelectFiles when select button is clicked", () => {
      const onSelectFiles = vi.fn();
      render(<FileSelector {...defaultProps} onSelectFiles={onSelectFiles} />);
      fireEvent.click(screen.getByText("手动选择目录"));
      expect(onSelectFiles).toHaveBeenCalled();
    });
  });

  // 空状态
  describe("Empty State", () => {
    it("shows empty state when no files", () => {
      render(<FileSelector {...defaultProps} />);
      expect(screen.getByText("暂无文件")).toBeInTheDocument();
    });
  });

  // 加载状态
  describe("Loading State", () => {
    it("shows loading indicator when loading", () => {
      render(<FileSelector {...defaultProps} loading={true} />);
      expect(screen.getByTestId("file-selector-loading")).toBeInTheDocument();
    });

    it("disables buttons when loading", () => {
      render(<FileSelector {...defaultProps} loading={true} />);
      expect(screen.getByText("扫描默认路径").closest("button")).toBeDisabled();
      expect(screen.getByText("手动选择目录").closest("button")).toBeDisabled();
    });
  });

  // 项目分组显示
  describe("Project Grouping", () => {
    it("displays project groups", () => {
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
        />
      );
      // 应显示两个项目
      expect(screen.getByText("myproject")).toBeInTheDocument();
      expect(screen.getByText("another-project")).toBeInTheDocument();
    });

    it("shows session count per project", () => {
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
        />
      );
      // myproject 有 2 个会话，another-project 有 1 个会话
      expect(screen.getByText("2 个会话")).toBeInTheDocument();
      expect(screen.getByText("1 个会话")).toBeInTheDocument();
    });
  });

  // 搜索过滤
  describe("Search Filter", () => {
    it("renders search input", () => {
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
        />
      );
      expect(screen.getByRole("searchbox")).toBeInTheDocument();
    });

    it("calls onSearchChange when search input changes", () => {
      const onSearchChange = vi.fn();
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onSearchChange={onSearchChange}
        />
      );
      fireEvent.change(screen.getByRole("searchbox"), {
        target: { value: "myproject" },
      });
      expect(onSearchChange).toHaveBeenCalledWith("myproject");
    });
  });

  // 统计栏
  describe("Selection Stats", () => {
    it("shows project count", () => {
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
        />
      );
      expect(screen.getByText("2 个项目")).toBeInTheDocument();
    });

    it("shows session count", () => {
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
        />
      );
      expect(screen.getByText("3 个会话")).toBeInTheDocument();
    });

    it("shows selected count", () => {
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set([mockFiles[0].path])}
        />
      );
      expect(screen.getAllByText(/已选.*1/).length).toBeGreaterThan(0);
    });
  });

  // 项目选择
  describe("Project Selection", () => {
    it("shows checkbox for each project", () => {
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
        />
      );
      expect(
        screen.getByTestId(`project-checkbox-${mockFiles[0].projectPath}`)
      ).toBeInTheDocument();
      expect(
        screen.getByTestId(`project-checkbox-${mockFiles[2].projectPath}`)
      ).toBeInTheDocument();
    });

    it("calls onToggleProject when project checkbox is clicked", () => {
      const onToggleProject = vi.fn();
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          onToggleProject={onToggleProject}
        />
      );

      fireEvent.click(
        screen.getByTestId(`project-checkbox-${mockFiles[0].projectPath}`)
      );
      expect(onToggleProject).toHaveBeenCalledWith(mockFiles[0].projectPath);
    });
  });

  // 项目展开/折叠
  describe("Project Expand/Collapse", () => {
    it("shows sessions when project is expanded", () => {
      render(
        <FileSelector
          {...defaultProps}
          files={mockFiles}
          selectedFiles={new Set(mockFiles.map((f) => f.path))}
          expandedProjects={new Set([mockFiles[0].projectPath])}
        />
      );
      // myproject 展开后应显示其会话
      expect(screen.getByText("conversation_123.json")).toBeInTheDocument();
      expect(screen.getByText("conversation_124.json")).toBeInTheDocument();
    });
  });
});
