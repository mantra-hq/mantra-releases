/**
 * PreviewMessageCard Component Tests
 * Story 10.3: Task 7.3
 *
 * 测试各种状态样式渲染
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PreviewMessageCard } from "./PreviewMessageCard";
import type { PreviewMessage } from "@/hooks/useCompressState";
import type { NarrativeMessage } from "@/types/message";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.previewCard.deleted": "Deleted",
        "compress.previewCard.modified": "Modified",
        "compress.previewCard.inserted": "Inserted",
        "compress.previewCard.savedTokens": "saved",
        "compress.previewCard.tokenDelta": "tokens",
        "compress.messageCard.expand": "Expand",
        "compress.messageCard.collapse": "Collapse",
        "compress.messageCard.tokens": "tokens",
        "compress.messageCard.user": "User",
        "compress.messageCard.assistant": "Assistant",
        "compress.messageCard.toolCall": "Tool Call",
        "compress.messageCard.toolResult": "Tool Result",
        "compress.messageCard.system": "System",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock token-counter
vi.mock("@/lib/token-counter", () => ({
  estimateTokenCount: vi.fn((text: string) => Math.ceil(text.length / 4)),
  formatTokenCount: vi.fn((count: number) => count.toString()),
}));

// Helper: Create test message
function createTestMessage(
  id: string,
  role: "user" | "assistant",
  content: string
): NarrativeMessage {
  return {
    id,
    role,
    timestamp: new Date().toISOString(),
    content: [{ type: "text", content }],
  };
}

// Helper: Create preview message
function createPreviewMessage(
  operation: "keep" | "modify" | "insert",
  message: NarrativeMessage,
  tokenDelta?: number
): PreviewMessage {
  return {
    id: message.id,
    operation,
    message,
    tokenDelta,
  };
}

describe("PreviewMessageCard", () => {
  describe("keep 状态 (AC #2)", () => {
    it("应渲染默认样式", () => {
      const message = createTestMessage("msg-1", "user", "Hello");
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      const card = screen.getByTestId("preview-message-card");
      expect(card).toHaveAttribute("data-operation", "keep");
      expect(card).toHaveClass("bg-card");
    });

    it("应显示用户角色标签", () => {
      const message = createTestMessage("msg-1", "user", "Hello");
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByText("User")).toBeInTheDocument();
    });

    it("应显示 token 数量", () => {
      const message = createTestMessage("msg-1", "user", "Hello World");
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByText(/tokens/)).toBeInTheDocument();
    });
  });

  describe("modify 状态 (AC #3)", () => {
    it("应渲染黄色边框样式", () => {
      const message = createTestMessage("msg-1", "user", "Modified content");
      const previewMessage = createPreviewMessage("modify", message, -5);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      const card = screen.getByTestId("preview-message-card");
      expect(card).toHaveAttribute("data-operation", "modify");
      expect(card).toHaveClass("border-yellow-500");
    });

    it("应显示修改图标和标签", () => {
      const message = createTestMessage("msg-1", "user", "Modified content");
      const previewMessage = createPreviewMessage("modify", message, -5);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByText("Modified")).toBeInTheDocument();
    });

    it("应显示 token 差异", () => {
      const message = createTestMessage("msg-1", "user", "Modified content");
      const previewMessage = createPreviewMessage("modify", message, -5);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByText(/-5 tokens/)).toBeInTheDocument();
    });
  });

  describe("insert 状态 (AC #4)", () => {
    it("应渲染绿色边框样式", () => {
      const message = createTestMessage("msg-1", "user", "Inserted content");
      const previewMessage = createPreviewMessage("insert", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      const card = screen.getByTestId("preview-message-card");
      expect(card).toHaveAttribute("data-operation", "insert");
      expect(card).toHaveClass("border-green-500");
    });

    it("应显示新增图标和标签", () => {
      const message = createTestMessage("msg-1", "user", "Inserted content");
      const previewMessage = createPreviewMessage("insert", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByText("Inserted")).toBeInTheDocument();
    });
  });

  describe("长内容折叠", () => {
    it("长内容应显示展开按钮", () => {
      const longContent = "A".repeat(300); // 超过 MAX_COLLAPSED_CHARS
      const message = createTestMessage("msg-1", "user", longContent);
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByText("Expand")).toBeInTheDocument();
    });

    it("点击展开后应显示收起按钮", () => {
      const longContent = "A".repeat(300);
      const message = createTestMessage("msg-1", "user", longContent);
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      fireEvent.click(screen.getByText("Expand"));

      expect(screen.getByText("Collapse")).toBeInTheDocument();
    });
  });

  describe("角色显示", () => {
    it("应正确显示 assistant 角色", () => {
      const message = createTestMessage("msg-1", "assistant", "Hello");
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByText("Assistant")).toBeInTheDocument();
    });

    it("应正确显示工具调用", () => {
      const message: NarrativeMessage = {
        id: "msg-1",
        role: "assistant",
        timestamp: new Date().toISOString(),
        content: [
          { type: "tool_use", content: "tool call", toolName: "read_file" },
        ],
      };
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByText("Tool Call")).toBeInTheDocument();
    });
  });

  describe("data-testid 和 data-index", () => {
    it("应有正确的 data-testid", () => {
      const message = createTestMessage("msg-1", "user", "Hello");
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} />);

      expect(screen.getByTestId("preview-message-card")).toBeInTheDocument();
    });

    it("应传递 index 属性", () => {
      const message = createTestMessage("msg-1", "user", "Hello");
      const previewMessage = createPreviewMessage("keep", message);

      render(<PreviewMessageCard previewMessage={previewMessage} index={5} />);

      const card = screen.getByTestId("preview-message-card");
      expect(card).toHaveAttribute("data-index", "5");
    });
  });
});
