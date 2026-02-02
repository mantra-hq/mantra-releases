/**
 * CompressModeContent - 压缩模式内容包装组件
 * Story 10.9: Task 3
 * Story 10.10: Task 6 - 集成快捷键支持
 *
 * 在 CompressStateProvider 内部使用，处理：
 * - 状态持久化逻辑
 * - beforeunload 事件监听
 * - 导航拦截 (AC3, AC4)
 * - 键盘快捷键 (Story 10.10)
 */

import * as React from "react";
import { DualStreamLayout, type DualStreamLayoutRef } from "@/components/layout";
import { OriginalMessageList, CompressPreviewList, TokenStatistics, UnsavedChangesDialog, KeyboardShortcutsHelp, EditMessageSheet, InsertMessageSheet } from "@/components/compress";
import { useCompressState } from "@/hooks/useCompressState";
import { useNavigationGuard } from "@/hooks/useNavigationGuard";
import { useMessageFocus } from "@/hooks/useMessageFocus";
import { useCompressHotkeys } from "@/hooks/useCompressHotkeys";
import { useCompressPersistStore } from "@/stores";
import type { NarrativeMessage } from "@/types/message";

interface CompressModeContentProps {
  /** 布局 ref */
  layoutRef: React.RefObject<DualStreamLayoutRef | null>;
  /** 消息列表 */
  messages: NarrativeMessage[];
  /** 当前会话 ID */
  sessionId: string;
  /** 导出完成回调 (用于 AC4: 导出并离开) */
  onExportRequest?: () => void;
}

/**
 * 压缩模式内容组件
 * 必须在 CompressStateProvider 内部使用
 */
export function CompressModeContent({
  layoutRef,
  messages,
  sessionId,
  onExportRequest,
}: CompressModeContentProps) {
  const { hasAnyChanges, exportSnapshot, resetAll, initializeFromSnapshot, setOperation, removeOperation, addInsertion } = useCompressState();
  const persistStore = useCompressPersistStore();

  // Story 10.9 AC3: 未保存更改对话框状态
  const [showUnsavedDialog, setShowUnsavedDialog] = React.useState(false);

  // Story 10.10: 快捷键帮助面板状态
  const [showHelpDialog, setShowHelpDialog] = React.useState(false);

  // Story 10.10: 消息列表容器 ref (用于 scrollIntoView)
  const messageListContainerRef = React.useRef<HTMLDivElement>(null);

  // Story 10.10: 焦点管理
  const focus = useMessageFocus({
    messageCount: messages.length,
    containerRef: messageListContainerRef,
  });

  // Story 10.10: 编辑对话框状态 (用于快捷键触发)
  const [editingMessageId, setEditingMessageId] = React.useState<string | null>(null);
  const [isEditDialogOpen, setIsEditDialogOpen] = React.useState(false);

  // Story 10.10: 插入对话框状态 (用于快捷键触发)
  const [insertAfterIndex, setInsertAfterIndex] = React.useState<number>(-1);
  const [isInsertDialogOpen, setIsInsertDialogOpen] = React.useState(false);

  // Story 10.10: 快捷键回调
  const handleKeep = React.useCallback(
    (messageId: string) => {
      removeOperation(messageId);
    },
    [removeOperation]
  );

  const handleDelete = React.useCallback(
    (messageId: string) => {
      const message = messages.find((m) => m.id === messageId);
      if (message) {
        setOperation(messageId, {
          type: "delete",
          originalMessage: message,
        });
      }
    },
    [messages, setOperation]
  );

  const handleEdit = React.useCallback(
    (messageId: string) => {
      // Story 10.10 AC1: E 键打开编辑对话框
      setEditingMessageId(messageId);
      setIsEditDialogOpen(true);
    },
    []
  );

  const handleInsert = React.useCallback(
    (afterIndex: number) => {
      // Story 10.10 AC1: I 键在当前位置后插入
      setInsertAfterIndex(afterIndex);
      setIsInsertDialogOpen(true);
    },
    []
  );

  // Story 10.10: 编辑确认回调
  const handleEditConfirm = React.useCallback(
    (modifiedContent: string) => {
      if (editingMessageId) {
        const message = messages.find((m) => m.id === editingMessageId);
        if (message) {
          setOperation(editingMessageId, {
            type: "modify",
            originalMessage: message,
            modifiedContent,
          });
        }
      }
      setEditingMessageId(null);
      setIsEditDialogOpen(false);
    },
    [editingMessageId, messages, setOperation]
  );

  // Story 10.10: 获取正在编辑的消息
  const editingMessage = React.useMemo(
    () => (editingMessageId ? messages.find((m) => m.id === editingMessageId) ?? null : null),
    [editingMessageId, messages]
  );

  // Story 10.10: 插入确认回调
  const handleInsertConfirm = React.useCallback(
    (message: NarrativeMessage) => {
      addInsertion(insertAfterIndex, message);
      setInsertAfterIndex(-1);
      setIsInsertDialogOpen(false);
    },
    [insertAfterIndex, addInsertion]
  );

  const handleToggleHelp = React.useCallback(() => {
    setShowHelpDialog((prev) => !prev);
  }, []);

  // Story 10.10: 集成快捷键
  useCompressHotkeys({
    enabled: true,
    focus,
    messages,
    onKeep: handleKeep,
    onDelete: handleDelete,
    onEdit: handleEdit,
    onInsert: handleInsert,
    onOpenExport: onExportRequest,
    onToggleHelp: handleToggleHelp,
  });

  // 使用 ref 存储最新值，避免 cleanup 函数中的闭包问题
  const hasAnyChangesRef = React.useRef(hasAnyChanges);
  const exportSnapshotRef = React.useRef(exportSnapshot);
  const sessionIdRef = React.useRef(sessionId);

  React.useEffect(() => {
    hasAnyChangesRef.current = hasAnyChanges;
  }, [hasAnyChanges]);

  React.useEffect(() => {
    exportSnapshotRef.current = exportSnapshot;
  }, [exportSnapshot]);

  React.useEffect(() => {
    sessionIdRef.current = sessionId;
  }, [sessionId]);

  // Story 10.9: 进入压缩模式时尝试恢复状态
  const hasInitializedRef = React.useRef(false);
  React.useEffect(() => {
    if (!hasInitializedRef.current) {
      const savedSnapshot = persistStore.loadState(sessionId);
      if (savedSnapshot) {
        initializeFromSnapshot(savedSnapshot);
        if (import.meta.env.DEV) {
          console.log("[CompressModeContent] 已恢复压缩状态:", sessionId);
        }
      }
      hasInitializedRef.current = true;
    }
  }, [sessionId, persistStore, initializeFromSnapshot]);

  // Story 10.9 AC5: beforeunload 事件监听
  React.useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (hasAnyChangesRef.current) {
        e.preventDefault();
        // 现代浏览器会显示通用提示，不再支持自定义消息
        e.returnValue = "";
      }
    };

    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, []);

  // Story 10.9: 组件卸载时保存状态 (模式切换时)
  React.useEffect(() => {
    return () => {
      // 使用 ref 获取最新值避免闭包问题
      if (hasAnyChangesRef.current) {
        const snapshot = exportSnapshotRef.current();
        persistStore.saveState(sessionIdRef.current, snapshot);
        if (import.meta.env.DEV) {
          console.log("[CompressModeContent] 已保存压缩状态:", sessionIdRef.current);
        }
      }
    };
  }, [persistStore]); // persistStore 是稳定的

  // Story 10.9 AC3: 导航拦截
  const { isBlocked, proceed, reset } = useNavigationGuard({
    shouldBlock: hasAnyChanges,
    onBlock: () => setShowUnsavedDialog(true),
  });

  // Story 10.9 AC4: 导出并离开
  const handleExportAndLeave = React.useCallback(() => {
    if (onExportRequest) {
      onExportRequest();
    }
    // 导出完成后由外部调用 proceed
  }, [onExportRequest]);

  // Story 10.9 AC3: 不保存直接离开
  const handleDiscardAndLeave = React.useCallback(() => {
    persistStore.clearState();
    resetAll();
    proceed();
  }, [persistStore, resetAll, proceed]);

  // Story 10.9 AC3: 取消离开
  const handleCancel = React.useCallback(() => {
    reset();
    setShowUnsavedDialog(false);
  }, [reset]);

  return (
    <div className="flex-1 min-h-0 overflow-hidden flex flex-col">
      {/* Story 10.6 AC #1: 主内容区域添加 padding-bottom 避免被统计栏遮挡 */}
      <div className="flex-1 min-h-0 overflow-hidden">
        <DualStreamLayout
          ref={layoutRef}
          // Story 10.2: 精简模式左侧显示原始消息列表
          narrativeContent={
            <OriginalMessageList
              messages={messages}
              focus={focus}
              containerRef={messageListContainerRef}
            />
          }
          // Story 10.3: 右侧显示压缩预览列表
          codeContent={
            <CompressPreviewList
              messages={messages}
              focusedOriginalIndex={focus.focusedIndex}
            />
          }
          // Story 10.1 AC #5: 精简模式隐藏时间轴
          showTimeline={false}
        />
      </div>
      {/* Story 10.6: Token 统计栏 (AC #1: 固定在底部) */}
      <TokenStatistics messages={messages} />

      {/* Story 10.9 AC3/AC4: 未保存更改确认对话框 */}
      <UnsavedChangesDialog
        open={showUnsavedDialog || isBlocked}
        onOpenChange={setShowUnsavedDialog}
        onExportAndLeave={handleExportAndLeave}
        onDiscardAndLeave={handleDiscardAndLeave}
        onCancel={handleCancel}
      />

      {/* Story 10.10 AC4: 快捷键帮助面板 */}
      <KeyboardShortcutsHelp
        open={showHelpDialog}
        onOpenChange={setShowHelpDialog}
      />

      {/* Story 10.10 AC1: 快捷键触发的编辑 Sheet - Story 12.1 改造 */}
      <EditMessageSheet
        open={isEditDialogOpen}
        onOpenChange={setIsEditDialogOpen}
        message={editingMessage}
        onConfirm={handleEditConfirm}
      />

      {/* Story 10.10 AC1: 快捷键触发的插入 Sheet - Story 12.1 改造 */}
      <InsertMessageSheet
        open={isInsertDialogOpen}
        onOpenChange={setIsInsertDialogOpen}
        onConfirm={handleInsertConfirm}
        insertPosition={insertAfterIndex.toString()}
      />
    </div>
  );
}

export default CompressModeContent;
