/**
 * useCompressHotkeys - 压缩模式快捷键 Hook
 * Story 10.10: Task 2
 *
 * 处理压缩模式下的键盘快捷键：
 * - 消息操作: K (保留), D (删除), E (编辑), I (插入)
 * - 导航: ↑/↓ (上下移动焦点)
 * - 全局: Ctrl+S/Cmd+S (导出), ? (帮助)
 * - 输入框排除: 在 input/textarea/contenteditable 中不触发
 */

import * as React from "react";
import { usePlatform } from "./usePlatform";
import type { UseMessageFocusReturn } from "./useMessageFocus";
import type { NarrativeMessage } from "@/types/message";

/**
 * useCompressHotkeys 选项
 */
export interface UseCompressHotkeysOptions {
  /** 是否启用快捷键 (仅压缩模式激活时) */
  enabled: boolean;
  /** 焦点管理 */
  focus: UseMessageFocusReturn;
  /** 消息列表 (用于获取焦点消息) */
  messages: NarrativeMessage[];
  /** 标记为保留回调 */
  onKeep?: (messageId: string) => void;
  /** 标记为删除回调 */
  onDelete?: (messageId: string) => void;
  /** 打开编辑对话框回调 */
  onEdit?: (messageId: string) => void;
  /** 在指定位置后插入回调 */
  onInsert?: (afterIndex: number) => void;
  /** 打开导出菜单回调 */
  onOpenExport?: () => void;
  /** 打开帮助面板回调 */
  onToggleHelp?: () => void;
}

/**
 * 检测是否为输入元素
 * AC5: 输入框排除逻辑
 */
function isInputElement(target: EventTarget | null): boolean {
  if (!target) return false;
  const element = target as HTMLElement;

  // 检查是否为输入元素
  if (
    element instanceof HTMLInputElement ||
    element instanceof HTMLTextAreaElement
  ) {
    return true;
  }

  // 检查是否为可编辑元素 (contenteditable)
  if (element.isContentEditable) {
    return true;
  }

  // 检查是否在 Dialog 内 (避免在对话框打开时触发快捷键)
  if (element.closest('[role="dialog"]')) {
    return true;
  }

  return false;
}

/**
 * 压缩模式快捷键 Hook
 *
 * @param options 选项
 *
 * @example
 * ```tsx
 * useCompressHotkeys({
 *   enabled: isCompressMode,
 *   focus,
 *   messages,
 *   onKeep: (id) => setOperation(id, { type: 'keep' }),
 *   onDelete: (id) => setOperation(id, { type: 'delete' }),
 *   onEdit: (id) => openEditDialog(id),
 *   onInsert: (index) => openInsertDialog(index),
 *   onOpenExport: () => setExportMenuOpen(true),
 *   onToggleHelp: () => setHelpOpen((prev) => !prev),
 * });
 * ```
 */
export function useCompressHotkeys(options: UseCompressHotkeysOptions): void {
  const {
    enabled,
    focus,
    messages,
    onKeep,
    onDelete,
    onEdit,
    onInsert,
    onOpenExport,
    onToggleHelp,
  } = options;

  const platform = usePlatform();

  React.useEffect(() => {
    if (!enabled) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // AC5: 输入框排除
      if (isInputElement(e.target)) return;

      // 跨平台修饰键检测
      const modKey = platform === "mac" ? e.metaKey : e.ctrlKey;

      // 获取当前焦点消息
      const focusedMessage =
        focus.focusedIndex >= 0 && focus.focusedIndex < messages.length
          ? messages[focus.focusedIndex]
          : null;

      // AC3: 导航快捷键 (↑/↓)
      if (e.key === "ArrowUp") {
        e.preventDefault();
        focus.focusPrevious();
        return;
      }

      if (e.key === "ArrowDown") {
        e.preventDefault();
        focus.focusNext();
        return;
      }

      // AC1: 消息操作快捷键 (需要焦点)
      if (focusedMessage) {
        const key = e.key.toLowerCase();

        switch (key) {
          case "k":
            // K - 标记为保留 (Keep)
            e.preventDefault();
            onKeep?.(focusedMessage.id);
            return;

          case "d":
            // D - 标记为删除 (Delete)
            e.preventDefault();
            onDelete?.(focusedMessage.id);
            return;

          case "e":
            // E - 打开编辑对话框 (Edit)
            e.preventDefault();
            onEdit?.(focusedMessage.id);
            return;

          case "i":
            // I - 在当前位置后插入 (Insert)
            e.preventDefault();
            onInsert?.(focus.focusedIndex);
            return;
        }
      }

      // AC2: 全局快捷键 - Ctrl+S / Cmd+S 打开导出菜单
      if (modKey && e.key.toLowerCase() === "s") {
        e.preventDefault();
        onOpenExport?.();
        return;
      }

      // AC4: 帮助快捷键 - ? 切换帮助面板
      // 需要检查 Shift 键因为 ? 需要 Shift+/
      if (e.key === "?" || (e.shiftKey && e.key === "/")) {
        e.preventDefault();
        onToggleHelp?.();
        return;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    enabled,
    platform,
    focus,
    messages,
    onKeep,
    onDelete,
    onEdit,
    onInsert,
    onOpenExport,
    onToggleHelp,
  ]);
}

export default useCompressHotkeys;
