/**
 * ProjectGroupList Component - 项目分组列表
 * Story 2.9 UX Redesign
 * Story 2.20: Import Status Enhancement (改进：使用 sessionId 匹配)
 * Story 2.26: 国际化支持
 *
 * 项目分组列表，支持大量项目时的流畅滚动
 */

import { useTranslation } from "react-i18next";
import type { ProjectGroup, ProjectImportStatus } from "@/types/import";
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
    /** 已导入会话 ID 集合 (Story 2.20 改进) */
    importedSessionIds?: Set<string>;
}

/**
 * 计算项目的导入状态
 * Story 2.20 改进: 基于 sessionId 匹配
 */
function getImportStatus(
    group: ProjectGroup,
    importedSessionIds?: Set<string>
): ProjectImportStatus | undefined {
    if (!importedSessionIds) return undefined;

    // 检查每个会话的导入状态
    let importedCount = 0;
    let totalCount = 0;

    for (const session of group.sessions) {
        totalCount++;
        // 如果会话有 sessionId 且该 sessionId 已导入
        if (session.sessionId && importedSessionIds.has(session.sessionId)) {
            importedCount++;
        }
    }

    if (totalCount === 0) return "new";
    if (importedCount === totalCount) return "imported";
    if (importedCount > 0) return "partial";
    return "new";
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
    importedSessionIds,
}: ProjectGroupListProps) {
    const { t } = useTranslation();

    if (groups.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                <span className="text-sm">{t("project.noMatch")}</span>
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
                const importStatus = getImportStatus(group, importedSessionIds);

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
                        importStatus={importStatus}
                        importedSessionIds={importedSessionIds}
                    />
                );
            })}
        </div>
    );
}
