/**
 * ProjectGroupItem Component - 项目分组卡片
 * Story 2.9 UX Redesign
 *
 * 可折叠的项目卡片，使用 Radix Collapsible
 * - 点击复选框选中/取消该项目下所有会话
 * - 点击展开按钮查看会话列表
 * - 默认显示最近 3 个会话
 */

import * as React from "react";
import { ChevronRight, Folder } from "lucide-react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { Checkbox } from "@/components/ui";
import { cn } from "@/lib/utils";
import type { ProjectGroup, ProjectSelectionState } from "@/types/import";
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
}: ProjectGroupItemProps) {
    const [showAll, setShowAll] = React.useState(false);

    const { isSelected, isPartiallySelected, selectedCount } = selectionState;

    // 显示的会话列表
    const visibleSessions =
        showAll || group.sessions.length <= DEFAULT_VISIBLE_COUNT
            ? group.sessions
            : group.sessions.slice(0, DEFAULT_VISIBLE_COUNT);

    const remainingCount = group.sessions.length - DEFAULT_VISIBLE_COUNT;

    return (
        <Collapsible.Root
            open={isExpanded}
            onOpenChange={onToggleExpand}
            data-testid={`project-group-${group.projectPath}`}
            className="border-b border-border/50 last:border-b-0"
        >
            {/* 项目头部 */}
            <div className="flex items-center gap-3 px-3 py-3 hover:bg-muted/30 transition-colors">
                {/* 展开/折叠按钮 */}
                <Collapsible.Trigger asChild>
                    <button
                        type="button"
                        className="p-1 rounded hover:bg-muted transition-colors"
                        aria-label={isExpanded ? "折叠" : "展开"}
                    >
                        <ChevronRight
                            className={cn(
                                "w-4 h-4 text-muted-foreground transition-transform duration-200",
                                isExpanded && "rotate-90"
                            )}
                        />
                    </button>
                </Collapsible.Trigger>

                {/* 项目复选框 */}
                <Checkbox
                    data-testid={`project-checkbox-${group.projectPath}`}
                    checked={isSelected}
                    data-state={
                        isSelected
                            ? "checked"
                            : isPartiallySelected
                                ? "indeterminate"
                                : "unchecked"
                    }
                    onCheckedChange={onToggleProject}
                    aria-label={`选择项目 ${group.projectName}`}
                />

                {/* 项目图标和名称 */}
                <Folder className="w-4 h-4 text-primary shrink-0" />
                <span className="text-sm font-medium text-foreground flex-1 truncate">
                    {group.projectName}
                </span>

                {/* 会话统计 */}
                <span className="text-xs text-muted-foreground shrink-0">
                    {selectedCount > 0 && selectedCount < group.sessions.length
                        ? `${selectedCount}/`
                        : ""}
                    {group.sessions.length} 个会话
                </span>
            </div>

            {/* 会话列表 */}
            <Collapsible.Content className="overflow-hidden data-[state=open]:animate-slideDown data-[state=closed]:animate-slideUp">
                <div className="bg-muted/20">
                    {visibleSessions.map((session) => (
                        <SessionListItem
                            key={session.path}
                            session={session}
                            selected={selectedFiles.has(session.path)}
                            onToggle={() => onToggleSession(session.path)}
                        />
                    ))}

                    {/* 显示更多按钮 */}
                    {!showAll && remainingCount > 0 && (
                        <button
                            type="button"
                            onClick={() => setShowAll(true)}
                            className="w-full py-2 text-xs text-primary hover:text-primary/80 hover:bg-muted/50 transition-colors"
                        >
                            显示更多 +{remainingCount}
                        </button>
                    )}
                </div>
            </Collapsible.Content>
        </Collapsible.Root>
    );
}
