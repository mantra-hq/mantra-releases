/**
 * OAuthConfigSheet 组件测试
 * Story 12.2: Dialog → Sheet 改造 - Code Review 补充测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { OAuthConfigSheet } from "./OAuthConfigSheet";

// Mock IPC adapter
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: vi.fn(),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (params?.name) return `${key} - ${params.name}`;
      return key;
    },
  }),
}));

// Mock feedback
vi.mock("@/lib/feedback", () => ({
  feedback: {
    error: vi.fn(),
  },
}));

// Mock sonner toast
vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
  },
}));

// Import after mocking
import { invoke } from "@/lib/ipc-adapter";
import { feedback } from "@/lib/feedback";
import { toast } from "sonner";

const mockInvoke = vi.mocked(invoke);

describe("OAuthConfigSheet", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue({
      service_id: "test-service",
      status: "disconnected",
      expires_at: null,
      scopes: [],
      last_refreshed: null,
    });
  });

  describe("渲染测试", () => {
    it("应该正确渲染 Sheet", async () => {
      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(screen.getByText("hub.oauth.title - Test Service")).toBeInTheDocument();
      });
    });

    it("open 为 false 时不应该渲染内容", () => {
      render(
        <OAuthConfigSheet
          open={false}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      expect(screen.queryByText("hub.oauth.title - Test Service")).not.toBeInTheDocument();
    });

    it("应该显示 OAuth 2.0 和 Bearer Token 选项卡", async () => {
      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(screen.getByText("OAuth 2.0")).toBeInTheDocument();
        expect(screen.getByText("Bearer Token")).toBeInTheDocument();
      });
    });
  });

  describe("状态加载", () => {
    it("应该在打开时加载 OAuth 状态", async () => {
      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("oauth_get_status", {
          serviceId: "test-service",
        });
      });
    });

    it("已连接状态应该显示 connected 徽章", async () => {
      mockInvoke.mockResolvedValue({
        service_id: "test-service",
        status: "connected",
        expires_at: "2025-01-01T00:00:00Z",
        scopes: ["read", "write"],
        last_refreshed: null,
      });

      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(screen.getByText("hub.oauth.statusConnected")).toBeInTheDocument();
      });
    });

    it("过期状态应该显示 expired 徽章", async () => {
      mockInvoke.mockResolvedValue({
        service_id: "test-service",
        status: "expired",
        expires_at: "2024-01-01T00:00:00Z",
        scopes: [],
        last_refreshed: null,
      });

      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(screen.getByText("hub.oauth.statusExpired")).toBeInTheDocument();
      });
    });
  });

  describe("OAuth 配置表单", () => {
    it("应该显示 OAuth 配置字段", async () => {
      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(screen.getByLabelText(/hub.oauth.clientId/)).toBeInTheDocument();
        expect(screen.getByLabelText(/hub.oauth.clientSecret/)).toBeInTheDocument();
        expect(screen.getByLabelText(/hub.oauth.authorizationUrl/)).toBeInTheDocument();
        expect(screen.getByLabelText(/hub.oauth.tokenUrl/)).toBeInTheDocument();
      });
    });

    it("缺少必填字段时连接按钮应禁用", async () => {
      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        const connectButton = screen.getByText("hub.oauth.connect").closest("button");
        expect(connectButton).toBeDisabled();
      });
    });
  });

  describe("Bearer Token 模式", () => {
    it("切换到 Bearer Token 应该显示 token 输入框", async () => {
      const user = userEvent.setup();

      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(screen.getByText("Bearer Token")).toBeInTheDocument();
      });

      await user.click(screen.getByText("Bearer Token"));

      await waitFor(() => {
        expect(screen.getByLabelText(/hub.oauth.bearerToken/)).toBeInTheDocument();
      });
    });
  });

  describe("连接操作", () => {
    it("应该调用 oauth_start_flow", async () => {
      const user = userEvent.setup();

      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(screen.getByLabelText(/hub.oauth.clientId/)).toBeInTheDocument();
      });

      // 填写必填字段
      await user.type(screen.getByLabelText(/hub.oauth.clientId/), "client-123");
      await user.type(screen.getByLabelText(/hub.oauth.authorizationUrl/), "https://auth.example.com/authorize");
      await user.type(screen.getByLabelText(/hub.oauth.tokenUrl/), "https://auth.example.com/token");

      // 点击连接
      const connectButton = screen.getByText("hub.oauth.connect").closest("button");
      await user.click(connectButton!);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("oauth_start_flow", expect.objectContaining({
          request: expect.objectContaining({
            service_id: "test-service",
            client_id: "client-123",
          }),
        }));
      });
    });
  });

  describe("断开连接操作", () => {
    it("已连接状态应该显示断开和刷新按钮", async () => {
      mockInvoke.mockResolvedValue({
        service_id: "test-service",
        status: "connected",
        expires_at: "2025-01-01T00:00:00Z",
        scopes: [],
        last_refreshed: null,
      });

      render(
        <OAuthConfigSheet
          open={true}
          onOpenChange={vi.fn()}
          serviceId="test-service"
          serviceName="Test Service"
        />
      );

      await waitFor(() => {
        expect(screen.getByText("hub.oauth.disconnect")).toBeInTheDocument();
        expect(screen.getByText("hub.oauth.refresh")).toBeInTheDocument();
      });
    });
  });
});
