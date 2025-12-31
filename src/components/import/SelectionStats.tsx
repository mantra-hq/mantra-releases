/**
 * SelectionStats Component - 选择统计栏
 * Story 2.9 UX Redesign
 *
 * 显示统计信息和批量操作按钮
 */

import { Folder, FileJson, CheckSquare } from "lucide-react";
import { Button } from "@/components/ui";

/** SelectionStats Props */
export interface SelectionStatsProps {
    /** 总项目数 */
    totalProjects: number;
    /** 总会话数 */
    totalSessions: number;
    /** 已选会话数 */
    selectedCount: number;
    /** 全选回调 */
    onSelectAll: () => void;
    /** 清空选择回调 */
    onClearAll: () => void;
}

/**
 * SelectionStats 组件
 */
export function SelectionStats({
    totalProjects,
    totalSessions,
    selectedCount,
    onSelectAll,
    onClearAll,
}: SelectionStatsProps) {
    const allSelected = selectedCount === totalSessions && totalSessions > 0;

    return (
        <div className="flex items-center justify-between px-3 py-2 bg-muted/30 border border-border rounded-lg">
            {/* 统计信息 */}
            <div className="flex items-center gap-4 text-sm text-muted-foreground">
                <span className="flex items-center gap-1.5">
                    <Folder className="w-4 h-4" />
                    {totalProjects} 个项目
                </span>
                <span className="flex items-center gap-1.5">
                    <FileJson className="w-4 h-4" />
                    {totalSessions} 个会话
                </span>
                <span className="flex items-center gap-1.5 text-primary">
                    <CheckSquare className="w-4 h-4" />
                    已选 {selectedCount} 个
                </span>
            </div>

            {/* 批量操作按钮 */}
            <div className="flex items-center gap-2">
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={allSelected ? onClearAll : onSelectAll}
                    className="text-xs h-7"
                >
                    {allSelected ? "清空选择" : "全选"}
                </Button>
            </div>
        </div>
    );
}
