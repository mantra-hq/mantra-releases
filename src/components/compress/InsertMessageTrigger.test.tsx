/**
 * InsertMessageTrigger Component Tests
 * Story 10.5: Task 8.1
 *
 * 测试消息插入触发器组件
 * - 悬停显示、点击触发、已插入状态
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InsertMessageTrigger } from "./InsertMessageTrigger";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.insertTrigger.tooltip": "Insert message here",
        "compress.insertTrigger.hasInsertion": "Message inserted",
        "compress.insertTrigger.removeTooltip": "Remove inserted message",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock Tooltip to simplify testing
vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  TooltipTrigger: ({ children, asChild }: { children: React.ReactNode; asChild?: boolean }) => <>{children}</>,
  TooltipContent: ({ children }: { children: React.ReactNode }) => <div data-testid="tooltip">{children}</div>,
}));

describe("InsertMessageTrigger", () => {
  const defaultProps = {
    afterIndex: 0,
    hasInsertion: false,
    onClick: vi.fn(),
    onRemoveInsertion: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("默认状态渲染 (AC #1)", () => {
    it("应渲染触发器元素", () => {
      render(<InsertMessageTrigger {...defaultProps} />);

      expect(screen.getByTestId("insert-message-trigger")).toBeInTheDocument();
    });

    it("应设置 data-after-index 属性", () => {
      render(<InsertMessageTrigger {...defaultProps} afterIndex={5} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      expect(trigger).toHaveAttribute("data-after-index", "5");
    });

    it("默认状态 data-has-insertion 应为 false", () => {
      render(<InsertMessageTrigger {...defaultProps} hasInsertion={false} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      expect(trigger).toHaveAttribute("data-has-insertion", "false");
    });

    it("默认状态应有较低高度", () => {
      render(<InsertMessageTrigger {...defaultProps} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      // AC1: 默认状态透明占位，高度约 8px (h-2)
      expect(trigger).toHaveClass("h-2");
    });
  });

  describe("悬停状态 (AC #1)", () => {
    it("悬停时应显示 Plus 图标和提示文字", async () => {
      const user = userEvent.setup();
      render(<InsertMessageTrigger {...defaultProps} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      await user.hover(trigger);

      expect(screen.getByText("Insert message here")).toBeInTheDocument();
    });

    it("悬停时应扩展高度", async () => {
      const user = userEvent.setup();
      render(<InsertMessageTrigger {...defaultProps} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      await user.hover(trigger);

      expect(trigger).toHaveClass("h-6");
    });

    it("悬停时应有虚线边框样式", async () => {
      const user = userEvent.setup();
      render(<InsertMessageTrigger {...defaultProps} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      await user.hover(trigger);

      expect(trigger).toHaveClass("border-dashed");
    });

    it("离开悬停后应恢复默认状态", async () => {
      const user = userEvent.setup();
      render(<InsertMessageTrigger {...defaultProps} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      await user.hover(trigger);
      await user.unhover(trigger);

      // AC1: 恢复默认状态 h-2
      expect(trigger).toHaveClass("h-2");
    });
  });

  describe("点击触发 (AC #1)", () => {
    it("点击应触发 onClick 回调", () => {
      const onClick = vi.fn();
      render(<InsertMessageTrigger {...defaultProps} onClick={onClick} />);

      fireEvent.click(screen.getByTestId("insert-message-trigger"));

      expect(onClick).toHaveBeenCalledTimes(1);
    });

    it("按 Enter 键应触发 onClick", () => {
      const onClick = vi.fn();
      render(<InsertMessageTrigger {...defaultProps} onClick={onClick} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      fireEvent.keyDown(trigger, { key: "Enter" });

      expect(onClick).toHaveBeenCalledTimes(1);
    });

    it("按 Space 键应触发 onClick", () => {
      const onClick = vi.fn();
      render(<InsertMessageTrigger {...defaultProps} onClick={onClick} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      fireEvent.keyDown(trigger, { key: " " });

      expect(onClick).toHaveBeenCalledTimes(1);
    });
  });

  describe("已插入状态 (AC #1)", () => {
    it("hasInsertion 为 true 时应显示已插入状态", () => {
      render(<InsertMessageTrigger {...defaultProps} hasInsertion={true} />);

      const trigger = screen.getByTestId("insert-message-trigger");
      expect(trigger).toHaveAttribute("data-has-insertion", "true");
    });

    it("已插入状态应显示删除按钮", () => {
      render(<InsertMessageTrigger {...defaultProps} hasInsertion={true} />);

      expect(screen.getByTestId("remove-insertion-button")).toBeInTheDocument();
    });

    it("已插入状态应显示提示文字", () => {
      render(<InsertMessageTrigger {...defaultProps} hasInsertion={true} />);

      expect(screen.getByText("Message inserted")).toBeInTheDocument();
    });

    it("点击删除按钮应触发 onRemoveInsertion", () => {
      const onRemoveInsertion = vi.fn();
      render(
        <InsertMessageTrigger
          {...defaultProps}
          hasInsertion={true}
          onRemoveInsertion={onRemoveInsertion}
        />
      );

      fireEvent.click(screen.getByTestId("remove-insertion-button"));

      expect(onRemoveInsertion).toHaveBeenCalledTimes(1);
    });
  });
});
