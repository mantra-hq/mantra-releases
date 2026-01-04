/**
 * SelectionStats Component - 选择统计栏（紧凑版）
 * Story 2.9 UX Redesign
 * Story 2.20: Import Status Enhancement
 * Story 2.24: 布局优化 - 紧凑统计，批量按钮移至 Footer
 * Story 2.26: 国际化支持
 *
 * 显示统计信息（批量操作按钮已移至 Footer）
 */

import { useTranslation } from "react-i18next";
import { Folder, FileJson, Import } from "lucide-react";

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
    /** 已导入项目数 (Story 2.20) */
    importedCount?: number;
}

/**
 * SelectionStats 组件（紧凑版）
 * 显示格式：X 个项目・已选 Y
 */
export function SelectionStats({
    totalProjects,
    totalSessions,
    selectedCount,
    selectedProjectCount,
    importedCount,
}: SelectionStatsProps) {
    const { t } = useTranslation();

    return (
        <div className="flex items-center gap-4 px-3 py-1.5 text-sm text-muted-foreground bg-muted/30 rounded-md">
            {/* 项目统计 */}
            <span className="flex items-center gap-1.5">
                <Folder className="w-3.5 h-3.5" />
                <span>{t("import.projectCount", { count: totalProjects })}</span>
                <span className="text-border">・</span>
                <span className="text-primary font-medium">{t("import.selected", { count: selectedProjectCount })}</span>
            </span>
            {/* 会话统计 */}
            <span className="flex items-center gap-1.5">
                <FileJson className="w-3.5 h-3.5" />
                <span>{t("import.sessionCount", { count: totalSessions })}</span>
                <span className="text-border">・</span>
                <span className="text-primary font-medium">{t("import.selected", { count: selectedCount })}</span>
            </span>
            {/* Story 2.20: 已导入统计 */}
            {importedCount !== undefined && importedCount > 0 && (
                <span
                    className="flex items-center gap-1.5"
                    data-testid="imported-count-stat"
                >
                    <Import className="w-3.5 h-3.5" />
                    <span>{t("import.importedCount", { count: importedCount })}</span>
                </span>
            )}
        </div>
    );
}
