/**
 * Import Utilities - 导入相关工具函数
 * Story 2.9 UX Redesign
 */

import type { DiscoveredFile } from "@/components/import";
import type { ProjectGroup, ProjectSelectionState } from "@/types/import";

/**
 * 按项目分组文件
 * - 按 projectPath 分组
 * - 每个分组内按 modifiedAt 倒序排列
 * - 分组按最新会话时间倒序排列
 */
export function groupByProject(files: DiscoveredFile[]): ProjectGroup[] {
    const grouped = new Map<string, DiscoveredFile[]>();

    for (const file of files) {
        const projectPath = file.projectPath;
        if (!grouped.has(projectPath)) {
            grouped.set(projectPath, []);
        }
        grouped.get(projectPath)!.push(file);
    }

    const groups = Array.from(grouped.entries()).map(([path, sessions]) => ({
        projectPath: path,
        projectName: getProjectName(path),
        sessions: sessions.sort((a, b) => b.modifiedAt - a.modifiedAt),
    }));

    // 按最新会话时间排序
    return groups.sort((a, b) => {
        const aTime = a.sessions[0]?.modifiedAt ?? 0;
        const bTime = b.sessions[0]?.modifiedAt ?? 0;
        return bTime - aTime;
    });
}

/**
 * 从路径提取项目名
 */
export function getProjectName(projectPath: string): string {
    const parts = projectPath.split("/").filter(Boolean);
    return parts[parts.length - 1] || projectPath;
}

/**
 * 搜索过滤项目分组
 * - 匹配项目名 → 显示整个项目
 * - 匹配会话名 → 仅显示匹配的会话
 */
export function filterGroups(
    groups: ProjectGroup[],
    query: string
): ProjectGroup[] {
    if (!query.trim()) return groups;

    const normalizedQuery = query.toLowerCase().trim();

    return groups
        .map((group) => {
            // 项目名完全匹配或包含 → 显示整个项目
            if (group.projectName.toLowerCase().includes(normalizedQuery)) {
                return group;
            }

            // 会话名匹配 → 仅显示匹配的会话
            const matchedSessions = group.sessions.filter((s) =>
                s.name.toLowerCase().includes(normalizedQuery)
            );

            if (matchedSessions.length > 0) {
                return { ...group, sessions: matchedSessions };
            }

            return null;
        })
        .filter((g): g is ProjectGroup => g !== null);
}

/**
 * 获取项目选择状态
 */
export function getProjectSelectionState(
    group: ProjectGroup,
    selectedFiles: Set<string>
): ProjectSelectionState {
    const totalSessions = group.sessions.length;
    const selectedCount = group.sessions.filter((s) =>
        selectedFiles.has(s.path)
    ).length;

    return {
        isSelected: selectedCount === totalSessions && totalSessions > 0,
        isPartiallySelected: selectedCount > 0 && selectedCount < totalSessions,
        selectedCount,
    };
}

/**
 * 统计分组中的总会话数
 */
export function getTotalSessionCount(groups: ProjectGroup[]): number {
    return groups.reduce((sum, g) => sum + g.sessions.length, 0);
}
