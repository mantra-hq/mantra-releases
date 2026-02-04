/**
 * TakeoverStatusCard - MCP 接管状态卡片
 * Story 11.16: 接管状态模块系统性重构
 *
 * 功能：
 * - 按 scope 分组显示（用户级/项目级）
 * - 折叠/展开功能
 * - 文件内容预览抽屉
 * - 一键恢复功能
 */

import React, { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { zhCN, enUS } from "date-fns/locale";
import { format, isValid, parseISO } from "date-fns";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  ActionSheet,
  ActionSheetContent,
  ActionSheetDescription,
  ActionSheetHeader,
  ActionSheetTitle,
} from "@/components/ui/action-sheet";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Archive,
  RotateCcw,
  FileText,
  Clock,
  Loader2,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Eye,
  User,
  FolderOpen,
  Copy,
  Check,
  AlertTriangle,
  Trash2,
} from "lucide-react";
import { feedback } from "@/lib/feedback";
import { SourceIcon } from "@/components/import/SourceIcons";

/**
 * 工具类型
 */
type ToolType = "claude_code" | "cursor" | "codex" | "gemini_cli";

/**
 * 接管作用域 (Story 11.16: AC1, AC2; Story 11.21: Local Scope)
 */
type TakeoverScope = "user" | "project" | "local";

/**
 * 带完整性信息的备份记录 (Story 11.22)
 */
interface TakeoverBackupIntegrity {
  id: string;
  toolType: ToolType;
  scope: TakeoverScope;
  projectPath: string | null;
  originalPath: string;
  backupPath: string;
  backupHash: string | null;
  takenOverAt: string;
  restoredAt: string | null;
  status: "active" | "restored";
  // 完整性字段
  backupFileExists: boolean;
  originalFileExists: boolean;
  hashValid: boolean | null;
}

/**
 * 获取工具类型显示名称
 */
function getToolLabel(toolType: ToolType): string {
  switch (toolType) {
    case "claude_code":
      return "Claude Code";
    case "cursor":
      return "Cursor";
    case "codex":
      return "Codex";
    case "gemini_cli":
      return "Gemini CLI";
    default:
      return toolType;
  }
}

/**
 * 将工具类型转换为 adapter_id
 */
function toolTypeToAdapterId(toolType: ToolType): string {
  switch (toolType) {
    case "claude_code":
      return "claude";
    case "gemini_cli":
      return "gemini";
    default:
      return toolType;
  }
}

/**
 * 缩短路径显示（显示 ~ 替代 home 目录）
 */
function shortenPath(path: string): string {
  const homeDir = path.match(/^\/(?:home\/[^/]+|Users\/[^/]+)/)?.[0];
  if (homeDir) {
    return path.replace(homeDir, "~");
  }
  return path;
}

/**
 * 从完整路径中提取项目名称
 */
function getProjectName(projectPath: string): string {
  return projectPath.split("/").pop() || projectPath;
}

/**
 * 格式化日期时间（使用 i18n locale）
 */
function formatDateTime(isoString: string | undefined | null, locale: string): string {
  if (!isoString) {
    return "-";
  }

  const date = parseISO(isoString);

  if (!isValid(date)) {
    console.warn("[TakeoverStatusCard] Invalid date string:", isoString);
    return isoString;
  }

  const dateLocale = locale === "zh-CN" ? zhCN : enUS;
  return format(date, "MM-dd HH:mm", { locale: dateLocale });
}

/**
 * 检测文件类型用于语法高亮
 * 支持 .mantra-backup.* 后缀的备份文件
 */
function detectFileType(path: string): "json" | "toml" | "text" {
  const basePath = path.replace(/\.mantra-backup\.\d+$/, "");
  if (basePath.endsWith(".json")) return "json";
  if (basePath.endsWith(".toml")) return "toml";
  return "text";
}

/**
 * 简单 JSON 语法高亮 (Story 11.16: AC5)
 */
function highlightJsonLine(line: string, lineKey: number): React.ReactNode {
  const regex = /("(?:[^"\\]|\\.)*")(\s*:)?|(\btrue\b|\bfalse\b|\bnull\b)|(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/g;
  const parts: React.ReactNode[] = [];
  let lastIndex = 0;
  let match;
  let key = 0;

  while ((match = regex.exec(line)) !== null) {
    if (match.index > lastIndex) {
      parts.push(<span key={`${lineKey}-t-${key++}`}>{line.slice(lastIndex, match.index)}</span>);
    }
    if (match[1] && match[2]) {
      parts.push(<span key={`${lineKey}-k-${key++}`} className="text-sky-400">{match[1]}</span>);
      parts.push(<span key={`${lineKey}-c-${key++}`}>{match[2]}</span>);
    } else if (match[1]) {
      parts.push(<span key={`${lineKey}-s-${key++}`} className="text-emerald-400">{match[1]}</span>);
    } else if (match[3]) {
      parts.push(<span key={`${lineKey}-b-${key++}`} className="text-purple-400">{match[3]}</span>);
    } else if (match[4]) {
      parts.push(<span key={`${lineKey}-n-${key++}`} className="text-amber-400">{match[4]}</span>);
    }
    lastIndex = regex.lastIndex;
  }

  if (lastIndex < line.length) {
    parts.push(<span key={`${lineKey}-e-${key}`}>{line.slice(lastIndex)}</span>);
  }

  return parts.length > 0 ? <>{parts}</> : line;
}

/**
 * 简单 TOML 语法高亮 (Story 11.16: AC5)
 */
function highlightTomlLine(line: string): React.ReactNode {
  const trimmed = line.trimStart();
  if (trimmed.startsWith("#")) {
    return <span className="text-muted-foreground italic">{line}</span>;
  }
  if (trimmed.startsWith("[")) {
    return <span className="text-sky-400 font-medium">{line}</span>;
  }
  const kvMatch = line.match(/^(\s*)([\w.-]+)(\s*=\s*)(.*)/);
  if (kvMatch) {
    return (
      <>
        {kvMatch[1]}
        <span className="text-sky-400">{kvMatch[2]}</span>
        {kvMatch[3]}
        <span className="text-emerald-400">{kvMatch[4]}</span>
      </>
    );
  }
  return line;
}

/**
 * 按文件类型高亮一行内容
 */
function highlightLine(line: string, fileType: "json" | "toml" | "text", lineIndex: number): React.ReactNode {
  if (fileType === "json") return highlightJsonLine(line, lineIndex);
  if (fileType === "toml") return highlightTomlLine(line);
  return line;
}

export interface TakeoverStatusCardProps {
  onRestore?: () => void;
}

export function TakeoverStatusCard({ onRestore }: TakeoverStatusCardProps) {
  const { t, i18n } = useTranslation();
  const [backups, setBackups] = useState<TakeoverBackupIntegrity[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [restoringId, setRestoringId] = useState<string | null>(null);
  const [isDeletingInvalid, setIsDeletingInvalid] = useState(false);

  // 分组展开状态
  const [userExpanded, setUserExpanded] = useState(true);
  const [projectExpanded, setProjectExpanded] = useState(false);
  const [localExpanded, setLocalExpanded] = useState(false);
  const [expandedProjects, setExpandedProjects] = useState<Set<string>>(new Set());
  const [expandedLocalProjects, setExpandedLocalProjects] = useState<Set<string>>(new Set());

  // 文件预览状态
  const [previewOpen, setPreviewOpen] = useState(false);
  const [previewPath, setPreviewPath] = useState<string>("");
  const [previewContent, setPreviewContent] = useState<string>("");
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const currentLocale = useMemo(() => i18n.language, [i18n.language]);

  // 按 scope 分组
  const groupedBackups = useMemo(() => {
    const userBackups = backups.filter((b) => b.scope === "user");
    const projectBackups = backups.filter((b) => b.scope === "project");
    const localBackups = backups.filter((b) => b.scope === "local");

    // 项目级按 projectPath 子分组
    const projectGroups = new Map<string, TakeoverBackupIntegrity[]>();
    for (const backup of projectBackups) {
      const path = backup.projectPath || "unknown";
      const existing = projectGroups.get(path) || [];
      projectGroups.set(path, [...existing, backup]);
    }

    // Local Scope 按 projectPath 子分组 (Story 11.21)
    const localGroups = new Map<string, TakeoverBackupIntegrity[]>();
    for (const backup of localBackups) {
      const path = backup.projectPath || "unknown";
      const existing = localGroups.get(path) || [];
      localGroups.set(path, [...existing, backup]);
    }

    return {
      user: userBackups,
      project: projectGroups,
      projectCount: projectBackups.length,
      local: localGroups,
      localCount: localBackups.length,
    };
  }, [backups]);

  // 加载活跃的接管记录（带完整性信息）
  const loadBackups = useCallback(async () => {
    try {
      const result = await invoke<TakeoverBackupIntegrity[]>("list_active_takeovers_with_integrity");
      setBackups(result);
    } catch (error) {
      console.error("[TakeoverStatusCard] Failed to load backups:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 计算无效备份数量 (Story 11.22)
  const invalidBackupCount = useMemo(() => {
    return backups.filter((b) => !b.backupFileExists || b.hashValid === false).length;
  }, [backups]);

  // 删除无效备份 (Story 11.22)
  const handleDeleteInvalidBackups = useCallback(async () => {
    setIsDeletingInvalid(true);
    try {
      const deletedCount = await invoke<number>("delete_invalid_takeover_backups");
      if (deletedCount > 0) {
        feedback.success(t("hub.takeover.deleteInvalidSuccess", { count: deletedCount }));
        await loadBackups();
      } else {
        feedback.success(t("hub.takeover.noInvalidBackups"));
      }
    } catch (error) {
      console.error("[TakeoverStatusCard] Failed to delete invalid backups:", error);
      feedback.error(t("hub.takeover.deleteInvalidError"), (error as Error).message);
    } finally {
      setIsDeletingInvalid(false);
    }
  }, [t, loadBackups]);

  // 读取文件内容预览
  const handlePreview = useCallback(async (path: string) => {
    setPreviewPath(path);
    setPreviewOpen(true);
    setPreviewLoading(true);
    setPreviewError(null);
    setPreviewContent("");
    setCopied(false);

    try {
      const content = await invoke<string>("read_config_file_content", { path });
      setPreviewContent(content);
    } catch (error) {
      console.error("[TakeoverStatusCard] Failed to read file:", error);
      setPreviewError((error as Error).message);
    } finally {
      setPreviewLoading(false);
    }
  }, []);

  // 复制到剪贴板
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(previewContent);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error("[TakeoverStatusCard] Failed to copy:", error);
    }
  }, [previewContent]);

  // 恢复配置
  const handleRestore = useCallback(async (backupId: string) => {
    setRestoringId(backupId);
    try {
      await invoke("restore_takeover", { backupId });
      try {
        const status = await invoke<{ running: boolean }>("get_gateway_status");
        if (status.running) {
          await invoke("restart_gateway", {});
        }
      } catch {
        // Gateway 操作失败不阻断恢复流程
      }
      feedback.success(t("hub.takeover.restoreSuccess"));
      await loadBackups();
      onRestore?.();
    } catch (error) {
      console.error("[TakeoverStatusCard] Failed to restore:", error);
      feedback.error(t("hub.takeover.restoreError"), (error as Error).message);
    } finally {
      setRestoringId(null);
    }
  }, [t, loadBackups, onRestore]);

  // 切换项目分组展开
  const toggleProjectGroup = useCallback((projectPath: string) => {
    setExpandedProjects((prev) => {
      const next = new Set(prev);
      if (next.has(projectPath)) {
        next.delete(projectPath);
      } else {
        next.add(projectPath);
      }
      return next;
    });
  }, []);

  // 切换 Local Scope 项目分组展开 (Story 11.21)
  const toggleLocalProjectGroup = useCallback((projectPath: string) => {
    setExpandedLocalProjects((prev) => {
      const next = new Set(prev);
      if (next.has(projectPath)) {
        next.delete(projectPath);
      } else {
        next.add(projectPath);
      }
      return next;
    });
  }, []);

  // 初始加载
  useEffect(() => {
    loadBackups();
  }, [loadBackups]);

  // 没有活跃接管记录时不显示
  if (!isLoading && backups.length === 0) {
    return null;
  }

  // 渲染单个接管记录
  const renderBackupItem = (backup: TakeoverBackupIntegrity) => {
    // 检查是否是无效备份 (Story 11.22)
    const isInvalid = !backup.backupFileExists || backup.hashValid === false;
    const canRestore = backup.backupFileExists && backup.hashValid !== false;

    return (
    <div
      key={backup.id}
      className={`flex items-center gap-2 py-1.5 px-2 hover:bg-muted/50 rounded text-sm ${isInvalid ? "bg-destructive/5" : ""}`}
      data-testid={`takeover-item-${backup.id}`}
    >
      {/* 完整性状态图标 (Story 11.22) */}
      {isInvalid && (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <AlertTriangle className="h-4 w-4 text-destructive shrink-0" />
            </TooltipTrigger>
            <TooltipContent side="top">
              <p className="text-xs">
                {!backup.backupFileExists
                  ? t("hub.takeover.backupFileMissing")
                  : t("hub.takeover.hashMismatch")}
              </p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}

      {/* 工具图标 */}
      <SourceIcon source={toolTypeToAdapterId(backup.toolType)} className="h-4 w-4 shrink-0" />

      {/* 工具名称 */}
      <span className="font-medium w-24 shrink-0">{getToolLabel(backup.toolType)}</span>

      {/* Local Scope 项目路径标签 (Story 11.21) */}
      {backup.scope === "local" && backup.projectPath && (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Badge variant="outline" className="text-[10px] shrink-0 max-w-[120px] truncate text-emerald-500 border-emerald-500/30">
                {getProjectName(backup.projectPath)}
              </Badge>
            </TooltipTrigger>
            <TooltipContent side="top">
              <p className="text-xs">{backup.projectPath}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}

      {/* 当前配置路径 + 预览按钮 */}
      <div className="flex items-center gap-1 flex-1 min-w-0">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-1 min-w-0">
                <FileText className="h-3.5 w-3.5 text-blue-500 shrink-0" />
                <code className="text-xs text-muted-foreground truncate">
                  {shortenPath(backup.originalPath)}
                </code>
              </div>
            </TooltipTrigger>
            <TooltipContent side="top" className="max-w-md">
              <p className="text-xs">{t("hub.takeover.currentConfig")}: {backup.originalPath}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
        <Button
          variant="ghost"
          size="icon"
          className="h-5 w-5 shrink-0"
          onClick={() => handlePreview(backup.originalPath)}
          title={t("hub.takeover.preview")}
        >
          <Eye className="h-3 w-3" />
        </Button>
      </div>

      {/* 原始备份路径 + 预览按钮 */}
      <div className="flex items-center gap-1 flex-1 min-w-0">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-1 min-w-0">
                <Archive className="h-3.5 w-3.5 text-amber-500 shrink-0" />
                <code className="text-xs text-muted-foreground truncate">
                  {shortenPath(backup.backupPath)}
                </code>
              </div>
            </TooltipTrigger>
            <TooltipContent side="top" className="max-w-md">
              <p className="text-xs">{t("hub.takeover.originalBackup")}: {backup.backupPath}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
        <Button
          variant="ghost"
          size="icon"
          className="h-5 w-5 shrink-0"
          onClick={() => handlePreview(backup.backupPath)}
          title={t("hub.takeover.preview")}
        >
          <Eye className="h-3 w-3" />
        </Button>
      </div>

      {/* 时间 */}
      <div className="flex items-center gap-1 text-xs text-muted-foreground shrink-0 w-28">
        <Clock className="h-3 w-3" />
        <span>{formatDateTime(backup.takenOverAt, currentLocale)}</span>
      </div>

      {/* 恢复按钮 */}
      <AlertDialog>
        <AlertDialogTrigger asChild>
          <Button
            variant="outline"
            size="sm"
            className="h-6 px-2"
            disabled={restoringId === backup.id || !canRestore}
            data-testid={`restore-button-${backup.id}`}
          >
            {restoringId === backup.id ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <>
                <RotateCcw className="h-3 w-3 mr-1" />
                <span className="text-xs">{t("hub.takeover.restore")}</span>
              </>
            )}
          </Button>
        </AlertDialogTrigger>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("hub.takeover.restoreConfirmTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              {backup.scope === "local"
                ? t("hub.takeover.restoreLocalScopeDescription", {
                    tool: getToolLabel(backup.toolType),
                    project: backup.projectPath ? getProjectName(backup.projectPath) : "",
                    defaultValue: `Restore {{tool}} local scope configuration for project "{{project}}". Only this project's MCP services will be restored, other projects and user scope will not be affected.`,
                  })
                : t("hub.takeover.restoreConfirmDescription", {
                    tool: getToolLabel(backup.toolType),
                  })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="py-4 space-y-2 text-sm">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="h-4 w-4 text-emerald-500" />
              <span>{t("hub.takeover.restoreWillDo1")}</span>
            </div>
            <div className="flex items-center gap-2">
              <CheckCircle2 className="h-4 w-4 text-emerald-500" />
              <span>{t("hub.takeover.restoreWillDo2")}</span>
            </div>
            <div className="flex items-center gap-2">
              <CheckCircle2 className="h-4 w-4 text-emerald-500" />
              <span>{t("hub.takeover.restoreWillDo3")}</span>
            </div>
            {/* Local Scope 影响范围提示 (Story 11.21) */}
            {backup.scope === "local" && backup.projectPath && (
              <div className="mt-2 p-2 rounded bg-emerald-500/10 border border-emerald-500/20">
                <p className="text-xs text-emerald-400">
                  {t("hub.takeover.restoreLocalScopeHint", {
                    project: backup.projectPath,
                    defaultValue: "Scope: only {{project}} local MCP configuration",
                  })}
                </p>
              </div>
            )}
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={() => handleRestore(backup.id)}>
              {t("hub.takeover.restoreConfirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
    );
  };

  return (
    <>
      <Card data-testid="takeover-status-card">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-md bg-blue-500/10">
                <Archive className="h-5 w-5 text-blue-500" />
              </div>
              <div>
                <CardTitle className="text-base">{t("hub.takeover.title")}</CardTitle>
                <CardDescription>{t("hub.takeover.description")}</CardDescription>
              </div>
            </div>
            <div className="flex items-center gap-2">
              {/* 删除无效记录按钮 (Story 11.22) */}
              {invalidBackupCount > 0 && (
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="text-destructive border-destructive/30 hover:bg-destructive/10"
                        onClick={handleDeleteInvalidBackups}
                        disabled={isDeletingInvalid}
                      >
                        {isDeletingInvalid ? (
                          <Loader2 className="h-3 w-3 animate-spin mr-1" />
                        ) : (
                          <Trash2 className="h-3 w-3 mr-1" />
                        )}
                        <span className="text-xs">{t("hub.takeover.deleteInvalid", { count: invalidBackupCount })}</span>
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent side="bottom">
                      <p className="text-xs">{t("hub.takeover.deleteInvalidTooltip")}</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
              <Badge variant="secondary" className="bg-blue-500/10 text-blue-500 border-blue-500/20">
                {t("hub.takeover.activeCount", { count: backups.length })}
              </Badge>
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-2">
          {isLoading ? (
            <div className="flex items-center justify-center py-4">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : (
            <>
              {/* 用户级配置分组 */}
              {groupedBackups.user.length > 0 && (
                <Collapsible open={userExpanded} onOpenChange={setUserExpanded}>
                  <CollapsibleTrigger className="flex items-center gap-2 w-full p-2 hover:bg-muted/50 rounded-md text-sm font-medium">
                    {userExpanded ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                    <User className="h-4 w-4 text-blue-500" />
                    <span>{t("hub.takeover.userLevel")}</span>
                    <Badge variant="secondary" className="ml-auto text-xs">
                      {groupedBackups.user.length}
                    </Badge>
                  </CollapsibleTrigger>
                  <CollapsibleContent className="pl-4 mt-1 space-y-0.5 border-l-2 border-muted ml-2">
                    {groupedBackups.user.map(renderBackupItem)}
                  </CollapsibleContent>
                </Collapsible>
              )}

              {/* 项目级配置分组 */}
              {groupedBackups.projectCount > 0 && (
                <Collapsible open={projectExpanded} onOpenChange={setProjectExpanded}>
                  <CollapsibleTrigger className="flex items-center gap-2 w-full p-2 hover:bg-muted/50 rounded-md text-sm font-medium">
                    {projectExpanded ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                    <FolderOpen className="h-4 w-4 text-amber-500" />
                    <span>{t("hub.takeover.projectLevel")}</span>
                    <Badge variant="secondary" className="ml-auto text-xs">
                      {groupedBackups.projectCount}
                    </Badge>
                  </CollapsibleTrigger>
                  <CollapsibleContent className="pl-4 mt-1 space-y-1 border-l-2 border-muted ml-2">
                    {Array.from(groupedBackups.project.entries()).map(([projectPath, items]) => (
                      <Collapsible
                        key={projectPath}
                        open={expandedProjects.has(projectPath)}
                        onOpenChange={() => toggleProjectGroup(projectPath)}
                      >
                        <CollapsibleTrigger className="flex items-center gap-2 w-full p-1.5 hover:bg-muted/50 rounded text-sm">
                          {expandedProjects.has(projectPath) ? (
                            <ChevronDown className="h-3 w-3" />
                          ) : (
                            <ChevronRight className="h-3 w-3" />
                          )}
                          <FolderOpen className="h-3.5 w-3.5 text-muted-foreground" />
                          <TooltipProvider>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <span className="text-muted-foreground truncate">
                                  {getProjectName(projectPath)}
                                </span>
                              </TooltipTrigger>
                              <TooltipContent side="top">
                                <p className="text-xs">{projectPath}</p>
                              </TooltipContent>
                            </Tooltip>
                          </TooltipProvider>
                          <Badge variant="outline" className="ml-auto text-xs">
                            {items.length}
                          </Badge>
                        </CollapsibleTrigger>
                        <CollapsibleContent className="pl-4 mt-0.5 space-y-0.5">
                          {items.map(renderBackupItem)}
                        </CollapsibleContent>
                      </Collapsible>
                    ))}
                  </CollapsibleContent>
                </Collapsible>
              )}

              {/* Local Scope 配置分组 (Story 11.21) */}
              {groupedBackups.localCount > 0 && (
                <Collapsible open={localExpanded} onOpenChange={setLocalExpanded}>
                  <CollapsibleTrigger className="flex items-center gap-2 w-full p-2 hover:bg-muted/50 rounded-md text-sm font-medium">
                    {localExpanded ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                    <FolderOpen className="h-4 w-4 text-emerald-500" />
                    <span>{t("hub.takeover.localLevel", "Local Scope")}</span>
                    <Badge variant="secondary" className="ml-auto text-xs bg-emerald-500/10 text-emerald-500">
                      {groupedBackups.localCount}
                    </Badge>
                  </CollapsibleTrigger>
                  <CollapsibleContent className="pl-4 mt-1 space-y-1 border-l-2 border-emerald-500/30 ml-2">
                    {Array.from(groupedBackups.local.entries()).map(([projectPath, items]) => (
                      <Collapsible
                        key={projectPath}
                        open={expandedLocalProjects.has(projectPath)}
                        onOpenChange={() => toggleLocalProjectGroup(projectPath)}
                      >
                        <CollapsibleTrigger className="flex items-center gap-2 w-full p-1.5 hover:bg-muted/50 rounded text-sm">
                          {expandedLocalProjects.has(projectPath) ? (
                            <ChevronDown className="h-3 w-3" />
                          ) : (
                            <ChevronRight className="h-3 w-3" />
                          )}
                          <FolderOpen className="h-3.5 w-3.5 text-emerald-500/70" />
                          <TooltipProvider>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <span className="text-muted-foreground truncate">
                                  {getProjectName(projectPath)}
                                </span>
                              </TooltipTrigger>
                              <TooltipContent side="top">
                                <p className="text-xs">{projectPath}</p>
                              </TooltipContent>
                            </Tooltip>
                          </TooltipProvider>
                          <Badge variant="outline" className="ml-auto text-xs text-emerald-500 border-emerald-500/30">
                            {items.length}
                          </Badge>
                        </CollapsibleTrigger>
                        <CollapsibleContent className="pl-4 mt-0.5 space-y-0.5">
                          {items.map(renderBackupItem)}
                        </CollapsibleContent>
                      </Collapsible>
                    ))}
                  </CollapsibleContent>
                </Collapsible>
              )}
            </>
          )}
        </CardContent>
      </Card>

      {/* 文件预览抽屉 */}
      <ActionSheet open={previewOpen} onOpenChange={setPreviewOpen}>
        <ActionSheetContent size="xl">
          <ActionSheetHeader>
            <ActionSheetTitle className="flex items-center gap-2">
              <FileText className="h-5 w-5" />
              {t("hub.takeover.filePreview")}
            </ActionSheetTitle>
            <ActionSheetDescription className="truncate font-mono text-xs">
              {previewPath}
            </ActionSheetDescription>
          </ActionSheetHeader>

          <div className="flex-1 overflow-hidden mt-4">
            {previewLoading ? (
              <div className="flex items-center justify-center h-full">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : previewError ? (
              <div className="p-4 rounded-md bg-destructive/10 text-destructive text-sm">
                <p className="font-medium">{t("hub.takeover.previewError")}</p>
                <p className="mt-1 text-xs">{previewError}</p>
              </div>
            ) : (
              <div className="relative h-full">
                <Button
                  variant="outline"
                  size="sm"
                  className="absolute top-2 right-2 z-10"
                  onClick={handleCopy}
                >
                  {copied ? (
                    <>
                      <Check className="h-3 w-3 mr-1" />
                      {t("common.copied")}
                    </>
                  ) : (
                    <>
                      <Copy className="h-3 w-3 mr-1" />
                      {t("common.copy")}
                    </>
                  )}
                </Button>
                <pre className="h-full overflow-auto p-4 rounded-md bg-muted/50 text-xs font-mono leading-relaxed">
                  <code>
                    {(() => {
                      const fileType = detectFileType(previewPath);
                      return previewContent.split("\n").map((line, i) => (
                        <div key={i} className="flex">
                          <span className="select-none text-muted-foreground w-8 pr-2 text-right shrink-0">
                            {i + 1}
                          </span>
                          <span className="flex-1 whitespace-pre-wrap break-all">
                            {highlightLine(line, fileType, i)}
                          </span>
                        </div>
                      ));
                    })()}
                  </code>
                </pre>
              </div>
            )}
          </div>
        </ActionSheetContent>
      </ActionSheet>
    </>
  );
}

export default TakeoverStatusCard;
