/**
 * InsertMessageSheet Tests
 * Story 10.5: Task 2 - 消息插入测试
 * Story 12.1: Task 4 - Dialog → Sheet 改造测试
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InsertMessageSheet } from "./InsertMessageSheet";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.insertDialog.title": "插入消息",
        "compress.insertDialog.titleEdit": "编辑插入的消息",
        "compress.insertDialog.description": "在此位置插入新消息",
        "compress.insertDialog.descriptionEdit": "编辑已插入的消息",
        "compress.insertDialog.roleLabel": "角色",
        "compress.insertDialog.roleUser": "用户",
        "compress.insertDialog.roleAssistant": "助手",
        "compress.insertDialog.contentLabel": "内容",
        "compress.insertDialog.placeholder": "输入消息内容...",
        "compress.insertDialog.tokens": "tokens",
        "compress.insertDialog.cancel": "取消",
        "compress.insertDialog.confirm": "插入",
        "compress.insertDialog.confirmEdit": "保存",
      };
      return translations[key] || key;
    },
    i18n: { language: "zh-CN" },
  }),
}));

// Mock token-counter
vi.mock("@/lib/token-counter", () => ({
  estimateTokenCount: vi.fn((text: string) => Math.ceil(text.length / 4)),
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

const defaultProps = {
  open: true,
  onOpenChange: vi.fn(),
  onConfirm: vi.fn(),
  insertPosition: "0",
};

describe("InsertMessageSheet", () => {
  describe("Sheet rendering", () => {
    it("renders sheet when open", async () => {
      render(<InsertMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByRole("dialog")).toBeInTheDocument();
      });
    });

    it("does not render when closed", () => {
      render(<InsertMessageSheet {...defaultProps} open={false} />);

      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    // Story 12.1: Sheet 特定测试
    it("renders with data-testid for sheet", async () => {
      render(<InsertMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("insert-message-sheet")).toBeInTheDocument();
      });
    });
  });

  describe("Role selection (AC #4)", () => {
    it("shows role toggle buttons", async () => {
      render(<InsertMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("role-user-button")).toBeInTheDocument();
        expect(screen.getByTestId("role-assistant-button")).toBeInTheDocument();
      });
    });

    it("defaults to user role", async () => {
      render(<InsertMessageSheet {...defaultProps} />);

      await waitFor(() => {
        const userButton = screen.getByTestId("role-user-button");
        expect(userButton).toHaveAttribute("data-state", "on");
      });
    });

    it("can switch to assistant role", async () => {
      const user = userEvent.setup();
      render(<InsertMessageSheet {...defaultProps} />);

      const assistantButton = screen.getByTestId("role-assistant-button");
      await user.click(assistantButton);

      await waitFor(() => {
        expect(assistantButton).toHaveAttribute("data-state", "on");
      });
    });
  });

  describe("Content input (AC #4)", () => {
    it("shows content input", async () => {
      render(<InsertMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("content-input")).toBeInTheDocument();
      });
    });

    it("shows token count", async () => {
      render(<InsertMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("token-count-display")).toBeInTheDocument();
      });
    });
  });

  describe("User interactions", () => {
    it("disables confirm when content is empty", async () => {
      render(<InsertMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("confirm-button")).toBeDisabled();
      });
    });

    it("enables confirm when content is entered", async () => {
      const user = userEvent.setup();
      render(<InsertMessageSheet {...defaultProps} />);

      const input = screen.getByTestId("content-input");
      await user.type(input, "Test content");

      await waitFor(() => {
        expect(screen.getByTestId("confirm-button")).not.toBeDisabled();
      });
    });

    it("calls onConfirm with message data", async () => {
      const onConfirm = vi.fn();
      const user = userEvent.setup();
      render(<InsertMessageSheet {...defaultProps} onConfirm={onConfirm} />);

      const input = screen.getByTestId("content-input");
      await user.type(input, "New message");

      const confirmButton = screen.getByTestId("confirm-button");
      await user.click(confirmButton);

      expect(onConfirm).toHaveBeenCalledWith(
        expect.objectContaining({
          role: "user",
          content: [{ type: "text", content: "New message" }],
        })
      );
    });

    it("calls onOpenChange(false) when cancel clicked", async () => {
      const onOpenChange = vi.fn();
      const user = userEvent.setup();
      render(<InsertMessageSheet {...defaultProps} onOpenChange={onOpenChange} />);

      const cancelButton = screen.getByTestId("cancel-button");
      await user.click(cancelButton);

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe("Keyboard navigation (AC #8)", () => {
    it("closes on Escape key press", async () => {
      const onOpenChange = vi.fn();
      const user = userEvent.setup();

      render(<InsertMessageSheet {...defaultProps} onOpenChange={onOpenChange} />);

      await waitFor(() => {
        expect(screen.getByRole("dialog")).toBeInTheDocument();
      });

      await user.keyboard("{Escape}");

      await waitFor(() => {
        expect(onOpenChange).toHaveBeenCalledWith(false);
      });
    });
  });

  describe("Edit mode", () => {
    it("shows edit mode title when initialMessage provided", async () => {
      const initialMessage = {
        id: "test-id",
        role: "assistant" as const,
        content: [{ type: "text" as const, content: "Existing content" }],
        timestamp: "2026-01-01T00:00:00Z",
      };

      render(<InsertMessageSheet {...defaultProps} initialMessage={initialMessage} />);

      await waitFor(() => {
        expect(screen.getByText("编辑插入的消息")).toBeInTheDocument();
      });
    });

    it("prefills content from initialMessage", async () => {
      const initialMessage = {
        id: "test-id",
        role: "user" as const,
        content: [{ type: "text" as const, content: "Prefilled content" }],
        timestamp: "2026-01-01T00:00:00Z",
      };

      render(<InsertMessageSheet {...defaultProps} initialMessage={initialMessage} />);

      await waitFor(() => {
        const input = screen.getByTestId("content-input");
        expect(input).toHaveValue("Prefilled content");
      });
    });
  });
});
