/**
 * CopyButton - 通用复制按钮组件
 * Story 2.22: Task 1
 *
 * 提供一键复制功能，支持成功反馈动画和无障碍支持
 */

import { useState, useCallback, useEffect, useRef } from "react";
import { Copy, Check } from "lucide-react";
import { cn } from "@/lib/utils";

export interface CopyButtonProps {
  /** 要复制的内容 */
  content: string;
  /** 按钮尺寸变体 */
  size?: "sm" | "md";
  /** 复制成功回调 */
  onSuccess?: () => void;
  /** 复制失败回调 */
  onError?: (error: Error) => void;
  /** 自定义 className */
  className?: string;
  /** 自定义 aria-label */
  ariaLabel?: string;
  /** 自定义 tooltip 文本 */
  tooltip?: string;
}

/**
 * 复制内容到剪贴板（带降级支持）
 */
async function copyToClipboard(text: string): Promise<void> {
  // 优先使用现代 Clipboard API
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  // 降级方案：使用 execCommand
  const textArea = document.createElement("textarea");
  textArea.value = text;
  textArea.style.position = "fixed";
  textArea.style.left = "-9999px";
  textArea.style.top = "-9999px";
  document.body.appendChild(textArea);
  textArea.select();

  try {
    const success = document.execCommand("copy");
    if (!success) {
      throw new Error("execCommand copy failed");
    }
  } finally {
    document.body.removeChild(textArea);
  }
}

/**
 * CopyButton 组件
 *
 * 功能:
 * - 点击复制内容到剪贴板 (AC1, AC2)
 * - 复制成功显示 ✓ 图标 2 秒后恢复 (AC4)
 * - 支持 sm/md 两种尺寸 (Task 1.6)
 * - 完整的无障碍支持 (AC5)
 * - 支持剪贴板 API 降级 (Task 6)
 */
export function CopyButton({
  content,
  size = "sm",
  onSuccess,
  onError,
  className,
  ariaLabel = "复制",
  tooltip = "复制",
}: CopyButtonProps) {
  const [copied, setCopied] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // 清理 timeout 防止内存泄漏 (H1 fix)
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const handleCopy = useCallback(async () => {
    if (!content) return;

    try {
      await copyToClipboard(content);
      setCopied(true);
      onSuccess?.();

      // 清除之前的 timeout（如果用户快速连续点击）
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      // 2 秒后恢复原状 (AC4)
      timeoutRef.current = setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      onError?.(error instanceof Error ? error : new Error("复制失败"));
    }
  }, [content, onSuccess, onError]);

  // 键盘事件处理 - 确保 Enter/Space 在所有环境下工作 (AC5)
  // 注: 原生 button 已支持，此处为兼容性/测试环境考虑
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLButtonElement>) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        handleCopy();
      }
    },
    [handleCopy]
  );

  // 尺寸配置
  const sizeClasses = {
    sm: "size-3",
    md: "size-4",
  };

  const iconSize = sizeClasses[size];
  const isDisabled = !content;

  return (
    <button
      type="button"
      onClick={handleCopy}
      onKeyDown={handleKeyDown}
      disabled={isDisabled}
      aria-label={copied ? "已复制" : ariaLabel}
      aria-pressed={copied}
      title={copied ? "已复制" : tooltip}
      className={cn(
        // Base styles
        "shrink-0 rounded p-1",
        "transition-colors duration-150",
        // Normal state
        "text-muted-foreground",
        // Hover state (非禁用)
        !isDisabled && "hover:bg-muted hover:text-foreground",
        // Disabled state
        isDisabled && "cursor-not-allowed opacity-50",
        // Focus state (AC5)
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        className
      )}
    >
      {copied ? (
        <Check
          className={cn(iconSize, "text-emerald-500")}
          aria-hidden="true"
        />
      ) : (
        <Copy className={iconSize} aria-hidden="true" />
      )}
      {/* 屏幕阅读器通知 (AC5) */}
      {copied && (
        <span className="sr-only" role="status" aria-live="polite">
          已复制到剪贴板
        </span>
      )}
    </button>
  );
}

export default CopyButton;
