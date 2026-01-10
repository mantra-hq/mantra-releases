/**
 * GlobalSearch - 全局搜索 Modal 组件
 * Story 2.10: Task 1
 * Story 2.26: 国际化支持
 *
 * Command Palette 风格的全局搜索框
 * 支持键盘快捷键、实时搜索、键盘导航
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import * as Dialog from "@radix-ui/react-dialog";
import { Search, X, Loader2 } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useSearchStore, type SearchResult, type RecentSession } from "@/stores/useSearchStore";
import { SearchResultList } from "./SearchResultList";
import { EmptySearchState } from "./EmptySearchState";
import { RecentSessions } from "./RecentSessions";
import { createDebouncedSearch } from "@/lib/search-ipc";
import { cn } from "@/lib/utils";

/**
 * 检测操作系统
 */
function isMacOS(): boolean {
    if (typeof navigator === "undefined") return false;
    return navigator.platform.toLowerCase().includes("mac");
}

/**
 * GlobalSearch 组件
 */
export function GlobalSearch() {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const inputRef = React.useRef<HTMLInputElement>(null);

    // Store state
    const isOpen = useSearchStore((state) => state.isOpen);
    const query = useSearchStore((state) => state.query);
    const results = useSearchStore((state) => state.results);
    const isLoading = useSearchStore((state) => state.isLoading);
    const selectedIndex = useSearchStore((state) => state.selectedIndex);
    const recentSessions = useSearchStore((state) => state.recentSessions);

    // Store actions
    const close = useSearchStore((state) => state.close);
    const setQuery = useSearchStore((state) => state.setQuery);
    const setResults = useSearchStore((state) => state.setResults);
    const setLoading = useSearchStore((state) => state.setLoading);
    const selectNext = useSearchStore((state) => state.selectNext);
    const selectPrev = useSearchStore((state) => state.selectPrev);
    const confirm = useSearchStore((state) => state.confirm);
    const addRecentSession = useSearchStore((state) => state.addRecentSession);

    // 创建防抖搜索函数
    const debouncedSearchRef = React.useRef(createDebouncedSearch(300));

    // 聚焦输入框
    React.useEffect(() => {
        if (isOpen && inputRef.current) {
            // 延迟聚焦，等待 Dialog 动画完成
            const timer = setTimeout(() => {
                inputRef.current?.focus();
            }, 50);
            return () => clearTimeout(timer);
        }
    }, [isOpen]);

    // 搜索处理
    React.useEffect(() => {
        const { debouncedSearch, cancel } = debouncedSearchRef.current;
        debouncedSearch(query, setResults, setLoading);

        return () => {
            cancel();
        };
    }, [query, setResults, setLoading]);

    // 输入变化处理
    const handleInputChange = React.useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            setQuery(e.target.value);
        },
        [setQuery]
    );

    // 跳转到会话
    const navigateToSession = React.useCallback(
        (sessionId: string, messageId?: string) => {
            close();
            // 添加到最近会话 (如果有结果信息)
            const targetResult = results.find((r) => r.sessionId === sessionId);
            if (targetResult) {
                addRecentSession({
                    projectId: targetResult.projectId,
                    projectName: targetResult.projectName,
                    sessionId: targetResult.sessionId,
                    sessionName: targetResult.sessionName,
                    accessedAt: Date.now(),
                });
            }
            // 跳转，如果有 messageId 则添加到 URL
            const url = messageId
                ? `/session/${sessionId}?messageId=${messageId}`
                : `/session/${sessionId}`;
            navigate(url);
        },
        [close, navigate, results, addRecentSession]
    );

    // 选择搜索结果
    const handleSelectResult = React.useCallback(
        (result: SearchResult) => {
            navigateToSession(result.sessionId, result.messageId);
        },
        [navigateToSession]
    );

    // 选择最近会话
    const handleSelectRecent = React.useCallback(
        (session: RecentSession) => {
            close();
            addRecentSession({
                ...session,
                accessedAt: Date.now(),
            });
            navigate(`/session/${session.sessionId}`);
        },
        [close, navigate, addRecentSession]
    );

    // 更新 hover 索引 (搜索结果)
    const handleResultHover = React.useCallback(
        (index: number) => {
            useSearchStore.setState({ selectedIndex: index });
        },
        []
    );

    // 键盘导航
    const handleKeyDown = React.useCallback(
        (e: React.KeyboardEvent) => {
            const hasQuery = query.trim().length > 0;
            const itemCount = hasQuery ? results.length : recentSessions.length;

            switch (e.key) {
                case "ArrowDown":
                    e.preventDefault();
                    if (selectedIndex < itemCount - 1) {
                        selectNext();
                    }
                    break;
                case "ArrowUp":
                    e.preventDefault();
                    selectPrev();
                    break;
                case "Enter":
                    e.preventDefault();
                    if (hasQuery) {
                        const result = confirm();
                        if (result) {
                            navigateToSession(result.sessionId, result.messageId);
                        }
                    } else if (recentSessions[selectedIndex]) {
                        handleSelectRecent(recentSessions[selectedIndex]);
                    }
                    break;
                case "Escape":
                    e.preventDefault();
                    close();
                    break;
            }
        },
        [
            query,
            results.length,
            recentSessions,
            selectedIndex,
            selectNext,
            selectPrev,
            confirm,
            navigateToSession,
            handleSelectRecent,
            close,
        ]
    );

    // 决定显示内容
    const hasQuery = query.trim().length > 0;
    const showResults = hasQuery && results.length > 0;
    const showEmpty = hasQuery && !isLoading && results.length === 0;
    const showRecent = !hasQuery;

    return (
        <Dialog.Root open={isOpen} onOpenChange={(open) => !open && close()}>
            <Dialog.Portal>
                {/* Overlay */}
                <Dialog.Overlay
                    className={cn(
                        "fixed inset-0 z-50",
                        "bg-black/50 backdrop-blur-sm",
                        "data-[state=open]:animate-in data-[state=closed]:animate-out",
                        "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
                    )}
                />

                {/* Content */}
                <Dialog.Content
                    data-testid="global-search"
                    aria-label={t("search.globalSearch")}
                    onKeyDown={handleKeyDown}
                    className={cn(
                        "fixed left-1/2 top-[15%] z-50 -translate-x-1/2",
                        "w-[640px] max-w-[90vw] max-h-[70vh]",
                        "bg-background border border-border rounded-xl shadow-2xl",
                        "overflow-hidden flex flex-col",
                        "data-[state=open]:animate-in data-[state=closed]:animate-out",
                        "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
                        "data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
                        "data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%]",
                        "data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%]",
                        "duration-200"
                    )}
                >
                    {/* Hidden Title for Accessibility */}
                    <Dialog.Title className="sr-only">{t("search.globalSearch")}</Dialog.Title>
                    <Dialog.Description className="sr-only">
                        {t("search.description")}
                    </Dialog.Description>

                    {/* Search Input */}
                    <div className="flex items-center gap-3 px-4 py-3 border-b border-border">
                        <Search className="w-5 h-5 text-muted-foreground shrink-0" />
                        <input
                            ref={inputRef}
                            data-testid="search-input"
                            type="text"
                            value={query}
                            onChange={handleInputChange}
                            placeholder={t("search.placeholder")}
                            aria-label={t("common.search")}
                            className={cn(
                                "flex-1 bg-transparent border-none outline-none",
                                "text-base text-foreground placeholder:text-muted-foreground"
                            )}
                        />
                        {isLoading && (
                            <Loader2 className="w-4 h-4 text-muted-foreground animate-spin shrink-0" />
                        )}
                        {/* 快捷键提示 */}
                        <div className="flex items-center gap-1 text-xs text-muted-foreground shrink-0">
                            <kbd className="px-1.5 py-0.5 bg-muted rounded text-[11px] font-mono">
                                {isMacOS() ? "⌘" : "Ctrl"}
                            </kbd>
                            <kbd className="px-1.5 py-0.5 bg-muted rounded text-[11px] font-mono">K</kbd>
                        </div>
                        <Dialog.Close asChild>
                            <button
                                type="button"
                                aria-label={t("search.closeSearch")}
                                className="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </Dialog.Close>
                    </div>

                    {/* Results Area */}
                    <div data-testid="search-results" className="flex-1 overflow-hidden">
                        {/* Loading skeleton */}
                        {isLoading && hasQuery && results.length === 0 && (
                            <div className="p-4 space-y-3">
                                {[1, 2, 3].map((i) => (
                                    <div key={i} className="animate-pulse">
                                        <div className="h-4 bg-muted rounded w-1/3 mb-2" />
                                        <div className="h-3 bg-muted rounded w-2/3" />
                                    </div>
                                ))}
                            </div>
                        )}

                        {/* Search Results */}
                        {showResults && (
                            <SearchResultList
                                results={results}
                                selectedIndex={selectedIndex}
                                onSelect={handleSelectResult}
                                onHover={handleResultHover}
                            />
                        )}

                        {/* Empty State */}
                        {showEmpty && <EmptySearchState query={query} />}

                        {/* Recent Sessions */}
                        {showRecent && (
                            <RecentSessions
                                sessions={recentSessions}
                                selectedIndex={selectedIndex}
                                onSelect={handleSelectRecent}
                                onHover={handleResultHover}
                            />
                        )}
                    </div>

                    {/* Footer */}
                    <div className="px-4 py-2 border-t border-border bg-muted/30 flex items-center justify-between text-xs text-muted-foreground">
                        <div className="flex items-center gap-4">
                            <span className="flex items-center gap-1">
                                <kbd className="px-1 py-0.5 bg-muted rounded text-[10px]">↑</kbd>
                                <kbd className="px-1 py-0.5 bg-muted rounded text-[10px]">↓</kbd>
                                <span className="ml-1">{t("search.navigate")}</span>
                            </span>
                            <span className="flex items-center gap-1">
                                <kbd className="px-1 py-0.5 bg-muted rounded text-[10px]">↵</kbd>
                                <span className="ml-1">{t("search.select")}</span>
                            </span>
                            <span className="flex items-center gap-1">
                                <kbd className="px-1 py-0.5 bg-muted rounded text-[10px]">Esc</kbd>
                                <span className="ml-1">{t("common.close")}</span>
                            </span>
                        </div>
                        {hasQuery && results.length > 0 && (
                            <span>{t("search.resultsCount", { count: results.length })}</span>
                        )}
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    );
}

export default GlobalSearch;
