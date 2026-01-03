/**
 * ProjectGroupItem 测试文件
 * Story 2.9: UX Redesign
 * Story 2.20: Import Status Enhancement
 *
 * 测试项目分组卡片组件
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ProjectGroupItem, type ProjectGroupItemProps } from "./ProjectGroupItem";
import type { ProjectGroup, ProjectSelectionState } from "@/types/import";
import type { DiscoveredFile } from "./FileSelector";

/** 测试用会话数据 */
const mockSessions: DiscoveredFile[] = [
  {
    path: "/home/user/.claude/projects/proj1/conversation_123.json",
    name: "conversation_123.json",
    size: 1024,
    modifiedAt: Date.now() - 3600000,
    projectPath: "/home/user/myproject",
  },
  {
    path: "/home/user/.claude/projects/proj1/conversation_124.json",
    name: "conversation_124.json",
    size: 512,
    modifiedAt: Date.now() - 7200000,
    projectPath: "/home/user/myproject",
  },
];

/** 测试用项目分组 */
const mockGroup: ProjectGroup = {
  projectPath: "/home/user/myproject",
  projectName: "myproject",
  sessions: mockSessions,
};

/** 默认选择状态 */
const defaultSelectionState: ProjectSelectionState = {
  isSelected: false,
  isPartiallySelected: false,
  selectedCount: 0,
};

/** 默认 props */
const defaultProps: ProjectGroupItemProps = {
  group: mockGroup,
  selectionState: defaultSelectionState,
  isExpanded: false,
  selectedFiles: new Set<string>(),
  onToggleProject: vi.fn(),
  onToggleExpand: vi.fn(),
  onToggleSession: vi.fn(),
};

describe("ProjectGroupItem", () => {
  // 基础渲染
  describe("Basic Rendering", () => {
    it("renders project name", () => {
      render(<ProjectGroupItem {...defaultProps} />);
      expect(screen.getByText("myproject")).toBeInTheDocument();
    });

    it("renders session count", () => {
      render(<ProjectGroupItem {...defaultProps} />);
      expect(screen.getByText("2 个会话")).toBeInTheDocument();
    });

    it("renders checkbox", () => {
      render(<ProjectGroupItem {...defaultProps} />);
      expect(
        screen.getByTestId(`project-checkbox-${mockGroup.projectPath}`)
      ).toBeInTheDocument();
    });
  });

  // Story 2.20: 导入状态测试
  describe("Import Status (Story 2.20)", () => {
    it("renders 'NEW' badge when importStatus is 'new'", () => {
      render(<ProjectGroupItem {...defaultProps} importStatus="new" />);
      expect(
        screen.getByTestId(`import-badge-new-${mockGroup.projectPath}`)
      ).toBeInTheDocument();
      expect(screen.getByText("NEW")).toBeInTheDocument();
    });

    it("renders '已导入' badge when importStatus is 'imported'", () => {
      render(<ProjectGroupItem {...defaultProps} importStatus="imported" />);
      expect(
        screen.getByTestId(`import-badge-imported-${mockGroup.projectPath}`)
      ).toBeInTheDocument();
      expect(screen.getByText("已导入")).toBeInTheDocument();
    });

    it("does not render badge when importStatus is undefined", () => {
      render(<ProjectGroupItem {...defaultProps} />);
      expect(
        screen.queryByTestId(`import-badge-new-${mockGroup.projectPath}`)
      ).not.toBeInTheDocument();
      expect(
        screen.queryByTestId(`import-badge-imported-${mockGroup.projectPath}`)
      ).not.toBeInTheDocument();
    });

    it("disables checkbox when importStatus is 'imported'", () => {
      render(<ProjectGroupItem {...defaultProps} importStatus="imported" />);
      const checkbox = screen.getByTestId(
        `project-checkbox-${mockGroup.projectPath}`
      );
      expect(checkbox).toBeDisabled();
    });

    it("enables checkbox when importStatus is 'new'", () => {
      render(<ProjectGroupItem {...defaultProps} importStatus="new" />);
      const checkbox = screen.getByTestId(
        `project-checkbox-${mockGroup.projectPath}`
      );
      expect(checkbox).not.toBeDisabled();
    });

    it("applies muted styling when imported", () => {
      render(<ProjectGroupItem {...defaultProps} importStatus="imported" />);
      const projectName = screen.getByText("myproject");
      expect(projectName).toHaveClass("text-muted-foreground");
    });
  });

  // 选择交互
  describe("Selection Interaction", () => {
    it("calls onToggleProject when checkbox is clicked", () => {
      const onToggleProject = vi.fn();
      render(
        <ProjectGroupItem {...defaultProps} onToggleProject={onToggleProject} />
      );
      fireEvent.click(
        screen.getByTestId(`project-checkbox-${mockGroup.projectPath}`)
      );
      expect(onToggleProject).toHaveBeenCalled();
    });

    it("does not call onToggleProject when checkbox is disabled (imported)", () => {
      const onToggleProject = vi.fn();
      render(
        <ProjectGroupItem
          {...defaultProps}
          onToggleProject={onToggleProject}
          importStatus="imported"
        />
      );
      fireEvent.click(
        screen.getByTestId(`project-checkbox-${mockGroup.projectPath}`)
      );
      // disabled checkbox should not trigger the callback
      expect(onToggleProject).not.toHaveBeenCalled();
    });
  });

  // 展开/折叠
  describe("Expand/Collapse", () => {
    it("shows sessions when expanded", () => {
      render(<ProjectGroupItem {...defaultProps} isExpanded={true} />);
      expect(screen.getByText("conversation_123.json")).toBeInTheDocument();
      expect(screen.getByText("conversation_124.json")).toBeInTheDocument();
    });

    it("hides sessions when collapsed", () => {
      render(<ProjectGroupItem {...defaultProps} isExpanded={false} />);
      // Sessions should not be visible when collapsed
      expect(
        screen.queryByText("conversation_123.json")
      ).not.toBeInTheDocument();
    });
  });
});
