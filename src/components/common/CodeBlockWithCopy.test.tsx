/**
 * CodeBlockWithCopy 单元测试
 * Story 2.22: Task 4.7
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { CodeBlockWithCopy } from "./CodeBlockWithCopy";

describe("CodeBlockWithCopy", () => {
  // Mock clipboard API
  const mockWriteText = vi.fn();

  beforeEach(() => {
    // 使用 vi.stubGlobal 来 mock navigator.clipboard
    vi.stubGlobal("navigator", {
      ...navigator,
      clipboard: {
        writeText: mockWriteText,
      },
    });
    mockWriteText.mockResolvedValue(undefined);
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.unstubAllGlobals();
  });

  describe("基础渲染", () => {
    it("应该渲染代码内容", () => {
      render(<CodeBlockWithCopy code="const x = 1;" />);
      expect(screen.getByText("const x = 1;")).toBeInTheDocument();
    });

    it("应该渲染语言标识", () => {
      render(<CodeBlockWithCopy code="const x = 1;" language="typescript" />);
      expect(screen.getByText("typescript")).toBeInTheDocument();
    });

    it("没有语言时不应该渲染语言标识", () => {
      render(<CodeBlockWithCopy code="const x = 1;" />);
      // 只应该有复制按钮，没有语言标识
      expect(screen.queryByText("javascript")).not.toBeInTheDocument();
    });

    it("应该渲染复制按钮", () => {
      render(<CodeBlockWithCopy code="const x = 1;" />);
      expect(
        screen.getByRole("button", { name: "复制代码" })
      ).toBeInTheDocument();
    });
  });

  describe("复制功能", () => {
    it("点击复制按钮应该复制代码内容", async () => {
      const code = `function hello() {
  console.log("Hello World");
}`;
      render(<CodeBlockWithCopy code={code} language="javascript" />);

      const button = screen.getByRole("button", { name: "复制代码" });
      await act(async () => {
        fireEvent.click(button);
      });

      expect(mockWriteText).toHaveBeenCalledWith(code);
    });

    it("复制时不应该包含语言标识", async () => {
      const code = "const x = 1;";
      render(<CodeBlockWithCopy code={code} language="typescript" />);

      await act(async () => {
        fireEvent.click(screen.getByRole("button"));
      });

      // 应该只复制代码，不包含 "typescript"
      expect(mockWriteText).toHaveBeenCalledWith(code);
      expect(mockWriteText).not.toHaveBeenCalledWith(
        expect.stringContaining("typescript")
      );
    });
  });

  describe("样式", () => {
    it("应该有相对定位容器", () => {
      const { container } = render(<CodeBlockWithCopy code="test" />);
      const wrapper = container.firstChild;
      expect(wrapper).toHaveClass("relative");
    });

    it("应该支持自定义 className", () => {
      const { container } = render(
        <CodeBlockWithCopy code="test" className="custom-class" />
      );
      const wrapper = container.firstChild;
      expect(wrapper).toHaveClass("custom-class");
    });
  });
});
