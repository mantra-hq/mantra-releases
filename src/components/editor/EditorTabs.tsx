/**
 * EditorTabs - 编辑器标签页组件
 * Story 2.13: Task 2 - AC #1, #2, #3, #4, #5
 *
 * 功能:
 * - 显示打开的文件标签页
 * - 标签切换和关闭
 * - 预览模式斜体样式
 * - 标签溢出滚动
 * - 历史模式指示器 (时钟图标)
 *
 * UX 优化 (方案 B): 纯标签管理，Diff 切换和返回当前已移至 Breadcrumbs
 */

import * as React from "react";
import { X, ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { useEditorStore, type EditorTab } from "@/stores/useEditorStore";
import { Button } from "@/components/ui/button";
import { getFileIcon } from "@/lib/file-icons";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import { SnapshotBadge } from "./SnapshotBadge";

export interface EditorTabsProps {
    /** 自定义类名 */
    className?: string;
}

/**
 * 编辑器标签页组件 (纯标签管理)
 */
export function EditorTabs({ className }: EditorTabsProps) {
    // 使用独立的选择器确保引用稳定
    const tabs = useEditorStore((state) => state.tabs);
    const activeTabId = useEditorStore((state) => state.activeTabId);
    const setActiveTab = useEditorStore((state) => state.setActiveTab);
    const closeTab = useEditorStore((state) => state.closeTab);
    const pinTab = useEditorStore((state) => state.pinTab);
    const scrollContainerRef = React.useRef<HTMLDivElement>(null);
    const [showLeftArrow, setShowLeftArrow] = React.useState(false);
    const [showRightArrow, setShowRightArrow] = React.useState(false);

    // 检测滚动状态
    React.useEffect(() => {
        const container = scrollContainerRef.current;
        if (!container) return;

        const checkScroll = () => {
            setShowLeftArrow(container.scrollLeft > 0);
            setShowRightArrow(
                container.scrollLeft < container.scrollWidth - container.clientWidth - 1
            );
        };

        checkScroll();
        container.addEventListener("scroll", checkScroll);
        window.addEventListener("resize", checkScroll);

        return () => {
            container.removeEventListener("scroll", checkScroll);
            window.removeEventListener("resize", checkScroll);
        };
    }, [tabs.length]);

    const scroll = (direction: "left" | "right") => {
        const container = scrollContainerRef.current;
        if (!container) return;
        const scrollAmount = 200;
        container.scrollBy({
            left: direction === "left" ? -scrollAmount : scrollAmount,
            behavior: "smooth",
        });
    };

    const handleTabClick = (tab: EditorTab, e: React.MouseEvent) => {
        e.stopPropagation();
        setActiveTab(tab.id);
    };

    const handleTabDoubleClick = (tab: EditorTab) => {
        if (tab.isPreview) {
            pinTab(tab.id);
        }
    };

    const handleCloseClick = (tab: EditorTab, e: React.MouseEvent) => {
        e.stopPropagation();
        closeTab(tab.id);
    };

    if (tabs.length === 0) return null;

    return (
        <div
            className={cn(
                "flex items-center border-b border-border bg-muted/30",
                className
            )}
        >
            {/* 左滚动箭头 */}
            {showLeftArrow && (
                <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-6 rounded-none flex-shrink-0"
                    onClick={() => scroll("left")}
                    aria-label="向左滚动"
                >
                    <ChevronLeft className="h-4 w-4" />
                </Button>
            )}

            {/* 标签容器 */}
            <div
                ref={scrollContainerRef}
                role="tablist"
                className="flex-1 flex overflow-x-auto scrollbar-none"
            >
                {tabs.map((tab) => {
                    const Icon = getFileIcon(tab.path);
                    const isActive = tab.id === activeTabId;
                    const isHistorical = !!tab.commitHash;

                    // 格式化历史时间提示
                    const historyTooltip = isHistorical
                        ? tab.timestamp
                            ? `历史版本 @ ${new Date(tab.timestamp).toLocaleString("zh-CN")}`
                            : `历史版本 (${tab.commitHash?.slice(0, 7)})`
                        : undefined;

                    return (
                        <Tooltip key={tab.id}>
                            <TooltipTrigger asChild>
                                <div
                                    data-tab
                                    data-active={isActive}
                                    data-historical={isHistorical}
                                    role="tab"
                                    aria-selected={isActive}
                                    onClick={(e) => handleTabClick(tab, e)}
                                    onDoubleClick={() => handleTabDoubleClick(tab)}
                                    className={cn(
                                        "group flex items-center gap-1.5 px-3 py-1.5 border-r border-border",
                                        "cursor-pointer select-none min-w-0 max-w-[200px]",
                                        "hover:bg-muted/50 transition-colors",
                                        isActive && "bg-background border-b-2 border-b-primary",
                                        tab.isPreview && "italic",
                                        isHistorical && "bg-amber-500/5"
                                    )}
                                >
                                    {/* 历史状态徽章 (会话快照或 Git 历史) */}
                                    {(tab.isSnapshot || isHistorical) && (
                                        <SnapshotBadge
                                            type={tab.isSnapshot ? "snapshot" : "git-history"}
                                            mode="icon"
                                        />
                                    )}
                                    <Icon className="h-4 w-4 flex-shrink-0 text-muted-foreground" />
                                    <span className="truncate text-sm">{tab.label}</span>
                                    {/* 关闭按钮 */}
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        className={cn(
                                            "h-4 w-4 p-0 ml-1 rounded-sm flex-shrink-0",
                                            "opacity-0 group-hover:opacity-100",
                                            isActive && "opacity-100"
                                        )}
                                        onClick={(e) => handleCloseClick(tab, e)}
                                        aria-label="关闭标签"
                                    >
                                        <X className="h-3 w-3" />
                                    </Button>
                                </div>
                            </TooltipTrigger>
                            {historyTooltip && (
                                <TooltipContent side="bottom">
                                    <p>{historyTooltip}</p>
                                </TooltipContent>
                            )}
                        </Tooltip>
                    );
                })}
            </div>

            {/* 右滚动箭头 */}
            {showRightArrow && (
                <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-6 rounded-none flex-shrink-0"
                    onClick={() => scroll("right")}
                    aria-label="向右滚动"
                >
                    <ChevronRight className="h-4 w-4" />
                </Button>
            )}
        </div>
    );
}

export default EditorTabs;




