/**
 * CodeSnapshotHeader - 代码快照头部组件
 * Story 2.5: Task 3
 *
 * 显示文件路径和历史状态指示器
 */

import { FileCode, History, Copy, Check } from "lucide-react";
import { cn } from "@/lib/utils";
import { useState, useCallback } from "react";

export interface CodeSnapshotHeaderProps {
  /** 文件路径 */
  filePath: string;
  /** 历史时间戳 */
  timestamp?: string;
  /** Commit Hash */
  commitHash?: string;
  /** 是否为历史快照 */
  isHistorical?: boolean;
}

/**
 * 格式化时间戳为友好显示
 * @param isoTimestamp - ISO 8601 格式时间戳
 * @returns 格式化后的时间字符串
 */
function formatTimestamp(isoTimestamp: string): string {
  try {
    const date = new Date(isoTimestamp);
    // 使用简洁格式: "12/30 14:30"
    return date.toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
  } catch {
    return isoTimestamp;
  }
}

/**
 * 代码快照头部组件
 *
 * 功能:
 * - 显示文件路径 (带 FileCode 图标) (AC3)
 * - 显示历史状态指示器 (时间戳 + "历史快照" 标签) (AC7)
 * - 可选的复制路径按钮 (Task 3.4)
 */
export function CodeSnapshotHeader({
  filePath,
  timestamp,
  commitHash,
  isHistorical = false,
}: CodeSnapshotHeaderProps) {
  const [copied, setCopied] = useState(false);

  // 复制文件路径到剪贴板
  const handleCopyPath = useCallback(async () => {
    if (!filePath) return;
    try {
      await navigator.clipboard.writeText(filePath);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // 复制失败时静默处理
    }
  }, [filePath]);

  // 显示的路径文本
  const displayPath = filePath || "未选择文件";

  // 格式化时间戳
  const formattedTime = timestamp ? formatTimestamp(timestamp) : null;

  return (
    <div
      className={cn(
        "flex items-center justify-between",
        "border-b border-border",
        "bg-muted/30 px-3 py-2"
      )}
    >
      {/* 左侧: 文件路径 */}
      <div className="flex min-w-0 flex-1 items-center gap-2">
        <FileCode className="size-4 shrink-0 text-muted-foreground" />
        <span
          className={cn(
            "truncate font-mono text-xs",
            filePath ? "text-foreground" : "text-muted-foreground"
          )}
          title={filePath}
        >
          {displayPath}
        </span>
        {/* 复制按钮 */}
        {filePath && (
          <button
            onClick={handleCopyPath}
            className={cn(
              "shrink-0 rounded p-1",
              "text-muted-foreground hover:bg-muted hover:text-foreground",
              "transition-colors duration-150"
            )}
            title="复制路径"
          >
            {copied ? (
              <Check className="size-3 text-emerald-500" />
            ) : (
              <Copy className="size-3" />
            )}
          </button>
        )}
      </div>

      {/* 右侧: 历史状态指示器 (AC7) */}
      {isHistorical && (
        <div
          className={cn(
            "ml-2 flex shrink-0 items-center gap-1.5",
            "rounded-md bg-blue-500/15 px-2 py-1",
            "text-[11px] text-blue-500"
          )}
        >
          <History className="size-3" />
          <span className="font-medium">历史快照</span>
          {formattedTime && (
            <span className="text-blue-400/80">· {formattedTime}</span>
          )}
          {commitHash && (
            <span className="font-mono text-blue-400/80">
              · {commitHash.slice(0, 7)}
            </span>
          )}
        </div>
      )}
    </div>
  );
}

export default CodeSnapshotHeader;

