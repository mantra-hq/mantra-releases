/**
 * FileEditDiff - 文件编辑差异视图组件
 * Story 8.11: Task 9 (AC: #9)
 *
 * 为 file_edit 类型的工具调用显示 inline diff 视图
 * 复用 sanitizer/diff-utils.ts 的 computeDiff() 函数
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { FileCode2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { computeDiff } from "@/components/sanitizer/diff-utils";
import type { DiffLine } from "@/components/sanitizer/types";

export interface FileEditDiffProps {
  /** 文件路径 */
  filePath: string;
  /** 原始内容 (被替换的文本) */
  oldString?: string;
  /** 新内容 (替换后的文本) */
  newString?: string;
  /** 自定义 className */
  className?: string;
}

/**
 * 渲染单行差异
 */
function DiffLineRow({ line }: { line: DiffLine }) {
  return (
    <div
      className={cn(
        "flex font-mono text-xs leading-5",
        line.type === "added" && "bg-green-500/10",
        line.type === "removed" && "bg-red-500/10"
      )}
    >
      {/* 行号 - 原始 */}
      <span className="w-8 text-muted-foreground text-right pr-2 select-none shrink-0 border-r border-border/50">
        {line.lineNumber.original ?? ""}
      </span>
      {/* 行号 - 新 */}
      <span className="w-8 text-muted-foreground text-right pr-2 select-none shrink-0 border-r border-border/50">
        {line.lineNumber.sanitized ?? ""}
      </span>
      {/* 变更标记 */}
      <span className="w-5 text-center select-none shrink-0">
        {line.type === "added" && (
          <span className="text-green-600 dark:text-green-400">+</span>
        )}
        {line.type === "removed" && (
          <span className="text-red-600 dark:text-red-400">-</span>
        )}
      </span>
      {/* 内容 */}
      <span className="flex-1 whitespace-pre-wrap break-all pl-1">
        {line.content || "\u00A0"}
      </span>
    </div>
  );
}

/**
 * FileEditDiff 组件
 *
 * 显示文件编辑的差异视图：
 * - 如果有 oldString 和 newString，显示 inline diff
 * - 如果只有 newString，显示完整新内容
 */
export function FileEditDiff({
  filePath,
  oldString,
  newString,
  className,
}: FileEditDiffProps) {
  const { t } = useTranslation();

  // 计算差异
  const diffLines = React.useMemo(() => {
    if (!newString) return [];

    // 如果没有 oldString，将所有行标记为新增
    if (!oldString) {
      const lines = newString.split("\n");
      return lines.map((content, idx): DiffLine => ({
        type: "added",
        content,
        lineNumber: { sanitized: idx + 1 },
      }));
    }

    // 有 oldString 和 newString，计算真正的 diff
    return computeDiff(oldString, newString);
  }, [oldString, newString]);

  // 统计
  const stats = React.useMemo(() => {
    const added = diffLines.filter((l) => l.type === "added").length;
    const removed = diffLines.filter((l) => l.type === "removed").length;
    return { added, removed };
  }, [diffLines]);

  // 没有内容
  if (!newString) {
    return (
      <div className={cn("text-muted-foreground text-xs italic", className)}>
        {t("message.noContentToShow", "无内容可显示")}
      </div>
    );
  }

  return (
    <div className={cn("rounded-md border border-border overflow-hidden", className)}>
      {/* 文件路径头部 */}
      <div className="flex items-center gap-2 px-3 py-1.5 bg-muted/50 border-b border-border text-xs">
        <FileCode2 className="h-3.5 w-3.5 text-muted-foreground" />
        <span className="font-mono text-muted-foreground truncate flex-1">
          {filePath}
        </span>
        {/* 统计徽章 */}
        {(stats.added > 0 || stats.removed > 0) && (
          <div className="flex items-center gap-1.5 shrink-0">
            {stats.added > 0 && (
              <span className="text-green-600 dark:text-green-400">
                +{stats.added}
              </span>
            )}
            {stats.removed > 0 && (
              <span className="text-red-600 dark:text-red-400">
                -{stats.removed}
              </span>
            )}
          </div>
        )}
      </div>

      {/* Diff 内容 */}
      <div className="max-h-[300px] overflow-auto bg-background">
        {diffLines.map((line, idx) => (
          <DiffLineRow key={idx} line={line} />
        ))}
      </div>
    </div>
  );
}

export default FileEditDiff;
