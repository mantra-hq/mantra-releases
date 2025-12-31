/**
 * ProjectGroupList Component - 项目分组列表
 * Story 2.9 UX Redesign
 *
 * 项目分组列表，支持大量项目时的流畅滚动
 */

import type { ProjectGroup } from "@/types/import";
import { getProjectSelectionState } from "@/lib/import-utils";
import { ProjectGroupItem } from "./ProjectGroupItem";

/** ProjectGroupList Props */
export interface ProjectGroupListProps {
    /** 项目分组列表 */
    groups: ProjectGroup[];
    /** 已选文件集合 */
    selectedFiles: Set<string>;
    /** 展开的项目集合 */
    expandedProjects: Set<string>;
    /** 切换项目选择 */
    onToggleProject: (projectPath: string) => void;
    /** 切换项目展开 */
    onToggleExpand: (projectPath: string) => void;
    /** 切换单个会话选择 */
    onToggleSession: (filePath: string) => void;
}

/**
 * ProjectGroupList 组件
 */
export function ProjectGroupList({
    groups,
    selectedFiles,
    expandedProjects,
    onToggleProject,
    onToggleExpand,
    onToggleSession,
}: ProjectGroupListProps) {
    if (groups.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                <span className="text-sm">没有找到匹配的项目</span>
            </div>
        );
    }

    return (
        <div
            className="max-h-[calc(70vh-280px)] min-h-[200px] overflow-y-auto border border-border rounded-lg"
            data-testid="project-group-list"
        >
            {groups.map((group) => {
                const selectionState = getProjectSelectionState(group, selectedFiles);

                return (
                    <ProjectGroupItem
                        key={group.projectPath}
                        group={group}
                        selectionState={selectionState}
                        isExpanded={expandedProjects.has(group.projectPath)}
                        selectedFiles={selectedFiles}
                        onToggleProject={() => onToggleProject(group.projectPath)}
                        onToggleExpand={() => onToggleExpand(group.projectPath)}
                        onToggleSession={onToggleSession}
                    />
                );
            })}
        </div>
    );
}
