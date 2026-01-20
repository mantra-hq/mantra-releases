/**
 * useCompressState - 压缩状态管理 Hook
 * Story 10.3: Task 1
 *
 * 管理压缩模式下的操作状态（保留/删除/修改/插入）
 * 使用 React Context 实现左右面板状态共享
 */

import * as React from "react";
import type { NarrativeMessage, ContentBlock } from "@/types/message";
import { estimateTokenCount } from "@/lib/token-counter";

// ===== 类型定义 =====

/** 操作类型 */
export type OperationType = "keep" | "delete" | "modify" | "insert";

/** 压缩操作 */
export interface CompressOperation {
  /** 操作类型 */
  type: OperationType;
  /** 原始消息 (keep/delete/modify 时存在) */
  originalMessage?: NarrativeMessage;
  /** 修改后的内容 (modify 时存在) */
  modifiedContent?: string;
  /** 插入的消息 (insert 时存在) */
  insertedMessage?: NarrativeMessage;
  /** 插入位置 - 在该 index 之后插入 (insert 时存在) */
  insertAfterIndex?: number;
}

/** 预览消息项 */
export interface PreviewMessage {
  /** 唯一标识 */
  id: string;
  /** 操作类型 */
  operation: OperationType;
  /** 显示的消息内容 */
  message: NarrativeMessage;
  /** 原始 token 数 (delete 时显示节省的 token) */
  originalTokens?: number;
  /** token 变化量 (modify 时显示变化) */
  tokenDelta?: number;
}

/** 变更统计 */
export interface ChangeStats {
  /** 删除数量 */
  deleted: number;
  /** 修改数量 */
  modified: number;
  /** 插入数量 */
  inserted: number;
}

/** 压缩状态 Context 值类型 */
export interface CompressStateContextValue {
  /** 操作映射表 (messageId -> CompressOperation) */
  operations: Map<string, CompressOperation>;
  /** 设置操作 */
  setOperation: (messageId: string, operation: CompressOperation) => void;
  /** 移除操作 (恢复保留) */
  removeOperation: (messageId: string) => void;
  /** 批量重置 */
  resetAll: () => void;
  /** 获取预览消息列表 */
  getPreviewMessages: (messages: NarrativeMessage[]) => PreviewMessage[];
  /** 获取变更统计 */
  getChangeStats: () => ChangeStats;
  /** 插入操作列表 (用于在特定位置插入消息) */
  insertions: Map<number, CompressOperation>;
  /** 添加插入操作 */
  addInsertion: (afterIndex: number, message: NarrativeMessage) => void;
  /** 移除插入操作 */
  removeInsertion: (afterIndex: number) => void;
  /** Story 10.4: 获取指定消息的操作 */
  getOperationForMessage: (messageId: string) => CompressOperation | undefined;
  /** Story 10.4: 获取指定消息的操作类型 (默认 keep) */
  getOperationType: (messageId: string) => OperationType;
}

// ===== Context 创建 =====

const CompressStateContext = React.createContext<CompressStateContextValue | null>(null);

// ===== Hook =====

/**
 * 使用压缩状态 Context
 * @throws 如果在 Provider 外部使用会抛出错误
 */
export function useCompressState(): CompressStateContextValue {
  const context = React.useContext(CompressStateContext);
  if (!context) {
    throw new Error("useCompressState must be used within a CompressStateProvider");
  }
  return context;
}

// ===== 工具函数 =====

/**
 * 获取消息的文本内容
 */
function getMessageTextContent(content: ContentBlock[]): string {
  return content
    .filter((block) => block.type === "text")
    .map((block) => block.content)
    .join("\n");
}

/**
 * 计算消息的 token 数
 */
function calculateMessageTokens(message: NarrativeMessage): number {
  const textContent = getMessageTextContent(message.content);
  return estimateTokenCount(textContent);
}

/**
 * 创建修改后的消息
 */
function createModifiedMessage(
  original: NarrativeMessage,
  newContent: string
): NarrativeMessage {
  return {
    ...original,
    content: [{ type: "text", content: newContent }],
  };
}

// ===== Provider =====

export interface CompressStateProviderProps {
  children: React.ReactNode;
}

/**
 * 压缩状态 Provider
 * 提供压缩操作的状态管理
 */
export function CompressStateProvider({ children }: CompressStateProviderProps) {
  // 操作映射表 (messageId -> CompressOperation)
  const [operations, setOperations] = React.useState<Map<string, CompressOperation>>(
    () => new Map()
  );

  // 插入操作列表 (afterIndex -> CompressOperation)
  const [insertions, setInsertions] = React.useState<Map<number, CompressOperation>>(
    () => new Map()
  );

  // 设置操作
  const setOperation = React.useCallback(
    (messageId: string, operation: CompressOperation) => {
      setOperations((prev) => {
        const next = new Map(prev);
        next.set(messageId, operation);
        return next;
      });
    },
    []
  );

  // 移除操作 (恢复保留)
  const removeOperation = React.useCallback((messageId: string) => {
    setOperations((prev) => {
      const next = new Map(prev);
      next.delete(messageId);
      return next;
    });
  }, []);

  // 批量重置
  const resetAll = React.useCallback(() => {
    setOperations(new Map());
    setInsertions(new Map());
  }, []);

  // 添加插入操作
  const addInsertion = React.useCallback(
    (afterIndex: number, message: NarrativeMessage) => {
      setInsertions((prev) => {
        const next = new Map(prev);
        next.set(afterIndex, {
          type: "insert",
          insertedMessage: message,
          insertAfterIndex: afterIndex,
        });
        return next;
      });
    },
    []
  );

  // 移除插入操作
  const removeInsertion = React.useCallback((afterIndex: number) => {
    setInsertions((prev) => {
      const next = new Map(prev);
      next.delete(afterIndex);
      return next;
    });
  }, []);

  // 获取预览消息列表
  const getPreviewMessages = React.useCallback(
    (messages: NarrativeMessage[]): PreviewMessage[] => {
      const result: PreviewMessage[] = [];

      for (let i = 0; i < messages.length; i++) {
        const message = messages[i];
        const operation = operations.get(message.id);

        // 检查是否有在当前位置之前的插入
        // 插入位置是 "在 afterIndex 之后"，所以 afterIndex = i-1 的插入应该在当前消息之前
        if (i > 0) {
          const insertOp = insertions.get(i - 1);
          if (insertOp?.insertedMessage) {
            result.push({
              id: `insert-${i - 1}`,
              operation: "insert",
              message: insertOp.insertedMessage,
            });
          }
        }

        if (!operation) {
          // 默认保留
          result.push({
            id: message.id,
            operation: "keep",
            message,
          });
        } else {
          switch (operation.type) {
            case "keep":
              result.push({
                id: message.id,
                operation: "keep",
                message,
              });
              break;

            case "delete":
              result.push({
                id: message.id,
                operation: "delete",
                message,
                originalTokens: calculateMessageTokens(message),
              });
              break;

            case "modify":
              if (operation.modifiedContent !== undefined) {
                const modifiedMessage = createModifiedMessage(
                  message,
                  operation.modifiedContent
                );
                const originalTokens = calculateMessageTokens(message);
                const newTokens = calculateMessageTokens(modifiedMessage);
                result.push({
                  id: message.id,
                  operation: "modify",
                  message: modifiedMessage,
                  originalTokens,
                  tokenDelta: newTokens - originalTokens,
                });
              }
              break;
          }
        }
      }

      // 检查末尾的插入
      const lastInsertOp = insertions.get(messages.length - 1);
      if (lastInsertOp?.insertedMessage) {
        result.push({
          id: `insert-${messages.length - 1}`,
          operation: "insert",
          message: lastInsertOp.insertedMessage,
        });
      }

      return result;
    },
    [operations, insertions]
  );

  // 获取变更统计
  const getChangeStats = React.useCallback((): ChangeStats => {
    let deleted = 0;
    let modified = 0;

    operations.forEach((op) => {
      if (op.type === "delete") deleted++;
      if (op.type === "modify") modified++;
    });

    const inserted = insertions.size;

    return { deleted, modified, inserted };
  }, [operations, insertions]);

  // Story 10.4: 获取指定消息的操作
  const getOperationForMessage = React.useCallback(
    (messageId: string): CompressOperation | undefined => {
      return operations.get(messageId);
    },
    [operations]
  );

  // Story 10.4: 获取指定消息的操作类型 (默认 keep)
  const getOperationType = React.useCallback(
    (messageId: string): OperationType => {
      const operation = operations.get(messageId);
      return operation?.type ?? "keep";
    },
    [operations]
  );

  // Context 值
  const contextValue = React.useMemo<CompressStateContextValue>(
    () => ({
      operations,
      setOperation,
      removeOperation,
      resetAll,
      getPreviewMessages,
      getChangeStats,
      insertions,
      addInsertion,
      removeInsertion,
      getOperationForMessage,
      getOperationType,
    }),
    [
      operations,
      setOperation,
      removeOperation,
      resetAll,
      getPreviewMessages,
      getChangeStats,
      insertions,
      addInsertion,
      removeInsertion,
      getOperationForMessage,
      getOperationType,
    ]
  );

  return (
    <CompressStateContext.Provider value={contextValue}>
      {children}
    </CompressStateContext.Provider>
  );
}

export default useCompressState;
