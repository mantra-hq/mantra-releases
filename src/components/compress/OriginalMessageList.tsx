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
import { useVirtualizer } from "@tanstack/react-virtual";
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
 * 估算消息高度
 * 根据消息内容长度估算渲染高度
 */
function estimateMessageSize(message: NarrativeMessage): number {
  const textLength = message.content
    .filter((block) => block.type === "text")
    .reduce((acc, block) => acc + block.content.length, 0);

  // 基础高度 (包含角色图标、元信息等)
  const baseHeight = 72;
  // 内容行数估算 (约 60 字符每行)
  const lineEstimate = Math.ceil(textLength / 60);
  // 每行约 20px
  const contentHeight = Math.min(lineEstimate * 20, 60); // 折叠状态最多 3 行

  // 返回估算高度，限制在 72-140px (折叠状态)
  return Math.min(Math.max(baseHeight + contentHeight, 72), 140);
}

/**
 * 估算列表项高度 (包含插入触发器和已插入消息)
 */
function estimateItemSize(
  message: NarrativeMessage,
  hasInsertion: boolean,
  insertedMessage?: NarrativeMessage
): number {
  // 消息本身的高度
  let height = estimateMessageSize(message);

  // 插入触发器高度 (默认 8px，有插入时 24px)
  height += hasInsertion ? 24 : 8;

  // 如果有插入的消息，增加其高度
  if (insertedMessage) {
    height += estimateMessageSize(insertedMessage);
  }

  return height;
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
  const scrollContainerRef = React.useRef<HTMLDivElement>(null);

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

  // 虚拟化器配置 (复用 NarrativeStream 模式)
  const virtualizer = useVirtualizer({
    count: messages.length,
    getScrollElement: () => scrollContainerRef.current,
    estimateSize: (index) => {
      const message = messages[index];
      const insertion = insertions.get(index);
      return estimateItemSize(
        message,
        !!insertion,
        insertion?.insertedMessage
      );
    },
    overscan: 5, // 预渲染 5 条消息优化滚动体验
  });

  // [Fix #5] 当插入状态变化时，通知虚拟化器重新测量
  React.useEffect(() => {
    // measure() 方法可能在某些版本或 mock 中不存在，安全调用
    if (typeof virtualizer.measure === "function") {
      virtualizer.measure();
    }
  }, [insertions, virtualizer]);

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

  // Story 10.5: 插入确认回调
  const handleInsertConfirm = React.useCallback(
    (message: NarrativeMessage) => {
      addInsertion(insertAfterIndex, message);
      setInsertAfterIndex(-1);
    },
    [insertAfterIndex, addInsertion]
  );

  // Story 10.5: 移除插入
  const handleRemoveInsertion = React.useCallback(
    (afterIndex: number) => {
      removeInsertion(afterIndex);
    },
    [removeInsertion]
  );

  // AC1: 空状态处理
  if (messages.length === 0) {
    return (
      <div className={cn("h-full", className)}>
        <EmptyState />
      </div>
    );
  }

  const virtualItems = virtualizer.getVirtualItems();

  return (
    <>
      <div
        ref={scrollContainerRef}
        data-testid="original-message-list"
        className={cn(
          "h-full overflow-y-auto",
          // 自定义滚动条样式
          "scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent",
          className
        )}
      >
        {/* 虚拟列表容器 */}
        <div
          className="relative w-full px-3 py-2"
          style={{
            height: `${virtualizer.getTotalSize()}px`,
          }}
        >
          {/* Story 10.5: 列表开头的插入触发器 (索引 -1) */}
          {/* [Fix #3] 修复 UI 重复问题：显示 InsertedMessageCard 后，触发器保持正常状态允许继续插入 */}
          <div className="px-3 mb-1">
            {insertions.get(-1) && (
              <InsertedMessageCard
                message={insertions.get(-1)!.insertedMessage!}
                onRemove={() => handleRemoveInsertion(-1)}
              />
            )}
            <InsertMessageTrigger
              afterIndex={-1}
              hasInsertion={false}
              onClick={() => handleOpenInsertDialog(-1)}
            />
          </div>

          {/* 渲染可见的虚拟项 */}
          {virtualItems.map((virtualItem) => {
            const message = messages[virtualItem.index];
            const handlers = createOperationHandlers(message);
            const currentOperation = getOperationType(message.id);
            const insertion = insertions.get(virtualItem.index);

            return (
              <div
                key={virtualItem.key}
                data-index={virtualItem.index}
                className="absolute left-0 top-0 w-full px-3"
                style={{
                  transform: `translateY(${virtualItem.start + 32}px)`, // +32 for header trigger
                }}
              >
                <OriginalMessageCard
                  message={message}
                  index={virtualItem.index}
                  measureElement={virtualizer.measureElement}
                  showActionButtons={true}
                  currentOperation={currentOperation}
                  onKeepClick={handlers.onKeepClick}
                  onDeleteClick={handlers.onDeleteClick}
                  onEditClick={handlers.onEditClick}
                />

                {/* Story 10.5: 已插入的消息卡片 */}
                {/* [Fix #3] 修复 UI 重复问题：显示 InsertedMessageCard 后，触发器保持正常状态 */}
                {insertion?.insertedMessage && (
                  <InsertedMessageCard
                    message={insertion.insertedMessage}
                    onRemove={() => handleRemoveInsertion(virtualItem.index)}
                  />
                )}

                {/* Story 10.5: 插入触发器 */}
                <InsertMessageTrigger
                  afterIndex={virtualItem.index}
                  hasInsertion={false}
                  onClick={() => handleOpenInsertDialog(virtualItem.index)}
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

      {/* Story 10.5: 插入对话框 */}
      <InsertMessageDialog
        open={isInsertDialogOpen}
        onOpenChange={setIsInsertDialogOpen}
        onConfirm={handleInsertConfirm}
        insertPosition={insertAfterIndex.toString()}
      />
    </>
  );
}

export default OriginalMessageList;
