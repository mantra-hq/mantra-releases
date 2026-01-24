/**
 * MessageActionButtons Component Tests
 * Story 10.4: Task 7.1
 *
 * 测试消息操作按钮组件
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MessageActionButtons } from "./MessageActionButtons";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.actions.keep": "Keep",
        "compress.actions.keepTooltip": "Keep this message (K)",
        "compress.actions.delete": "Delete",
        "compress.actions.deleteTooltip": "Mark for deletion (D)",
        "compress.actions.edit": "Edit",
        "compress.actions.editTooltip": "Edit message content (E)",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock Tooltip to simplify testing
vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  TooltipTrigger: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  TooltipContent: ({ children }: { children: React.ReactNode }) => <div data-testid="tooltip">{children}</div>,
}));

describe("MessageActionButtons", () => {
  const defaultProps = {
    messageId: "msg-1",
    currentOperation: "keep" as const,
    onKeepClick: vi.fn(),
    onDeleteClick: vi.fn(),
    onEditClick: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("按钮渲染 (AC #1)", () => {
    it("应渲染三个操作按钮", () => {
      render(<MessageActionButtons {...defaultProps} />);

      expect(screen.getByTestId("action-keep")).toBeInTheDocument();
      expect(screen.getByTestId("action-delete")).toBeInTheDocument();
      expect(screen.getByTestId("action-edit")).toBeInTheDocument();
    });

    it("应有 message-action-buttons testid", () => {
      render(<MessageActionButtons {...defaultProps} />);

      expect(screen.getByTestId("message-action-buttons")).toBeInTheDocument();
    });

    it("应设置 data-message-id 属性", () => {
      render(<MessageActionButtons {...defaultProps} messageId="test-msg-123" />);

      const container = screen.getByTestId("message-action-buttons");
      expect(container).toHaveAttribute("data-message-id", "test-msg-123");
    });
  });

  describe("状态高亮 (AC #1)", () => {
    it("keep 状态时保留按钮应激活", () => {
      render(<MessageActionButtons {...defaultProps} currentOperation="keep" />);

      const keepButton = screen.getByTestId("action-keep");
      expect(keepButton).toHaveAttribute("aria-pressed", "true");
    });

    it("delete 状态时删除按钮应激活", () => {
      render(<MessageActionButtons {...defaultProps} currentOperation="delete" />);

      const deleteButton = screen.getByTestId("action-delete");
      expect(deleteButton).toHaveAttribute("aria-pressed", "true");
    });

    it("modify 状态时修改按钮应激活", () => {
      render(<MessageActionButtons {...defaultProps} currentOperation="modify" />);

      const editButton = screen.getByTestId("action-edit");
      expect(editButton).toHaveAttribute("aria-pressed", "true");
    });
  });

  describe("点击回调 (AC #2, #5)", () => {
    it("点击保留按钮应触发 onKeepClick", () => {
      const onKeepClick = vi.fn();
      render(<MessageActionButtons {...defaultProps} onKeepClick={onKeepClick} />);

      fireEvent.click(screen.getByTestId("action-keep"));

      expect(onKeepClick).toHaveBeenCalledTimes(1);
    });

    it("点击删除按钮应触发 onDeleteClick", () => {
      const onDeleteClick = vi.fn();
      render(<MessageActionButtons {...defaultProps} onDeleteClick={onDeleteClick} />);

      fireEvent.click(screen.getByTestId("action-delete"));

      expect(onDeleteClick).toHaveBeenCalledTimes(1);
    });

    it("点击修改按钮应触发 onEditClick", () => {
      const onEditClick = vi.fn();
      render(<MessageActionButtons {...defaultProps} onEditClick={onEditClick} />);

      fireEvent.click(screen.getByTestId("action-edit"));

      expect(onEditClick).toHaveBeenCalledTimes(1);
    });
  });

  describe("无障碍访问", () => {
    it("按钮应有 aria-label", () => {
      render(<MessageActionButtons {...defaultProps} />);

      expect(screen.getByTestId("action-keep")).toHaveAttribute("aria-label", "Keep");
      expect(screen.getByTestId("action-delete")).toHaveAttribute("aria-label", "Delete");
      expect(screen.getByTestId("action-edit")).toHaveAttribute("aria-label", "Edit");
    });
  });
});
