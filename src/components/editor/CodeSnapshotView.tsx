/**
 * CodeSnapshotView - 代码快照视图组件
 * Story 2.5: Task 2
 * Story 2.7: Task 3 集成 - AC #5 Diff 高亮
 * Story 2.12: Task 5 - AC #5 文件不存在处理
 *
 * 使用 Monaco Editor 以只读模式显示历史代码快照
 * 支持语法高亮、主题切换、代码变化动画、Diff 高亮
 * 文件不存在时显示 FileNotFoundBanner
 */

import { useEffect, useState, useRef, useMemo, useCallback } from "react";
import Editor, { DiffEditor, type OnMount, type DiffOnMount } from "@monaco-editor/react";
import type { editor } from "monaco-editor";
import { useTheme } from "@/lib/theme-provider";
import { cn } from "@/lib/utils";
// UX 优化: 移除 CodeSnapshotHeader 和 HistoryBanner，功能已上移到 EditorTabs/Breadcrumbs
import { EmptyCodeState } from "./EmptyCodeState";
import { FileNotFoundBanner } from "./FileNotFoundBanner";
import { DiffModeToggle } from "./DiffModeToggle";
import {
  computeDiffDecorations,
  toMonacoDecorations,
  useDiffFadeOut,
} from "./DiffHighlighter";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useEditorStore } from "@/stores/useEditorStore";

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
  /** 文件未找到标志 (Story 2.12 AC #5) */
  fileNotFound?: boolean;
  /** 未找到的文件路径 (Story 2.12 AC #5) */
  notFoundPath?: string;
  /** 清除文件不存在状态回调 (Story 2.12 AC #5) */
  onDismissNotFound?: () => void;
  /** 自定义 className */
  className?: string;
  /** Monaco ViewState (Story 2.13 AC #5) */
  viewState?: editor.ICodeEditorViewState | null;
  /** ViewState 变更回调 (Story 2.13 AC #5) */
  onViewStateChange?: (viewState: editor.ICodeEditorViewState) => void;
}

/**
 * Monaco Editor 只读配置
 */
const EDITOR_OPTIONS: editor.IStandaloneEditorConstructionOptions = {
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
 * Monaco DiffEditor 配置
 */
const DIFF_EDITOR_OPTIONS: editor.IDiffEditorConstructionOptions = {
  readOnly: true,
  renderSideBySide: true,
  enableSplitViewResizing: true,
  minimap: { enabled: false },
  scrollBeyondLastLine: false,
  fontSize: 13,
  lineNumbers: "on" as const,
  automaticLayout: true,
  scrollbar: {
    vertical: "auto" as const,
    horizontal: "auto" as const,
  },
  // 禁用编辑
  domReadOnly: true,
  // Diff 特有选项
  renderIndicators: true,
  renderMarginRevertIcon: false,
  ignoreTrimWhitespace: false,
  renderOverviewRuler: true,
  diffWordWrap: "off" as const,
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
  // UX 优化: commitHash/commitMessage/onReturnToCurrent 已上移到 EditorTabs/Breadcrumbs
  // commitHash,
  // UX 优化: commitMessage 和 onReturnToCurrent 已上移到 EditorTabs/Breadcrumbs
  // commitMessage,
  // isHistoricalMode = false,
  // onReturnToCurrent,
  previousCode,
  fileNotFound = false,
  notFoundPath,
  onDismissNotFound,
  className,
  viewState,
  onViewStateChange,
}: CodeSnapshotViewProps) {
  const { resolvedTheme } = useTheme();
  const diffMode = useEditorStore((state) => state.diffMode);
  const [isTransitioning, setIsTransitioning] = useState(false);
  const previousCodeRef = useRef(code);
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const diffEditorRef = useRef<editor.IStandaloneDiffEditor | null>(null);
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

  // 是否有可用的 Diff 数据
  const hasDiffData = !!(previousCode && previousCode !== code);

  // 是否使用并排 Diff 模式
  const useSideBySideDiff = hasDiffData && diffMode === "side-by-side";

  // 事件监听器清理引用
  const disposablesRef = useRef<Array<{ dispose: () => void }>>([]);
  // ViewState 回调 ref (避免依赖变化导致重新绑定)
  const onViewStateChangeRef = useRef(onViewStateChange);
  onViewStateChangeRef.current = onViewStateChange;
  // 跟踪是否需要恢复 ViewState
  const pendingViewStateRef = useRef<editor.ICodeEditorViewState | null>(null);

  // 编辑器挂载回调 (不依赖 viewState，避免重新挂载)
  const handleEditorMount: OnMount = useCallback((editor, _monaco) => {
    editorRef.current = editor;

    // 清理旧的监听器
    disposablesRef.current.forEach(d => d.dispose());
    disposablesRef.current = [];

    // Story 2.13 AC #5: 如果有待恢复的 ViewState，立即恢复
    if (pendingViewStateRef.current) {
      editor.restoreViewState(pendingViewStateRef.current);
      pendingViewStateRef.current = null;
    }

    // Story 2.13 AC #5: 监听光标/滚动变化，保存 ViewState
    const saveViewState = () => {
      const state = editor.saveViewState();
      if (state && onViewStateChangeRef.current) {
        onViewStateChangeRef.current(state);
      }
    };

    // 监听光标位置变化 (保存 disposable 用于清理)
    disposablesRef.current.push(editor.onDidChangeCursorPosition(saveViewState));
    // 监听滚动变化
    disposablesRef.current.push(editor.onDidScrollChange(saveViewState));
  }, []);

  // DiffEditor 挂载回调
  const handleDiffEditorMount: DiffOnMount = useCallback((diffEditor, _monaco) => {
    diffEditorRef.current = diffEditor;
    // 获取修改后的编辑器 (右侧)
    const modifiedEditor = diffEditor.getModifiedEditor();
    editorRef.current = modifiedEditor;

    // 清理旧的监听器
    disposablesRef.current.forEach(d => d.dispose());
    disposablesRef.current = [];

    // 恢复 ViewState (如果有)
    if (pendingViewStateRef.current) {
      modifiedEditor.restoreViewState(pendingViewStateRef.current);
      pendingViewStateRef.current = null;
    }

    // 监听滚动/光标变化
    const saveViewState = () => {
      const state = modifiedEditor.saveViewState();
      if (state && onViewStateChangeRef.current) {
        onViewStateChangeRef.current(state);
      }
    };

    disposablesRef.current.push(modifiedEditor.onDidChangeCursorPosition(saveViewState));
    disposablesRef.current.push(modifiedEditor.onDidScrollChange(saveViewState));
  }, []);

  // Story 2.13 AC #5: ViewState 变化时恢复 (独立 effect，避免 onMount 重绑定)
  useEffect(() => {
    if (viewState && editorRef.current) {
      // 编辑器已挂载，直接恢复
      editorRef.current.restoreViewState(viewState);
    } else if (viewState) {
      // 编辑器未挂载，存储待恢复状态
      pendingViewStateRef.current = viewState;
    }
  }, [viewState]);

  // 组件卸载时清理事件监听器
  useEffect(() => {
    return () => {
      disposablesRef.current.forEach(d => d.dispose());
      disposablesRef.current = [];
    };
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

  // 空状态处理 (AC6) - UX 优化: 移除冗余的 Header 和 Banner
  if (!code) {
    return (
      <div className={cn("flex h-full flex-col", className)}>
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
      {/* UX 优化: 移除 HistoryBanner 和 CodeSnapshotHeader，功能已上移 */}
      {/* Diff 模式切换按钮 (仅在有 Diff 数据时显示) */}
      {hasDiffData && (
        <div className="flex items-center justify-end px-2 py-1 border-b border-border bg-muted/30">
          <DiffModeToggle visible={hasDiffData} />
        </div>
      )}

      {/* 编辑器容器 (AC1) */}
      <div
        className={cn(
          "flex-1 overflow-hidden relative",
          isTransitioning && "animate-fade-in"
        )}
      >
        {/* 文件未找到遮罩层 (Story 2.12 AC #5) */}
        {fileNotFound && (
          <div className="absolute inset-0 z-20 flex flex-col">
            {/* FileNotFoundBanner */}
            <FileNotFoundBanner
              filePath={notFoundPath || filePath}
              timestamp={timestampMs || undefined}
              onDismiss={onDismissNotFound}
              onKeepCurrent={onDismissNotFound}
            />
            {/* 半透明遮罩 - 保持上一个有效代码可见 */}
            <div className="flex-1 bg-background/60 backdrop-blur-[1px]" />
          </div>
        )}

        {/* 并排 Diff 模式 */}
        {useSideBySideDiff ? (
          <DiffEditor
            height="100%"
            language={language}
            original={previousCode || ""}
            modified={code}
            theme={monacoTheme}
            options={DIFF_EDITOR_OPTIONS}
            onMount={handleDiffEditorMount}
            loading={
              <div className="flex h-full items-center justify-center text-muted-foreground">
                加载 Diff 编辑器中...
              </div>
            }
          />
        ) : (
          /* 普通编辑器 (inline diff 使用装饰器) */
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
        )}

        {/* Diff 关闭按钮 - 仅在 inline 模式显示 (Story 2.7 AC #5) */}
        {shouldShowDiff && !useSideBySideDiff && (
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
