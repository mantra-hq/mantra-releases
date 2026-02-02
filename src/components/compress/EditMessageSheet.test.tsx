/**
 * EditMessageSheet Tests
 * Story 10.4: Task 3 - 消息编辑测试
 * Story 12.1: Task 3 - Dialog → Sheet 改造测试
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { EditMessageSheet } from "./EditMessageSheet";
import type { NarrativeMessage } from "@/types/message";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.editDialog.title": "编辑消息",
        "compress.editDialog.description": "编辑消息内容",
        "compress.editDialog.original": "原始内容",
        "compress.editDialog.modified": "修改后",
        "compress.editDialog.placeholder": "输入修改后的内容...",
        "compress.editDialog.cancel": "取消",
        "compress.editDialog.confirm": "确认",
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

// Mock message-utils
vi.mock("@/lib/message-utils", () => ({
  getMessageDisplayContent: vi.fn((content) => {
    if (Array.isArray(content)) {
      return content
        .filter((block: { type: string }) => block.type === "text")
        .map((block: { content: string }) => block.content)
        .join("\n");
    }
    return String(content);
  }),
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

const createMockMessage = (overrides: Partial<NarrativeMessage> = {}): NarrativeMessage => ({
  id: "test-message-id",
  role: "user",
  content: [{ type: "text", content: "Test message content" }],
  timestamp: "2026-01-01T00:00:00Z",
  ...overrides,
});

const defaultProps = {
  open: true,
  onOpenChange: vi.fn(),
  message: createMockMessage(),
  onConfirm: vi.fn(),
};

describe("EditMessageSheet", () => {
  describe("Sheet rendering", () => {
    it("renders sheet when open with message", async () => {
      render(<EditMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByRole("dialog")).toBeInTheDocument();
      });
    });

    it("does not render when closed", () => {
      render(<EditMessageSheet {...defaultProps} open={false} />);

      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    it("does not render when message is null", () => {
      render(<EditMessageSheet {...defaultProps} message={null} />);

      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    // Story 12.1: Sheet 特定测试
    it("renders with data-testid for sheet", async () => {
      render(<EditMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("edit-message-sheet")).toBeInTheDocument();
      });
    });
  });

  describe("Content display", () => {
    it("shows original content", async () => {
      const message = createMockMessage({
        content: [{ type: "text", content: "Original text" }],
      });
      render(<EditMessageSheet {...defaultProps} message={message} />);

      await waitFor(() => {
        expect(screen.getByTestId("original-content")).toHaveTextContent("Original text");
      });
    });

    it("shows editable input with initial content", async () => {
      const message = createMockMessage({
        content: [{ type: "text", content: "Edit me" }],
      });
      render(<EditMessageSheet {...defaultProps} message={message} />);

      await waitFor(() => {
        const input = screen.getByTestId("modified-content-input");
        expect(input).toHaveValue("Edit me");
      });
    });
  });

  describe("Token display", () => {
    it("shows token count", async () => {
      render(<EditMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("token-display")).toBeInTheDocument();
      });
    });
  });

  describe("User interactions", () => {
    it("enables confirm button when content changes", async () => {
      const user = userEvent.setup();
      render(<EditMessageSheet {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId("confirm-button")).toBeDisabled();
      });

      const input = screen.getByTestId("modified-content-input");
      await user.clear(input);
      await user.type(input, "New content");

      await waitFor(() => {
        expect(screen.getByTestId("confirm-button")).not.toBeDisabled();
      });
    });

    it("calls onConfirm with modified content", async () => {
      const onConfirm = vi.fn();
      const user = userEvent.setup();
      render(<EditMessageSheet {...defaultProps} onConfirm={onConfirm} />);

      const input = screen.getByTestId("modified-content-input");
      await user.clear(input);
      await user.type(input, "Modified content");

      const confirmButton = screen.getByTestId("confirm-button");
      await user.click(confirmButton);

      expect(onConfirm).toHaveBeenCalledWith("Modified content");
    });

    it("calls onOpenChange(false) when cancel clicked", async () => {
      const onOpenChange = vi.fn();
      const user = userEvent.setup();
      render(<EditMessageSheet {...defaultProps} onOpenChange={onOpenChange} />);

      const cancelButton = screen.getByTestId("cancel-button");
      await user.click(cancelButton);

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe("Keyboard navigation (AC #8)", () => {
    it("closes on Escape key press", async () => {
      const onOpenChange = vi.fn();
      const user = userEvent.setup();

      render(<EditMessageSheet {...defaultProps} onOpenChange={onOpenChange} />);

      await waitFor(() => {
        expect(screen.getByRole("dialog")).toBeInTheDocument();
      });

      await user.keyboard("{Escape}");

      await waitFor(() => {
        expect(onOpenChange).toHaveBeenCalledWith(false);
      });
    });
  });
});
