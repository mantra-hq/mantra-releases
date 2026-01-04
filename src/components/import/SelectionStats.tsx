/**
 * SelectionStats Component - 选择统计栏
 * Story 2.9 UX Redesign
 * Story 2.20: Import Status Enhancement
 * Story 2.26: 国际化支持
 *
 * 显示统计信息和批量操作按钮
 */

import { useTranslation } from "react-i18next";
import { Folder, FileJson, Import } from "lucide-react";
import { Button } from "@/components/ui";

/** SelectionStats Props */
export interface SelectionStatsProps {
    /** 总项目数 */
    totalProjects: number;
    /** 总会话数 */
    totalSessions: number;
    /** 已选会话数 */
    selectedCount: number;
    /** 已选项目数 (Story 2.24 AC2) */
    selectedProjectCount: number;
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
    selectedProjectCount,
    onSelectAll,
    onClearAll,
    onInvertSelection,
    importedCount,
    newProjectCount,
    onSelectAllNew,
}: SelectionStatsProps) {
    const { t } = useTranslation();
    // Story 2.20: 判断是否有导入状态区分
    const hasImportStatus = importedCount !== undefined && newProjectCount !== undefined;

    // 计算全选状态（只针对新项目的会话）
    const allSelected = selectedCount === totalSessions && totalSessions > 0;
    const noneSelected = selectedCount === 0;

    return (
        <div className="flex items-center justify-between px-3 py-2 bg-muted/30 border border-border rounded-lg">
            {/* Story 2.24 AC2: 统计信息 - 分别显示项目和会话的已选数 */}
            <div className="flex items-center gap-4 text-sm text-muted-foreground whitespace-nowrap">
                <span className="flex items-center gap-1.5">
                    <Folder className="w-4 h-4" />
                    {t("import.projectCount", { count: totalProjects })}
                    <span className="text-primary">{t("import.selected", { count: selectedProjectCount })}</span>
                </span>
                <span className="text-border">|</span>
                <span className="flex items-center gap-1.5">
                    <FileJson className="w-4 h-4" />
                    {t("import.sessionCount", { count: totalSessions })}
                    <span className="text-primary">{t("import.selected", { count: selectedCount })}</span>
                </span>
                {/* Story 2.20: 已导入统计 */}
                {hasImportStatus && importedCount > 0 && (
                    <span
                        className="flex items-center gap-1.5"
                        data-testid="imported-count-stat"
                    >
                        <Import className="w-4 h-4" />
                        {t("import.importedCount", { count: importedCount })}
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
                        {t("import.selectAllNew")}
                    </Button>
                ) : (
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={onSelectAll}
                        disabled={allSelected}
                        className="text-xs h-7 px-2"
                    >
                        {t("import.selectAll")}
                    </Button>
                )}
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onClearAll}
                    disabled={noneSelected}
                    className="text-xs h-7 px-2"
                >
                    {t("import.clearSelection")}
                </Button>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onInvertSelection}
                    disabled={totalSessions === 0}
                    className="text-xs h-7 px-2"
                >
                    {t("import.invertSelection")}
                </Button>
            </div>
        </div>
    );
}
