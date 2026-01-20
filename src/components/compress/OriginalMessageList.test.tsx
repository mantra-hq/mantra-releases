/**
 * OriginalMessageList Component Tests
 * Story 10.2: Task 7.1
 *
 * 测试原始消息列表组件的虚拟化渲染和空状态
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { OriginalMessageList } from "./OriginalMessageList";
import type { NarrativeMessage } from "@/types/message";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.originalList.title": "Original Messages",
        "compress.originalList.empty": "No messages to display",
        "compress.originalList.emptyHint": "Import a session to view messages",
        "compress.messageCard.expand": "Expand",
        "compress.messageCard.collapse": "Collapse",
        "compress.messageCard.tokens": "tokens",
        "compress.messageCard.user": "User",
        "compress.messageCard.assistant": "Assistant",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock token-counter
vi.mock("@/lib/token-counter", () => ({
  estimateTokenCount: vi.fn(() => 10),
}));

// Mock @tanstack/react-virtual
vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: vi.fn(() => ({
    getVirtualItems: () => [],
    getTotalSize: () => 0,
    measureElement: vi.fn(),
  })),
}));

describe("OriginalMessageList", () => {
  const createMessages = (count: number): NarrativeMessage[] =>
    Array.from({ length: count }, (_, i) => ({
      id: `msg-${i}`,
      role: i % 2 === 0 ? "user" : "assistant",
      timestamp: new Date(2024, 0, 1, 0, i).toISOString(),
      content: [{ type: "text", content: `Message ${i}` }],
    }));

  describe("空状态 (AC #1)", () => {
    it("无消息时应显示空状态", () => {
      render(<OriginalMessageList messages={[]} />);

      expect(screen.getByText("No messages to display")).toBeInTheDocument();
      expect(
        screen.getByText("Import a session to view messages")
      ).toBeInTheDocument();
    });

    it("空状态应显示图标", () => {
      const { container } = render(<OriginalMessageList messages={[]} />);

      // MessageSquare icon 应该存在
      expect(container.querySelector("svg")).toBeInTheDocument();
    });
  });

  describe("data-testid", () => {
    it("有消息时应有 original-message-list testid", () => {
      const messages = createMessages(1);
      render(<OriginalMessageList messages={messages} />);

      expect(screen.getByTestId("original-message-list")).toBeInTheDocument();
    });
  });

  describe("虚拟化配置 (AC #4)", () => {
    it("应使用 useVirtualizer hook", async () => {
      const { useVirtualizer } = await import("@tanstack/react-virtual");
      const messages = createMessages(5);

      render(<OriginalMessageList messages={messages} />);

      expect(useVirtualizer).toHaveBeenCalledWith(
        expect.objectContaining({
          count: 5,
          overscan: 5,
        })
      );
    });
  });

  describe("className 传递", () => {
    it("应接受自定义 className", () => {
      render(
        <OriginalMessageList messages={[]} className="custom-class" />
      );

      const container = document.querySelector(".custom-class");
      expect(container).toBeInTheDocument();
    });
  });
});
