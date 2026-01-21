/**
 * InsertedMessageCard Component Tests
 * Story 10.5: Task 8.3
 *
 * 测试已插入消息卡片组件
 * - 渲染样式、删除按钮
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InsertedMessageCard } from "./InsertedMessageCard";
import type { NarrativeMessage } from "@/types/message";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.insertedCard.inserted": "Inserted",
        "compress.insertedCard.removeTooltip": "Remove this inserted message",
        "compress.messageCard.user": "User",
        "compress.messageCard.assistant": "Assistant",
        "compress.messageCard.tokens": "tokens",
        "compress.messageCard.expand": "Expand",
        "compress.messageCard.collapse": "Collapse",
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

// Mock message-utils
vi.mock("@/lib/message-utils", () => ({
  getMessageTextContent: vi.fn((content) => {
    return content
      .filter((block: { type: string }) => block.type === "text")
      .map((block: { content: string }) => block.content)
      .join("\n");
  }),
}));

// Mock Tooltip to simplify testing
vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  TooltipTrigger: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  TooltipContent: ({ children }: { children: React.ReactNode }) => <div data-testid="tooltip">{children}</div>,
}));

describe("InsertedMessageCard", () => {
  const createMessage = (
    content: string,
    role: "user" | "assistant" = "user"
  ): NarrativeMessage => ({
    id: `inserted-msg-${Date.now()}`,
    role,
    timestamp: "2024-01-01T00:00:00Z",
    content: [{ type: "text", content }],
  });

  const defaultProps = {
    message: createMessage("Test inserted message"),
    onRemove: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("卡片渲染 (AC #3)", () => {
    it("应渲染卡片元素", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      expect(screen.getByTestId("inserted-message-card")).toBeInTheDocument();
    });

    it("应设置 data-message-id 属性", () => {
      const message = createMessage("Test");
      render(<InsertedMessageCard {...defaultProps} message={message} />);

      const card = screen.getByTestId("inserted-message-card");
      expect(card).toHaveAttribute("data-message-id", message.id);
    });

    it("应显示消息内容", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      expect(screen.getByText("Test inserted message")).toBeInTheDocument();
    });
  });

  describe("绿色边框样式 (AC #3)", () => {
    it("应有绿色边框类", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      const card = screen.getByTestId("inserted-message-card");
      expect(card).toHaveClass("border-green-500");
    });

    it("应有绿色背景类", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      const card = screen.getByTestId("inserted-message-card");
      expect(card).toHaveClass("bg-green-500/5");
    });
  });

  describe("插入标识 (AC #3)", () => {
    it("应显示 Inserted 标识文字", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      expect(screen.getByText("Inserted")).toBeInTheDocument();
    });
  });

  describe("角色显示", () => {
    it("user 角色应显示 User 标签", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      expect(screen.getByText("User")).toBeInTheDocument();
    });

    it("assistant 角色应显示 Assistant 标签", () => {
      const message = createMessage("AI response", "assistant");
      render(<InsertedMessageCard {...defaultProps} message={message} />);

      expect(screen.getByText("Assistant")).toBeInTheDocument();
    });
  });

  describe("Token 显示", () => {
    it("应显示 Token 数量", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      expect(screen.getByText(/tokens/)).toBeInTheDocument();
    });
  });

  describe("删除按钮 (AC #4)", () => {
    it("应显示删除按钮", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      expect(screen.getByTestId("remove-inserted-button")).toBeInTheDocument();
    });

    it("点击删除按钮应触发 onRemove", () => {
      const onRemove = vi.fn();
      render(<InsertedMessageCard {...defaultProps} onRemove={onRemove} />);

      fireEvent.click(screen.getByTestId("remove-inserted-button"));

      expect(onRemove).toHaveBeenCalledTimes(1);
    });
  });

  describe("长内容折叠", () => {
    it("长内容应显示折叠按钮", () => {
      const longContent = "A".repeat(250);
      const message = createMessage(longContent);
      render(<InsertedMessageCard {...defaultProps} message={message} />);

      expect(screen.getByText("Expand")).toBeInTheDocument();
    });

    it("点击展开按钮应展开内容", async () => {
      const user = userEvent.setup();
      const longContent = "A".repeat(250);
      const message = createMessage(longContent);
      render(<InsertedMessageCard {...defaultProps} message={message} />);

      await user.click(screen.getByText("Expand"));

      expect(screen.getByText("Collapse")).toBeInTheDocument();
    });

    it("短内容不应显示折叠按钮", () => {
      render(<InsertedMessageCard {...defaultProps} />);

      expect(screen.queryByText("Expand")).not.toBeInTheDocument();
    });
  });
});
