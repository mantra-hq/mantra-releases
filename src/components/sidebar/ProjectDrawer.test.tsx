/**
 * ProjectDrawer Tests
 * Story 2.18: Task 2.9
 * Story 1.12: Phase 5 - 更新为逻辑项目视图接口
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ProjectDrawer } from "./ProjectDrawer";
import type { LogicalProjectStats } from "@/types/project";

// Mock logical projects data (Story 1.12)
const mockLogicalProjects: LogicalProjectStats[] = [
  {
    physical_path: "/home/user/projects/mantra",
    project_count: 1,
    project_ids: ["proj-1"],
    total_sessions: 3,
    last_activity: "2024-01-02T00:00:00Z",
    display_name: "mantra",
    path_type: "local",
    path_exists: true,
    needs_association: false,
    has_git_repo: true,
  },
  {
    physical_path: "/home/user/projects/other",
    project_count: 1,
    project_ids: ["proj-2"],
    total_sessions: 1,
    last_activity: "2024-01-01T12:00:00Z",
    display_name: "other-project",
    path_type: "local",
    path_exists: true,
    needs_association: false,
    has_git_repo: false,
  },
];

const mockSessions = [
  {
    id: "sess-1",
    source: "claude" as const,
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-02T00:00:00Z",
    message_count: 10,
    is_empty: false,
  },
  {
    id: "sess-2",
    source: "gemini" as const,
    created_at: "2024-01-01T12:00:00Z",
    updated_at: "2024-01-01T18:00:00Z",
    message_count: 5,
    is_empty: false,
  },
];

describe("ProjectDrawer", () => {
  const defaultProps = {
    isOpen: true,
    onOpenChange: vi.fn(),
    logicalProjects: mockLogicalProjects,
    isLoading: false,
    currentSessionId: undefined,
    onSessionSelect: vi.fn(),
    onImportClick: vi.fn(),
    getLogicalProjectSessions: vi.fn().mockResolvedValue(mockSessions),
  };

  it("renders drawer when open", () => {
    render(<ProjectDrawer {...defaultProps} />);
    expect(screen.getByTestId("project-drawer")).toBeInTheDocument();
    expect(screen.getByText("我的项目")).toBeInTheDocument();
  });

  it("does not render content when closed", () => {
    render(<ProjectDrawer {...defaultProps} isOpen={false} />);
    expect(screen.queryByText("我的项目")).not.toBeInTheDocument();
  });

  it("displays logical project list", () => {
    render(<ProjectDrawer {...defaultProps} />);
    expect(screen.getByText("mantra")).toBeInTheDocument();
    expect(screen.getByText("other-project")).toBeInTheDocument();
  });

  it("shows empty state when no projects", () => {
    render(<ProjectDrawer {...defaultProps} logicalProjects={[]} />);
    expect(screen.getByTestId("project-drawer-empty")).toBeInTheDocument();
    expect(screen.getByText("还没有导入任何项目")).toBeInTheDocument();
  });

  it("shows loading state", () => {
    render(<ProjectDrawer {...defaultProps} logicalProjects={[]} isLoading={true} />);
    expect(screen.getByText("加载中...")).toBeInTheDocument();
  });

  it("shows import button at bottom", () => {
    render(<ProjectDrawer {...defaultProps} />);
    expect(screen.getByTestId("project-drawer-import-button")).toBeInTheDocument();
  });

  it("calls onImportClick when import button clicked", () => {
    const onImportClick = vi.fn();
    render(<ProjectDrawer {...defaultProps} onImportClick={onImportClick} />);

    fireEvent.click(screen.getByTestId("project-drawer-import-button"));
    expect(onImportClick).toHaveBeenCalled();
  });

  it("expands logical project and loads sessions on click", async () => {
    const getLogicalProjectSessions = vi.fn().mockResolvedValue(mockSessions);
    render(
      <ProjectDrawer {...defaultProps} getLogicalProjectSessions={getLogicalProjectSessions} />
    );

    // Click to expand first logical project (using physical path as key)
    const toggleButton = screen.getByTestId("logical-project-toggle--home-user-projects-mantra");
    fireEvent.click(toggleButton);

    await waitFor(() => {
      expect(getLogicalProjectSessions).toHaveBeenCalledWith("/home/user/projects/mantra");
    });
  });

  it("filters logical projects by search keyword", () => {
    render(<ProjectDrawer {...defaultProps} />);

    const searchInput = screen.getByTestId("drawer-search-input");
    fireEvent.change(searchInput, { target: { value: "mantra" } });

    expect(screen.getByText("mantra")).toBeInTheDocument();
    expect(screen.queryByText("other-project")).not.toBeInTheDocument();
  });
});
