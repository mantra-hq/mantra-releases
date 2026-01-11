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
// UX 优化: DiffModeToggle 也已移至 EditorTabs
import { EmptyCodeState } from "./EmptyCodeState";
import { FileNotFoundBanner } from "./FileNotFoundBanner";
import {
  computeDiffDecorations,
  toMonacoDecorations,
  useDiffFadeOut,
} from "./DiffHighlighter";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useEditorStore } from "@/stores/useEditorStore";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Components } from "react-markdown";
import { CodeBlockWithCopy } from "@/components/common/CodeBlockWithCopy";

// 后备语言映射表 (Monaco 未加载时使用)
const FALLBACK_LANGUAGE_MAP: Record<string, string> = {
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
  ".dart": "dart",
  ".kt": "kotlin",
  ".swift": "swift",
  ".rb": "ruby",
  ".php": "php",
  ".java": "java",
  ".c": "c",
  ".cpp": "cpp",
  ".h": "c",
  ".hpp": "cpp",
  ".cs": "csharp",
};

// 缓存 Monaco 语言扩展名映射 (运行时从 Monaco 获取)
let languageExtensionCache: Map<string, string> | null = null;

/**
 * 初始化语言扩展名缓存 (从 Monaco 运行时获取)
 */
function initLanguageCache(): Map<string, string> | null {
  if (languageExtensionCache) return languageExtensionCache;

  const monaco = (window as any).monaco;
  if (!monaco?.languages?.getLanguages) return null;

  languageExtensionCache = new Map();
  for (const lang of monaco.languages.getLanguages()) {
    if (lang.extensions) {
      for (const ext of lang.extensions) {
        // 扩展名可能带或不带点，统一为带点格式
        const normalizedExt = ext.startsWith('.') ? ext.toLowerCase() : `.${ext.toLowerCase()}`;
        languageExtensionCache.set(normalizedExt, lang.id);
      }
    }
  }
  return languageExtensionCache;
}

/**
 * 根据文件路径获取语言标识 (优先使用 Monaco 内置语言注册表，后备使用静态映射)
 * @param filePath - 文件路径
 * @returns Monaco Editor 语言标识符
 */
export function getLanguageFromPath(filePath: string): string {
  if (!filePath) return "plaintext";
  const lastDotIndex = filePath.lastIndexOf(".");
  if (lastDotIndex === -1) return "plaintext";
  const ext = filePath.slice(lastDotIndex).toLowerCase();

  // 优先使用 Monaco 运行时缓存
  const cache = initLanguageCache();
  if (cache) {
    const lang = cache.get(ext);
    if (lang) return lang;
  }

  // 后备使用静态映射
  return FALLBACK_LANGUAGE_MAP[ext] || "plaintext";
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
  /** Story 3.4: 编辑器 ref 回调 (用于外部行跳转) */
  onEditorRef?: (editor: editor.IStandaloneCodeEditor | null) => void;
  /** Story 3.4: 强制使用并排 Diff 模式 (用于脱敏预览) */
  forceSideBySide?: boolean;
  /** Markdown 预览模式 (由外部控制) */
  markdownMode?: 'source' | 'preview';
  // Story 2.30: snapshotSource 已移至 Breadcrumbs 组件
}

/**
 * Monaco Editor 只读配置
 */
const EDITOR_OPTIONS: editor.IStandaloneEditorConstructionOptions = {
  readOnly: true,
  minimap: { enabled: false },
  scrollBeyondLastLine: false,
  fontFamily: "JetBrains Mono, Consolas, monospace",
  fontSize: 13,
  fontLigatures: false,
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
  fontFamily: "JetBrains Mono, Consolas, monospace",
  fontSize: 13,
  fontLigatures: false,
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
  onEditorRef,
  forceSideBySide = false,
  markdownMode = 'source',
}: CodeSnapshotViewProps) {
  const { resolvedTheme } = useTheme();
  const diffMode = useEditorStore((state) => state.diffMode);
  const [isTransitioning, setIsTransitioning] = useState(false);
  const previousCodeRef = useRef(code);
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const diffEditorRef = useRef<editor.IStandaloneDiffEditor | null>(null);
  const decorationsRef = useRef<string[]>([]);

  // 检测是否为 Markdown 文件
  const isMarkdown = useMemo(() => {
    if (!filePath) return false;
    const ext = filePath.slice(filePath.lastIndexOf('.')).toLowerCase();
    return ext === '.md' || ext === '.markdown' || ext === '.mdx';
  }, [filePath]);

  // Markdown 渲染组件配置
  const markdownComponents: Components = useMemo(() => ({
    code({ className, children, ...props }) {
      const match = /language-(\w+)/.exec(className || "");
      const language = match ? match[1] : undefined;
      const codeString = String(children).replace(/\n$/, "");
      const isCodeBlock = className?.includes("language-") || codeString.includes("\n");

      if (isCodeBlock) {
        return <CodeBlockWithCopy code={codeString} language={language} />;
      }
      return <code className={className} {...props}>{children}</code>;
    },
    pre({ children }) {
      return <>{children}</>;
    },
  }), []);

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
  // Story 3.4: forceSideBySide 用于脱敏预览，即使内容相同也显示并排视图
  const useSideBySideDiff = (hasDiffData && diffMode === "side-by-side") ||
    (forceSideBySide && previousCode !== undefined);

  // 事件监听器清理引用
  const disposablesRef = useRef<Array<{ dispose: () => void }>>([]);
  // ViewState 回调 ref (避免依赖变化导致重新绑定)
  const onViewStateChangeRef = useRef(onViewStateChange);
  onViewStateChangeRef.current = onViewStateChange;
  // Story 3.4: 编辑器 ref 回调引用 (避免依赖变化导致重新绑定)
  const onEditorRefRef = useRef(onEditorRef);
  onEditorRefRef.current = onEditorRef;
  // 跟踪是否需要恢复 ViewState
  const pendingViewStateRef = useRef<editor.ICodeEditorViewState | null>(null);
  // 跟踪是否正在恢复 ViewState（避免循环）
  const isRestoringViewStateRef = useRef(false);

  // 编辑器挂载回调 (不依赖 viewState，避免重新挂载)
  const handleEditorMount: OnMount = useCallback((editor, _monaco) => {
    editorRef.current = editor;
    // Story 3.4: 通知外部编辑器 ref
    onEditorRefRef.current?.(editor);

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
      // 正在恢复 ViewState 时跳过保存，避免循环
      if (isRestoringViewStateRef.current) return;
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
    // Story 3.4: 通知外部编辑器 ref
    onEditorRefRef.current?.(modifiedEditor);

    // 清理旧的监听器
    disposablesRef.current.forEach(d => d.dispose());
    disposablesRef.current = [];

    // 恢复 ViewState (如果有)
    if (pendingViewStateRef.current) {
      isRestoringViewStateRef.current = true;
      modifiedEditor.restoreViewState(pendingViewStateRef.current);
      pendingViewStateRef.current = null;
      requestAnimationFrame(() => {
        isRestoringViewStateRef.current = false;
      });
    }

    // 监听滚动/光标变化
    const saveViewState = () => {
      // 正在恢复 ViewState 时跳过保存，避免循环
      if (isRestoringViewStateRef.current) return;
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
      // 标记正在恢复，避免触发 saveViewState 循环
      isRestoringViewStateRef.current = true;
      editorRef.current.restoreViewState(viewState);
      // 延迟重置标记，确保事件处理完成
      requestAnimationFrame(() => {
        isRestoringViewStateRef.current = false;
      });
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
      // 清理 editor ref，通知外部
      if (onEditorRefRef.current) {
        onEditorRefRef.current(null);
      }
    };
  }, []);

  // Story 3.4: 编辑器类型切换时清理 ref，避免 TextModel disposed 错误
  useEffect(() => {
    // 当切换编辑器类型时，清理旧的 editor ref
    return () => {
      editorRef.current = null;
      diffEditorRef.current = null;
    };
  }, [useSideBySideDiff]);

  // 是否已初始化（避免首次挂载时触发动画）
  const isInitializedRef = useRef(false);

  // 代码变化处理 (动画 + Diff 高亮)
  useEffect(() => {
    // 首次挂载时跳过动画，只初始化 ref
    if (!isInitializedRef.current) {
      isInitializedRef.current = true;
      previousCodeRef.current = code;
      return;
    }

    const prevCode = previousCodeRef.current;

    if (prevCode !== code) {
      // 触发过渡动画
      setIsTransitioning(true);
      const timer = setTimeout(() => setIsTransitioning(false), 150);
      previousCodeRef.current = code;

      // 计算 Diff 装饰器 (Story 2.7 AC #5)
      // 检查 monaco 是否已加载，避免 "Cannot read properties of undefined (reading 'Range')" 错误
      if (editorRef.current && previousCode && (window as any).monaco) {
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
      data-testid="code-panel"
      className={cn(
        "flex h-full flex-col bg-background",
        shouldShowDiff && "diff-fade-out",
        className
      )}
    >
      {/* UX 优化: 移除 HistoryBanner/CodeSnapshotHeader/DiffModeToggle，功能已上移至 EditorTabs */}
      {/* Story 2.30: 来源 badge 已移至 Breadcrumbs 组件 */}

      {/* 编辑器容器 (AC1) */}
      <div
        data-testid="code-content"
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

        {/* Markdown 预览模式 */}
        {isMarkdown && markdownMode === 'preview' ? (
          <div className="h-full overflow-auto p-6 bg-background">
            <div
              className={cn(
                "prose prose-sm dark:prose-invert max-w-none",
                "prose-p:my-2 prose-p:leading-relaxed",
                "prose-pre:bg-transparent prose-pre:p-0",
                "prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-sm",
                "prose-code:text-foreground prose-code:font-normal",
                "prose-code:before:content-none prose-code:after:content-none",
                "prose-blockquote:border-l-primary prose-blockquote:text-foreground/80",
                "prose-ul:my-2 prose-ol:my-2",
                "prose-li:my-0.5",
                "prose-headings:mt-4 prose-headings:mb-2",
                "prose-th:text-foreground prose-td:text-foreground/90",
                "prose-strong:text-foreground prose-em:text-foreground/90",
                "prose-img:rounded-lg prose-img:shadow-md"
              )}
            >
              <ReactMarkdown remarkPlugins={[remarkGfm]} components={markdownComponents}>
                {code}
              </ReactMarkdown>
            </div>
          </div>
        ) : useSideBySideDiff ? (
          /* 并排 Diff 模式 */
          <DiffEditor
            key="diff-editor"
            height="100%"
            language={language}
            original={previousCode || ""}
            modified={code}
            theme={monacoTheme}
            options={DIFF_EDITOR_OPTIONS}
            onMount={handleDiffEditorMount}
            keepCurrentOriginalModel={true}
            keepCurrentModifiedModel={true}
            loading={
              <div className="flex h-full items-center justify-center text-muted-foreground">
                加载 Diff 编辑器中...
              </div>
            }
          />
        ) : (
          /* 普通编辑器 (inline diff 使用装饰器) */
          <Editor
            key="normal-editor"
            height="100%"
            language={language}
            value={code}
            theme={monacoTheme}
            options={EDITOR_OPTIONS}
            onMount={handleEditorMount}
            keepCurrentModel={true}
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
