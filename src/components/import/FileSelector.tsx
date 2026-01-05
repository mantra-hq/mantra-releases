/**
 * FileSelector Component - 文件选择组件 (重构版)
 * Story 2.9: Task 3 + UX Redesign
 * Story 2.20: Import Status Enhancement
 * Story 2.26: 国际化支持
 *
 * 显示发现的日志文件列表，使用项目分组展示：
 * - 按项目分组
 * - 实时搜索过滤
 * - 批量选择
 * - 识别已导入项目
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Radar, FolderOpen, Loader2, FileJson } from "lucide-react";
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
  /** 会话 ID (用于识别已导入状态) */
  sessionId?: string;
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
  /** 切换项目选择 */
  onToggleProject: (projectPath: string) => void;
  /** 切换项目展开 */
  onToggleProjectExpand: (projectPath: string) => void;
  /** 设置搜索查询 */
  onSearchChange: (query: string) => void;
  /** 是否正在加载 */
  loading: boolean;
  /** 已导入会话 ID 集合 (Story 2.20 改进) */
  importedSessionIds?: Set<string>;
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
  onToggleProject,
  onToggleProjectExpand,
  onSearchChange,
  loading,
  importedSessionIds,
}: FileSelectorProps) {
  const { t } = useTranslation();

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

  // Story 2.20 改进: 计算已导入会话数量
  const importedCount = React.useMemo(() => {
    if (!importedSessionIds) {
      return undefined;
    }
    let count = 0;
    for (const file of files) {
      if (file.sessionId && importedSessionIds.has(file.sessionId)) {
        count++;
      }
    }
    return count;
  }, [files, importedSessionIds]);

  // 统计数据
  const totalProjects = projectGroups.length;
  const totalSessions = files.length;
  const filteredSessionCount = getTotalSessionCount(filteredGroups);

  // Story 2.24 AC2: 计算已选项目数（至少有一个会话被选中的项目）
  const selectedProjectCount = React.useMemo(() => {
    let count = 0;
    for (const group of projectGroups) {
      const hasSelectedSession = group.sessions.some((s) => selectedFiles.has(s.path));
      if (hasSelectedSession) {
        count++;
      }
    }
    return count;
  }, [projectGroups, selectedFiles]);

  // Story 2.23: 根据是否已有文件决定按钮文案
  const scanButtonText = files.length > 0 ? t("import.rescan") : t("import.scanDefault");

  return (
    <div data-testid="file-selector" className="space-y-3">
      {/* 工具栏：操作按钮 + 搜索框 */}
      <div className="flex items-center justify-between gap-3">
        {/* 左侧：操作按钮 */}
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={onScan}
            disabled={loading}
            className="gap-1.5"
          >
            <Radar className="w-4 h-4" />
            {scanButtonText}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={onSelectFiles}
            disabled={loading}
            className="gap-1.5"
          >
            <FolderOpen className="w-4 h-4" />
            {t("import.selectManually")}
          </Button>
        </div>
        {/* 右侧：搜索框（仅有文件时显示） */}
        {files.length > 0 && (
          <div className="w-[300px]">
            <SearchFilter
              value={searchQuery}
              onChange={onSearchChange}
              resultCount={filteredSessionCount}
              totalCount={totalSessions}
            />
          </div>
        )}
      </div>

      {/* 加载状态 */}
      {loading && (
        <div
          data-testid="file-selector-loading"
          className="flex items-center justify-center gap-2 py-12 text-muted-foreground"
        >
          <Loader2 className="w-4 h-4 animate-spin" />
          <span className="text-sm">{t("import.scanning")}</span>
        </div>
      )}

      {/* 空状态 */}
      {!loading && files.length === 0 && (
        <div className="flex flex-col items-center justify-center py-12 text-muted-foreground border border-border rounded-lg">
          <FileJson className="w-10 h-10 mb-2 opacity-50" />
          <span className="text-sm">{t("import.noFiles")}</span>
          <span className="text-xs mt-1">{t("import.scanHint")}</span>
        </div>
      )}

      {/* 有文件时显示统计和列表 */}
      {!loading && files.length > 0 && (
        <>
          {/* 统计栏 */}
          <SelectionStats
            totalProjects={totalProjects}
            totalSessions={totalSessions}
            selectedCount={selectedFiles.size}
            selectedProjectCount={selectedProjectCount}
            importedCount={importedCount}
          />

          {/* 项目分组列表 */}
          <ProjectGroupList
            groups={filteredGroups}
            selectedFiles={selectedFiles}
            expandedProjects={expandedProjects}
            onToggleProject={onToggleProject}
            onToggleExpand={onToggleProjectExpand}
            onToggleSession={onToggleFile}
            importedSessionIds={importedSessionIds}
          />
        </>
      )}
    </div>
  );
}
