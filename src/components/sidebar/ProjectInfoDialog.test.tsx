/**
 * ProjectInfoDialog Tests
 * Story 2.27: 项目元信息对话框
 * Story 1.9: Task 8.9 - 设置工作目录功能测试
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { ProjectInfoDialog } from "./ProjectInfoDialog";
import type { Project } from "@/types/project";

// Helper to render with router context
const renderWithRouter = (ui: React.ReactElement) => {
    return render(
        <MemoryRouter>{ui}</MemoryRouter>
    );
};

// Mock Tauri dialog API
vi.mock("@tauri-apps/plugin-dialog", () => ({
    open: vi.fn(),
}));

// Mock project-ipc
vi.mock("@/lib/project-ipc", () => ({
    updateProjectCwd: vi.fn(),
}));

// Mock sonner toast
vi.mock("sonner", () => ({
    toast: {
        success: vi.fn(),
        error: vi.fn(),
    },
}));

// Mock useProjects hook - Story 1.12 paths management
vi.mock("@/hooks/useProjects", () => ({
    useProjectPaths: vi.fn(() => ({
        paths: [],
        isLoading: false,
        refetch: vi.fn(),
    })),
    addProjectPath: vi.fn(() => Promise.resolve()),
    removeProjectPath: vi.fn(() => Promise.resolve()),
    getProjectPaths: vi.fn(() => Promise.resolve([])),
    getProjectsByPhysicalPath: vi.fn(() => Promise.resolve([])),
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => {
            const translations: Record<string, string> = {
                "projectInfo.paths": "项目路径",
                "projectInfo.invalidCwdWarning": "无法识别的路径格式，请手动设置正确的工作目录",
                "projectInfo.changePath": "更换路径",
                "projectInfo.associatePath": "关联真实路径",
                "projectInfo.aggregatedSources": "聚合来源",
                "projectInfo.sessions": "会话",
                "projectInfo.createdAt": "创建时间",
                "projectInfo.lastActivity": "最后活动",
                "projectInfo.gitRemoteUrl": "Git 仓库 URL",
                "projectInfo.gitPath": "Git 仓库根目录",
                "projectInfo.invalidCwdTitle": "无法识别的路径",
                "projectInfo.description": "项目详细信息",
                "common.loading": "加载中",
            };
            return translations[key] || fallback || key;
        },
        i18n: { language: "zh-CN" },
    }),
}));

// Radix UI PointerEvent polyfill
beforeAll(() => {
    class MockPointerEvent extends MouseEvent {
        constructor(type: string, props: PointerEventInit = {}) {
            super(type, props);
            Object.assign(this, {
                pointerId: props.pointerId ?? 0,
                width: props.width ?? 1,
                height: props.height ?? 1,
                pressure: props.pressure ?? 0,
                tangentialPressure: props.tangentialPressure ?? 0,
                tiltX: props.tiltX ?? 0,
                tiltY: props.tiltY ?? 0,
                twist: props.twist ?? 0,
                pointerType: props.pointerType ?? "mouse",
                isPrimary: props.isPrimary ?? true,
            });
        }
    }
    window.PointerEvent = MockPointerEvent as unknown as typeof PointerEvent;
    window.HTMLElement.prototype.scrollIntoView = vi.fn();
    window.HTMLElement.prototype.hasPointerCapture = vi.fn();
    window.HTMLElement.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => {
    cleanup();
    vi.clearAllMocks();
});

const createMockProject = (overrides: Partial<Project> = {}): Project => ({
    id: "test-project-id",
    name: "test-project",
    cwd: "/home/user/test-project",
    session_count: 5,
    created_at: "2026-01-01T00:00:00Z",
    last_activity: "2026-01-06T00:00:00Z",
    git_repo_path: null,
    has_git_repo: false,
    git_remote_url: null,
    ...overrides,
});

const defaultProps = {
    isOpen: true,
    onOpenChange: vi.fn(),
    getProjectSessions: vi.fn().mockResolvedValue([]),
    onProjectUpdated: vi.fn(),
};

describe("ProjectInfoDialog", () => {
    describe("Dialog rendering", () => {
        it("renders dialog when open with project", async () => {
            const project = createMockProject();
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByRole("dialog")).toBeInTheDocument();
            });
        });

        it("does not render when closed", () => {
            const project = createMockProject();
            renderWithRouter(<ProjectInfoDialog {...defaultProps} isOpen={false} project={project} />);

            expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
        });

        it("does not render when project is null", () => {
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={null} />);

            expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
        });
    });

    describe("Project name display (Task 8.7)", () => {
        it("shows project name in title", async () => {
            const project = createMockProject({ name: "my-project" });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByText("my-project")).toBeInTheDocument();
            });
        });

        it("truncates long project name with ellipsis", async () => {
            const longName = "a".repeat(50);
            const project = createMockProject({ name: longName });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                // Should show truncated name with "..."
                expect(screen.getByText(/^a{30}\.\.\.$/)).toBeInTheDocument();
            });
        });
    });

    describe("CWD path display (Task 8.8)", () => {
        it("shows project cwd path", async () => {
            const project = createMockProject({ cwd: "/home/user/project" });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByText(/项目路径/)).toBeInTheDocument();
            });
        });

        it("truncates long cwd path with ellipsis", async () => {
            const longPath = "/home/user/" + "a".repeat(60);
            const project = createMockProject({ cwd: longPath });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                // Path should be truncated
                const pathElements = screen.getAllByText(/\/home\/user\//);
                expect(pathElements.length).toBeGreaterThan(0);
            });
        });
    });

    describe("Invalid CWD warning (Task 8.6)", () => {
        it("shows warning for gemini-project: placeholder", async () => {
            const project = createMockProject({ cwd: "gemini-project:abc123" });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByText(/无法识别的路径/)).toBeInTheDocument();
            });
        });

        it("shows warning for empty cwd", async () => {
            const project = createMockProject({ cwd: "" });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByText(/无法识别的路径/)).toBeInTheDocument();
            });
        });

        it("shows warning for unknown cwd", async () => {
            const project = createMockProject({ cwd: "unknown" });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByText(/无法识别的路径/)).toBeInTheDocument();
            });
        });

        it("does not show warning for valid cwd", async () => {
            const project = createMockProject({ cwd: "/home/user/valid-project" });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.queryByText(/无法识别的路径/)).not.toBeInTheDocument();
            });
        });
    });

    describe("Change path button (Task 8.4, 8.5, Story 1.12)", () => {
        it("shows change path button", async () => {
            const project = createMockProject();
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                // Button should be present (look for the FolderEdit icon button with title="更换路径")
                const buttons = screen.getAllByRole("button");
                const changePathButton = buttons.find(
                    (btn) => btn.getAttribute("title") === "更换路径"
                );
                expect(changePathButton).toBeTruthy();
            });
        });

        it("opens directory picker when clicked", async () => {
            const { open } = await import("@tauri-apps/plugin-dialog");
            const mockOpen = open as ReturnType<typeof vi.fn>;
            mockOpen.mockResolvedValue("/new/path");

            const project = createMockProject();
            const user = userEvent.setup();
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                const buttons = screen.getAllByRole("button");
                const changePathButton = buttons.find(
                    (btn) => btn.getAttribute("title") === "更换路径"
                );
                expect(changePathButton).toBeTruthy();
            });

            const buttons = screen.getAllByRole("button");
            const changePathButton = buttons.find(
                (btn) => btn.getAttribute("title") === "更换路径"
            )!;

            await user.click(changePathButton);

            expect(mockOpen).toHaveBeenCalledWith({
                directory: true,
                multiple: false,
                title: expect.any(String),
            });
        });
    });

    describe("Session info display", () => {
        it("loads sessions when dialog opens", async () => {
            const mockGetSessions = vi.fn().mockResolvedValue([
                { id: "s1", source: "claude", message_count: 10 },
                { id: "s2", source: "gemini", message_count: 5 },
            ]);

            const project = createMockProject();
            renderWithRouter(
                <ProjectInfoDialog
                    {...defaultProps}
                    project={project}
                    getProjectSessions={mockGetSessions}
                />
            );

            await waitFor(() => {
                expect(mockGetSessions).toHaveBeenCalledWith(project.id);
            });
        });

        it("shows aggregated sources section with session count", async () => {
            // Story 1.12 V10: session count is now shown in "聚合来源" section
            const mockGetSessions = vi.fn().mockResolvedValue([
                { id: "s1", source: "claude", message_count: 10 },
            ]);
            const project = createMockProject({ session_count: 10 });
            renderWithRouter(
                <ProjectInfoDialog
                    {...defaultProps}
                    project={project}
                    getProjectSessions={mockGetSessions}
                />
            );

            await waitFor(() => {
                // Story 1.12 V10: Session info is shown in "聚合来源" section
                expect(screen.getByText(/聚合来源/)).toBeInTheDocument();
            });
        });
    });

    describe("Date display", () => {
        it("shows created date", async () => {
            const project = createMockProject({ created_at: "2026-01-01T00:00:00Z" });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByText(/创建时间/)).toBeInTheDocument();
            });
        });

        it("shows last activity date", async () => {
            const project = createMockProject({ last_activity: "2026-01-06T00:00:00Z" });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByText(/最后活动/)).toBeInTheDocument();
            });
        });
    });

    describe("Git info display", () => {
        it("shows git remote url when available (Story 1.9)", async () => {
            const project = createMockProject({
                has_git_repo: true,
                git_repo_path: "/home/user/test-project",
                git_remote_url: "https://github.com/user/repo",
            });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.getByText(/Git 仓库 URL/)).toBeInTheDocument();
                expect(screen.getByText("https://github.com/user/repo")).toBeInTheDocument();
            });
        });

        it("does not show git repo path when same as cwd", async () => {
            const project = createMockProject({
                cwd: "/home/user/project",
                has_git_repo: true,
                git_repo_path: "/home/user/project", // same as cwd
                git_remote_url: "https://github.com/user/repo",
            });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                // Should show git remote url
                expect(screen.getByText(/Git 仓库 URL/)).toBeInTheDocument();
                // Should NOT show git repo path (it's same as cwd)
                expect(screen.queryByText(/Git 仓库根目录/)).not.toBeInTheDocument();
            });
        });

        it("shows git repo path when different from cwd (subdirectory case)", async () => {
            const project = createMockProject({
                cwd: "/home/user/project/packages/client", // subdirectory
                has_git_repo: true,
                git_repo_path: "/home/user/project", // git root is parent
                git_remote_url: "https://github.com/user/repo",
            });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                // Should show git remote url
                expect(screen.getByText(/Git 仓库 URL/)).toBeInTheDocument();
                expect(screen.getByText("https://github.com/user/repo")).toBeInTheDocument();

                // Should also show git repo path when different from cwd
                // The InfoRow title attribute contains the path value
                const pathElement = screen.queryByTitle("/home/user/project");
                expect(pathElement).toBeInTheDocument();
            });
        });

        it("does not show git info when no repo", async () => {
            const project = createMockProject({
                has_git_repo: false,
                git_repo_path: null,
                git_remote_url: null,
            });
            renderWithRouter(<ProjectInfoDialog {...defaultProps} project={project} />);

            await waitFor(() => {
                expect(screen.queryByText(/Git 仓库 URL/)).not.toBeInTheDocument();
                expect(screen.queryByText(/Git 仓库根目录/)).not.toBeInTheDocument();
            });
        });
    });
});
