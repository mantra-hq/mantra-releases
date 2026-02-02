/**
 * EnvVariableSheet 组件测试
 * Story 12.2: Dialog → Sheet 改造 - Code Review 补充测试
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { EnvVariableSheet } from "./EnvVariableSheet";
import type { EnvVariable } from "@/lib/env-variable-ipc";

// Mock IPC
vi.mock("@/lib/env-variable-ipc", () => ({
  setEnvVariable: vi.fn(),
  validateEnvVarNameSync: vi.fn(() => ({ is_valid: true, error_message: null, suggestion: null })),
  getEnvVariableDecrypted: vi.fn(() => Promise.resolve("decrypted-value")),
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
    saved: vi.fn(),
    error: vi.fn(),
  },
}));

// Import after mocking
import { setEnvVariable, validateEnvVarNameSync, getEnvVariableDecrypted } from "@/lib/env-variable-ipc";
import { feedback } from "@/lib/feedback";

const mockSetEnvVariable = vi.mocked(setEnvVariable);
const mockValidateEnvVarNameSync = vi.mocked(validateEnvVarNameSync);
const mockGetEnvVariableDecrypted = vi.mocked(getEnvVariableDecrypted);

const mockVariable: EnvVariable = {
  name: "TEST_API_KEY",
  description: "Test description",
  created_at: "2024-01-01T00:00:00Z",
  updated_at: "2024-01-01T00:00:00Z",
};

describe("EnvVariableSheet", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockValidateEnvVarNameSync.mockReturnValue({ is_valid: true, error_message: null, suggestion: null });
    mockGetEnvVariableDecrypted.mockResolvedValue("decrypted-value");
    mockSetEnvVariable.mockResolvedValue(undefined);
  });

  describe("添加模式", () => {
    it("应该显示空表单", async () => {
      render(
        <EnvVariableSheet
          open={true}
          onOpenChange={vi.fn()}
          editVariable={null}
          onSuccess={vi.fn()}
        />
      );

      expect(screen.getByText("hub.envVariables.addTitle")).toBeInTheDocument();
      expect(screen.getByTestId("env-variable-name-input")).toHaveValue("");
    });

    it("点击保存应该调用 setEnvVariable", async () => {
      const user = userEvent.setup();
      const onSuccess = vi.fn();
      const onOpenChange = vi.fn();

      render(
        <EnvVariableSheet
          open={true}
          onOpenChange={onOpenChange}
          editVariable={null}
          onSuccess={onSuccess}
        />
      );

      // 填写表单
      await user.type(screen.getByTestId("env-variable-name-input"), "NEW_KEY");
      await user.type(screen.getByTestId("env-variable-value-input"), "new-value");

      // 提交
      await user.click(screen.getByTestId("env-variable-save-button"));

      await waitFor(() => {
        expect(mockSetEnvVariable).toHaveBeenCalledWith("NEW_KEY", "new-value", undefined);
      });

      expect(feedback.saved).toHaveBeenCalled();
      expect(onSuccess).toHaveBeenCalled();
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe("编辑模式", () => {
    it("应该加载并显示已有变量数据", async () => {
      render(
        <EnvVariableSheet
          open={true}
          onOpenChange={vi.fn()}
          editVariable={mockVariable}
          onSuccess={vi.fn()}
        />
      );

      expect(screen.getByText("hub.envVariables.editTitle")).toBeInTheDocument();

      await waitFor(() => {
        expect(screen.getByTestId("env-variable-name-input")).toHaveValue("TEST_API_KEY");
      });

      // 名称输入框在编辑模式下应禁用
      expect(screen.getByTestId("env-variable-name-input")).toBeDisabled();
    });

    it("应该调用 getEnvVariableDecrypted 加载解密值", async () => {
      render(
        <EnvVariableSheet
          open={true}
          onOpenChange={vi.fn()}
          editVariable={mockVariable}
          onSuccess={vi.fn()}
        />
      );

      await waitFor(() => {
        expect(mockGetEnvVariableDecrypted).toHaveBeenCalledWith("TEST_API_KEY");
      });
    });
  });

  describe("表单验证", () => {
    it("变量名无效时应该显示错误", async () => {
      mockValidateEnvVarNameSync.mockReturnValue({
        is_valid: false,
        error_message: "Invalid name",
        suggestion: "VALID_NAME",
      });

      const user = userEvent.setup();

      render(
        <EnvVariableSheet
          open={true}
          onOpenChange={vi.fn()}
          editVariable={null}
          onSuccess={vi.fn()}
        />
      );

      await user.type(screen.getByTestId("env-variable-name-input"), "invalid-name");

      await waitFor(() => {
        expect(screen.getByText("Invalid name")).toBeInTheDocument();
      });
    });

    it("应该显示建议的变量名", async () => {
      mockValidateEnvVarNameSync.mockReturnValue({
        is_valid: false,
        error_message: "Invalid name",
        suggestion: "VALID_NAME",
      });

      const user = userEvent.setup();

      render(
        <EnvVariableSheet
          open={true}
          onOpenChange={vi.fn()}
          editVariable={null}
          onSuccess={vi.fn()}
        />
      );

      await user.type(screen.getByTestId("env-variable-name-input"), "invalid");

      await waitFor(() => {
        expect(screen.getByText("VALID_NAME")).toBeInTheDocument();
      });
    });
  });

  describe("错误处理", () => {
    it("保存失败时应该显示错误提示", async () => {
      mockSetEnvVariable.mockRejectedValue(new Error("Save failed"));

      const user = userEvent.setup();

      render(
        <EnvVariableSheet
          open={true}
          onOpenChange={vi.fn()}
          editVariable={null}
          onSuccess={vi.fn()}
        />
      );

      await user.type(screen.getByTestId("env-variable-name-input"), "TEST_KEY");
      await user.type(screen.getByTestId("env-variable-value-input"), "test-value");

      await user.click(screen.getByTestId("env-variable-save-button"));

      await waitFor(() => {
        expect(feedback.error).toHaveBeenCalled();
      });
    });
  });
});
