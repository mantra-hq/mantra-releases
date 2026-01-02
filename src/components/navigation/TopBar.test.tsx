/**
 * TopBar Tests - TopBar 组件测试
 * Story 2.17: Task 1
 */

import { describe, it, expect, vi, beforeEach, beforeAll, afterAll } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { TopBar, type SessionSummary } from "./TopBar";

// Mock ResizeObserver for cmdk
class ResizeObserverMock {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeAll(() => {
  vi.stubGlobal("ResizeObserver", ResizeObserverMock);
});

afterAll(() => {
  vi.unstubAllGlobals();
});

// Mock ThemeToggle
vi.mock("@/components/theme-toggle", () => ({
  ThemeToggle: () => <button data-testid="theme-toggle">Toggle Theme</button>,
}));

// 测试数据
const mockSessions: SessionSummary[] = [
  {
    id: "session-1",
    name: "Session abc12345",
    messageCount: 10,
    lastActiveAt: Date.now() - 3600000, // 1 hour ago
  },
  {
    id: "session-2",
    name: "Session def67890",
    messageCount: 5,
    lastActiveAt: Date.now() - 7200000, // 2 hours ago
  },
];

const defaultProps = {
  sessionId: "session-1",
  sessionName: "Session abc12345",
  messageCount: 10,
  projectId: "project-1",
  projectName: "test-project",
  sessions: mockSessions,
  onDrawerOpen: vi.fn(),
  onSessionSelect: vi.fn(),
  onSync: vi.fn(),
  onImport: vi.fn(),
};

// 测试包装器
const renderWithRouter = (ui: React.ReactElement) => {
  return render(<MemoryRouter>{ui}</MemoryRouter>);
};

describe("TopBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("UI 展示", () => {
    it("应该显示项目名", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      expect(screen.getByTestId("topbar-project-name")).toHaveTextContent(
        "test-project"
      );
    });

    it("应该显示会话名和消息数", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      expect(screen.getByTestId("session-dropdown-trigger")).toHaveTextContent(
        "Session abc12345"
      );
      expect(screen.getByTestId("session-dropdown-trigger")).toHaveTextContent(
        "(10)"
      );
    });

    it("应该显示汉堡菜单按钮", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      expect(screen.getByTestId("topbar-menu-button")).toBeInTheDocument();
    });

    it("应该显示同步按钮", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      expect(screen.getByTestId("topbar-sync-button")).toBeInTheDocument();
    });

    it("应该显示导入按钮", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      expect(screen.getByTestId("topbar-import-button")).toBeInTheDocument();
    });

    it("应该显示主题切换按钮", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      expect(screen.getByTestId("theme-toggle")).toBeInTheDocument();
    });
  });

  describe("交互", () => {
    it("点击汉堡菜单应该触发 onDrawerOpen", async () => {
      const user = userEvent.setup();
      renderWithRouter(<TopBar {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-menu-button"));
      expect(defaultProps.onDrawerOpen).toHaveBeenCalledTimes(1);
    });

    it("点击项目名应该触发 onDrawerOpen", async () => {
      const user = userEvent.setup();
      renderWithRouter(<TopBar {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-project-name"));
      expect(defaultProps.onDrawerOpen).toHaveBeenCalledTimes(1);
    });

    it("点击同步按钮应该触发 onSync", async () => {
      const user = userEvent.setup();
      renderWithRouter(<TopBar {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-sync-button"));
      expect(defaultProps.onSync).toHaveBeenCalledTimes(1);
    });

    it("点击导入按钮应该触发 onImport", async () => {
      const user = userEvent.setup();
      renderWithRouter(<TopBar {...defaultProps} />);

      await user.click(screen.getByTestId("topbar-import-button"));
      expect(defaultProps.onImport).toHaveBeenCalledTimes(1);
    });

    // 注意: cmdk 库在 jsdom 环境中有一些限制
    // 复杂的 Popover 交互测试应在 E2E 测试中进行
    it("会话下拉按钮应该渲染", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      expect(screen.getByTestId("session-dropdown-trigger")).toBeInTheDocument();
    });
  });

  describe("样式", () => {
    it("应该是 sticky 定位", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      const header = screen.getByTestId("top-bar");
      expect(header).toHaveClass("sticky");
    });

    it("应该有边框", () => {
      renderWithRouter(<TopBar {...defaultProps} />);
      const header = screen.getByTestId("top-bar");
      expect(header).toHaveClass("border-b");
    });
  });
});
