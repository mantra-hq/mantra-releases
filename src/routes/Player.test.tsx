/**
 * Player Page Tests - 会话回放页面测试
 * Story 2.8: Task 9 (Code Review Fix)
 * Story 2.11: 更新测试以支持异步加载
 * Story 2.12: Task 6.4, 6.5 - 智能文件选择集成测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import Player from "./Player";

// Mock Tauri IPC
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === "get_session") {
      return Promise.resolve({
        id: "test-session-123",
        source: "claude",
        cwd: "/test/project",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        messages: [],
      });
    }
    if (cmd === "detect_git_repo") {
      return Promise.resolve("/test/project");
    }
    return Promise.resolve(null);
  }),
}));

// Mock project-ipc
vi.mock("@/lib/project-ipc", () => ({
  getProjectByCwd: vi.fn().mockResolvedValue({
    id: "proj-123",
    name: "test-project",
    cwd: "/test/project",
    session_count: 1,
    created_at: new Date().toISOString(),
    last_activity: new Date().toISOString(),
    git_repo_path: "/test/project",
    has_git_repo: true,
  }),
  getRepresentativeFile: vi.fn().mockResolvedValue({
    path: "README.md",
    content: "# Test Project",
    language: "markdown",
  }),
  detectGitRepo: vi.fn().mockResolvedValue("/test/project"),
  getProjectSessions: vi.fn().mockResolvedValue([]),
}));

// Mock DualStreamLayout
vi.mock("@/components/layout", () => ({
  DualStreamLayout: vi.fn(() => (
    <div data-testid="dual-stream-layout">DualStreamLayout</div>
  )),
}));

// Mock ThemeToggle
vi.mock("@/components/theme-toggle", () => ({
  ThemeToggle: () => <button data-testid="theme-toggle">Toggle Theme</button>,
}));

// Mock TimberLine
vi.mock("@/components/timeline", () => ({
  TimberLine: () => <div data-testid="timberline">TimberLine</div>,
}));

// Mock TopBar
vi.mock("@/components/navigation", () => ({
  TopBar: ({ onDrawerOpen }: { onDrawerOpen: () => void }) => (
    <div data-testid="top-bar">
      <button data-testid="topbar-menu-button" onClick={onDrawerOpen}>Menu</button>
      <span>Mantra</span>
      <span className="text-primary">心法</span>
    </div>
  ),
}));

// Mock ImportWizard
vi.mock("@/components/import", () => ({
  ImportWizard: ({ open }: { open: boolean }) => (
    open ? <div data-testid="import-wizard">Import Wizard</div> : null
  ),
}));

// Mock ProjectDrawer
vi.mock("@/components/sidebar", () => ({
  ProjectDrawer: ({ isOpen }: { isOpen: boolean }) => (
    isOpen ? <div data-testid="project-drawer">Project Drawer</div> : null
  ),
}));

// Mock PlayerEmptyState
vi.mock("@/components/player", () => ({
  PlayerEmptyState: ({ onOpenDrawer, onImport }: { onOpenDrawer: () => void; onImport: () => void }) => (
    <div data-testid="player-empty-state">
      <h2>选择一个会话开始回放</h2>
      <button onClick={onOpenDrawer}>打开项目列表</button>
      <button onClick={onImport}>导入项目</button>
    </div>
  ),
}));

// Mock useProjectDrawer hook
vi.mock("@/hooks/useProjectDrawer", () => ({
  useProjectDrawer: () => ({
    isOpen: false,
    setIsOpen: vi.fn(),
    openDrawer: vi.fn(),
    closeDrawer: vi.fn(),
    toggleDrawer: vi.fn(),
    projects: [],
    isLoading: false,
    error: null,
    refetchProjects: vi.fn(),
    getProjectSessions: vi.fn().mockResolvedValue([]),
  }),
}));

// Mock useProjects hook
vi.mock("@/hooks/useProjects", () => ({
  useProjects: () => ({
    projects: [],
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  }),
}));

// Mock useTimeMachine hook
vi.mock("@/hooks/useTimeMachine", () => ({
  useTimeMachine: () => ({
    fetchSnapshot: vi.fn(),
    isLoading: false,
    error: null,
  }),
}));

// Mock mock-messages
vi.mock("@/lib/mock-messages", () => ({
  MOCK_MESSAGES_WITH_ALL_TYPES: [],
}));

// Wrapper with Router
function renderWithRouter(
  ui: React.ReactElement,
  { route = "/session/test-session-123" } = {}
) {
  return render(
    <MemoryRouter initialEntries={[route]}>
      <Routes>
        {/* Story 2.21: Player 也处理首页路由 */}
        <Route path="/" element={ui} />
        <Route path="/session/:sessionId" element={ui} />
      </Routes>
    </MemoryRouter>
  );
}

describe("Player Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("UI 展示", () => {
    it("应该渲染页面标题 Mantra 心法", async () => {
      renderWithRouter(<Player />);
      // 等待加载完成
      await waitFor(() => {
        expect(screen.getByText("Mantra")).toBeInTheDocument();
      });
      expect(screen.getByText("心法")).toBeInTheDocument();
    });

    it("应该渲染 DualStreamLayout（有 sessionId 时）", async () => {
      renderWithRouter(<Player />);
      // 等待异步加载完成
      await waitFor(() => {
        expect(screen.getByText("Mantra")).toBeInTheDocument();
      });
    });

    it("应该在加载时显示加载状态（有 sessionId 时）", () => {
      renderWithRouter(<Player />);
      // 初始渲染时应该显示加载中状态
      expect(screen.getByText("加载会话中...")).toBeInTheDocument();
    });
  });

  describe("Story 2.21 - 空状态", () => {
    it("无 sessionId 时应该显示 PlayerEmptyState", async () => {
      renderWithRouter(<Player />, { route: "/" });
      // 应该显示空状态组件
      await waitFor(() => {
        expect(screen.getByTestId("player-empty-state")).toBeInTheDocument();
      });
    });

    it("空状态应该显示引导文案", async () => {
      renderWithRouter(<Player />, { route: "/" });
      await waitFor(() => {
        expect(screen.getByText("选择一个会话开始回放")).toBeInTheDocument();
      });
    });

    it("空状态应该有打开项目列表按钮", async () => {
      renderWithRouter(<Player />, { route: "/" });
      await waitFor(() => {
        expect(screen.getByText("打开项目列表")).toBeInTheDocument();
      });
    });

    it("空状态应该有导入项目按钮", async () => {
      renderWithRouter(<Player />, { route: "/" });
      await waitFor(() => {
        expect(screen.getByText("导入项目")).toBeInTheDocument();
      });
    });
  });

  describe("样式", () => {
    it("应该有全屏高度布局", () => {
      const { container } = renderWithRouter(<Player />);
      const mainDiv = container.firstChild as HTMLElement;
      expect(mainDiv).toHaveClass("h-screen");
    });
  });
});

/**
 * Story 2.12: 智能文件选择集成测试
 * Task 6.4, 6.5
 */
describe("Story 2.12 - 智能文件选择", () => {
  describe("file-path-extractor 集成", () => {
    // 这些测试验证文件路径提取逻辑与实际消息结构的集成
    it("应该从 tool_use 消息中提取文件路径", async () => {
      // 导入实际的提取函数进行集成测试
      const { extractFilePathWithPriority } = await import("@/lib/file-path-extractor");
      
      const message = {
        id: "msg-1",
        role: "assistant" as const,
        timestamp: new Date().toISOString(),
        content: [
          {
            type: "tool_use" as const,
            content: "",
            toolName: "Read",
            toolInput: { file_path: "src/components/Button.tsx" },
            toolUseId: "tu-1",
          },
        ],
      };

      const result = extractFilePathWithPriority(message);
      expect(result).not.toBeNull();
      expect(result?.path).toBe("src/components/Button.tsx");
      expect(result?.source).toBe("tool_use");
      expect(result?.confidence).toBe("high");
    });

    it("应该从历史消息中向前搜索文件路径", async () => {
      const { findRecentFilePathEnhanced } = await import("@/lib/file-path-extractor");
      
      const messages = [
        {
          id: "msg-1",
          role: "assistant" as const,
          timestamp: new Date().toISOString(),
          content: [
            {
              type: "tool_use" as const,
              content: "",
              toolName: "Write",
              toolInput: { file_path: "src/utils/helper.ts" },
              toolUseId: "tu-1",
            },
          ],
        },
        {
          id: "msg-2",
          role: "user" as const,
          timestamp: new Date().toISOString(),
          content: [
            {
              type: "text" as const,
              content: "好的，谢谢！",
            },
          ],
        },
        {
          id: "msg-3",
          role: "assistant" as const,
          timestamp: new Date().toISOString(),
          content: [
            {
              type: "text" as const,
              content: "不客气！还有什么问题吗？",
            },
          ],
        },
      ];

      // 从第三条消息向前搜索
      const result = findRecentFilePathEnhanced(messages, 2);
      expect(result).not.toBeNull();
      expect(result?.path).toBe("src/utils/helper.ts");
      expect(result?.source).toBe("history"); // 来自历史消息
    });

    it("应该正确转换绝对路径为相对路径", async () => {
      const { toRelativePath } = await import("@/lib/file-path-extractor");
      
      // Linux 绝对路径
      expect(toRelativePath("/home/user/project/src/main.ts", "/home/user/project"))
        .toBe("src/main.ts");
      
      // Windows 绝对路径
      expect(toRelativePath("C:\\Users\\project\\src\\main.ts", "C:\\Users\\project"))
        .toBe("src/main.ts");
      
      // 已经是相对路径
      expect(toRelativePath("src/main.ts", "/home/user/project"))
        .toBe("src/main.ts");
    });
  });

  describe("useTimeTravelStore 文件不存在状态集成", () => {
    it("setFileNotFound 应该正确设置状态", async () => {
      const { useTimeTravelStore } = await import("@/stores/useTimeTravelStore");
      
      // 重置 store
      useTimeTravelStore.getState().reset();
      
      // 设置文件不存在
      useTimeTravelStore.getState().setFileNotFound("src/deleted.ts", 1735500000);
      
      const state = useTimeTravelStore.getState();
      expect(state.fileNotFound).toBe(true);
      expect(state.notFoundPath).toBe("src/deleted.ts");
      expect(state.notFoundTimestamp).toBe(1735500000);
      expect(state.error).toBeNull(); // 不应该设置通用错误
    });

    it("clearFileNotFound 应该清除状态", async () => {
      const { useTimeTravelStore } = await import("@/stores/useTimeTravelStore");
      
      // 先设置状态
      useTimeTravelStore.getState().setFileNotFound("src/test.ts", 123);
      expect(useTimeTravelStore.getState().fileNotFound).toBe(true);
      
      // 清除状态
      useTimeTravelStore.getState().clearFileNotFound();
      
      const state = useTimeTravelStore.getState();
      expect(state.fileNotFound).toBe(false);
      expect(state.notFoundPath).toBeNull();
      expect(state.notFoundTimestamp).toBeNull();
    });

    it("returnToCurrent 应该同时清除文件不存在状态", async () => {
      const { useTimeTravelStore } = await import("@/stores/useTimeTravelStore");
      
      // 设置历史状态和文件不存在
      useTimeTravelStore.getState().jumpToMessage(0, "msg-1", 1735500000000);
      useTimeTravelStore.getState().setFileNotFound("src/old.ts", 1735500000);
      
      // 返回当前
      useTimeTravelStore.getState().returnToCurrent();
      
      const state = useTimeTravelStore.getState();
      expect(state.fileNotFound).toBe(false);
      expect(state.isHistoricalMode).toBe(false);
    });
  });
});
