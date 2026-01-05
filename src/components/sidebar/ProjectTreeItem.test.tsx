/**
 * ProjectTreeItem Tests
 * Story 2.18: Task 3.5
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ProjectTreeItem } from "./ProjectTreeItem";
import type { Project } from "@/types/project";

const mockProject: Project = {
  id: "proj-1",
  name: "test-project",
  cwd: "/home/user/test-project",
  session_count: 5,
  created_at: "2024-01-01T00:00:00Z",
  last_activity: "2024-01-02T00:00:00Z",
  git_repo_path: "/home/user/test-project",
  has_git_repo: true,
};

const mockSessions = [
  {
    id: "sess-1",
    source: "claude" as const,
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-02T00:00:00Z",
    message_count: 10,
    is_empty: false,
  },
];

describe("ProjectTreeItem", () => {
  const defaultProps = {
    project: mockProject,
    isExpanded: false,
    isLoading: false,
    sessions: [],
    currentSessionId: undefined,
    searchKeyword: undefined,
    onToggle: vi.fn(),
    onSessionSelect: vi.fn(),
  };

  it("renders project name and session count", () => {
    render(<ProjectTreeItem {...defaultProps} />);
    expect(screen.getByText("test-project")).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
  });

  it("calls onToggle when clicked", () => {
    const onToggle = vi.fn();
    render(<ProjectTreeItem {...defaultProps} onToggle={onToggle} />);

    fireEvent.click(screen.getByTestId("project-toggle-proj-1"));
    expect(onToggle).toHaveBeenCalled();
  });

  it("shows sessions when expanded", () => {
    render(
      <ProjectTreeItem
        {...defaultProps}
        isExpanded={true}
        sessions={mockSessions}
      />
    );
    expect(screen.getByTestId("session-tree-item-sess-1")).toBeInTheDocument();
  });

  it("shows loading state when loading", () => {
    render(
      <ProjectTreeItem {...defaultProps} isExpanded={true} isLoading={true} />
    );
    expect(screen.getByText("加载中...")).toBeInTheDocument();
  });

  it("shows empty message when no sessions", () => {
    render(
      <ProjectTreeItem {...defaultProps} isExpanded={true} sessions={[]} />
    );
    expect(screen.getByText("暂无会话")).toBeInTheDocument();
  });

  it("highlights search keyword in project name", () => {
    render(<ProjectTreeItem {...defaultProps} searchKeyword="test" />);
    const mark = document.querySelector("mark");
    expect(mark).toBeInTheDocument();
    expect(mark?.textContent).toBe("test");
  });

  it("shows settings button on hover when onSettingsClick provided", () => {
    const onSettingsClick = vi.fn();
    render(
      <ProjectTreeItem {...defaultProps} onSettingsClick={onSettingsClick} />
    );

    // Hover on the parent div container, not the button
    const container = screen.getByTestId("project-tree-item-proj-1").firstChild as HTMLElement;
    fireEvent.mouseEnter(container);

    expect(screen.getByTestId("project-settings-proj-1")).toBeInTheDocument();
  });
});
