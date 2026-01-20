/**
 * OriginalMessageCard Component Tests
 * Story 10.2: Task 7.2
 *
 * 测试原始消息卡片组件
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { OriginalMessageCard } from "./OriginalMessageCard";
import type { NarrativeMessage } from "@/types/message";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.messageCard.expand": "Expand",
        "compress.messageCard.collapse": "Collapse",
        "compress.messageCard.tokens": "tokens",
        "compress.messageCard.toolCall": "Tool Call",
        "compress.messageCard.toolResult": "Tool Result",
        "compress.messageCard.system": "System",
        "compress.messageCard.user": "User",
        "compress.messageCard.assistant": "Assistant",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock token-counter
vi.mock("@/lib/token-counter", () => ({
  estimateTokenCount: vi.fn(() => 42),
  formatTokenCount: vi.fn((count: number) => count.toString()),
}));

describe("OriginalMessageCard", () => {
  const createMessage = (
    role: "user" | "assistant",
    content: string,
    hasToolUse = false
  ): NarrativeMessage => ({
    id: "msg-1",
    role,
    timestamp: "2024-01-01T00:00:00Z",
    content: hasToolUse
      ? [
          { type: "tool_use", content: "", toolName: "Read" },
          { type: "text", content },
        ]
      : [{ type: "text", content }],
  });

  describe("角色图标显示 (AC #2)", () => {
    it("用户消息应显示 User 标签", () => {
      render(<OriginalMessageCard message={createMessage("user", "Hello")} />);

      expect(screen.getByText("User")).toBeInTheDocument();
    });

    it("助手消息应显示 Assistant 标签", () => {
      render(
        <OriginalMessageCard message={createMessage("assistant", "Hi there")} />
      );

      expect(screen.getByText("Assistant")).toBeInTheDocument();
    });

    it("工具调用应显示 Tool Call 标签和工具名称", () => {
      render(
        <OriginalMessageCard
          message={createMessage("assistant", "Reading file", true)}
        />
      );

      expect(screen.getByText("Tool Call")).toBeInTheDocument();
      expect(screen.getByText("· Read")).toBeInTheDocument();
    });
  });

  describe("Token 数量显示 (AC #1)", () => {
    it("应显示 Token 数量徽章", () => {
      render(<OriginalMessageCard message={createMessage("user", "Hello")} />);

      expect(screen.getByText("42 tokens")).toBeInTheDocument();
    });
  });

  describe("内容显示 (AC #1)", () => {
    it("应显示消息文本内容", () => {
      const content = "This is a test message";
      render(<OriginalMessageCard message={createMessage("user", content)} />);

      expect(screen.getByText(content)).toBeInTheDocument();
    });
  });

  describe("长内容折叠功能 (AC #3)", () => {
    const longContent = `${"Line of text\n".repeat(10)}`;

    it("长内容默认应折叠并显示展开按钮", () => {
      render(
        <OriginalMessageCard message={createMessage("user", longContent)} />
      );

      expect(screen.getByText("Expand")).toBeInTheDocument();
    });

    it("点击展开按钮后应显示收起按钮", () => {
      render(
        <OriginalMessageCard message={createMessage("user", longContent)} />
      );

      fireEvent.click(screen.getByText("Expand"));

      expect(screen.getByText("Collapse")).toBeInTheDocument();
    });

    it("点击收起按钮后应恢复折叠状态", () => {
      render(
        <OriginalMessageCard message={createMessage("user", longContent)} />
      );

      // 展开
      fireEvent.click(screen.getByText("Expand"));
      expect(screen.getByText("Collapse")).toBeInTheDocument();

      // 收起
      fireEvent.click(screen.getByText("Collapse"));
      expect(screen.getByText("Expand")).toBeInTheDocument();
    });

    it("短内容不应显示折叠按钮", () => {
      render(
        <OriginalMessageCard message={createMessage("user", "Short text")} />
      );

      expect(screen.queryByText("Expand")).not.toBeInTheDocument();
      expect(screen.queryByText("Collapse")).not.toBeInTheDocument();
    });
  });

  describe("data-testid", () => {
    it("应有 original-message-card testid", () => {
      render(<OriginalMessageCard message={createMessage("user", "Hello")} />);

      expect(screen.getByTestId("original-message-card")).toBeInTheDocument();
    });
  });

  describe("虚拟化支持", () => {
    it("应接受 measureElement 回调", () => {
      const measureElement = vi.fn();

      render(
        <OriginalMessageCard
          message={createMessage("user", "Hello")}
          measureElement={measureElement}
          index={0}
        />
      );

      // measureElement 应该在渲染时被调用
      expect(measureElement).toHaveBeenCalled();
    });

    it("应设置 data-index 属性", () => {
      render(
        <OriginalMessageCard
          message={createMessage("user", "Hello")}
          index={5}
        />
      );

      const card = screen.getByTestId("original-message-card");
      expect(card).toHaveAttribute("data-index", "5");
    });
  });
});
