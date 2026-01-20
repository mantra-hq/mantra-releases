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
        "compress.actions.edited": "Edited",
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

// Mock MessageActionButtons for simpler testing
vi.mock("./MessageActionButtons", () => ({
  MessageActionButtons: ({ messageId, onKeepClick, onDeleteClick, onEditClick }: any) => (
    <div data-testid="mock-action-buttons" data-message-id={messageId}>
      <button data-testid="mock-keep" onClick={onKeepClick}>Keep</button>
      <button data-testid="mock-delete" onClick={onDeleteClick}>Delete</button>
      <button data-testid="mock-edit" onClick={onEditClick}>Edit</button>
    </div>
  ),
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

  // Story 10.4: 操作按钮和状态样式测试
  describe("操作按钮集成 (Story 10.4 AC #1)", () => {
    it("showActionButtons=true 时应显示操作按钮", () => {
      const onKeepClick = vi.fn();
      const onDeleteClick = vi.fn();
      const onEditClick = vi.fn();

      render(
        <OriginalMessageCard
          message={createMessage("user", "Hello")}
          showActionButtons={true}
          currentOperation="keep"
          onKeepClick={onKeepClick}
          onDeleteClick={onDeleteClick}
          onEditClick={onEditClick}
        />
      );

      expect(screen.getByTestId("mock-action-buttons")).toBeInTheDocument();
    });

    it("showActionButtons=false 时不应显示操作按钮", () => {
      render(
        <OriginalMessageCard
          message={createMessage("user", "Hello")}
          showActionButtons={false}
        />
      );

      expect(screen.queryByTestId("mock-action-buttons")).not.toBeInTheDocument();
    });

    it("点击删除按钮应触发 onDeleteClick", () => {
      const onDeleteClick = vi.fn();

      render(
        <OriginalMessageCard
          message={createMessage("user", "Hello")}
          showActionButtons={true}
          currentOperation="keep"
          onKeepClick={vi.fn()}
          onDeleteClick={onDeleteClick}
          onEditClick={vi.fn()}
        />
      );

      fireEvent.click(screen.getByTestId("mock-delete"));
      expect(onDeleteClick).toHaveBeenCalledTimes(1);
    });
  });

  describe("删除状态样式 (Story 10.4 AC #2)", () => {
    it("删除状态应设置 data-operation 属性", () => {
      render(
        <OriginalMessageCard
          message={createMessage("user", "Hello")}
          currentOperation="delete"
        />
      );

      const card = screen.getByTestId("original-message-card");
      expect(card).toHaveAttribute("data-operation", "delete");
    });
  });

  describe("修改状态样式 (Story 10.4 AC #4)", () => {
    it("修改状态应设置 data-operation 属性", () => {
      render(
        <OriginalMessageCard
          message={createMessage("user", "Hello")}
          currentOperation="modify"
        />
      );

      const card = screen.getByTestId("original-message-card");
      expect(card).toHaveAttribute("data-operation", "modify");
    });

    it("修改状态应显示 Edited 标识", () => {
      render(
        <OriginalMessageCard
          message={createMessage("user", "Hello")}
          currentOperation="modify"
        />
      );

      expect(screen.getByText("Edited")).toBeInTheDocument();
    });
  });

  describe("保留状态 (Story 10.4 AC #5)", () => {
    it("默认应为 keep 状态", () => {
      render(<OriginalMessageCard message={createMessage("user", "Hello")} />);

      const card = screen.getByTestId("original-message-card");
      expect(card).toHaveAttribute("data-operation", "keep");
    });
  });
});
