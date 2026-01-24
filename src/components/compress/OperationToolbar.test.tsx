/**
 * OperationToolbar Component Tests
 * Story 10.8: Task 8.2
 *
 * 测试按钮渲染、禁用状态、点击事件、键盘快捷键
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import * as React from "react";
import { OperationToolbar } from "./OperationToolbar";
import { CompressStateProvider, useCompressState, type CompressStateContextValue } from "@/hooks/useCompressState";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.operations.undo": "Undo",
        "compress.operations.redo": "Redo",
        "compress.operations.reset": "Reset All",
        "compress.operations.resetConfirm.title": "Reset All Changes?",
        "compress.operations.resetConfirm.description": "This will discard all your edits.",
        "compress.operations.resetConfirm.cancel": "Cancel",
        "compress.operations.resetConfirm.confirm": "Reset",
      };
      return translations[key] || key;
    },
  }),
}));

// 测试辅助组件 - 用于操作 CompressState
function TestHelper({
  onSetup,
}: {
  onSetup: (context: CompressStateContextValue) => void;
}) {
  const context = useCompressState();
  React.useEffect(() => {
    onSetup(context);
  }, [context, onSetup]);
  return null;
}

// Wrapper 组件
const TestWrapper = ({ children }: { children: React.ReactNode }) => (
  <CompressStateProvider>{children}</CompressStateProvider>
);

describe("OperationToolbar", () => {
  describe("渲染", () => {
    it("应渲染三个按钮", () => {
      render(
        <TestWrapper>
          <OperationToolbar />
        </TestWrapper>
      );

      expect(screen.getByTestId("undo-button")).toBeInTheDocument();
      expect(screen.getByTestId("redo-button")).toBeInTheDocument();
      expect(screen.getByTestId("reset-button")).toBeInTheDocument();
    });

    it("应接受自定义 className", () => {
      const { container } = render(
        <TestWrapper>
          <OperationToolbar className="custom-class" />
        </TestWrapper>
      );

      expect(container.querySelector(".custom-class")).toBeInTheDocument();
    });
  });

  describe("禁用状态 (AC5)", () => {
    it("初始状态下所有按钮应禁用", () => {
      render(
        <TestWrapper>
          <OperationToolbar />
        </TestWrapper>
      );

      expect(screen.getByTestId("undo-button")).toBeDisabled();
      expect(screen.getByTestId("redo-button")).toBeDisabled();
      expect(screen.getByTestId("reset-button")).toBeDisabled();
    });

    it("有操作后撤销按钮应启用", () => {
      const setupContext = { current: null as CompressStateContextValue | null };

      render(
        <TestWrapper>
          <TestHelper onSetup={(ctx) => { setupContext.current = ctx; }} />
          <OperationToolbar />
        </TestWrapper>
      );

      // 执行一个操作
      act(() => {
        setupContext.current?.setOperation("msg-1", { type: "delete" });
      });

      expect(screen.getByTestId("undo-button")).not.toBeDisabled();
      expect(screen.getByTestId("reset-button")).not.toBeDisabled();
    });
  });

  describe("点击事件", () => {
    it("点击重置按钮应打开确认对话框", () => {
      const setupContext = { current: null as CompressStateContextValue | null };

      render(
        <TestWrapper>
          <TestHelper onSetup={(ctx) => { setupContext.current = ctx; }} />
          <OperationToolbar />
        </TestWrapper>
      );

      // 先执行操作使重置按钮启用
      act(() => {
        setupContext.current?.setOperation("msg-1", { type: "delete" });
      });

      // 点击重置按钮
      fireEvent.click(screen.getByTestId("reset-button"));

      // 确认对话框应该打开
      expect(screen.getByTestId("reset-confirm-dialog")).toBeInTheDocument();
      expect(screen.getByText("Reset All Changes?")).toBeInTheDocument();
    });

    it("确认重置后对话框应关闭", () => {
      const setupContext = { current: null as CompressStateContextValue | null };

      render(
        <TestWrapper>
          <TestHelper onSetup={(ctx) => { setupContext.current = ctx; }} />
          <OperationToolbar />
        </TestWrapper>
      );

      // 先执行操作
      act(() => {
        setupContext.current?.setOperation("msg-1", { type: "delete" });
      });

      // 打开对话框
      fireEvent.click(screen.getByTestId("reset-button"));

      // 点击确认
      fireEvent.click(screen.getByTestId("reset-confirm-button"));

      // 对话框应关闭
      expect(screen.queryByTestId("reset-confirm-dialog")).not.toBeInTheDocument();
    });

    it("点击撤销按钮应调用 undo", () => {
      const setupContext = { current: null as CompressStateContextValue | null };

      render(
        <TestWrapper>
          <TestHelper onSetup={(ctx) => { setupContext.current = ctx; }} />
          <OperationToolbar />
        </TestWrapper>
      );

      // 先执行操作
      act(() => {
        setupContext.current?.setOperation("msg-1", { type: "delete" });
      });

      expect(setupContext.current?.operations.has("msg-1")).toBe(true);

      // 点击撤销
      fireEvent.click(screen.getByTestId("undo-button"));

      // 操作应被撤销
      expect(setupContext.current?.operations.has("msg-1")).toBe(false);
    });

    it("点击重做按钮应调用 redo", () => {
      const setupContext = { current: null as CompressStateContextValue | null };

      render(
        <TestWrapper>
          <TestHelper onSetup={(ctx) => { setupContext.current = ctx; }} />
          <OperationToolbar />
        </TestWrapper>
      );

      // 先执行操作然后撤销
      act(() => {
        setupContext.current?.setOperation("msg-1", { type: "delete" });
      });

      act(() => {
        setupContext.current?.undo();
      });

      expect(setupContext.current?.operations.has("msg-1")).toBe(false);

      // 点击重做
      fireEvent.click(screen.getByTestId("redo-button"));

      // 操作应被恢复
      expect(setupContext.current?.operations.has("msg-1")).toBe(true);
    });
  });

  describe("键盘快捷键 (AC2, AC3)", () => {
    beforeEach(() => {
      // Mock navigator.platform for consistent testing
      vi.stubGlobal("navigator", { platform: "Win32" });
    });

    afterEach(() => {
      vi.unstubAllGlobals();
    });

    it("Ctrl+Z 应触发撤销", () => {
      const setupContext = { current: null as CompressStateContextValue | null };

      render(
        <TestWrapper>
          <TestHelper onSetup={(ctx) => { setupContext.current = ctx; }} />
          <OperationToolbar />
        </TestWrapper>
      );

      // 先执行操作
      act(() => {
        setupContext.current?.setOperation("msg-1", { type: "delete" });
      });

      expect(setupContext.current?.operations.has("msg-1")).toBe(true);

      // 触发 Ctrl+Z
      fireEvent.keyDown(window, { key: "z", ctrlKey: true });

      // 操作应被撤销
      expect(setupContext.current?.operations.has("msg-1")).toBe(false);
    });

    it("Ctrl+Shift+Z 应触发重做", () => {
      const setupContext = { current: null as CompressStateContextValue | null };

      render(
        <TestWrapper>
          <TestHelper onSetup={(ctx) => { setupContext.current = ctx; }} />
          <OperationToolbar />
        </TestWrapper>
      );

      // 先执行操作然后撤销
      act(() => {
        setupContext.current?.setOperation("msg-1", { type: "delete" });
      });

      act(() => {
        setupContext.current?.undo();
      });

      expect(setupContext.current?.operations.has("msg-1")).toBe(false);

      // 触发 Ctrl+Shift+Z
      fireEvent.keyDown(window, { key: "z", ctrlKey: true, shiftKey: true });

      // 操作应被恢复
      expect(setupContext.current?.operations.has("msg-1")).toBe(true);
    });

    it("在输入框中按 Ctrl+Z 不应触发撤销", () => {
      const setupContext = { current: null as CompressStateContextValue | null };

      render(
        <TestWrapper>
          <TestHelper onSetup={(ctx) => { setupContext.current = ctx; }} />
          <OperationToolbar />
          <input data-testid="test-input" />
        </TestWrapper>
      );

      // 先执行操作
      act(() => {
        setupContext.current?.setOperation("msg-1", { type: "delete" });
      });

      const input = screen.getByTestId("test-input");

      // 在输入框中触发 Ctrl+Z
      fireEvent.keyDown(input, { key: "z", ctrlKey: true });

      // 操作不应被撤销
      expect(setupContext.current?.operations.has("msg-1")).toBe(true);
    });
  });
});
