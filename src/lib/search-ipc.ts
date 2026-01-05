/**
 * search-ipc - 搜索 IPC 封装
 * Story 2.10: Task 4
 *
 * 封装 Tauri IPC 调用，用于全局搜索功能
 * 与后端 Tanzou 搜索引擎交互
 */

import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "@/stores/useSearchStore";

/**
 * Tanzou 搜索结果 (后端格式)
 */
interface TanzouSearchResult {
    id: string;
    session_id: string;
    project_id: string;
    project_name: string;
    session_name: string;
    message_id: string;
    content: string;
    match_positions: Array<[number, number]>;
    timestamp: number;
}

/**
 * 搜索会话内容
 * 调用后端 Tanzou 搜索引擎
 *
 * @param query 搜索关键词
 * @param limit 返回结果数量限制 (默认 50)
 * @returns 搜索结果列表
 */
export async function searchSessions(
    query: string,
    limit: number = 50
): Promise<SearchResult[]> {
    if (!query.trim()) {
        return [];
    }

    try {
        console.log("[search-ipc] Calling search_sessions with query:", query);
        const results = await invoke<TanzouSearchResult[]>("search_sessions", {
            query: query.trim(),
            limit,
        });

        console.log("[search-ipc] Received results:", results);

        return results.map((r) => ({
            id: r.id,
            projectId: r.project_id,
            projectName: r.project_name,
            sessionId: r.session_id,
            sessionName: r.session_name,
            messageId: r.message_id,
            snippet: r.content,
            highlightRanges: r.match_positions,
            timestamp: r.timestamp,
        }));
    } catch (error) {
        console.error("[search-ipc] Search failed:", error);
        // 如果后端尚未实现，返回空数组
        return [];
    }
}

/**
 * 使用防抖的搜索 Hook 辅助函数
 * 创建一个防抖搜索函数
 *
 * @param delayMs 防抖延迟 (毫秒)
 * @returns 防抖搜索函数和取消函数
 */
export function createDebouncedSearch(delayMs: number = 300) {
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    const debouncedSearch = async (
        query: string,
        onResult: (results: SearchResult[]) => void,
        onLoading: (loading: boolean) => void
    ) => {
        // 清除之前的定时器
        if (timeoutId) {
            clearTimeout(timeoutId);
        }

        // 空查询立即返回
        if (!query.trim()) {
            onResult([]);
            onLoading(false);
            return;
        }

        // 设置加载状态
        onLoading(true);

        // 防抖执行搜索
        timeoutId = setTimeout(async () => {
            try {
                const results = await searchSessions(query);
                onResult(results);
            } catch (error) {
                console.error("[search-ipc] Debounced search failed:", error);
                onResult([]);
            } finally {
                onLoading(false);
            }
        }, delayMs);
    };

    const cancel = () => {
        if (timeoutId) {
            clearTimeout(timeoutId);
            timeoutId = null;
        }
    };

    return { debouncedSearch, cancel };
}
