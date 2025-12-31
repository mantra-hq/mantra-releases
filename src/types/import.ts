/**
 * Import Types - 导入相关类型定义
 * Story 2.9 UX Redesign
 */

import type { DiscoveredFile } from "@/components/import";

/**
 * 项目分组
 */
export interface ProjectGroup {
    /** 项目完整路径 */
    projectPath: string;
    /** 项目名称 (最后一级目录) */
    projectName: string;
    /** 该项目下的会话列表 (按时间倒序) */
    sessions: DiscoveredFile[];
}

/**
 * 项目选择状态
 */
export interface ProjectSelectionState {
    /** 是否全选 */
    isSelected: boolean;
    /** 是否部分选择 */
    isPartiallySelected: boolean;
    /** 已选数量 */
    selectedCount: number;
}
