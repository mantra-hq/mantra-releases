/**
 * KeyboardShortcutsHelp - 快捷键帮助面板
 * Story 10.10: Task 3
 *
 * 显示压缩模式可用的快捷键列表
 * 按类别分组: 消息操作、导航、全局
 * 支持 Esc 或 ? 键关闭
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { usePlatform, getModifierKey, getShiftKey } from "@/hooks/usePlatform";

/**
 * KeyboardShortcutsHelp 组件 Props
 */
export interface KeyboardShortcutsHelpProps {
  /** 是否打开 */
  open: boolean;
  /** 打开状态变更回调 */
  onOpenChange: (open: boolean) => void;
}

/**
 * 快捷键项
 */
interface ShortcutItem {
  key: string;
  description: string;
}

/**
 * 快捷键分组
 */
interface ShortcutGroup {
  category: string;
  items: ShortcutItem[];
}

/**
 * KeyboardShortcutsHelp - 快捷键帮助面板组件
 *
 * AC4: 显示快捷键帮助
 * - 按 ? 键显示
 * - 按 Esc 或再次按 ? 关闭
 * - 分组展示快捷键
 * - 根据平台显示 ⌘ 或 Ctrl
 */
export function KeyboardShortcutsHelp({
  open,
  onOpenChange,
}: KeyboardShortcutsHelpProps) {
  const { t } = useTranslation();
  const platform = usePlatform();

  // AC4: Esc 或 ? 键关闭面板
  React.useEffect(() => {
    if (!open) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Esc 键关闭
      if (e.key === "Escape") {
        e.preventDefault();
        onOpenChange(false);
        return;
      }

      // ? 键关闭 (再次按下)
      if (e.key === "?" || (e.shiftKey && e.key === "/")) {
        e.preventDefault();
        onOpenChange(false);
        return;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onOpenChange]);

  // 获取平台相关的修饰键符号
  const modKey = getModifierKey(platform);
  const shiftKey = getShiftKey(platform);

  // 快捷键列表 (分组)
  const shortcuts: ShortcutGroup[] = React.useMemo(
    () => [
      {
        category: t("compress.shortcuts.categories.messageOps"),
        items: [
          { key: "K", description: t("compress.shortcuts.keep") },
          { key: "D", description: t("compress.shortcuts.delete") },
          { key: "E", description: t("compress.shortcuts.edit") },
          { key: "I", description: t("compress.shortcuts.insert") },
        ],
      },
      {
        category: t("compress.shortcuts.categories.navigation"),
        items: [
          { key: "↑", description: t("compress.shortcuts.prevMessage") },
          { key: "↓", description: t("compress.shortcuts.nextMessage") },
        ],
      },
      {
        category: t("compress.shortcuts.categories.global"),
        items: [
          {
            key: `${modKey}+Z`,
            description: t("compress.shortcuts.undo"),
          },
          {
            key: `${modKey}+${shiftKey}Z`,
            description: t("compress.shortcuts.redo"),
          },
          {
            key: `${modKey}+S`,
            description: t("compress.shortcuts.export"),
          },
          { key: "?", description: t("compress.shortcuts.help") },
        ],
      },
    ],
    [t, modKey, shiftKey]
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="sm:max-w-md"
        data-testid="keyboard-shortcuts-help"
      >
        <DialogHeader>
          <DialogTitle>{t("compress.shortcuts.title")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {shortcuts.map((group) => (
            <div key={group.category}>
              {/* 分组标题 */}
              <h4 className="text-sm font-medium text-muted-foreground mb-2">
                {group.category}
              </h4>

              {/* 快捷键列表 */}
              <div className="space-y-1">
                {group.items.map((item) => (
                  <div
                    key={item.key}
                    className="flex items-center justify-between py-1"
                  >
                    <span className="text-sm">{item.description}</span>
                    <kbd className="px-2 py-1 text-xs font-mono bg-muted rounded border border-border">
                      {item.key}
                    </kbd>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* 底部提示 */}
        <div className="text-xs text-muted-foreground text-center pt-2 border-t">
          {t("compress.shortcuts.closeHint")}
        </div>
      </DialogContent>
    </Dialog>
  );
}

export default KeyboardShortcutsHelp;
