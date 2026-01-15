/**
 * search-ipc - 搜索 IPC 封装
 * Story 2.10: Task 4
 * Story 9.2: Task 5.3 (使用 IPC 适配器)
 * Story 2.33: Task 5 (添加 filters 参数)
 *
 * 封装 Tauri IPC 调用，用于全局搜索功能
 * 与后端 Tanzou 搜索引擎交互
 */

import { invoke } from "@/lib/ipc-adapter";
import type { ContentType, SearchFilters, SearchResult, TimePreset } from "@/stores/useSearchStore";

/**
 * 搜索筛选器 (后端格式)
 */
interface BackendSearchFilters {
    contentType: ContentType;
    projectId: string | null;
    timePreset: TimePreset | null;
}

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
    content_type?: ContentType;
}

/**
 * 搜索会话内容
 * 调用后端 Tanzou 搜索引擎
 *
 * @param query 搜索关键词
 * @param limit 返回结果数量限制 (默认 50)
 * @param filters 搜索筛选器 (Story 2.33)
 * @returns 搜索结果列表
 */
export async function searchSessions(
    query: string,
    limit: number = 50,
    filters?: SearchFilters
): Promise<SearchResult[]> {
    if (!query.trim()) {
        return [];
    }

    try {
        console.log("[search-ipc] Calling search_sessions with query:", query, "filters:", filters);

        // 转换筛选器为后端格式
        const backendFilters: BackendSearchFilters | undefined = filters
            ? {
                  contentType: filters.contentType,
                  projectId: filters.projectId,
                  timePreset: filters.timePreset === "all" ? null : filters.timePreset,
              }
            : undefined;

        const results = await invoke<TanzouSearchResult[]>("search_sessions", {
            query: query.trim(),
            limit,
            filters: backendFilters,
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
            contentType: r.content_type,
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
        onLoading: (loading: boolean) => void,
        filters?: SearchFilters
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
                const results = await searchSessions(query, 50, filters);
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
