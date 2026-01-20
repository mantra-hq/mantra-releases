/**
 * EditMessageDialog Component Tests
 * Story 10.4: Task 7.2
 *
 * 测试消息编辑对话框组件
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { EditMessageDialog } from "./EditMessageDialog";
import type { NarrativeMessage } from "@/types/message";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.editDialog.title": "Edit Message",
        "compress.editDialog.original": "Original Content",
        "compress.editDialog.modified": "Modified Content",
        "compress.editDialog.placeholder": "Enter modified content...",
        "compress.editDialog.cancel": "Cancel",
        "compress.editDialog.confirm": "Confirm Edit",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock token-counter
vi.mock("@/lib/token-counter", () => ({
  estimateTokenCount: vi.fn((text: string) => Math.ceil(text.length / 4)),
}));

describe("EditMessageDialog", () => {
  const createMessage = (content: string): NarrativeMessage => ({
    id: "msg-1",
    role: "user",
    timestamp: "2024-01-01T00:00:00Z",
    content: [{ type: "text", content }],
  });

  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    message: createMessage("Original message content"),
    onConfirm: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("对话框渲染 (AC #3)", () => {
    it("打开时应渲染对话框", () => {
      render(<EditMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("edit-message-dialog")).toBeInTheDocument();
    });

    it("关闭时不应渲染对话框内容", () => {
      render(<EditMessageDialog {...defaultProps} open={false} />);

      expect(screen.queryByTestId("edit-message-dialog")).not.toBeInTheDocument();
    });

    it("应显示对话框标题", () => {
      render(<EditMessageDialog {...defaultProps} />);

      expect(screen.getByText("Edit Message")).toBeInTheDocument();
    });

    it("message 为 null 时不应渲染", () => {
      render(<EditMessageDialog {...defaultProps} message={null} />);

      expect(screen.queryByTestId("edit-message-dialog")).not.toBeInTheDocument();
    });
  });

  describe("原始内容显示 (AC #3)", () => {
    it("应显示原始内容区域", () => {
      render(<EditMessageDialog {...defaultProps} />);

      expect(screen.getByText("Original Content")).toBeInTheDocument();
    });

    it("应显示原始消息内容", () => {
      render(<EditMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("original-content")).toHaveTextContent(
        "Original message content"
      );
    });
  });

  describe("可编辑区域 (AC #3)", () => {
    it("应显示修改内容区域", () => {
      render(<EditMessageDialog {...defaultProps} />);

      expect(screen.getByText("Modified Content")).toBeInTheDocument();
    });

    it("应显示可编辑的文本输入框", () => {
      render(<EditMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("modified-content-input");
      expect(textarea).toBeInTheDocument();
      expect(textarea).toHaveValue("Original message content");
    });

    it("应能编辑内容", async () => {
      const user = userEvent.setup();
      render(<EditMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("modified-content-input");
      await user.clear(textarea);
      await user.type(textarea, "New content");

      expect(textarea).toHaveValue("New content");
    });
  });

  describe("Token 显示 (AC #3)", () => {
    it("应显示 Token 数量", () => {
      render(<EditMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("token-display")).toBeInTheDocument();
    });

    it("内容修改后应显示 Token 变化", async () => {
      const user = userEvent.setup();
      render(<EditMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("modified-content-input");
      await user.clear(textarea);
      await user.type(textarea, "Short");

      // 应显示 Token delta
      await waitFor(() => {
        expect(screen.getByTestId("token-delta")).toBeInTheDocument();
      });
    });
  });

  describe("按钮操作 (AC #4)", () => {
    it("应显示取消和确认按钮", () => {
      render(<EditMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("cancel-button")).toBeInTheDocument();
      expect(screen.getByTestId("confirm-button")).toBeInTheDocument();
    });

    it("无变更时确认按钮应禁用", () => {
      render(<EditMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("confirm-button")).toBeDisabled();
    });

    it("有变更时确认按钮应启用", async () => {
      const user = userEvent.setup();
      render(<EditMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("modified-content-input");
      await user.type(textarea, " extra");

      expect(screen.getByTestId("confirm-button")).not.toBeDisabled();
    });

    it("点击取消按钮应关闭对话框", () => {
      const onOpenChange = vi.fn();
      render(<EditMessageDialog {...defaultProps} onOpenChange={onOpenChange} />);

      fireEvent.click(screen.getByTestId("cancel-button"));

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("点击确认按钮应调用 onConfirm 并关闭对话框", async () => {
      const user = userEvent.setup();
      const onConfirm = vi.fn();
      const onOpenChange = vi.fn();
      render(
        <EditMessageDialog
          {...defaultProps}
          onConfirm={onConfirm}
          onOpenChange={onOpenChange}
        />
      );

      const textarea = screen.getByTestId("modified-content-input");
      await user.type(textarea, " modified");

      fireEvent.click(screen.getByTestId("confirm-button"));

      expect(onConfirm).toHaveBeenCalledWith("Original message content modified");
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });
});
