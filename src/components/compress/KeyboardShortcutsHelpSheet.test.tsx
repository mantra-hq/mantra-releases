/**
 * KeyboardShortcutsHelpSheet 组件单元测试
 * Story 10.10: Task 9.3
 * Story 12.3: Dialog → Sheet 改造
 */

import React from "react";
import { render, screen } from "@testing-library/react";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { KeyboardShortcutsHelpSheet } from "./KeyboardShortcutsHelpSheet";

// Mock dependencies
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.shortcuts.title": "Keyboard Shortcuts",
        "compress.shortcuts.categories.messageOps": "Message Operations",
        "compress.shortcuts.categories.navigation": "Navigation",
        "compress.shortcuts.categories.global": "Global",
        "compress.shortcuts.keep": "Keep message",
        "compress.shortcuts.delete": "Delete message",
        "compress.shortcuts.edit": "Edit message",
        "compress.shortcuts.insert": "Insert message after",
        "compress.shortcuts.prevMessage": "Previous message",
        "compress.shortcuts.nextMessage": "Next message",
        "compress.shortcuts.undo": "Undo",
        "compress.shortcuts.redo": "Redo",
        "compress.shortcuts.export": "Open export menu",
        "compress.shortcuts.help": "Show keyboard shortcuts",
        "compress.shortcuts.closeHint": "Press Esc or ? to close",
      };
      return translations[key] || key;
    },
  }),
}));

vi.mock("@/hooks/usePlatform", () => ({
  usePlatform: () => "other",
  getModifierKey: () => "Ctrl",
  getShiftKey: () => "Shift+",
}));

// Mock Sheet component
vi.mock("@/components/ui/sheet", () => ({
  Sheet: ({ children, open }: { children: React.ReactNode; open: boolean }) =>
    open ? <div data-testid="sheet">{children}</div> : null,
  SheetContent: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div data-testid="sheet-content" className={className}>
      {children}
    </div>
  ),
  SheetHeader: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sheet-header">{children}</div>
  ),
  SheetTitle: ({ children }: { children: React.ReactNode }) => (
    <h2 data-testid="sheet-title">{children}</h2>
  ),
  SheetDescription: ({ children }: { children: React.ReactNode }) => (
    <p data-testid="sheet-description">{children}</p>
  ),
  SheetFooter: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div data-testid="sheet-footer" className={className}>{children}</div>
  ),
  SheetClose: ({ children }: { children: React.ReactNode }) => (
    <button data-testid="sheet-close">{children}</button>
  ),
}));

describe("KeyboardShortcutsHelpSheet", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("渲染", () => {
    it("open=true 时应显示面板", () => {
      render(<KeyboardShortcutsHelpSheet {...defaultProps} />);

      expect(screen.getByTestId("sheet")).toBeInTheDocument();
      expect(screen.getByTestId("sheet-title")).toHaveTextContent(
        "Keyboard Shortcuts"
      );
    });

    it("open=false 时不应显示面板", () => {
      render(<KeyboardShortcutsHelpSheet {...defaultProps} open={false} />);

      expect(screen.queryByTestId("sheet")).not.toBeInTheDocument();
    });

    it("应显示所有分组", () => {
      render(<KeyboardShortcutsHelpSheet {...defaultProps} />);

      expect(screen.getByText("Message Operations")).toBeInTheDocument();
      expect(screen.getByText("Navigation")).toBeInTheDocument();
      expect(screen.getByText("Global")).toBeInTheDocument();
    });

    it("应显示消息操作快捷键", () => {
      render(<KeyboardShortcutsHelpSheet {...defaultProps} />);

      expect(screen.getByText("Keep message")).toBeInTheDocument();
      expect(screen.getByText("Delete message")).toBeInTheDocument();
      expect(screen.getByText("Edit message")).toBeInTheDocument();
      expect(screen.getByText("Insert message after")).toBeInTheDocument();
    });

    it("应显示导航快捷键", () => {
      render(<KeyboardShortcutsHelpSheet {...defaultProps} />);

      expect(screen.getByText("Previous message")).toBeInTheDocument();
      expect(screen.getByText("Next message")).toBeInTheDocument();
    });

    it("应显示全局快捷键", () => {
      render(<KeyboardShortcutsHelpSheet {...defaultProps} />);

      expect(screen.getByText("Undo")).toBeInTheDocument();
      expect(screen.getByText("Redo")).toBeInTheDocument();
      expect(screen.getByText("Open export menu")).toBeInTheDocument();
      expect(screen.getByText("Show keyboard shortcuts")).toBeInTheDocument();
    });

    it("应显示关闭提示", () => {
      render(<KeyboardShortcutsHelpSheet {...defaultProps} />);

      expect(screen.getByText("Press Esc or ? to close")).toBeInTheDocument();
    });
  });

  describe("键盘事件", () => {
    it("Esc 键关闭由 shadcn/ui Sheet 处理", () => {
      // 注意: Escape 键关闭由 shadcn/ui Sheet 组件内部处理
      // 此测试仅验证组件正确渲染，Escape 功能由 Sheet 组件保证
      const onOpenChange = vi.fn();
      render(<KeyboardShortcutsHelpSheet open={true} onOpenChange={onOpenChange} />);

      expect(screen.getByTestId("sheet")).toBeInTheDocument();
    });

    it("? 键应关闭面板", () => {
      const onOpenChange = vi.fn();
      render(<KeyboardShortcutsHelpSheet open={true} onOpenChange={onOpenChange} />);

      // 模拟 ? 键
      const event = new KeyboardEvent("keydown", { key: "?" });
      Object.defineProperty(event, "preventDefault", { value: vi.fn() });
      window.dispatchEvent(event);

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });
});
