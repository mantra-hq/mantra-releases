/**
 * CodeSnapshotView - 代码快照视图组件
 * Story 2.5: Task 2
 * Story 2.7: Task 3 集成 - AC #5 Diff 高亮
 *
 * 使用 Monaco Editor 以只读模式显示历史代码快照
 * 支持语法高亮、主题切换、代码变化动画、Diff 高亮
 */

import { useEffect, useState, useRef, useMemo, useCallback } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";
import { useTheme } from "@/lib/theme-provider";
import { cn } from "@/lib/utils";
import { CodeSnapshotHeader } from "./CodeSnapshotHeader";
import { EmptyCodeState } from "./EmptyCodeState";
import { HistoryBanner } from "./HistoryBanner";
import {
  computeDiffDecorations,
  toMonacoDecorations,
  useDiffFadeOut,
} from "./DiffHighlighter";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";

/**
 * 语言映射表 - 根据文件扩展名识别语言
 */
const LANGUAGE_MAP: Record<string, string> = {
  ".ts": "typescript",
  ".tsx": "typescript",
  ".js": "javascript",
  ".jsx": "javascript",
  ".json": "json",
  ".md": "markdown",
  ".css": "css",
  ".scss": "scss",
  ".html": "html",
  ".rs": "rust",
  ".go": "go",
  ".py": "python",
  ".yaml": "yaml",
  ".yml": "yaml",
  ".toml": "toml",
  ".sql": "sql",
  ".sh": "shell",
  ".bash": "shell",
  ".xml": "xml",
  ".vue": "vue",
  ".svelte": "svelte",
};

/**
 * 根据文件路径获取语言标识
 * @param filePath - 文件路径
 * @returns Monaco Editor 语言标识符
 */
export function getLanguageFromPath(filePath: string): string {
  if (!filePath) return "plaintext";
  const lastDotIndex = filePath.lastIndexOf(".");
  if (lastDotIndex === -1) return "plaintext";
  const ext = filePath.slice(lastDotIndex).toLowerCase();
  return LANGUAGE_MAP[ext] || "plaintext";
}

export interface CodeSnapshotViewProps {
  /** 代码内容 */
  code: string;
  /** 文件路径 (用于语言检测和显示) */
  filePath: string;
  /** 历史时间戳 (ISO 8601 或 Unix ms) */
  timestamp?: string | number;
  /** Commit Hash (短格式) */
  commitHash?: string;
  /** Commit 消息 */
  commitMessage?: string;
  /** 是否处于历史模式 (Story 2.7 AC #6) */
  isHistoricalMode?: boolean;
  /** 返回当前回调 (Story 2.7 AC #6) */
  onReturnToCurrent?: () => void;
  /** 前一个代码内容 (用于 Diff 计算, Story 2.7 AC #5) */
  previousCode?: string | null;
  /** 自定义 className */
  className?: string;
}

/**
 * Monaco Editor 只读配置
 */
const EDITOR_OPTIONS = {
  readOnly: true,
  minimap: { enabled: false },
  scrollBeyondLastLine: false,
  fontSize: 13,
  lineNumbers: "on" as const,
  renderLineHighlight: "all" as const,
  automaticLayout: true,
  scrollbar: {
    vertical: "auto" as const,
    horizontal: "auto" as const,
  },
  padding: {
    top: 8,
    bottom: 8,
  },
  // 禁用编辑相关功能
  domReadOnly: true,
  cursorStyle: "line" as const,
  // 优化只读性能
  renderValidationDecorations: "off" as const,
  folding: true,
  foldingStrategy: "auto" as const,
  wordWrap: "off" as const,
  // Diff 高亮需要 glyph margin
  glyphMargin: true,
};

/**
 * 代码快照视图组件
 *
 * 功能:
 * - Monaco Editor 只读模式显示代码
 * - 自动语法高亮 (基于文件扩展名)
 * - 深色/浅色主题自动切换
 * - 代码变化时的淡入动画 (150ms)
 * - Diff 高亮 (新增绿色，删除红色，3秒淡出)
 * - 空代码状态处理
 * - 历史快照信息显示
 * - 历史模式 Banner
 */
export function CodeSnapshotView({
  code,
  filePath,
  timestamp,
  commitHash,
  commitMessage,
  isHistoricalMode = false,
  onReturnToCurrent,
  previousCode,
  className,
}: CodeSnapshotViewProps) {
  const { resolvedTheme } = useTheme();
  const [isTransitioning, setIsTransitioning] = useState(false);
  const previousCodeRef = useRef(code);
  const editorRef = useRef<any>(null);
  const decorationsRef = useRef<string[]>([]);

  // Diff 淡出控制 (Story 2.7 AC #5)
  const { shouldShow: shouldShowDiff, triggerFadeOut, cancelFadeOut } = useDiffFadeOut(3000);

  // 根据应用主题映射 Monaco 主题 (AC5)
  const monacoTheme = resolvedTheme === "dark" ? "vs-dark" : "light";

  // 检测语言 (AC2)
  const language = useMemo(() => getLanguageFromPath(filePath), [filePath]);

  // 计算时间戳
  const timestampMs = useMemo(() => {
    if (typeof timestamp === "number") return timestamp;
    if (typeof timestamp === "string") {
      const parsed = Date.parse(timestamp);
      return isNaN(parsed) ? null : parsed;
    }
    return null;
  }, [timestamp]);

  // 是否为历史快照
  const isHistorical = isHistoricalMode || !!(timestamp || commitHash);

  // 格式化时间戳显示
  const formattedTimestamp = useMemo(() => {
    if (!timestamp) return undefined;
    if (typeof timestamp === "string") return timestamp;
    try {
      return new Date(timestamp).toISOString();
    } catch {
      return undefined;
    }
  }, [timestamp]);

  // 编辑器挂载回调
  const handleEditorMount: OnMount = useCallback((editor, _monaco) => {
    editorRef.current = editor;
  }, []);

  // 代码变化处理 (动画 + Diff 高亮)
  useEffect(() => {
    const prevCode = previousCodeRef.current;

    if (prevCode !== code) {
      // 触发过渡动画
      setIsTransitioning(true);
      const timer = setTimeout(() => setIsTransitioning(false), 150);
      previousCodeRef.current = code;

      // 计算 Diff 装饰器 (Story 2.7 AC #5)
      if (editorRef.current && previousCode) {
        const diffDecorations = computeDiffDecorations(previousCode, code);

        if (diffDecorations.length > 0) {
          const monacoDecorations = toMonacoDecorations(diffDecorations);

          // 应用装饰器
          decorationsRef.current = editorRef.current.deltaDecorations(
            decorationsRef.current,
            monacoDecorations.map((d) => ({
              range: new (window as any).monaco.Range(
                d.range.startLineNumber,
                d.range.startColumn,
                d.range.endLineNumber,
                d.range.endColumn
              ),
              options: d.options,
            }))
          );

          // 触发淡出
          triggerFadeOut();
        }
      }

      return () => clearTimeout(timer);
    }
  }, [code, previousCode, triggerFadeOut]);

  // 清除 Diff 装饰器 (淡出后)
  useEffect(() => {
    if (!shouldShowDiff && editorRef.current && decorationsRef.current.length > 0) {
      decorationsRef.current = editorRef.current.deltaDecorations(
        decorationsRef.current,
        []
      );
    }
  }, [shouldShowDiff]);

  // 空状态处理 (AC6)
  if (!code) {
    return (
      <div className={cn("flex h-full flex-col", className)}>
        {/* 历史模式 Banner */}
        {isHistoricalMode && timestampMs && onReturnToCurrent && (
          <HistoryBanner
            timestamp={timestampMs}
            commitHash={commitHash}
            commitMessage={commitMessage}
            onReturnToCurrent={onReturnToCurrent}
          />
        )}
        <CodeSnapshotHeader
          filePath={filePath || ""}
          timestamp={formattedTimestamp}
          commitHash={commitHash}
          isHistorical={isHistorical}
        />
        <EmptyCodeState />
      </div>
    );
  }

  return (
    <div
      className={cn(
        "flex h-full flex-col bg-background",
        shouldShowDiff && "diff-fade-out",
        className
      )}
    >
      {/* 历史模式 Banner (Story 2.7 AC #6) */}
      {isHistoricalMode && timestampMs && onReturnToCurrent && (
        <HistoryBanner
          timestamp={timestampMs}
          commitHash={commitHash}
          commitMessage={commitMessage}
          onReturnToCurrent={onReturnToCurrent}
        />
      )}

      {/* 头部: 文件路径 + 历史状态指示器 (AC3, AC7) */}
      <CodeSnapshotHeader
        filePath={filePath}
        timestamp={formattedTimestamp}
        commitHash={commitHash}
        isHistorical={isHistorical}
      />

      {/* 编辑器容器 (AC1) */}
      <div
        className={cn(
          "flex-1 overflow-hidden relative",
          isTransitioning && "animate-fade-in"
        )}
      >
        <Editor
          height="100%"
          language={language}
          value={code}
          theme={monacoTheme}
          options={EDITOR_OPTIONS}
          onMount={handleEditorMount}
          loading={
            <div className="flex h-full items-center justify-center text-muted-foreground">
              加载编辑器中...
            </div>
          }
        />

        {/* Diff 关闭按钮 (Story 2.7 AC #5) */}
        {shouldShowDiff && (
          <Button
            variant="secondary"
            size="sm"
            className="absolute top-2 right-2 z-10 h-7 gap-1 text-xs opacity-80 hover:opacity-100"
            onClick={cancelFadeOut}
          >
            <X className="h-3 w-3" />
            隐藏 Diff
          </Button>
        )}
      </div>
    </div>
  );
}

export default CodeSnapshotView;
