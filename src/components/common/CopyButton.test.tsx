/**
 * CopyButton 单元测试
 * Story 2.22: Task 1.9
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { CopyButton } from "./CopyButton";

describe("CopyButton", () => {
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
    it("应该渲染 Copy 图标", () => {
      render(<CopyButton content="test" />);
      const button = screen.getByRole("button", { name: "复制" });
      expect(button).toBeInTheDocument();
    });

    it("内容为空时应该禁用按钮", () => {
      render(<CopyButton content="" />);
      const button = screen.getByRole("button");
      expect(button).toBeDisabled();
    });

    it("应该支持 sm 和 md 尺寸", () => {
      const { rerender } = render(<CopyButton content="test" size="sm" />);
      expect(screen.getByRole("button")).toBeInTheDocument();

      rerender(<CopyButton content="test" size="md" />);
      expect(screen.getByRole("button")).toBeInTheDocument();
    });
  });

  describe("复制功能", () => {
    it("点击应该复制内容到剪贴板", async () => {
      render(<CopyButton content="Hello World" />);

      const button = screen.getByRole("button");
      await act(async () => {
        fireEvent.click(button);
      });

      expect(mockWriteText).toHaveBeenCalledWith("Hello World");
    });

    it("复制成功后图标应该变为 Check", async () => {
      render(<CopyButton content="test" />);

      const button = screen.getByRole("button");
      await act(async () => {
        fireEvent.click(button);
      });

      expect(button).toHaveAttribute("aria-label", "已复制");
      expect(button).toHaveAttribute("title", "已复制");
    });

    it("复制成功 2 秒后应该恢复原状 (AC4)", async () => {
      vi.useFakeTimers();

      render(<CopyButton content="test" />);
      const button = screen.getByRole("button");

      // 点击复制
      await act(async () => {
        fireEvent.click(button);
      });

      // 验证已复制状态
      expect(button).toHaveAttribute("aria-label", "已复制");

      // 快进 2 秒
      await act(async () => {
        vi.advanceTimersByTime(2000);
      });

      // 验证恢复原状
      expect(button).toHaveAttribute("aria-label", "复制");

      vi.useRealTimers();
    });

    it("复制成功应该调用 onSuccess 回调", async () => {
      const onSuccess = vi.fn();
      render(<CopyButton content="test" onSuccess={onSuccess} />);

      await act(async () => {
        fireEvent.click(screen.getByRole("button"));
      });

      expect(onSuccess).toHaveBeenCalledTimes(1);
    });

    it("复制失败应该调用 onError 回调", async () => {
      const error = new Error("Copy failed");
      mockWriteText.mockRejectedValueOnce(error);

      const onError = vi.fn();
      render(<CopyButton content="test" onError={onError} />);

      await act(async () => {
        fireEvent.click(screen.getByRole("button"));
      });

      await waitFor(() => {
        expect(onError).toHaveBeenCalledWith(error);
      });
    });
  });

  describe("无障碍支持", () => {
    it("应该有正确的 aria-label", () => {
      render(<CopyButton content="test" ariaLabel="复制代码" />);
      expect(screen.getByRole("button")).toHaveAttribute(
        "aria-label",
        "复制代码"
      );
    });

    it("应该有 aria-pressed 属性", () => {
      render(<CopyButton content="test" />);
      expect(screen.getByRole("button")).toHaveAttribute("aria-pressed");
    });

    it("Enter 键应该触发复制", async () => {
      render(<CopyButton content="test" />);
      const button = screen.getByRole("button");

      await act(async () => {
        fireEvent.keyDown(button, { key: "Enter" });
      });

      expect(mockWriteText).toHaveBeenCalledWith("test");
    });

    it("Space 键应该触发复制", async () => {
      render(<CopyButton content="test" />);
      const button = screen.getByRole("button");

      await act(async () => {
        fireEvent.keyDown(button, { key: " " });
      });

      expect(mockWriteText).toHaveBeenCalledWith("test");
    });

    it("复制成功后应该有屏幕阅读器通知", async () => {
      render(<CopyButton content="test" />);

      await act(async () => {
        fireEvent.click(screen.getByRole("button"));
      });

      expect(screen.getByRole("status")).toHaveTextContent("已复制到剪贴板");
    });
  });

  describe("tooltip 支持", () => {
    it("应该显示默认 tooltip", () => {
      render(<CopyButton content="test" />);
      expect(screen.getByRole("button")).toHaveAttribute("title", "复制");
    });

    it("应该支持自定义 tooltip", () => {
      render(<CopyButton content="test" tooltip="复制文件路径" />);
      expect(screen.getByRole("button")).toHaveAttribute(
        "title",
        "复制文件路径"
      );
    });
  });
});
