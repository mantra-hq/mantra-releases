/**
 * ResetConfirmDialog Component Tests
 * Story 10.8: Task 8.3
 *
 * 测试对话框打开/关闭、确认重置
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ResetConfirmDialog } from "./ResetConfirmDialog";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.operations.resetConfirm.title": "Reset All Changes?",
        "compress.operations.resetConfirm.description": "This will discard all your edits and restore the original message list. This action cannot be undone.",
        "compress.operations.resetConfirm.cancel": "Cancel",
        "compress.operations.resetConfirm.confirm": "Reset",
      };
      return translations[key] || key;
    },
  }),
}));

describe("ResetConfirmDialog", () => {
  describe("渲染", () => {
    it("open=false 时不应渲染对话框内容", () => {
      render(
        <ResetConfirmDialog
          open={false}
          onOpenChange={() => {}}
          onConfirm={() => {}}
        />
      );

      expect(screen.queryByTestId("reset-confirm-dialog")).not.toBeInTheDocument();
    });

    it("open=true 时应渲染对话框", () => {
      render(
        <ResetConfirmDialog
          open={true}
          onOpenChange={() => {}}
          onConfirm={() => {}}
        />
      );

      expect(screen.getByTestId("reset-confirm-dialog")).toBeInTheDocument();
      expect(screen.getByText("Reset All Changes?")).toBeInTheDocument();
      expect(screen.getByText(/This will discard all your edits/)).toBeInTheDocument();
    });

    it("应渲染取消和确认按钮", () => {
      render(
        <ResetConfirmDialog
          open={true}
          onOpenChange={() => {}}
          onConfirm={() => {}}
        />
      );

      expect(screen.getByTestId("reset-cancel-button")).toBeInTheDocument();
      expect(screen.getByTestId("reset-confirm-button")).toBeInTheDocument();
    });
  });

  describe("交互", () => {
    it("点击取消按钮应调用 onOpenChange(false)", () => {
      const onOpenChange = vi.fn();

      render(
        <ResetConfirmDialog
          open={true}
          onOpenChange={onOpenChange}
          onConfirm={() => {}}
        />
      );

      fireEvent.click(screen.getByTestId("reset-cancel-button"));

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("点击确认按钮应调用 onConfirm", () => {
      const onConfirm = vi.fn();

      render(
        <ResetConfirmDialog
          open={true}
          onOpenChange={() => {}}
          onConfirm={onConfirm}
        />
      );

      fireEvent.click(screen.getByTestId("reset-confirm-button"));

      expect(onConfirm).toHaveBeenCalled();
    });
  });
});
