/**
 * GatewayStatusCard 组件测试
 * Story 11.6: Task 9.1 - GatewayStatusCard 组件测试
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { GatewayStatusCard } from "./GatewayStatusCard";

// Mock IPC adapter
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: vi.fn(),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
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

// Import after mocking
import { invoke } from "@/lib/ipc-adapter";
import { feedback } from "@/lib/feedback";

const mockInvokeFn = vi.mocked(invoke);

describe("GatewayStatusCard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // 默认返回停止状态
    mockInvokeFn.mockResolvedValue({
      running: false,
      port: null,
      auth_token: "test-token-1234",
      active_connections: 0,
      total_connections: 0,
      total_requests: 0,
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("状态显示", () => {
    it("应该显示 Gateway 已停止状态", async () => {
      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-status-badge")).toHaveTextContent(
          "hub.gateway.stopped"
        );
      });
    });

    it("应该显示 Gateway 运行中状态", async () => {
      mockInvokeFn.mockResolvedValue({
        running: true,
        port: 51234,
        auth_token: "test-token-1234",
        active_connections: 3,
        total_connections: 10,
        total_requests: 100,
      });

      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-status-badge")).toHaveTextContent(
          "hub.gateway.running"
        );
      });
    });

    it("应该显示端口号", async () => {
      mockInvokeFn.mockResolvedValue({
        running: true,
        port: 51234,
        auth_token: "test-token",
        active_connections: 0,
        total_connections: 0,
        total_requests: 0,
      });

      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-port")).toHaveTextContent("51234");
      });
    });

    it("应该显示连接数", async () => {
      mockInvokeFn.mockResolvedValue({
        running: true,
        port: 51234,
        auth_token: "test-token",
        active_connections: 5,
        total_connections: 10,
        total_requests: 100,
      });

      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-connections")).toHaveTextContent("5");
      });
    });
  });

  describe("启动/停止操作", () => {
    it("停止状态时应该显示启动按钮", async () => {
      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-start-button")).toBeInTheDocument();
      });
    });

    it("运行状态时应该显示停止按钮", async () => {
      mockInvokeFn.mockResolvedValue({
        running: true,
        port: 51234,
        auth_token: "test-token",
        active_connections: 0,
        total_connections: 0,
        total_requests: 0,
      });

      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-stop-button")).toBeInTheDocument();
      });
    });

    it("点击启动按钮应该调用 start_gateway", async () => {
      const user = userEvent.setup();

      // 首次加载返回停止状态
      mockInvokeFn.mockResolvedValueOnce({
        running: false,
        port: null,
        auth_token: "test-token",
        active_connections: 0,
        total_connections: 0,
        total_requests: 0,
      });

      // 启动后返回运行状态
      mockInvokeFn.mockResolvedValueOnce({
        running: true,
        port: 51234,
        auth_token: "test-token",
        active_connections: 0,
        total_connections: 0,
        total_requests: 0,
      });

      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-start-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("gateway-start-button"));

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("start_gateway");
      });
    });

    it("点击停止按钮应该调用 stop_gateway", async () => {
      const user = userEvent.setup();

      mockInvokeFn.mockResolvedValue({
        running: true,
        port: 51234,
        auth_token: "test-token",
        active_connections: 0,
        total_connections: 0,
        total_requests: 0,
      });

      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("gateway-stop-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("gateway-stop-button"));

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("stop_gateway");
      });
    });
  });

  describe("复制功能", () => {
    beforeEach(() => {
      mockInvokeFn.mockResolvedValue({
        running: true,
        port: 51234,
        auth_token: "test-token-1234",
        active_connections: 0,
        total_connections: 0,
        total_requests: 0,
      });
    });

    it("运行时应该显示复制 URL 按钮", async () => {
      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("copy-url-button")).toBeInTheDocument();
      });
    });

    it("运行时应该显示复制 Token 按钮", async () => {
      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("copy-token-button")).toBeInTheDocument();
      });
    });
  });

  describe("Token 重新生成", () => {
    beforeEach(() => {
      mockInvokeFn.mockResolvedValue({
        running: true,
        port: 51234,
        auth_token: "old-token",
        active_connections: 0,
        total_connections: 0,
        total_requests: 0,
      });
    });

    it("点击重新生成按钮应该调用 regenerate_gateway_token", async () => {
      const user = userEvent.setup();

      // 重新生成返回新 Token
      mockInvokeFn.mockImplementation((cmd: string) => {
        if (cmd === "regenerate_gateway_token") {
          return Promise.resolve("new-token");
        }
        return Promise.resolve({
          running: true,
          port: 51234,
          auth_token: "old-token",
          active_connections: 0,
          total_connections: 0,
          total_requests: 0,
        });
      });

      render(<GatewayStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("regenerate-token-button")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("regenerate-token-button"));

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("regenerate_gateway_token");
      });

      expect(feedback.success).toHaveBeenCalled();
    });
  });
});
