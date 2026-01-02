/**
 * BreadcrumbItem Tests - 面包屑项组件测试
 * Story 2.17: Task 2
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BreadcrumbItem } from "./BreadcrumbItem";
import { FolderOpen } from "lucide-react";

describe("BreadcrumbItem", () => {
  const mockOnClick = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("UI 展示", () => {
    it("应该显示标签文字", () => {
      render(<BreadcrumbItem label="test-label" testId="test-item" />);
      expect(screen.getByTestId("test-item")).toHaveTextContent("test-label");
    });

    it("应该显示图标", () => {
      render(
        <BreadcrumbItem
          icon={<FolderOpen data-testid="icon" />}
          label="test"
          testId="test-item"
        />
      );
      expect(screen.getByTestId("icon")).toBeInTheDocument();
    });

    it("应该显示子元素", () => {
      render(
        <BreadcrumbItem testId="test-item">
          <span data-testid="child">Child</span>
        </BreadcrumbItem>
      );
      expect(screen.getByTestId("child")).toBeInTheDocument();
    });
  });

  describe("可点击状态", () => {
    it("有 onClick 时应该渲染为按钮", () => {
      render(
        <BreadcrumbItem
          label="test"
          onClick={mockOnClick}
          testId="test-item"
        />
      );
      const button = screen.getByTestId("test-item");
      expect(button.tagName).toBe("BUTTON");
    });

    it("点击按钮应该触发 onClick", async () => {
      const user = userEvent.setup();
      render(
        <BreadcrumbItem
          label="test"
          onClick={mockOnClick}
          testId="test-item"
        />
      );

      await user.click(screen.getByTestId("test-item"));
      expect(mockOnClick).toHaveBeenCalledTimes(1);
    });

    it("无 onClick 时应该渲染为 span", () => {
      render(<BreadcrumbItem label="test" testId="test-item" />);
      const span = screen.getByTestId("test-item");
      expect(span.tagName).toBe("SPAN");
    });
  });

  describe("截断功能", () => {
    it("truncate=true 时应该有 truncate 类", () => {
      render(
        <BreadcrumbItem
          label="very-long-label-that-should-be-truncated"
          truncate
          testId="test-item"
        />
      );
      const labelSpan = screen.getByTestId("test-item").querySelector("span");
      expect(labelSpan).toHaveClass("truncate");
    });

    it("truncate=false 时不应该有 truncate 类", () => {
      render(<BreadcrumbItem label="short" testId="test-item" />);
      const labelSpan = screen.getByTestId("test-item").querySelector("span");
      expect(labelSpan).not.toHaveClass("truncate");
    });
  });

  describe("无障碍", () => {
    it("应该支持 aria-label", () => {
      render(
        <BreadcrumbItem
          label="test"
          onClick={mockOnClick}
          aria-label="自定义标签"
          testId="test-item"
        />
      );
      expect(screen.getByTestId("test-item")).toHaveAttribute(
        "aria-label",
        "自定义标签"
      );
    });
  });
});
