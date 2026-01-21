/**
 * InsertMessageDialog Component Tests
 * Story 10.5: Task 8.2
 *
 * 测试消息插入对话框组件
 * - 角色选择、内容输入、Token 计算、快捷键
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InsertMessageDialog } from "./InsertMessageDialog";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.insertDialog.title": "Insert Message",
        "compress.insertDialog.description": "Add a new message to the conversation context.",
        "compress.insertDialog.roleLabel": "Role",
        "compress.insertDialog.roleUser": "User",
        "compress.insertDialog.roleAssistant": "Assistant",
        "compress.insertDialog.contentLabel": "Content",
        "compress.insertDialog.placeholder": "Enter message content...",
        "compress.insertDialog.tokens": "tokens",
        "compress.insertDialog.cancel": "Cancel",
        "compress.insertDialog.confirm": "Insert",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock token-counter
vi.mock("@/lib/token-counter", () => ({
  estimateTokenCount: vi.fn((text: string) => Math.ceil(text.length / 4)),
}));

describe("InsertMessageDialog", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    onConfirm: vi.fn(),
    insertPosition: "0",
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("对话框渲染 (AC #2)", () => {
    it("打开时应渲染对话框", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("insert-message-dialog")).toBeInTheDocument();
    });

    it("关闭时不应渲染对话框内容", () => {
      render(<InsertMessageDialog {...defaultProps} open={false} />);

      expect(screen.queryByTestId("insert-message-dialog")).not.toBeInTheDocument();
    });

    it("应显示对话框标题", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByText("Insert Message")).toBeInTheDocument();
    });

    it("应显示对话框描述", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByText("Add a new message to the conversation context.")).toBeInTheDocument();
    });
  });

  describe("角色选择 (AC #2)", () => {
    it("应显示角色选择区域", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByText("Role")).toBeInTheDocument();
    });

    it("应显示 User 和 Assistant 选项", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("role-user-button")).toBeInTheDocument();
      expect(screen.getByTestId("role-assistant-button")).toBeInTheDocument();
    });

    it("默认应选中 User", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      const userButton = screen.getByTestId("role-user-button");
      expect(userButton).toHaveAttribute("data-state", "on");
    });

    it("点击 Assistant 应切换选择", async () => {
      const user = userEvent.setup();
      render(<InsertMessageDialog {...defaultProps} />);

      await user.click(screen.getByTestId("role-assistant-button"));

      expect(screen.getByTestId("role-assistant-button")).toHaveAttribute("data-state", "on");
    });
  });

  describe("内容输入 (AC #2)", () => {
    it("应显示内容输入区域", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByText("Content")).toBeInTheDocument();
    });

    it("应显示可编辑的文本输入框", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("content-input");
      expect(textarea).toBeInTheDocument();
    });

    it("应能输入内容", async () => {
      const user = userEvent.setup();
      render(<InsertMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("content-input");
      await user.type(textarea, "Hello world");

      expect(textarea).toHaveValue("Hello world");
    });

    it("输入框应有 placeholder", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("content-input");
      expect(textarea).toHaveAttribute("placeholder", "Enter message content...");
    });
  });

  describe("Token 计算 (AC #2)", () => {
    it("应显示 Token 数量", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("token-count-display")).toBeInTheDocument();
    });

    it("初始 Token 应为 0", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("token-count-display")).toHaveTextContent("0");
    });

    it("输入内容后应更新 Token 数量", async () => {
      const user = userEvent.setup();
      render(<InsertMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("content-input");
      await user.type(textarea, "Hello world test content");

      await waitFor(() => {
        const display = screen.getByTestId("token-count-display");
        expect(display.textContent).not.toBe("0 tokens");
      });
    });
  });

  describe("按钮操作 (AC #2, #3)", () => {
    it("应显示取消和确认按钮", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("cancel-button")).toBeInTheDocument();
      expect(screen.getByTestId("confirm-button")).toBeInTheDocument();
    });

    it("空内容时确认按钮应禁用", () => {
      render(<InsertMessageDialog {...defaultProps} />);

      expect(screen.getByTestId("confirm-button")).toBeDisabled();
    });

    it("有内容时确认按钮应启用", async () => {
      const user = userEvent.setup();
      render(<InsertMessageDialog {...defaultProps} />);

      const textarea = screen.getByTestId("content-input");
      await user.type(textarea, "Hello");

      expect(screen.getByTestId("confirm-button")).not.toBeDisabled();
    });

    it("点击取消按钮应关闭对话框", () => {
      const onOpenChange = vi.fn();
      render(<InsertMessageDialog {...defaultProps} onOpenChange={onOpenChange} />);

      fireEvent.click(screen.getByTestId("cancel-button"));

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("点击确认按钮应调用 onConfirm 并关闭对话框", async () => {
      const user = userEvent.setup();
      const onConfirm = vi.fn();
      const onOpenChange = vi.fn();
      render(
        <InsertMessageDialog
          {...defaultProps}
          onConfirm={onConfirm}
          onOpenChange={onOpenChange}
        />
      );

      const textarea = screen.getByTestId("content-input");
      await user.type(textarea, "Test message");

      fireEvent.click(screen.getByTestId("confirm-button"));

      expect(onConfirm).toHaveBeenCalledTimes(1);
      expect(onConfirm.mock.calls[0][0]).toMatchObject({
        role: "user",
        content: [{ type: "text", content: "Test message" }],
      });
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("确认时消息应包含正确的角色", async () => {
      const user = userEvent.setup();
      const onConfirm = vi.fn();
      render(<InsertMessageDialog {...defaultProps} onConfirm={onConfirm} />);

      // 切换到 Assistant
      await user.click(screen.getByTestId("role-assistant-button"));

      const textarea = screen.getByTestId("content-input");
      await user.type(textarea, "AI response");

      fireEvent.click(screen.getByTestId("confirm-button"));

      expect(onConfirm.mock.calls[0][0]).toMatchObject({
        role: "assistant",
      });
    });
  });

  describe("键盘快捷键 (AC #2)", () => {
    it("Ctrl+Enter 有内容时应触发确认", async () => {
      const user = userEvent.setup();
      const onConfirm = vi.fn();
      const onOpenChange = vi.fn();
      render(
        <InsertMessageDialog
          {...defaultProps}
          onConfirm={onConfirm}
          onOpenChange={onOpenChange}
        />
      );

      const textarea = screen.getByTestId("content-input");
      await user.type(textarea, "Test content");

      fireEvent.keyDown(screen.getByTestId("insert-message-dialog"), {
        key: "Enter",
        ctrlKey: true,
      });

      expect(onConfirm).toHaveBeenCalledTimes(1);
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("Ctrl+Enter 空内容时不应触发确认", () => {
      const onConfirm = vi.fn();
      render(<InsertMessageDialog {...defaultProps} onConfirm={onConfirm} />);

      fireEvent.keyDown(screen.getByTestId("insert-message-dialog"), {
        key: "Enter",
        ctrlKey: true,
      });

      expect(onConfirm).not.toHaveBeenCalled();
    });

    it("Escape 应触发取消", () => {
      const onOpenChange = vi.fn();
      render(<InsertMessageDialog {...defaultProps} onOpenChange={onOpenChange} />);

      fireEvent.keyDown(screen.getByTestId("insert-message-dialog"), {
        key: "Escape",
      });

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("Meta+Enter (Mac) 有内容时应触发确认", async () => {
      const user = userEvent.setup();
      const onConfirm = vi.fn();
      render(<InsertMessageDialog {...defaultProps} onConfirm={onConfirm} />);

      const textarea = screen.getByTestId("content-input");
      await user.type(textarea, "Test");

      fireEvent.keyDown(screen.getByTestId("insert-message-dialog"), {
        key: "Enter",
        metaKey: true,
      });

      expect(onConfirm).toHaveBeenCalledTimes(1);
    });
  });

  describe("状态重置", () => {
    it("对话框重新打开时应重置状态", async () => {
      const user = userEvent.setup();
      const { rerender } = render(<InsertMessageDialog {...defaultProps} />);

      // 输入内容
      const textarea = screen.getByTestId("content-input");
      await user.type(textarea, "Some content");

      // 关闭对话框
      rerender(<InsertMessageDialog {...defaultProps} open={false} />);

      // 重新打开
      rerender(<InsertMessageDialog {...defaultProps} open={true} />);

      // 内容应被重置
      expect(screen.getByTestId("content-input")).toHaveValue("");
    });
  });
});
