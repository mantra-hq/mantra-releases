/**
 * ProjectDrawer Tests
 * Story 2.18: Task 2.9
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ProjectDrawer } from "./ProjectDrawer";
import type { Project } from "@/types/project";

// Mock projects data
const mockProjects: Project[] = [
  {
    id: "proj-1",
    name: "mantra",
    cwd: "/home/user/projects/mantra",
    session_count: 3,
    created_at: "2024-01-01T00:00:00Z",
    last_activity: "2024-01-02T00:00:00Z",
    git_repo_path: "/home/user/projects/mantra",
    git_remote_url: "https://github.com/user/mantra.git",
    has_git_repo: true,
  },
  {
    id: "proj-2",
    name: "other-project",
    cwd: "/home/user/projects/other",
    session_count: 1,
    created_at: "2024-01-01T00:00:00Z",
    last_activity: "2024-01-01T12:00:00Z",
    git_repo_path: null,
    git_remote_url: null,
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
  },
  {
    id: "sess-2",
    source: "gemini" as const,
    created_at: "2024-01-01T12:00:00Z",
    updated_at: "2024-01-01T18:00:00Z",
    message_count: 5,
  },
];

describe("ProjectDrawer", () => {
  const defaultProps = {
    isOpen: true,
    onOpenChange: vi.fn(),
    projects: mockProjects,
    isLoading: false,
    currentSessionId: undefined,
    onSessionSelect: vi.fn(),
    onImportClick: vi.fn(),
    getProjectSessions: vi.fn().mockResolvedValue(mockSessions),
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

  it("displays project list", () => {
    render(<ProjectDrawer {...defaultProps} />);
    expect(screen.getByText("mantra")).toBeInTheDocument();
    expect(screen.getByText("other-project")).toBeInTheDocument();
  });

  it("shows empty state when no projects", () => {
    render(<ProjectDrawer {...defaultProps} projects={[]} />);
    expect(screen.getByTestId("project-drawer-empty")).toBeInTheDocument();
    expect(screen.getByText("还没有导入任何项目")).toBeInTheDocument();
  });

  it("shows loading state", () => {
    render(<ProjectDrawer {...defaultProps} projects={[]} isLoading={true} />);
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

  it("expands project and loads sessions on click", async () => {
    const getProjectSessions = vi.fn().mockResolvedValue(mockSessions);
    render(
      <ProjectDrawer {...defaultProps} getProjectSessions={getProjectSessions} />
    );

    // Click to expand first project
    fireEvent.click(screen.getByTestId("project-toggle-proj-1"));

    await waitFor(() => {
      expect(getProjectSessions).toHaveBeenCalledWith("proj-1");
    });
  });

  it("filters projects by search keyword", () => {
    render(<ProjectDrawer {...defaultProps} />);

    const searchInput = screen.getByTestId("drawer-search-input");
    fireEvent.change(searchInput, { target: { value: "mantra" } });

    expect(screen.getByText("mantra")).toBeInTheDocument();
    expect(screen.queryByText("other-project")).not.toBeInTheDocument();
  });
});
