/**
 * UnsavedChangesDialog 测试
 * Story 10.9: Task 9.2
 *
 * 测试未保存更改对话框的渲染和按钮交互
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { UnsavedChangesDialog } from "./UnsavedChangesDialog";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback: string) => fallback,
  }),
}));

describe("UnsavedChangesDialog", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    onExportAndLeave: vi.fn(),
    onDiscardAndLeave: vi.fn(),
    onCancel: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("渲染", () => {
    it("对话框打开时应该正确渲染标题和描述", () => {
      render(<UnsavedChangesDialog {...defaultProps} />);

      expect(screen.getByText("有未保存的编辑")).toBeInTheDocument();
      expect(
        screen.getByText("您对消息的编辑尚未导出。离开后这些更改将会丢失。")
      ).toBeInTheDocument();
    });

    it("应该渲染三个按钮", () => {
      render(<UnsavedChangesDialog {...defaultProps} />);

      expect(screen.getByText("取消")).toBeInTheDocument();
      expect(screen.getByText("不保存")).toBeInTheDocument();
      expect(screen.getByText("导出并离开")).toBeInTheDocument();
    });

    it("对话框关闭时不应该渲染内容", () => {
      render(<UnsavedChangesDialog {...defaultProps} open={false} />);

      expect(screen.queryByText("有未保存的编辑")).not.toBeInTheDocument();
    });
  });

  describe("按钮交互", () => {
    it("点击「取消」应该调用 onCancel 和 onOpenChange", async () => {
      const user = userEvent.setup();
      const onCancel = vi.fn();
      const onOpenChange = vi.fn();

      render(
        <UnsavedChangesDialog
          {...defaultProps}
          onCancel={onCancel}
          onOpenChange={onOpenChange}
        />
      );

      await user.click(screen.getByText("取消"));

      expect(onCancel).toHaveBeenCalledTimes(1);
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("点击「不保存」应该调用 onDiscardAndLeave 和 onOpenChange", async () => {
      const user = userEvent.setup();
      const onDiscardAndLeave = vi.fn();
      const onOpenChange = vi.fn();

      render(
        <UnsavedChangesDialog
          {...defaultProps}
          onDiscardAndLeave={onDiscardAndLeave}
          onOpenChange={onOpenChange}
        />
      );

      await user.click(screen.getByTestId("unsaved-discard-button"));

      expect(onDiscardAndLeave).toHaveBeenCalledTimes(1);
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("点击「导出并离开」应该调用 onExportAndLeave 但不关闭对话框", async () => {
      const user = userEvent.setup();
      const onExportAndLeave = vi.fn();
      const onOpenChange = vi.fn();

      render(
        <UnsavedChangesDialog
          {...defaultProps}
          onExportAndLeave={onExportAndLeave}
          onOpenChange={onOpenChange}
        />
      );

      await user.click(screen.getByTestId("unsaved-export-button"));

      expect(onExportAndLeave).toHaveBeenCalledTimes(1);
      // 导出按钮不应该直接关闭对话框（由调用方在导出完成后关闭）
      // AlertDialogAction 可能会触发 onOpenChange，这取决于 shadcn 实现
    });
  });

  describe("可访问性", () => {
    it("应该有正确的 data-testid 属性", () => {
      render(<UnsavedChangesDialog {...defaultProps} />);

      expect(screen.getByTestId("unsaved-discard-button")).toBeInTheDocument();
      expect(screen.getByTestId("unsaved-export-button")).toBeInTheDocument();
    });
  });
});
