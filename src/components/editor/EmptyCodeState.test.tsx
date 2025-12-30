/**
 * EmptyCodeState 组件测试
 * Story 2.5: Task 6 - 验证与测试
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { EmptyCodeState } from "./EmptyCodeState";

describe("EmptyCodeState", () => {
  describe("AC6 - 空状态处理", () => {
    it("should render empty state UI with icon, title and description", () => {
      render(<EmptyCodeState />);

      // 标题
      expect(screen.getByText("暂无代码")).toBeInTheDocument();

      // 说明
      expect(
        screen.getByText("选择一条对话消息，查看当时的代码快照")
      ).toBeInTheDocument();
    });

    it("should show action guide", () => {
      render(<EmptyCodeState />);

      expect(screen.getByText("点击左侧对话消息")).toBeInTheDocument();
    });

    it("should accept custom className", () => {
      const { container } = render(<EmptyCodeState className="custom-class" />);

      expect(container.firstChild).toHaveClass("custom-class");
    });
  });
});

