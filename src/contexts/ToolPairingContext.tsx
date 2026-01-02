/**
 * ToolPairingContext - 工具配对上下文
 * Story 2.15: AC #5, #6
 *
 * 提供工具调用和输出的配对信息，支持跳转和高亮
 */

import * as React from "react";
import type { NarrativeMessage } from "@/types/message";
import { useDetailPanelStore } from "@/stores/useDetailPanelStore";

/** 配对信息 */
export interface ToolPairInfo {
  toolUseId: string;
  toolName: string;
  toolInput?: Record<string, unknown>;
  outputContent?: string;
  isError?: boolean;
  /** tool_use 所在的消息 ID */
  callMessageId?: string;
  /** tool_result 所在的消息 ID */
  outputMessageId?: string;
}

/** 滚动回调类型 */
export type ScrollToMessageCallback = (messageId: string) => void;

/** Context 值 */
interface ToolPairingContextValue {
  /** 配对 Map，key 为 toolUseId */
  pairs: Map<string, ToolPairInfo>;
  /** 获取配对的输出内容 */
  getOutput: (toolUseId: string) => string | undefined;
  /** 获取配对的错误状态 */
  getIsError: (toolUseId: string) => boolean;
  /** 滚动到工具调用或输出 */
  scrollTo: (toolUseId: string, target: "call" | "output") => void;
  /** 注册滚动回调 (由 NarrativePanel 调用) */
  registerScrollCallback: (callback: ScrollToMessageCallback) => void;
}

const ToolPairingContext = React.createContext<ToolPairingContextValue | null>(null);

/** 从消息列表构建配对 Map */
function buildPairMap(messages: NarrativeMessage[]): Map<string, ToolPairInfo> {
  const map = new Map<string, ToolPairInfo>();

  for (const message of messages) {
    for (const block of message.content) {
      if (block.type === "tool_use" && block.toolUseId) {
        map.set(block.toolUseId, {
          toolUseId: block.toolUseId,
          toolName: block.toolName || "Unknown",
          toolInput: block.toolInput,
          callMessageId: message.id,
        });
      } else if (block.type === "tool_result" && block.toolUseId) {
        const existing = map.get(block.toolUseId);
        if (existing) {
          existing.outputContent = block.content;
          existing.isError = block.isError;
          existing.outputMessageId = message.id;
        } else {
          // tool_result 先于 tool_use 出现（理论上不应该发生）
          map.set(block.toolUseId, {
            toolUseId: block.toolUseId,
            toolName: block.associatedToolName || "Unknown",
            outputContent: block.content,
            isError: block.isError,
            outputMessageId: message.id,
          });
        }
      }
    }
  }

  return map;
}

/** Provider Props */
interface ToolPairingProviderProps {
  messages: NarrativeMessage[];
  children: React.ReactNode;
}

/**
 * ToolPairingProvider
 *
 * 包裹消息列表，提供配对信息给子组件
 */
export function ToolPairingProvider({ messages, children }: ToolPairingProviderProps) {
  const pairs = React.useMemo(() => buildPairMap(messages), [messages]);
  const scrollCallbackRef = React.useRef<ScrollToMessageCallback | null>(null);
  const setHighlightedToolId = useDetailPanelStore((state) => state.setHighlightedToolId);

  const getOutput = React.useCallback(
    (toolUseId: string) => pairs.get(toolUseId)?.outputContent,
    [pairs]
  );

  const getIsError = React.useCallback(
    (toolUseId: string) => pairs.get(toolUseId)?.isError ?? false,
    [pairs]
  );

  const registerScrollCallback = React.useCallback((callback: ScrollToMessageCallback) => {
    scrollCallbackRef.current = callback;
  }, []);

  const scrollTo = React.useCallback(
    (toolUseId: string, target: "call" | "output") => {
      const pair = pairs.get(toolUseId);
      if (!pair) return;

      const messageId = target === "call" ? pair.callMessageId : pair.outputMessageId;
      if (!messageId) return;

      // 1. 设置高亮状态 (持续 5 秒)
      setHighlightedToolId(toolUseId);

      // 2. 使用虚拟化列表的滚动方法
      if (scrollCallbackRef.current) {
        scrollCallbackRef.current(messageId);
      }

      // 3. 滚动完成后，延迟查找元素并确保可见
      // 虚拟化列表需要时间渲染目标元素
      setTimeout(() => {
        const selector =
          target === "call"
            ? `[data-tool-use-id="${toolUseId}"]`
            : `[data-tool-output-id="${toolUseId}"]`;

        const element = document.querySelector(selector);
        if (element) {
          // 微调滚动位置确保元素在视口中央
          element.scrollIntoView({
            behavior: "smooth",
            block: "center",
          });
        }
      }, 300);

      // 4. 5 秒后清除高亮
      setTimeout(() => {
        setHighlightedToolId(null);
      }, 5000);
    },
    [pairs, setHighlightedToolId]
  );

  const value = React.useMemo(
    () => ({ pairs, getOutput, getIsError, scrollTo, registerScrollCallback }),
    [pairs, getOutput, getIsError, scrollTo, registerScrollCallback]
  );

  return (
    <ToolPairingContext.Provider value={value}>
      {children}
    </ToolPairingContext.Provider>
  );
}

/**
 * useToolPairingContext
 *
 * 获取配对上下文
 */
export function useToolPairingContext(): ToolPairingContextValue | null {
  return React.useContext(ToolPairingContext);
}

export default ToolPairingContext;
