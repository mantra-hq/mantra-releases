/**
 * OriginalMessageList - 原始消息列表组件
 * Story 10.2: Task 1
 * Story 10.4: Task 5 - 集成操作按钮和编辑对话框
 * Story 10.5: Task 4 - 集成消息插入功能
 *
 * 使用 @tanstack/react-virtual 实现大量消息的高性能虚拟化渲染
 * 在压缩模式下显示完整的原始会话消息
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { MessageSquare } from "lucide-react";
import { cn } from "@/lib/utils";
import { OriginalMessageCard } from "./OriginalMessageCard";
import { EditMessageDialog } from "./EditMessageDialog";
import { InsertMessageTrigger } from "./InsertMessageTrigger";
import { InsertMessageDialog } from "./InsertMessageDialog";
import { InsertedMessageCard } from "./InsertedMessageCard";
import type { NarrativeMessage } from "@/types/message";
import { useCompressState } from "@/hooks/useCompressState";

/**
 * OriginalMessageList 组件 Props
 */
export interface OriginalMessageListProps {
  /** 消息列表 */
  messages: NarrativeMessage[];
  /** 自定义 className */
  className?: string;
}

/**
 * 空状态组件
 */
function EmptyState() {
  const { t } = useTranslation();
  return (
    <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
      <div className="flex flex-col items-center gap-4 p-8 text-center">
        <div className="rounded-full bg-muted p-4">
          <MessageSquare className="size-8" />
        </div>
        <div className="space-y-2">
          <h3 className="text-lg font-semibold text-foreground">
            {t("compress.originalList.empty")}
          </h3>
          <p className="text-sm max-w-xs">
            {t("compress.originalList.emptyHint")}
          </p>
        </div>
      </div>
    </div>
  );
}

/**
 * OriginalMessageList - 虚拟化原始消息列表
 *
 * AC1: 显示完整的原始消息列表
 * AC4: 使用虚拟化保持性能
 * Story 10.4: 集成操作按钮和编辑对话框
 * Story 10.5: 集成消息插入功能
 */
export function OriginalMessageList({
  messages,
  className,
}: OriginalMessageListProps) {
  // Story 10.4: 压缩状态管理
  const {
    setOperation,
    removeOperation,
    getOperationType,
    insertions,
    addInsertion,
    removeInsertion,
  } = useCompressState();

  // Story 10.4: 编辑对话框状态
  const [editingMessage, setEditingMessage] = React.useState<NarrativeMessage | null>(null);
  const [isEditDialogOpen, setIsEditDialogOpen] = React.useState(false);

  // Story 10.5: 插入对话框状态
  const [isInsertDialogOpen, setIsInsertDialogOpen] = React.useState(false);
  const [insertAfterIndex, setInsertAfterIndex] = React.useState<number>(-1);
  // 编辑已插入消息的状态
  const [editingInsertedMessage, setEditingInsertedMessage] = React.useState<NarrativeMessage | null>(null);

  // Story 10.4: 操作回调生成器
  const createOperationHandlers = React.useCallback(
    (message: NarrativeMessage) => ({
      onKeepClick: () => {
        removeOperation(message.id);
      },
      onDeleteClick: () => {
        setOperation(message.id, {
          type: "delete",
          originalMessage: message,
        });
      },
      onEditClick: () => {
        setEditingMessage(message);
        setIsEditDialogOpen(true);
      },
    }),
    [setOperation, removeOperation]
  );

  // Story 10.4: 编辑确认回调
  const handleEditConfirm = React.useCallback(
    (modifiedContent: string) => {
      if (editingMessage) {
        setOperation(editingMessage.id, {
          type: "modify",
          originalMessage: editingMessage,
          modifiedContent,
        });
      }
      setEditingMessage(null);
    },
    [editingMessage, setOperation]
  );

  // Story 10.5: 打开插入对话框
  const handleOpenInsertDialog = React.useCallback((afterIndex: number) => {
    setInsertAfterIndex(afterIndex);
    setIsInsertDialogOpen(true);
  }, []);

  // Story 10.5: 移除插入
  const handleRemoveInsertion = React.useCallback(
    (afterIndex: number) => {
      removeInsertion(afterIndex);
    },
    [removeInsertion]
  );

  // Story 10.5: 编辑已插入的消息
  const handleEditInsertion = React.useCallback(
    (afterIndex: number) => {
      const insertion = insertions.get(afterIndex);
      if (insertion?.insertedMessage) {
        setEditingInsertedMessage(insertion.insertedMessage);
        setInsertAfterIndex(afterIndex);
        setIsInsertDialogOpen(true);
      }
    },
    [insertions]
  );

  // Story 10.5: 插入/编辑确认回调 (支持二次编辑)
  const handleInsertDialogConfirm = React.useCallback(
    (message: NarrativeMessage) => {
      // 如果是编辑已有的插入消息，先删除再重新添加
      if (editingInsertedMessage) {
        removeInsertion(insertAfterIndex);
      }
      addInsertion(insertAfterIndex, message);
      setInsertAfterIndex(-1);
      setEditingInsertedMessage(null);
    },
    [insertAfterIndex, addInsertion, removeInsertion, editingInsertedMessage]
  );

  // 关闭插入对话框时清理状态
  const handleInsertDialogOpenChange = React.useCallback(
    (open: boolean) => {
      setIsInsertDialogOpen(open);
      if (!open) {
        setEditingInsertedMessage(null);
      }
    },
    []
  );

  // AC1: 空状态处理
  if (messages.length === 0) {
    return (
      <div className={cn("h-full", className)}>
        <EmptyState />
      </div>
    );
  }

  return (
    <>
      <div
        data-testid="original-message-list"
        className={cn(
          "h-full overflow-y-auto",
          // 自定义滚动条样式
          "scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent",
          className
        )}
      >
        {/* 消息列表容器 - 使用普通流式布局避免重叠问题 */}
        <div className="w-full px-3 py-2 space-y-1">
          {/* Story 10.5: 列表开头的插入触发器 (索引 -1) */}
          {insertions.get(-1) && (
            <InsertedMessageCard
              message={insertions.get(-1)!.insertedMessage!}
              onRemove={() => handleRemoveInsertion(-1)}
              onEdit={() => handleEditInsertion(-1)}
            />
          )}
          <InsertMessageTrigger
            afterIndex={-1}
            hasInsertion={false}
            onClick={() => handleOpenInsertDialog(-1)}
          />

          {/* 渲染消息列表 */}
          {messages.map((message, index) => {
            const handlers = createOperationHandlers(message);
            const currentOperation = getOperationType(message.id);
            const insertion = insertions.get(index);

            return (
              <div key={message.id} data-index={index}>
                <OriginalMessageCard
                  message={message}
                  index={index}
                  showActionButtons={true}
                  currentOperation={currentOperation}
                  onKeepClick={handlers.onKeepClick}
                  onDeleteClick={handlers.onDeleteClick}
                  onEditClick={handlers.onEditClick}
                />

                {/* Story 10.5: 已插入的消息卡片 */}
                {insertion?.insertedMessage && (
                  <InsertedMessageCard
                    message={insertion.insertedMessage}
                    onRemove={() => handleRemoveInsertion(index)}
                    onEdit={() => handleEditInsertion(index)}
                  />
                )}

                {/* Story 10.5: 插入触发器 */}
                <InsertMessageTrigger
                  afterIndex={index}
                  hasInsertion={false}
                  onClick={() => handleOpenInsertDialog(index)}
                />
              </div>
            );
          })}
        </div>
      </div>

      {/* Story 10.4: 编辑对话框 */}
      <EditMessageDialog
        open={isEditDialogOpen}
        onOpenChange={setIsEditDialogOpen}
        message={editingMessage}
        onConfirm={handleEditConfirm}
      />

      {/* Story 10.5: 插入对话框 (支持新建和编辑) */}
      <InsertMessageDialog
        open={isInsertDialogOpen}
        onOpenChange={handleInsertDialogOpenChange}
        onConfirm={handleInsertDialogConfirm}
        insertPosition={insertAfterIndex.toString()}
        initialMessage={editingInsertedMessage}
      />
    </>
  );
}

export default OriginalMessageList;
