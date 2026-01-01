/**
 * useToolPairing - 工具调用配对 hook
 * Story 2.15: Task 5
 *
 * 通过 tool_use_id 关联 Call 和 Output，提供跳转和高亮功能
 * AC: #5, #6
 */

import * as React from "react";

/** 工具调用消息 */
export interface ToolCallMessage {
    id: string;
    type: "tool_call";
    toolUseId: string;
    toolName: string;
    toolInput?: Record<string, unknown>;
}

/** 工具输出消息 */
export interface ToolOutputMessage {
    id: string;
    type: "tool_output";
    toolUseId: string;
    content: string;
    isError?: boolean;
}

/** 配对结果 */
export interface ToolPair {
    call: ToolCallMessage;
    output?: ToolOutputMessage;
}

/** 配对 Map */
export type ToolPairMap = Map<string, ToolPair>;

/** Hook 返回值 */
export interface UseToolPairingResult {
    /** 配对 Map，key 为 toolUseId */
    pairs: ToolPairMap;
    /** 当前高亮的 toolUseId */
    highlightedId: string | null;
    /** 设置高亮 ID */
    setHighlightedId: (id: string | null) => void;
    /** 滚动到指定 toolUseId 的元素 */
    scrollTo: (toolUseId: string, target: "call" | "output") => void;
    /** 获取配对 */
    getPair: (toolUseId: string) => ToolPair | undefined;
    /** 检查是否有配对输出 */
    hasOutput: (toolUseId: string) => boolean;
}

/**
 * 构建工具配对 Map
 */
function buildPairMap(
    calls: ToolCallMessage[],
    outputs: ToolOutputMessage[]
): ToolPairMap {
    const map = new Map<string, ToolPair>();

    // 先添加所有 call
    for (const call of calls) {
        map.set(call.toolUseId, { call });
    }

    // 关联 output
    for (const output of outputs) {
        const pair = map.get(output.toolUseId);
        if (pair) {
            pair.output = output;
        }
    }

    return map;
}

/**
 * useToolPairing hook
 *
 * 管理工具调用和输出的配对关系，支持：
 * - 通过 toolUseId 关联 Call 和 Output
 * - 悬停时高亮配对消息
 * - 点击链接跳转到配对位置
 */
export function useToolPairing(
    calls: ToolCallMessage[],
    outputs: ToolOutputMessage[]
): UseToolPairingResult {
    const [highlightedId, setHighlightedId] = React.useState<string | null>(null);

    // 构建配对 Map
    const pairs = React.useMemo(
        () => buildPairMap(calls, outputs),
        [calls, outputs]
    );

    // 滚动到指定元素
    const scrollTo = React.useCallback(
        (toolUseId: string, target: "call" | "output") => {
            const selector =
                target === "call"
                    ? `[data-tool-use-id="${toolUseId}"]`
                    : `[data-tool-output-id="${toolUseId}"]`;

            const element = document.querySelector(selector);
            if (element) {
                element.scrollIntoView({
                    behavior: "smooth",
                    block: "center",
                });
                // 短暂高亮
                setHighlightedId(toolUseId);
                setTimeout(() => setHighlightedId(null), 2000);
            }
        },
        []
    );

    // 获取配对
    const getPair = React.useCallback(
        (toolUseId: string) => pairs.get(toolUseId),
        [pairs]
    );

    // 检查是否有配对输出
    const hasOutput = React.useCallback(
        (toolUseId: string) => {
            const pair = pairs.get(toolUseId);
            return Boolean(pair?.output);
        },
        [pairs]
    );

    return {
        pairs,
        highlightedId,
        setHighlightedId,
        scrollTo,
        getPair,
        hasOutput,
    };
}

export default useToolPairing;
