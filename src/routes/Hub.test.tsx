/**
 * Hub 页面集成测试
 * Story 11.6: Task 9.6 - Hub 页面集成测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { Hub } from "./Hub";

// Mock navigate
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

// Mock IPC adapter
const mockInvokeFn = vi.fn();
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: (...args: unknown[]) => mockInvokeFn(...args),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en" },
  }),
}));

// Mock feedback
vi.mock("@/lib/feedback", () => ({
  feedback: {
    success: vi.fn(),
    copied: vi.fn(),
    error: vi.fn(),
  },
}));

// Wrapper with Router
function renderWithRouter(ui: React.ReactElement) {
  return render(<MemoryRouter>{ui}</MemoryRouter>);
}

describe("Hub Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockNavigate.mockReset();

    // Mock Gateway status
    mockInvokeFn.mockImplementation((cmd: string) => {
      switch (cmd) {
        case "get_gateway_status":
          return Promise.resolve({
            running: false,
            port: null,
            auth_token: "test-token",
            active_connections: 0,
            total_connections: 0,
            total_requests: 0,
          });
        case "list_mcp_services":
          return Promise.resolve([]);
        case "list_takeover_backups_with_version":
          return Promise.resolve([]);
        case "get_backup_stats":
          return Promise.resolve({ totalCount: 0, totalSize: 0, groups: [] });
        default:
          return Promise.resolve(null);
      }
    });
  });

  describe("页面结构", () => {
    it("应该显示页面标题", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByText("hub.title")).toBeInTheDocument();
      });
    });

    it("应该显示返回按钮", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("hub-back-button")).toBeInTheDocument();
      });
    });

    it("应该包含 Gateway 状态区域", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("hub-gateway-section")).toBeInTheDocument();
      });
    });

    it("应该包含接管状态区域 (Story 11.15)", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("hub-takeover-section")).toBeInTheDocument();
      });
    });

    it("应该包含 MCP 服务区域", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("hub-services-section")).toBeInTheDocument();
      });
    });

    it("应该包含环境变量区域", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("hub-env-section")).toBeInTheDocument();
      });
    });
  });

  describe("组件加载", () => {
    it("应该加载 Gateway 状态", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("get_gateway_status");
      });
    });

    it("应该加载 MCP 服务列表", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("list_mcp_services");
      });
    });
  });

  describe("环境变量链接卡片", () => {
    it("应该显示环境变量链接卡片", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("hub-env-settings-link")).toBeInTheDocument();
      });
    });

    it("点击链接卡片应导航到 Settings 页面", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("hub-env-settings-link")).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId("hub-env-settings-link"));

      expect(mockNavigate).toHaveBeenCalledWith("/settings");
    });
  });

  describe("数据展示", () => {
    it("应该显示 Gateway 状态卡片", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-status-card")).toBeInTheDocument();
      });
    });

    it("应该显示 MCP 服务列表", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByTestId("mcp-service-list")).toBeInTheDocument();
      });
    });

    it("应该显示 MCP 服务列表标题", async () => {
      renderWithRouter(<Hub />);

      // MCP 服务列表组件加载后应显示标题
      await waitFor(() => {
        expect(screen.getByText("hub.services.title")).toBeInTheDocument();
      });
    });
  });

  describe("AC #1: Hub 页面结构", () => {
    it("应该显示 Gateway 状态卡片（AC1）", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByText("hub.gateway.title")).toBeInTheDocument();
      });
    });

    it("应该显示 MCP 服务列表（AC1）", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByText("hub.services.title")).toBeInTheDocument();
      });
    });

    it("应该显示环境变量管理（AC1）", async () => {
      renderWithRouter(<Hub />);

      await waitFor(() => {
        expect(screen.getByText("hub.envVariables.title")).toBeInTheDocument();
      });
    });
  });
});
