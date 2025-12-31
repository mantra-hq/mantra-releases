/**
 * QuickOpen - 快速打开命令面板
 * Story 2.13: Task 6 - AC #15
 *
 * 功能:
 * - 模糊搜索文件名
 * - 键盘上下选择 + 回车确认
 * - 文件列表展示
 */

import * as React from "react";
import { Search, File, FileCode, Folder } from "lucide-react";
import { cn } from "@/lib/utils";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { getFileIcon } from "@/lib/file-icons";

/**
 * 防抖 Hook
 * @param value - 要防抖的值
 * @param delay - 延迟时间 (ms)
 */
function useDebouncedValue<T>(value: T, delay: number): T {
    const [debouncedValue, setDebouncedValue] = React.useState(value);

    React.useEffect(() => {
        const timer = setTimeout(() => {
            setDebouncedValue(value);
        }, delay);

        return () => clearTimeout(timer);
    }, [value, delay]);

    return debouncedValue;
}

export interface QuickOpenProps {
    /** 是否打开 */
    open: boolean;
    /** 打开状态变化回调 */
    onOpenChange: (open: boolean) => void;
    /** 文件路径列表 */
    files: string[];
    /** 选择文件回调 */
    onSelect: (path: string) => void;
    /** 加载中状态 */
    loading?: boolean;
}

/**
 * 快速打开命令面板
 */
export function QuickOpen({
    open,
    onOpenChange,
    files,
    onSelect,
    loading = false,
}: QuickOpenProps) {
    const [query, setQuery] = React.useState("");
    const [selectedIndex, setSelectedIndex] = React.useState(0);
    const inputRef = React.useRef<HTMLInputElement>(null);
    const listRef = React.useRef<HTMLDivElement>(null);

    // 防抖搜索 (150ms) - 优化大型仓库性能
    const debouncedQuery = useDebouncedValue(query, 150);

    // 模糊搜索过滤 (使用防抖后的 query)
    const filteredFiles = React.useMemo(() => {
        if (!debouncedQuery) return files.slice(0, 100); // 限制初始显示
        const lowerQuery = debouncedQuery.toLowerCase();
        return files
            .filter((f) => f.toLowerCase().includes(lowerQuery))
            .slice(0, 50);
    }, [files, debouncedQuery]);

    // 重置选中索引当过滤结果变化时
    React.useEffect(() => {
        setSelectedIndex(0);
    }, [filteredFiles]);

    // 打开时聚焦输入框
    React.useEffect(() => {
        if (open) {
            setQuery("");
            setSelectedIndex(0);
            // 延迟聚焦以确保 Dialog 已经渲染
            setTimeout(() => inputRef.current?.focus(), 50);
        }
    }, [open]);

    // 滚动选中项到可见区域
    React.useEffect(() => {
        const list = listRef.current;
        if (!list) return;

        const selectedItem = list.querySelector(`[data-index="${selectedIndex}"]`);
        if (selectedItem) {
            selectedItem.scrollIntoView({ block: "nearest" });
        }
    }, [selectedIndex]);

    const handleSelect = (path: string) => {
        onSelect(path);
        onOpenChange(false);
        setQuery("");
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        switch (e.key) {
            case "ArrowDown":
                e.preventDefault();
                setSelectedIndex((i) => Math.min(i + 1, filteredFiles.length - 1));
                break;
            case "ArrowUp":
                e.preventDefault();
                setSelectedIndex((i) => Math.max(i - 1, 0));
                break;
            case "Enter":
                e.preventDefault();
                if (filteredFiles[selectedIndex]) {
                    handleSelect(filteredFiles[selectedIndex]);
                }
                break;
            case "Escape":
                e.preventDefault();
                onOpenChange(false);
                break;
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="p-0 max-w-2xl gap-0" onKeyDown={handleKeyDown} showCloseButton={false} aria-describedby={undefined}>
                {/* 隐藏的标题 (无障碍) */}
                <DialogTitle className="sr-only">快速打开文件</DialogTitle>
                {/* 搜索输入 */}
                <div className="flex items-center border-b border-border px-3">
                    <Search className="h-4 w-4 text-muted-foreground shrink-0" />
                    <Input
                        ref={inputRef}
                        placeholder="输入文件名搜索..."
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        className="border-0 focus-visible:ring-0 focus-visible:ring-offset-0"
                    />
                </div>

                {/* 文件列表 */}
                <ScrollArea className="max-h-[400px]">
                    <div ref={listRef} className="py-1">
                        {loading ? (
                            <div className="px-3 py-8 text-center text-muted-foreground">
                                加载文件列表...
                            </div>
                        ) : filteredFiles.length === 0 ? (
                            <div className="px-3 py-8 text-center text-muted-foreground">
                                未找到匹配的文件
                            </div>
                        ) : (
                            filteredFiles.map((file, index) => {
                                const Icon = getFileIcon(file);
                                const isSelected = index === selectedIndex;

                                return (
                                    <div
                                        key={file}
                                        data-index={index}
                                        onClick={() => handleSelect(file)}
                                        className={cn(
                                            "flex items-center gap-2 px-3 py-2 cursor-pointer",
                                            "hover:bg-muted/50 transition-colors",
                                            isSelected && "bg-accent text-accent-foreground"
                                        )}
                                    >
                                        <Icon className="h-4 w-4 text-muted-foreground shrink-0" />
                                        <span className="truncate text-sm">{file}</span>
                                    </div>
                                );
                            })
                        )}
                    </div>
                </ScrollArea>

                {/* 底部提示 */}
                <div className="border-t border-border px-3 py-2 text-xs text-muted-foreground flex items-center gap-4">
                    <span>
                        <kbd className="px-1 bg-muted rounded">↑↓</kbd> 选择
                    </span>
                    <span>
                        <kbd className="px-1 bg-muted rounded">Enter</kbd> 打开
                    </span>
                    <span>
                        <kbd className="px-1 bg-muted rounded">Esc</kbd> 关闭
                    </span>
                </div>
            </DialogContent>
        </Dialog>
    );
}

export default QuickOpen;

