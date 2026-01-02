/**
 * FileSelector Component - 文件选择组件 (重构版)
 * Story 2.9: Task 3 + UX Redesign
 *
 * 显示发现的日志文件列表，使用项目分组展示：
 * - 按项目分组
 * - 实时搜索过滤
 * - 批量选择
 */

import * as React from "react";
import { Search, FolderOpen, Loader2, FileJson } from "lucide-react";
import { Button } from "@/components/ui";
import { useDebouncedValue } from "@/hooks/useDebouncedValue";
import { groupByProject, filterGroups, getTotalSessionCount } from "@/lib/import-utils";
import { SearchFilter } from "./SearchFilter";
import { SelectionStats } from "./SelectionStats";
import { ProjectGroupList } from "./ProjectGroupList";

/** 发现的文件信息 */
export interface DiscoveredFile {
  /** 文件完整路径 */
  path: string;
  /** 文件名 */
  name: string;
  /** 文件大小 (bytes) */
  size: number;
  /** 修改时间 (Unix timestamp ms) */
  modifiedAt: number;
  /** 项目路径 (从 cwd 推断) */
  projectPath: string;
}

/** FileSelector Props */
export interface FileSelectorProps {
  /** 发现的文件列表 */
  files: DiscoveredFile[];
  /** 已选择的文件路径集合 */
  selectedFiles: Set<string>;
  /** 展开的项目路径集合 */
  expandedProjects: Set<string>;
  /** 搜索查询 */
  searchQuery: string;
  /** 扫描默认路径回调 */
  onScan: () => void;
  /** 手动选择目录回调 */
  onSelectFiles: () => void;
  /** 切换单个文件选择 */
  onToggleFile: (path: string) => void;
  /** 全选 */
  onSelectAll: () => void;
  /** 清空选择 */
  onClearAll: () => void;
  /** 反选 */
  onInvertSelection: () => void;
  /** 切换项目选择 */
  onToggleProject: (projectPath: string) => void;
  /** 切换项目展开 */
  onToggleProjectExpand: (projectPath: string) => void;
  /** 设置搜索查询 */
  onSearchChange: (query: string) => void;
  /** 是否正在加载 */
  loading: boolean;
}

/**
 * FileSelector 组件 (重构版)
 * 使用项目分组展示文件列表
 */
export function FileSelector({
  files,
  selectedFiles,
  expandedProjects,
  searchQuery,
  onScan,
  onSelectFiles,
  onToggleFile,
  onSelectAll,
  onClearAll,
  onInvertSelection,
  onToggleProject,
  onToggleProjectExpand,
  onSearchChange,
  loading,
}: FileSelectorProps) {
  // 防抖搜索
  const debouncedQuery = useDebouncedValue(searchQuery, 150);

  // 按项目分组
  const projectGroups = React.useMemo(
    () => groupByProject(files),
    [files]
  );

  // 过滤后的分组
  const filteredGroups = React.useMemo(
    () => filterGroups(projectGroups, debouncedQuery),
    [projectGroups, debouncedQuery]
  );

  // 统计数据
  const totalProjects = projectGroups.length;
  const totalSessions = files.length;
  const filteredSessionCount = getTotalSessionCount(filteredGroups);

  return (
    <div data-testid="file-selector" className="space-y-4">
      {/* 操作按钮 */}
      <div className="flex gap-3">
        <Button
          variant="outline"
          onClick={onScan}
          disabled={loading}
          className="gap-2"
        >
          <Search className="w-4 h-4" />
          扫描默认路径
        </Button>
        <Button
          variant="outline"
          onClick={onSelectFiles}
          disabled={loading}
          className="gap-2"
        >
          <FolderOpen className="w-4 h-4" />
          手动选择目录
        </Button>
      </div>

      {/* 加载状态 */}
      {loading && (
        <div
          data-testid="file-selector-loading"
          className="flex items-center justify-center gap-2 py-12 text-muted-foreground"
        >
          <Loader2 className="w-4 h-4 animate-spin" />
          <span className="text-sm">正在扫描...</span>
        </div>
      )}

      {/* 空状态 */}
      {!loading && files.length === 0 && (
        <div className="flex flex-col items-center justify-center py-12 text-muted-foreground border border-border rounded-lg">
          <FileJson className="w-10 h-10 mb-2 opacity-50" />
          <span className="text-sm">暂无文件</span>
          <span className="text-xs mt-1">点击上方按钮扫描或选择目录</span>
        </div>
      )}

      {/* 有文件时显示搜索、统计和列表 */}
      {!loading && files.length > 0 && (
        <>
          {/* 搜索框 */}
          <SearchFilter
            value={searchQuery}
            onChange={onSearchChange}
            resultCount={filteredSessionCount}
            totalCount={totalSessions}
          />

          {/* 统计栏 */}
          <SelectionStats
            totalProjects={totalProjects}
            totalSessions={totalSessions}
            selectedCount={selectedFiles.size}
            onSelectAll={onSelectAll}
            onClearAll={onClearAll}
            onInvertSelection={onInvertSelection}
          />

          {/* 项目分组列表 */}
          <ProjectGroupList
            groups={filteredGroups}
            selectedFiles={selectedFiles}
            expandedProjects={expandedProjects}
            onToggleProject={onToggleProject}
            onToggleExpand={onToggleProjectExpand}
            onToggleSession={onToggleFile}
          />
        </>
      )}
    </div>
  );
}
