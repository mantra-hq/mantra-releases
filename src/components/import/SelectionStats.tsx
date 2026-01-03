/**
 * SelectionStats Component - 选择统计栏
 * Story 2.9 UX Redesign
 * Story 2.20: Import Status Enhancement
 *
 * 显示统计信息和批量操作按钮
 */

import { Folder, FileJson, CheckSquare, Import } from "lucide-react";
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
    /** 反选回调 */
    onInvertSelection: () => void;
    /** 已导入项目数 (Story 2.20) */
    importedCount?: number;
    /** 新项目总数 (Story 2.20) */
    newProjectCount?: number;
    /** 全选新项目回调 (Story 2.20) */
    onSelectAllNew?: () => void;
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
    onInvertSelection,
    importedCount,
    newProjectCount,
    onSelectAllNew,
}: SelectionStatsProps) {
    // Story 2.20: 判断是否有导入状态区分
    const hasImportStatus = importedCount !== undefined && newProjectCount !== undefined;

    // 计算全选状态（只针对新项目的会话）
    const allSelected = selectedCount === totalSessions && totalSessions > 0;
    const noneSelected = selectedCount === 0;

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
                    已选 {selectedCount} 个{hasImportStatus && "新项目"}
                </span>
                {/* Story 2.20: 已导入统计 */}
                {hasImportStatus && importedCount > 0 && (
                    <span
                        className="flex items-center gap-1.5"
                        data-testid="imported-count-stat"
                    >
                        <Import className="w-4 h-4" />
                        {importedCount} 个已导入
                    </span>
                )}
            </div>

            {/* 批量操作按钮 */}
            <div className="flex items-center gap-1">
                {/* Story 2.20: 全选新项目按钮 */}
                {hasImportStatus && onSelectAllNew ? (
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={onSelectAllNew}
                        disabled={newProjectCount === 0}
                        className="text-xs h-7 px-2"
                        data-testid="select-all-new-button"
                    >
                        全选新项目
                    </Button>
                ) : (
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={onSelectAll}
                        disabled={allSelected}
                        className="text-xs h-7 px-2"
                    >
                        全选
                    </Button>
                )}
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onClearAll}
                    disabled={noneSelected}
                    className="text-xs h-7 px-2"
                >
                    清空
                </Button>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onInvertSelection}
                    disabled={totalSessions === 0}
                    className="text-xs h-7 px-2"
                >
                    反选
                </Button>
            </div>
        </div>
    );
}
