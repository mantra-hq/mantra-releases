/**
 * CompressPreviewList Component Tests
 * Story 10.3: Task 7.2
 *
 * 测试虚拟化渲染、模式切换、空状态
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import * as React from "react";
import { CompressPreviewList } from "./CompressPreviewList";
import { CompressStateProvider } from "@/hooks/useCompressState";
import type { NarrativeMessage } from "@/types/message";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.previewList.title": "Compressed Preview",
        "compress.previewList.empty": "No changes yet",
        "compress.previewList.emptyHint": "Mark messages as delete/modify on the left panel",
        "compress.previewList.modeFull": "Full",
        "compress.previewList.modeChanges": "Changes Only",
        "compress.previewList.modeHideDeleted": "Hide Deleted",
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

// Mock @tanstack/react-virtual
vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: vi.fn(() => ({
    getVirtualItems: () => [],
    getTotalSize: () => 0,
    measureElement: vi.fn(),
  })),
}));

// Helper: Create test messages
function createMessages(count: number): NarrativeMessage[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `msg-${i}`,
    role: i % 2 === 0 ? "user" : "assistant",
    timestamp: new Date(2024, 0, 1, 0, i).toISOString(),
    content: [{ type: "text", content: `Message ${i}` }],
  }));
}

// Wrapper component
const TestWrapper = ({ children }: { children: React.ReactNode }) => (
  <CompressStateProvider>{children}</CompressStateProvider>
);

describe("CompressPreviewList", () => {
  describe("空状态", () => {
    it("无消息时应显示空状态", () => {
      render(
        <TestWrapper>
          <CompressPreviewList messages={[]} />
        </TestWrapper>
      );

      expect(screen.getByText("No changes yet")).toBeInTheDocument();
      expect(
        screen.getByText("Mark messages as delete/modify on the left panel")
      ).toBeInTheDocument();
    });
  });

  describe("预览模式切换 (AC #5)", () => {
    it("应默认显示完整预览模式", () => {
      const messages = createMessages(2);
      render(
        <TestWrapper>
          <CompressPreviewList messages={messages} />
        </TestWrapper>
      );

      // 检查 "Full" 按钮存在
      expect(screen.getByText("Full")).toBeInTheDocument();
    });

    it("应显示所有三个预览模式选项", () => {
      const messages = createMessages(2);
      render(
        <TestWrapper>
          <CompressPreviewList messages={messages} />
        </TestWrapper>
      );

      expect(screen.getByText("Full")).toBeInTheDocument();
      expect(screen.getByText("Changes Only")).toBeInTheDocument();
      expect(screen.getByText("Hide Deleted")).toBeInTheDocument();
    });

    it("应显示标题", () => {
      const messages = createMessages(2);
      render(
        <TestWrapper>
          <CompressPreviewList messages={messages} />
        </TestWrapper>
      );

      expect(screen.getByText("Compressed Preview")).toBeInTheDocument();
    });
  });

  describe("data-testid", () => {
    it("有消息时应有 compress-preview-list testid", () => {
      const messages = createMessages(1);
      render(
        <TestWrapper>
          <CompressPreviewList messages={messages} />
        </TestWrapper>
      );

      expect(screen.getByTestId("compress-preview-list")).toBeInTheDocument();
    });
  });

  describe("虚拟化配置 (AC #6)", () => {
    it("应使用 useVirtualizer hook", async () => {
      const { useVirtualizer } = await import("@tanstack/react-virtual");
      const messages = createMessages(5);

      render(
        <TestWrapper>
          <CompressPreviewList messages={messages} />
        </TestWrapper>
      );

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
        <TestWrapper>
          <CompressPreviewList messages={[]} className="custom-class" />
        </TestWrapper>
      );

      const container = document.querySelector(".custom-class");
      expect(container).toBeInTheDocument();
    });
  });
});
