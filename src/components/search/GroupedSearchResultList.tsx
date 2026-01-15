/**
 * GroupedSearchResultList - 按项目分组的搜索结果列表
 * Story 2.33: Task 6
 *
 * AC4: 搜索结果按项目分组显示
 * - 每组显示项目名称作为分组标题
 * - 分组可折叠/展开
 * - 显示每组结果数量
 * - 虚拟滚动保持流畅
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ChevronDown, ChevronRight, FolderOpen } from "lucide-react";
import { cn, formatSessionName } from "@/lib/utils";
import type { SearchResult } from "@/stores/useSearchStore";

export interface GroupedSearchResultListProps {
    /** 搜索结果列表 */
    results: SearchResult[];
    /** 当前选中的结果索引 */
    selectedIndex: number;
    /** 选择结果回调 */
    onSelect: (result: SearchResult) => void;
    /** hover 事件回调 */
    onHover: (index: number) => void;
}

/**
 * 按项目分组结果
 */
interface ProjectGroup {
    projectId: string;
    projectName: string;
    results: SearchResult[];
    isExpanded: boolean;
}

/**
 * 高亮文本的渲染函数
 */
function renderHighlightedText(
    text: string,
    ranges: Array<[number, number]>
): React.ReactNode {
    if (!ranges || ranges.length === 0) {
        return text;
    }

    const sortedRanges = [...ranges].sort((a, b) => a[0] - b[0]);
    const parts: React.ReactNode[] = [];
    let lastIndex = 0;

    sortedRanges.forEach(([start, end], i) => {
        if (start > lastIndex) {
            parts.push(
                <span key={`text-${i}`}>{text.slice(lastIndex, start)}</span>
            );
        }
        parts.push(
            <span
                key={`highlight-${i}`}
                className="bg-primary/20 text-primary rounded px-0.5"
            >
                {text.slice(start, end)}
            </span>
        );
        lastIndex = end;
    });

    if (lastIndex < text.length) {
        parts.push(<span key="text-end">{text.slice(lastIndex)}</span>);
    }

    return parts;
}

/**
 * 格式化时间戳
 */
function formatTimestamp(
    timestamp: number,
    locale: string,
    t: (key: string, options?: Record<string, unknown>) => string
): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
        return date.toLocaleTimeString(locale, {
            hour: "2-digit",
            minute: "2-digit",
        });
    } else if (diffDays === 1) {
        return t("time.yesterday");
    } else if (diffDays < 7) {
        return t("time.daysAgo", { count: diffDays });
    } else {
        return date.toLocaleDateString(locale, {
            month: "short",
            day: "numeric",
        });
    }
}

/**
 * 按项目分组搜索结果
 */
function groupResultsByProject(results: SearchResult[]): ProjectGroup[] {
    const groupMap = new Map<string, ProjectGroup>();

    results.forEach((result) => {
        if (!groupMap.has(result.projectId)) {
            groupMap.set(result.projectId, {
                projectId: result.projectId,
                projectName: result.projectName,
                results: [],
                isExpanded: true,
            });
        }
        groupMap.get(result.projectId)!.results.push(result);
    });

    return Array.from(groupMap.values());
}

/**
 * 分组结果列表组件
 */
export function GroupedSearchResultList({
    results,
    selectedIndex,
    onSelect,
    onHover,
}: GroupedSearchResultListProps) {
    const { t, i18n } = useTranslation();
    const parentRef = React.useRef<HTMLDivElement>(null);

    // 分组状态
    const [expandedGroups, setExpandedGroups] = React.useState<Set<string>>(
        new Set()
    );

    // 初始化时展开所有组
    React.useEffect(() => {
        const groups = groupResultsByProject(results);
        setExpandedGroups(new Set(groups.map((g) => g.projectId)));
    }, [results]);

    // 计算分组
    const groups = React.useMemo(() => groupResultsByProject(results), [results]);

    // 构建扁平化的可视项目列表（包含组头和结果项）
    type FlatItem =
        | { type: "header"; group: ProjectGroup }
        | { type: "result"; result: SearchResult; globalIndex: number };

    const flatItems = React.useMemo(() => {
        const items: FlatItem[] = [];
        let globalIndex = 0;

        groups.forEach((group) => {
            items.push({ type: "header", group });
            if (expandedGroups.has(group.projectId)) {
                group.results.forEach((result) => {
                    items.push({ type: "result", result, globalIndex });
                    globalIndex++;
                });
            } else {
                globalIndex += group.results.length;
            }
        });

        return items;
    }, [groups, expandedGroups]);

    // 虚拟化
    const virtualizer = useVirtualizer({
        count: flatItems.length,
        getScrollElement: () => parentRef.current,
        estimateSize: (index) => {
            const item = flatItems[index];
            return item.type === "header" ? 36 : 72;
        },
        overscan: 5,
    });

    // 切换组展开状态
    const toggleGroup = (projectId: string) => {
        setExpandedGroups((prev) => {
            const next = new Set(prev);
            if (next.has(projectId)) {
                next.delete(projectId);
            } else {
                next.add(projectId);
            }
            return next;
        });
    };

    return (
        <div
            ref={parentRef}
            className="flex-1 overflow-y-auto"
            style={{ maxHeight: "calc(80vh - 180px)" }}
        >
            <div
                style={{
                    height: `${virtualizer.getTotalSize()}px`,
                    width: "100%",
                    position: "relative",
                }}
            >
                {virtualizer.getVirtualItems().map((virtualRow) => {
                    const item = flatItems[virtualRow.index];

                    if (item.type === "header") {
                        const { group } = item;
                        const isExpanded = expandedGroups.has(group.projectId);

                        return (
                            <div
                                key={`header-${group.projectId}`}
                                style={{
                                    position: "absolute",
                                    top: 0,
                                    left: 0,
                                    width: "100%",
                                    height: `${virtualRow.size}px`,
                                    transform: `translateY(${virtualRow.start}px)`,
                                }}
                            >
                                <button
                                    type="button"
                                    onClick={() => toggleGroup(group.projectId)}
                                    className={cn(
                                        "flex items-center gap-2 w-full px-4 py-2",
                                        "bg-muted/50 hover:bg-muted transition-colors",
                                        "text-sm font-medium text-muted-foreground"
                                    )}
                                >
                                    {isExpanded ? (
                                        <ChevronDown className="w-4 h-4" />
                                    ) : (
                                        <ChevronRight className="w-4 h-4" />
                                    )}
                                    <FolderOpen className="w-4 h-4" />
                                    <span className="truncate flex-1 text-left">
                                        {group.projectName}
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        ({group.results.length})
                                    </span>
                                </button>
                            </div>
                        );
                    }

                    // Result item
                    const { result, globalIndex } = item;
                    const isSelected = globalIndex === selectedIndex;
                    const displaySessionName = formatSessionName(
                        result.sessionId,
                        result.sessionName
                    );

                    return (
                        <div
                            key={result.id}
                            style={{
                                position: "absolute",
                                top: 0,
                                left: 0,
                                width: "100%",
                                height: `${virtualRow.size}px`,
                                transform: `translateY(${virtualRow.start}px)`,
                            }}
                        >
                            <div
                                role="option"
                                aria-selected={isSelected}
                                onClick={() => onSelect(result)}
                                onMouseEnter={() => onHover(globalIndex)}
                                className={cn(
                                    "flex flex-col gap-1 px-4 py-2 pl-10 cursor-pointer transition-colors h-full",
                                    isSelected ? "bg-primary/10" : "hover:bg-accent"
                                )}
                            >
                                {/* Header: Session / Time */}
                                <div className="flex items-center gap-2 text-sm">
                                    <span
                                        className="text-foreground truncate flex-1"
                                        title={displaySessionName}
                                    >
                                        {displaySessionName}
                                    </span>
                                    <span className="text-xs text-muted-foreground shrink-0">
                                        {formatTimestamp(result.timestamp, i18n.language, t)}
                                    </span>
                                </div>

                                {/* Snippet */}
                                <div className="text-sm text-muted-foreground line-clamp-2 leading-relaxed">
                                    {renderHighlightedText(
                                        result.snippet,
                                        result.highlightRanges
                                    )}
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

export default GroupedSearchResultList;
