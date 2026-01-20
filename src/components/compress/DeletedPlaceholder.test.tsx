/**
 * DeletedPlaceholder Component Tests
 * Story 10.3: Task 7.4
 *
 * 测试删除占位符渲染
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { DeletedPlaceholder } from "./DeletedPlaceholder";
import type { NarrativeMessage } from "@/types/message";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.previewCard.deleted": "Deleted",
        "compress.previewCard.savedTokens": "saved",
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

describe("DeletedPlaceholder", () => {
  describe("渲染 (AC #2)", () => {
    it("应渲染删除占位符", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      expect(screen.getByTestId("deleted-placeholder")).toBeInTheDocument();
    });

    it("应显示 Deleted 标签", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      expect(screen.getByText("Deleted:")).toBeInTheDocument();
    });

    it("应显示消息类型", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      expect(screen.getByText("User")).toBeInTheDocument();
    });

    it("应显示节省的 token 数", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(<DeletedPlaceholder originalMessage={message} savedTokens={25} />);

      expect(screen.getByText("-25")).toBeInTheDocument();
      expect(screen.getByText("saved")).toBeInTheDocument();
    });
  });

  describe("样式 (AC #2)", () => {
    it("应有虚线边框样式", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      const placeholder = screen.getByTestId("deleted-placeholder");
      expect(placeholder).toHaveClass("border-dashed");
    });

    it("应有淡化样式", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      const placeholder = screen.getByTestId("deleted-placeholder");
      expect(placeholder).toHaveClass("opacity-50");
    });
  });

  describe("消息类型显示", () => {
    it("应正确显示 assistant 类型", () => {
      const message = createTestMessage("msg-1", "assistant", "Hello");

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      expect(screen.getByText("Assistant")).toBeInTheDocument();
    });

    it("应正确显示工具调用类型", () => {
      const message: NarrativeMessage = {
        id: "msg-1",
        role: "assistant",
        timestamp: new Date().toISOString(),
        content: [
          { type: "tool_use", content: "tool call", toolName: "read_file" },
        ],
      };

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      expect(screen.getByText("Tool Call")).toBeInTheDocument();
    });

    it("应正确显示工具结果类型", () => {
      const message: NarrativeMessage = {
        id: "msg-1",
        role: "assistant",
        timestamp: new Date().toISOString(),
        content: [{ type: "tool_result", content: "result" }],
      };

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      expect(screen.getByText("Tool Result")).toBeInTheDocument();
    });
  });

  describe("data-testid 和 data-index", () => {
    it("应有正确的 data-testid", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(<DeletedPlaceholder originalMessage={message} savedTokens={10} />);

      expect(screen.getByTestId("deleted-placeholder")).toBeInTheDocument();
    });

    it("应传递 index 属性", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(
        <DeletedPlaceholder
          originalMessage={message}
          savedTokens={10}
          index={3}
        />
      );

      const placeholder = screen.getByTestId("deleted-placeholder");
      expect(placeholder).toHaveAttribute("data-index", "3");
    });
  });

  describe("className 传递", () => {
    it("应接受自定义 className", () => {
      const message = createTestMessage("msg-1", "user", "Hello");

      render(
        <DeletedPlaceholder
          originalMessage={message}
          savedTokens={10}
          className="custom-class"
        />
      );

      const placeholder = screen.getByTestId("deleted-placeholder");
      expect(placeholder).toHaveClass("custom-class");
    });
  });

  describe("measureElement 回调", () => {
    it("应调用 measureElement 回调", () => {
      const message = createTestMessage("msg-1", "user", "Hello");
      const measureElement = vi.fn();

      render(
        <DeletedPlaceholder
          originalMessage={message}
          savedTokens={10}
          measureElement={measureElement}
        />
      );

      expect(measureElement).toHaveBeenCalled();
    });
  });
});
