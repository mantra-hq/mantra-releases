/**
 * McpServiceForm 组件测试
 * Story 11.6: Task 9.3 - McpServiceForm 组件测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { McpServiceForm } from "./McpServiceForm";
import type { McpService } from "./McpServiceList";

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
    error: vi.fn(),
  },
}));

// Import after mocking
import { invoke } from "@/lib/ipc-adapter";
import { feedback } from "@/lib/feedback";

const mockInvokeFn = vi.mocked(invoke);

const mockService: McpService = {
  id: "service-1",
  name: "git-mcp",
  command: "npx",
  args: ["--yes", "@anthropic/mcp-server-git"],
  env: { GITHUB_TOKEN: "$GITHUB_TOKEN" },
  source: "manual",
  source_file: null,
  enabled: true,
  created_at: "2024-01-01T00:00:00Z",
  updated_at: "2024-01-01T00:00:00Z",
};

describe("McpServiceForm", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("添加模式", () => {
    it("应该显示空表单", async () => {
      render(
        <McpServiceForm
          open={true}
          onOpenChange={vi.fn()}
          editService={null}
          onSuccess={vi.fn()}
        />
      );

      expect(screen.getByText("hub.services.form.addTitle")).toBeInTheDocument();
      expect(screen.getByTestId("mcp-service-name-input")).toHaveValue("");
      expect(screen.getByTestId("mcp-service-command-input")).toHaveValue("");
    });

    it("点击创建按钮应该调用 create_mcp_service", async () => {
      const user = userEvent.setup();
      const onSuccess = vi.fn();
      const onOpenChange = vi.fn();

      mockInvokeFn.mockResolvedValue(mockService);

      render(
        <McpServiceForm
          open={true}
          onOpenChange={onOpenChange}
          editService={null}
          onSuccess={onSuccess}
        />
      );

      // 填写表单
      await user.type(screen.getByTestId("mcp-service-name-input"), "test-service");
      await user.type(screen.getByTestId("mcp-service-command-input"), "npx");

      // 提交
      await user.click(screen.getByTestId("mcp-service-submit-button"));

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("create_mcp_service", {
          request: {
            name: "test-service",
            command: "npx",
            args: null,
            env: null,
          },
        });
      });

      expect(feedback.success).toHaveBeenCalled();
      expect(onSuccess).toHaveBeenCalled();
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe("编辑模式", () => {
    it("应该显示已有服务数据", async () => {
      render(
        <McpServiceForm
          open={true}
          onOpenChange={vi.fn()}
          editService={mockService}
          onSuccess={vi.fn()}
        />
      );

      expect(screen.getByText("hub.services.form.editTitle")).toBeInTheDocument();
      expect(screen.getByTestId("mcp-service-name-input")).toHaveValue("git-mcp");
      expect(screen.getByTestId("mcp-service-command-input")).toHaveValue("npx");
    });

    it("点击保存按钮应该调用 update_mcp_service", async () => {
      const user = userEvent.setup();
      const onSuccess = vi.fn();
      const onOpenChange = vi.fn();

      mockInvokeFn.mockResolvedValue({ ...mockService, name: "updated-service" });

      render(
        <McpServiceForm
          open={true}
          onOpenChange={onOpenChange}
          editService={mockService}
          onSuccess={onSuccess}
        />
      );

      // 修改名称
      const nameInput = screen.getByTestId("mcp-service-name-input");
      await user.clear(nameInput);
      await user.type(nameInput, "updated-service");

      // 提交
      await user.click(screen.getByTestId("mcp-service-submit-button"));

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("update_mcp_service", {
          id: "service-1",
          updates: expect.objectContaining({
            name: "updated-service",
          }),
        });
      });
    });
  });

  describe("表单验证", () => {
    it("名称为空时应该显示错误", async () => {
      const user = userEvent.setup();

      render(
        <McpServiceForm
          open={true}
          onOpenChange={vi.fn()}
          editService={null}
          onSuccess={vi.fn()}
        />
      );

      // 只填命令
      await user.type(screen.getByTestId("mcp-service-command-input"), "npx");

      // 提交
      await user.click(screen.getByTestId("mcp-service-submit-button"));

      await waitFor(() => {
        expect(screen.getByText("hub.services.form.nameRequired")).toBeInTheDocument();
      });
    });

    it("命令为空时应该显示错误", async () => {
      const user = userEvent.setup();

      render(
        <McpServiceForm
          open={true}
          onOpenChange={vi.fn()}
          editService={null}
          onSuccess={vi.fn()}
        />
      );

      // 只填名称
      await user.type(screen.getByTestId("mcp-service-name-input"), "test");

      // 提交
      await user.click(screen.getByTestId("mcp-service-submit-button"));

      await waitFor(() => {
        expect(screen.getByText("hub.services.form.commandRequired")).toBeInTheDocument();
      });
    });

    it("参数 JSON 无效时应该显示错误", async () => {
      const user = userEvent.setup();

      render(
        <McpServiceForm
          open={true}
          onOpenChange={vi.fn()}
          editService={null}
          onSuccess={vi.fn()}
        />
      );

      await user.type(screen.getByTestId("mcp-service-name-input"), "test");
      await user.type(screen.getByTestId("mcp-service-command-input"), "npx");
      await user.type(screen.getByTestId("mcp-service-args-input"), "invalid json");

      await user.click(screen.getByTestId("mcp-service-submit-button"));

      await waitFor(() => {
        expect(screen.getByText("hub.services.form.invalidJson")).toBeInTheDocument();
      });
    });
  });

  describe("错误处理", () => {
    it("创建失败时应该显示错误提示", async () => {
      const user = userEvent.setup();

      mockInvokeFn.mockRejectedValue(new Error("Create failed"));

      render(
        <McpServiceForm
          open={true}
          onOpenChange={vi.fn()}
          editService={null}
          onSuccess={vi.fn()}
        />
      );

      await user.type(screen.getByTestId("mcp-service-name-input"), "test");
      await user.type(screen.getByTestId("mcp-service-command-input"), "npx");

      await user.click(screen.getByTestId("mcp-service-submit-button"));

      await waitFor(() => {
        expect(feedback.error).toHaveBeenCalledWith(
          "hub.services.createError",
          "Create failed"
        );
      });
    });

    it("更新失败时应该显示错误提示", async () => {
      const user = userEvent.setup();

      mockInvokeFn.mockRejectedValue(new Error("Update failed"));

      render(
        <McpServiceForm
          open={true}
          onOpenChange={vi.fn()}
          editService={mockService}
          onSuccess={vi.fn()}
        />
      );

      await user.click(screen.getByTestId("mcp-service-submit-button"));

      await waitFor(() => {
        expect(feedback.error).toHaveBeenCalledWith(
          "hub.services.updateError",
          "Update failed"
        );
      });
    });
  });
});
