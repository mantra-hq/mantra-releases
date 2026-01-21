/**
 * useCompressState - 压缩状态管理 Hook
 * Story 10.3: Task 1
 * Story 10.8: 添加撤销/重做/重置功能
 *
 * 管理压缩模式下的操作状态（保留/删除/修改/插入）
 * 使用 React Context 实现左右面板状态共享
 */

import * as React from "react";
import type { NarrativeMessage } from "@/types/message";
import { estimateTokenCount } from "@/lib/token-counter";
import { getMessageDisplayContent } from "@/lib/message-utils";

// ===== 类型定义 =====

/** 操作类型 */
export type OperationType = "keep" | "delete" | "modify" | "insert";

/**
 * Story 10.8: 状态快照类型
 * 用于历史栈存储
 */
export interface StateSnapshot {
  operations: Map<string, CompressOperation>;
  insertions: Map<number, CompressOperation>;
}

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

/** 
 * Token 统计数据 
 * Story 10.6: Task 3.3
 */
export interface TokenStats {
  /** 原始 Token 总数 */
  originalTotal: number;
  /** 压缩后 Token 总数 */
  compressedTotal: number;
  /** 节省的 Token 数 */
  savedTokens: number;
  /** 节省百分比 (0-100) */
  savedPercentage: number;
  /** 变更统计 */
  changeStats: ChangeStats;
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
  /** Story 10.6: 获取 Token 统计 */
  getTokenStats: (messages: NarrativeMessage[]) => TokenStats;
  /** Story 10.8: 撤销操作 */
  undo: () => void;
  /** Story 10.8: 重做操作 */
  redo: () => void;
  /** Story 10.8: 是否可以撤销 */
  canUndo: boolean;
  /** Story 10.8: 是否可以重做 */
  canRedo: boolean;
  /** Story 10.8: 是否有任何变更 */
  hasAnyChanges: boolean;
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
 * Story 10.8: 深拷贝 Map 的辅助函数
 */
function cloneMap<K, V extends object>(map: Map<K, V>): Map<K, V> {
  return new Map(Array.from(map.entries()).map(([k, v]) => [k, { ...v }]));
}

/** Story 10.8: 最大历史栈深度 (AC6) */
const MAX_HISTORY_SIZE = 50;

/**
 * 计算消息的 token 数
 * Story 10.6/10-2 Fix: 使用 getMessageDisplayContent 支持所有内容类型
 */
function calculateMessageTokens(message: NarrativeMessage): number {
  const textContent = getMessageDisplayContent(message.content);
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

  // Story 10.8: 撤销/重做历史栈
  const [undoStack, setUndoStack] = React.useState<StateSnapshot[]>([]);
  const [redoStack, setRedoStack] = React.useState<StateSnapshot[]>([]);

  // Story 10.8: 使用 Ref 跟踪最新状态 (避免闭包陷阱)
  const operationsRef = React.useRef(operations);
  const insertionsRef = React.useRef(insertions);

  React.useEffect(() => {
    operationsRef.current = operations;
  }, [operations]);

  React.useEffect(() => {
    insertionsRef.current = insertions;
  }, [insertions]);

  // Story 10.8: 推入历史栈 (内部方法)
  const pushHistory = React.useCallback(() => {
    // 创建当前状态快照
    const snapshot: StateSnapshot = {
      operations: cloneMap(operationsRef.current),
      insertions: cloneMap(insertionsRef.current),
    };

    // 推入撤销栈
    setUndoStack((prev) => {
      const next = [...prev, snapshot];
      // AC6: 限制 50 步
      if (next.length > MAX_HISTORY_SIZE) {
        return next.slice(-MAX_HISTORY_SIZE);
      }
      return next;
    });

    // AC7: 清空重做栈
    setRedoStack([]);
  }, []);

  // Story 10.8: 撤销操作 (AC2)
  const undo = React.useCallback(() => {
    setUndoStack((prev) => {
      if (prev.length === 0) return prev;

      const snapshot = prev[prev.length - 1];
      const newStack = prev.slice(0, -1);

      // 当前状态推入重做栈
      setRedoStack((redoPrev) => [
        ...redoPrev,
        {
          operations: cloneMap(operationsRef.current),
          insertions: cloneMap(insertionsRef.current),
        },
      ]);

      // 恢复快照状态
      setOperations(snapshot.operations);
      setInsertions(snapshot.insertions);

      return newStack;
    });
  }, []);

  // Story 10.8: 重做操作 (AC3)
  const redo = React.useCallback(() => {
    setRedoStack((prev) => {
      if (prev.length === 0) return prev;

      const snapshot = prev[prev.length - 1];
      const newStack = prev.slice(0, -1);

      // 当前状态推入撤销栈
      setUndoStack((undoPrev) => [
        ...undoPrev,
        {
          operations: cloneMap(operationsRef.current),
          insertions: cloneMap(insertionsRef.current),
        },
      ]);

      // 恢复快照状态
      setOperations(snapshot.operations);
      setInsertions(snapshot.insertions);

      return newStack;
    });
  }, []);

  // Story 10.8: 计算派生状态 (AC5)
  const canUndo = undoStack.length > 0;
  const canRedo = redoStack.length > 0;
  const hasAnyChanges = operations.size > 0 || insertions.size > 0;

  // 设置操作 (修改: 操作前保存历史)
  const setOperation = React.useCallback(
    (messageId: string, operation: CompressOperation) => {
      pushHistory(); // Story 10.8: 保存历史
      setOperations((prev) => {
        const next = new Map(prev);
        next.set(messageId, operation);
        return next;
      });
    },
    [pushHistory]
  );

  // 移除操作 (恢复保留) (修改: 操作前保存历史)
  const removeOperation = React.useCallback((messageId: string) => {
    pushHistory(); // Story 10.8: 保存历史
    setOperations((prev) => {
      const next = new Map(prev);
      next.delete(messageId);
      return next;
    });
  }, [pushHistory]);

  // 批量重置 (修改: 同时清空历史栈)
  const resetAll = React.useCallback(() => {
    setOperations(new Map());
    setInsertions(new Map());
    // Story 10.8 AC4: 重置时清空操作栈
    setUndoStack([]);
    setRedoStack([]);
  }, []);

  // 添加插入操作 (修改: 操作前保存历史)
  const addInsertion = React.useCallback(
    (afterIndex: number, message: NarrativeMessage) => {
      pushHistory(); // Story 10.8: 保存历史
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
    [pushHistory]
  );

  // 移除插入操作 (修改: 操作前保存历史)
  const removeInsertion = React.useCallback((afterIndex: number) => {
    pushHistory(); // Story 10.8: 保存历史
    setInsertions((prev) => {
      const next = new Map(prev);
      next.delete(afterIndex);
      return next;
    });
  }, [pushHistory]);

  // 获取预览消息列表
  const getPreviewMessages = React.useCallback(
    (messages: NarrativeMessage[]): PreviewMessage[] => {
      const result: PreviewMessage[] = [];

      // [Fix #1] 检查列表开头的插入 (index=-1，在第一条消息之前)
      const firstInsertOp = insertions.get(-1);
      if (firstInsertOp?.insertedMessage) {
        result.push({
          id: `insert--1`,
          operation: "insert",
          message: firstInsertOp.insertedMessage,
        });
      }

      for (let i = 0; i < messages.length; i++) {
        const message = messages[i];
        const operation = operations.get(message.id);

        // 检查是否有在当前位置之前的插入
        // 插入位置是 "在 afterIndex 之后"，所以 afterIndex = i-1 的插入应该在当前消息之前
        // 注意: index=-1 的插入已在循环前处理
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

  // Story 10.6: 获取 Token 统计
  const getTokenStats = React.useCallback(
    (messages: NarrativeMessage[]): TokenStats => {
      // 计算原始 Token 总数
      const originalTotal = messages.reduce((total, message) => {
        const textContent = getMessageDisplayContent(message.content);
        return total + estimateTokenCount(textContent);
      }, 0);

      // 计算压缩后 Token 总数
      let compressedTotal = 0;

      // 遍历原始消息，考虑删除/修改操作
      messages.forEach((message) => {
        const operation = operations.get(message.id);

        if (!operation || operation.type === "keep") {
          // 保留: 计入原始 token
          const textContent = getMessageDisplayContent(message.content);
          compressedTotal += estimateTokenCount(textContent);
        } else if (operation.type === "modify" && operation.modifiedContent) {
          // 修改: 计入修改后的 token
          compressedTotal += estimateTokenCount(operation.modifiedContent);
        }
        // delete: 不计入
      });

      // 添加插入的消息 token
      insertions.forEach((insertion) => {
        if (insertion.insertedMessage) {
          const textContent = getMessageDisplayContent(insertion.insertedMessage.content);
          compressedTotal += estimateTokenCount(textContent);
        }
      });

      // 计算节省量
      const savedTokens = originalTotal - compressedTotal;
      const savedPercentage = originalTotal > 0
        ? (savedTokens / originalTotal) * 100
        : 0;

      // 获取变更统计
      const changeStats = getChangeStats();

      return {
        originalTotal,
        compressedTotal,
        savedTokens,
        savedPercentage,
        changeStats,
      };
    },
    [operations, insertions, getChangeStats]
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
      getTokenStats,
      // Story 10.8: undo/redo
      undo,
      redo,
      canUndo,
      canRedo,
      hasAnyChanges,
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
      getTokenStats,
      // Story 10.8: undo/redo
      undo,
      redo,
      canUndo,
      canRedo,
      hasAnyChanges,
    ]
  );

  return (
    <CompressStateContext.Provider value={contextValue}>
      {children}
    </CompressStateContext.Provider>
  );
}

export default useCompressState;
