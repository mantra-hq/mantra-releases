/**
 * CodeSnapshotView - 代码快照视图组件
 * Story 2.5: Task 2
 *
 * 使用 Monaco Editor 以只读模式显示历史代码快照
 * 支持语法高亮、主题切换、代码变化动画
 */

import { useEffect, useState, useRef, useMemo } from "react";
import Editor from "@monaco-editor/react";
import { useTheme } from "@/lib/theme-provider";
import { cn } from "@/lib/utils";
import { CodeSnapshotHeader } from "./CodeSnapshotHeader";
import { EmptyCodeState } from "./EmptyCodeState";

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
  /** 历史时间戳 (ISO 8601) */
  timestamp?: string;
  /** Commit Hash (短格式) */
  commitHash?: string;
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
} as const;

/**
 * 代码快照视图组件
 *
 * 功能:
 * - Monaco Editor 只读模式显示代码
 * - 自动语法高亮 (基于文件扩展名)
 * - 深色/浅色主题自动切换
 * - 代码变化时的淡入动画 (150ms)
 * - 空代码状态处理
 * - 历史快照信息显示
 */
export function CodeSnapshotView({
  code,
  filePath,
  timestamp,
  commitHash,
  className,
}: CodeSnapshotViewProps) {
  const { resolvedTheme } = useTheme();
  const [isTransitioning, setIsTransitioning] = useState(false);
  const previousCodeRef = useRef(code);

  // 根据应用主题映射 Monaco 主题 (AC5)
  const monacoTheme = resolvedTheme === "dark" ? "vs-dark" : "light";

  // 检测语言 (AC2)
  const language = useMemo(() => getLanguageFromPath(filePath), [filePath]);

  // 是否为历史快照 (AC7)
  const isHistorical = !!(timestamp || commitHash);

  // 代码变化动画 (AC4)
  useEffect(() => {
    if (previousCodeRef.current !== code) {
      setIsTransitioning(true);
      const timer = setTimeout(() => setIsTransitioning(false), 150);
      previousCodeRef.current = code;
      return () => clearTimeout(timer);
    }
  }, [code]);

  // 空状态处理 (AC6)
  if (!code) {
    return (
      <div className={cn("flex h-full flex-col", className)}>
        <CodeSnapshotHeader
          filePath={filePath || ""}
          timestamp={timestamp}
          commitHash={commitHash}
          isHistorical={isHistorical}
        />
        <EmptyCodeState />
      </div>
    );
  }

  return (
    <div className={cn("flex h-full flex-col bg-background", className)}>
      {/* 头部: 文件路径 + 历史状态指示器 (AC3, AC7) */}
      <CodeSnapshotHeader
        filePath={filePath}
        timestamp={timestamp}
        commitHash={commitHash}
        isHistorical={isHistorical}
      />

      {/* 编辑器容器 (AC1) */}
      <div
        className={cn(
          "flex-1 overflow-hidden",
          isTransitioning && "animate-fade-in"
        )}
      >
        <Editor
          height="100%"
          language={language}
          value={code}
          theme={monacoTheme}
          options={EDITOR_OPTIONS}
          loading={
            <div className="flex h-full items-center justify-center text-muted-foreground">
              加载编辑器中...
            </div>
          }
        />
      </div>
    </div>
  );
}

export default CodeSnapshotView;

