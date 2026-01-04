/**
 * ProjectGroupItem Component - 项目分组卡片
 * Story 2.9 UX Redesign
 * Story 2.20: Import Status Enhancement
 * Story 2.26: 国际化支持
 *
 * 可折叠的项目卡片，使用 Radix Collapsible
 * - 点击复选框选中/取消该项目下所有会话
 * - 点击展开按钮查看会话列表
 * - 默认显示最近 3 个会话
 * - 支持已导入/新项目状态显示
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { ChevronRight, Folder } from "lucide-react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { Checkbox } from "@/components/ui";
import { cn } from "@/lib/utils";
import type { ProjectGroup, ProjectImportStatus, ProjectSelectionState } from "@/types/import";
import { SessionListItem } from "./SessionListItem";

/** 默认显示的会话数量 */
const DEFAULT_VISIBLE_COUNT = 3;

/** ProjectGroupItem Props */
export interface ProjectGroupItemProps {
    /** 项目分组数据 */
    group: ProjectGroup;
    /** 选择状态 */
    selectionState: ProjectSelectionState;
    /** 是否展开 */
    isExpanded: boolean;
    /** 已选文件集合 */
    selectedFiles: Set<string>;
    /** 切换项目选择 */
    onToggleProject: () => void;
    /** 切换展开状态 */
    onToggleExpand: () => void;
    /** 切换单个会话选择 */
    onToggleSession: (path: string) => void;
    /** 导入状态 (Story 2.20) */
    importStatus?: ProjectImportStatus;
}

/**
 * ProjectGroupItem 组件
 */
export function ProjectGroupItem({
    group,
    selectionState,
    isExpanded,
    selectedFiles,
    onToggleProject,
    onToggleExpand,
    onToggleSession,
    importStatus,
}: ProjectGroupItemProps) {
    const { t } = useTranslation();
    const [showAll, setShowAll] = React.useState(false);

    const { isSelected, isPartiallySelected, selectedCount } = selectionState;

    // Story 2.20: 导入状态判断
    const isImported = importStatus === "imported";
    const isNew = importStatus === "new";

    // 显示的会话列表
    const visibleSessions =
        showAll || group.sessions.length <= DEFAULT_VISIBLE_COUNT
            ? group.sessions
            : group.sessions.slice(0, DEFAULT_VISIBLE_COUNT);

    const remainingCount = group.sessions.length - DEFAULT_VISIBLE_COUNT;

    // 处理复选框点击，阻止事件冒泡以避免触发折叠
    const handleCheckboxClick = React.useCallback(
        (e: React.MouseEvent) => {
            e.stopPropagation();
        },
        []
    );

    return (
        <Collapsible.Root
            open={isExpanded}
            onOpenChange={onToggleExpand}
            data-testid={`project-group-${group.projectPath}`}
            className="border-b border-border/50 last:border-b-0"
        >
            {/* 项目头部 - 整行可点击展开/折叠 */}
            <Collapsible.Trigger asChild>
                <div
                    className={cn(
                        "flex items-center gap-3 px-3 py-3 w-full",
                        "cursor-pointer hover:bg-muted/30 transition-colors",
                        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
                    )}
                    role="button"
                    tabIndex={0}
                    aria-expanded={isExpanded}
                    aria-label={isExpanded ? t("common.collapse") : t("common.expand")}
                    onKeyDown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                        }
                    }}
                >
                    {/* 展开/折叠箭头 */}
                    <ChevronRight
                        className={cn(
                            "w-4 h-4 text-muted-foreground transition-transform duration-200 shrink-0",
                            isExpanded && "rotate-90"
                        )}
                    />

                    {/* 项目复选框 - 独立点击区域 */}
                    <div onClick={handleCheckboxClick}>
                        <Checkbox
                            data-testid={`project-checkbox-${group.projectPath}`}
                            checked={isSelected}
                            disabled={isImported}
                            data-state={
                                isSelected
                                    ? "checked"
                                    : isPartiallySelected
                                        ? "indeterminate"
                                        : "unchecked"
                            }
                            onCheckedChange={onToggleProject}
                            aria-label={t("import.selectProject", { name: group.projectName })}
                            className={cn(
                                "cursor-pointer",
                                isImported && "cursor-not-allowed opacity-50"
                            )}
                        />
                    </div>

                    {/* 项目图标和名称 */}
                    <Folder className={cn(
                        "w-4 h-4 shrink-0",
                        isImported ? "text-muted-foreground" : "text-primary"
                    )} />
                    <span
                        className={cn(
                            "text-sm font-medium flex-1 truncate text-left",
                            isImported ? "text-muted-foreground" : "text-foreground"
                        )}
                        title={group.projectName}
                    >
                        {group.projectName}
                    </span>

                    {/* Story 2.20: 导入状态标签 */}
                    {isImported && (
                        <span
                            data-testid={`import-badge-imported-${group.projectPath}`}
                            className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded shrink-0"
                        >
                            {t("import.imported")}
                        </span>
                    )}
                    {isNew && (
                        <span
                            data-testid={`import-badge-new-${group.projectPath}`}
                            className="text-xs text-emerald-500 bg-emerald-500/10 px-1.5 py-0.5 rounded font-medium shrink-0"
                        >
                            NEW
                        </span>
                    )}

                    {/* 会话统计 */}
                    <span className="text-xs text-muted-foreground shrink-0">
                        {selectedCount > 0 && selectedCount < group.sessions.length
                            ? `${selectedCount}/`
                            : ""}
                        {t("import.sessionCount", { count: group.sessions.length })}
                    </span>
                </div>
            </Collapsible.Trigger>

            {/* 会话列表 */}
            <Collapsible.Content className="overflow-hidden data-[state=open]:animate-slideDown data-[state=closed]:animate-slideUp">
                <div className="bg-muted/20">
                    {visibleSessions.map((session) => (
                        <SessionListItem
                            key={session.path}
                            session={session}
                            selected={selectedFiles.has(session.path)}
                            onToggle={() => onToggleSession(session.path)}
                            disabled={isImported}
                        />
                    ))}

                    {/* 显示更多按钮 */}
                    {!showAll && remainingCount > 0 && (
                        <button
                            type="button"
                            onClick={() => setShowAll(true)}
                            className="w-full py-2 text-xs text-primary hover:text-primary/80 hover:bg-muted/50 transition-colors"
                        >
                            {t("import.showMore", { count: remainingCount })}
                        </button>
                    )}
                </div>
            </Collapsible.Content>
        </Collapsible.Root>
    );
}
