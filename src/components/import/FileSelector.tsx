/**
 * FileSelector Component - 文件选择组件
 * Story 2.9: Task 3
 *
 * 显示发现的日志文件列表，支持：
 * - 扫描默认路径
 * - 手动选择文件
 * - 全选/反选
 * - 单个文件选择
 */

import { Search, FolderOpen, Loader2, FileJson } from "lucide-react";
import { Button, Checkbox } from "@/components/ui";

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
  /** 扫描默认路径回调 */
  onScan: () => void;
  /** 手动选择文件回调 */
  onSelectFiles: () => void;
  /** 切换单个文件选择 */
  onToggleFile: (path: string) => void;
  /** 切换全选 */
  onToggleAll: () => void;
  /** 是否正在加载 */
  loading: boolean;
}

/**
 * 格式化文件大小
 */
function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * 格式化相对时间
 */
function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;

  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 60) return `${minutes} 分钟前`;
  if (hours < 24) return `${hours} 小时前`;
  return `${days} 天前`;
}

/**
 * 从完整路径提取项目名
 */
function getProjectName(projectPath: string): string {
  const parts = projectPath.split("/").filter(Boolean);
  return parts[parts.length - 1] || projectPath;
}

/**
 * 文件列表项组件
 */
function FileItem({
  file,
  selected,
  onToggle,
}: {
  file: DiscoveredFile;
  selected: boolean;
  onToggle: () => void;
}) {
  return (
    <div className="flex items-center gap-3 px-3 py-2.5 border-b border-border/50 last:border-b-0 hover:bg-muted/30 transition-colors">
      {/* 复选框 */}
      <Checkbox
        data-testid={`file-checkbox-${file.path}`}
        checked={selected}
        onCheckedChange={onToggle}
      />

      {/* 文件图标 */}
      <FileJson className="w-4 h-4 text-muted-foreground shrink-0" />

      {/* 文件信息 */}
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium text-foreground truncate">
          {file.name}
        </div>
        <div className="flex items-center gap-2 mt-0.5">
          <span className="text-xs text-muted-foreground">
            {formatFileSize(file.size)}
          </span>
          <span className="text-xs text-muted-foreground">·</span>
          <span className="text-xs text-muted-foreground">
            {formatRelativeTime(file.modifiedAt)}
          </span>
        </div>
      </div>

      {/* 项目名 */}
      <span className="text-xs text-primary font-mono shrink-0">
        {getProjectName(file.projectPath)}
      </span>
    </div>
  );
}

/**
 * FileSelector 组件
 * 显示和选择要导入的文件
 */
export function FileSelector({
  files,
  selectedFiles,
  onScan,
  onSelectFiles,
  onToggleFile,
  onToggleAll,
  loading,
}: FileSelectorProps) {
  const allSelected = files.length > 0 && selectedFiles.size === files.length;
  const someSelected = selectedFiles.size > 0 && selectedFiles.size < files.length;

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

      {/* 文件列表 */}
      <div className="border border-border rounded-lg overflow-hidden">
        {/* 列表头部 */}
        {files.length > 0 && (
          <div className="flex items-center gap-3 px-3 py-2 bg-muted/50 border-b border-border">
            <Checkbox
              data-testid="select-all-checkbox"
              checked={allSelected}
              // indeterminate 状态通过 data-state 处理
              data-state={allSelected ? "checked" : someSelected ? "indeterminate" : "unchecked"}
              onCheckedChange={onToggleAll}
            />
            <span className="text-sm font-medium text-foreground">
              已选择 {selectedFiles.size} 个 / 共 {files.length} 个文件
            </span>
          </div>
        )}

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
          <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <FileJson className="w-10 h-10 mb-2 opacity-50" />
            <span className="text-sm">暂无文件</span>
            <span className="text-xs mt-1">点击上方按钮扫描或选择文件</span>
          </div>
        )}

        {/* 文件列表 */}
        {!loading && files.length > 0 && (
          <div className="max-h-[300px] overflow-y-auto">
            {files.map((file) => (
              <FileItem
                key={file.path}
                file={file}
                selected={selectedFiles.has(file.path)}
                onToggle={() => onToggleFile(file.path)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
