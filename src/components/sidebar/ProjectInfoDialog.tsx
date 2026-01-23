/**
 * ProjectInfoDialog Component - 项目元信息对话框
 * Story 2.27: Task 1 - 项目元信息查看
 * Story 1.9: Task 8.4-8.8 - 设置工作目录功能
 * Story 1.12: 多路径支持
 *
 * 展示项目详细信息：名称、路径、来源、会话数、创建时间等
 * 支持手动设置工作目录（修复 Gemini 等占位符路径问题）
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import {
    FolderOpen,
    Calendar,
    Clock,
    GitBranch,
    Hash,
    MapPin,
    Loader2,
    AlertTriangle,
    FolderEdit,
    Link2,
    Plus,
    Trash2,
    Star,
} from "lucide-react";
import type { Project, LogicalProjectStats } from "@/types/project";
import type { SessionSummary } from "@/lib/project-ipc";
import { updateProjectCwd } from "@/lib/project-ipc";
import { useProjectPaths, addProjectPath, removeProjectPath, setProjectPrimaryPath } from "@/hooks/useProjects";
import { SourceIcon } from "@/components/import/SourceIcons";
import { toast } from "sonner";

/**
 * ProjectInfoDialog Props
 * Story 1.12: Phase 5 - 支持逻辑项目视图
 */
export interface ProjectInfoDialogProps {
    /** 是否打开 */
    isOpen: boolean;
    /** 打开状态变化回调 */
    onOpenChange: (open: boolean) => void;
    /** 项目信息 (存储层视图) - 向后兼容 */
    project?: Project | null;
    /** 获取项目会话列表 (存储层) - 向后兼容 */
    getProjectSessions?: (projectId: string) => Promise<SessionSummary[]>;
    /** 逻辑项目信息 (视图层) - Story 1.12 */
    logicalProject?: LogicalProjectStats | null;
    /** 获取逻辑项目会话列表 (视图层) - Story 1.12 */
    getLogicalProjectSessions?: (physicalPath: string) => Promise<SessionSummary[]>;
    /** 项目更新回调 */
    onProjectUpdated?: (project?: Project) => void;
}

/**
 * 检测 cwd 是否为占位符格式（无法自动识别的情况）
 * Story 1.9: AC11
 */
function isPlaceholderCwd(cwd: string): boolean {
    return (
        cwd.startsWith("gemini-project:") ||
        cwd === "" ||
        cwd === "unknown" ||
        cwd.startsWith("placeholder:")
    );
}

/**
 * 格式化日期时间
 * @param isoString - ISO 格式时间字符串
 * @param locale - 用户语言环境 (zh-CN 或 en)
 */
function formatDateTime(isoString: string, locale: string): string {
    const date = new Date(isoString);
    const localeCode = locale === "en" ? "en-US" : "zh-CN";
    return date.toLocaleString(localeCode, {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
    });
}

/**
 * 截断长文本并添加 tooltip
 * Story 1.9: Task 8.7, 8.8
 */
function TruncatedText({
    text,
    maxLength = 40,
    mono = false,
    className = "",
}: {
    text: string;
    maxLength?: number;
    mono?: boolean;
    className?: string;
}) {
    const isTruncated = text.length > maxLength;
    const displayText = isTruncated ? `${text.slice(0, maxLength)}...` : text;

    if (!isTruncated) {
        return (
            <span className={`${mono ? "font-mono" : ""} ${className}`}>
                {displayText}
            </span>
        );
    }

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <span
                    className={`${mono ? "font-mono" : ""} ${className} cursor-help`}
                >
                    {displayText}
                </span>
            </TooltipTrigger>
            <TooltipContent side="bottom" className="max-w-md break-all font-mono text-xs">
                {text}
            </TooltipContent>
        </Tooltip>
    );
}

/**
 * 信息行组件
 */
function InfoRow({
    icon: Icon,
    label,
    value,
    mono = false,
    action,
    warning,
}: {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    value: string | React.ReactNode;
    mono?: boolean;
    action?: React.ReactNode;
    warning?: string;
}) {
    return (
        <div className="flex items-start gap-3 py-2">
            <Icon className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
            <div className="flex-1 min-w-0">
                <div className="text-xs text-muted-foreground mb-0.5 flex items-center gap-1">
                    {label}
                    {warning && (
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <AlertTriangle className="h-3 w-3 text-yellow-500" />
                            </TooltipTrigger>
                            <TooltipContent side="right" className="max-w-xs">
                                {warning}
                            </TooltipContent>
                        </Tooltip>
                    )}
                </div>
                <div className="flex items-center gap-2">
                    <div
                        className={`text-sm ${mono ? "font-mono" : ""} break-all flex-1 min-w-0`}
                        title={typeof value === "string" ? value : undefined}
                    >
                        {typeof value === "string" ? (
                            <TruncatedText text={value} maxLength={50} mono={mono} />
                        ) : (
                            value
                        )}
                    </div>
                    {action}
                </div>
            </div>
        </div>
    );
}

/**
 * 来源名称映射
 */
function getSourceLabel(source: string): string {
    switch (source) {
        case "claude":
            return "Claude Code";
        case "gemini":
            return "Gemini CLI";
        case "cursor":
            return "Cursor";
        case "codex":
            return "Codex";
        case "antigravity":
            return "Antigravity";
        default:
            return source;
    }
}

/**
 * 按来源统计会话数
 */
function countSessionsBySource(sessions: SessionSummary[]): Record<string, number> {
    const counts: Record<string, number> = {};
    for (const session of sessions) {
        const source = session.source || "unknown";
        counts[source] = (counts[source] || 0) + 1;
    }
    return counts;
}

/**
 * ProjectInfoDialog 组件
 * 显示项目的元信息
 * Story 1.12: 支持逻辑项目视图
 */
export function ProjectInfoDialog({
    isOpen,
    onOpenChange,
    project,
    getProjectSessions,
    logicalProject,
    getLogicalProjectSessions,
    onProjectUpdated,
}: ProjectInfoDialogProps) {
    const { t, i18n } = useTranslation();
    const [sessions, setSessions] = React.useState<SessionSummary[]>([]);
    const [isLoading, setIsLoading] = React.useState(false);
    const [isUpdatingCwd, setIsUpdatingCwd] = React.useState(false);
    const [currentProject, setCurrentProject] = React.useState<Project | null>(project ?? null);
    const [isAddingPath, setIsAddingPath] = React.useState(false);
    const [removingPathId, setRemovingPathId] = React.useState<string | null>(null);

    // Story 1.12: 判断是否使用逻辑项目视图
    const isLogicalView = Boolean(logicalProject);
    const displayName = isLogicalView ? logicalProject?.display_name : currentProject?.name;

    // Story 1.12: 获取项目的所有关联路径
    // 逻辑项目视图下使用第一个存储层项目 ID
    const projectIdForPaths = isLogicalView
        ? (logicalProject?.project_ids[0] ?? null)
        : (currentProject?.id ?? null);
    const { paths, refetch: refetchPaths } = useProjectPaths(projectIdForPaths);

    // 当 project prop 变化时更新内部状态
    React.useEffect(() => {
        setCurrentProject(project ?? null);
    }, [project]);

    // 当对话框打开时加载会话
    React.useEffect(() => {
        if (!isOpen) {
            setSessions([]);
            return;
        }

        setIsLoading(true);

        // 根据视图模式选择不同的加载方式
        if (isLogicalView && logicalProject && getLogicalProjectSessions) {
            getLogicalProjectSessions(logicalProject.physical_path)
                .then(setSessions)
                .catch(console.error)
                .finally(() => setIsLoading(false));
        } else if (currentProject && getProjectSessions) {
            getProjectSessions(currentProject.id)
                .then(setSessions)
                .catch(console.error)
                .finally(() => setIsLoading(false));
        } else {
            setIsLoading(false);
        }
    }, [isOpen, currentProject, getProjectSessions, logicalProject, getLogicalProjectSessions, isLogicalView]);

    /**
     * 处理设置工作目录
     * Story 1.9: Task 8.4, 8.5
     */
    const handleSetCwd = async () => {
        if (!currentProject) return;

        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: t("projectInfo.selectDirectory", "选择项目工作目录"),
            });

            if (!selected || typeof selected !== "string") return;

            setIsUpdatingCwd(true);

            const updatedProject = await updateProjectCwd(currentProject.id, selected);
            setCurrentProject(updatedProject);
            onProjectUpdated?.(updatedProject);
            refetchPaths();

            toast.success(t("projectInfo.cwdUpdated", "工作目录已更新"));
        } catch (error) {
            console.error("Failed to update project cwd:", error);
            toast.error(
                t("projectInfo.cwdUpdateFailed", "更新工作目录失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setIsUpdatingCwd(false);
        }
    };

    /**
     * Story 1.12: 添加新路径
     * 支持逻辑项目视图：使用第一个存储层项目 ID
     */
    const handleAddPath = async () => {
        // 获取 project_id：优先使用 currentProject，否则使用逻辑项目的第一个存储层项目
        const projectId = currentProject?.id ?? logicalProject?.project_ids[0];
        if (!projectId) return;

        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: t("projectInfo.addPath", "添加项目路径"),
            });

            if (!selected || typeof selected !== "string") return;

            setIsAddingPath(true);
            await addProjectPath(projectId, selected, false);
            refetchPaths();
            onProjectUpdated?.();
            toast.success(t("projectInfo.pathAdded", "路径已添加"));
        } catch (error) {
            console.error("Failed to add path:", error);
            toast.error(
                t("projectInfo.addPathFailed", "添加路径失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setIsAddingPath(false);
        }
    };

    /**
     * Story 1.12 fix: 关联真实路径
     * 对于 needs_association 项目，选择目录后直接设置为主路径
     */
    const handleAssociatePath = async () => {
        const projectId = currentProject?.id ?? logicalProject?.project_ids[0];
        if (!projectId) return;

        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: t("projectInfo.associatePath", "关联真实路径"),
            });

            if (!selected || typeof selected !== "string") return;

            setIsAddingPath(true);
            // 直接添加为主路径
            await addProjectPath(projectId, selected, true);
            refetchPaths();
            onProjectUpdated?.();

            // 显示聚合提示，包含会话数和目标路径
            const targetName = selected.split("/").pop() || selected;
            const sessionCount = sessions.length;
            toast.success(
                t("projectInfo.pathAssociatedWithCount", "已关联到 {{name}}，{{count}} 个会话已聚合", {
                    name: targetName,
                    count: sessionCount,
                })
            );
            // 关联成功后关闭对话框
            onOpenChange(false);
        } catch (error) {
            console.error("Failed to associate path:", error);
            toast.error(
                t("projectInfo.associatePathFailed", "关联路径失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setIsAddingPath(false);
        }
    };

    /**
     * Story 1.12: 移除路径
     */
    const handleRemovePath = async (pathId: string) => {
        try {
            setRemovingPathId(pathId);
            await removeProjectPath(pathId);
            refetchPaths();
            toast.success(t("projectInfo.pathRemoved", "路径已移除"));
        } catch (error) {
            console.error("Failed to remove path:", error);
            toast.error(
                t("projectInfo.removePathFailed", "移除路径失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setRemovingPathId(null);
        }
    };

    /**
     * Story 1.12: 设置为主路径
     * 支持逻辑项目视图：使用第一个存储层项目 ID
     */
    const handleSetPrimaryPath = async (path: string) => {
        // 获取 project_id：优先使用 currentProject，否则使用逻辑项目的第一个存储层项目
        const projectId = currentProject?.id ?? logicalProject?.project_ids[0];
        if (!projectId) return;

        try {
            setIsUpdatingCwd(true);
            await setProjectPrimaryPath(projectId, path);
            refetchPaths();
            onProjectUpdated?.();
            toast.success(t("projectInfo.primaryPathSet", "主路径已设置"));
        } catch (error) {
            console.error("Failed to set primary path:", error);
            toast.error(
                t("projectInfo.setPrimaryFailed", "设置主路径失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setIsUpdatingCwd(false);
        }
    };

    // 如果没有任何项目数据，不渲染
    if (!currentProject && !logicalProject) return null;

    const isPlaceholder = isLogicalView
        ? logicalProject?.needs_association ?? false
        : isPlaceholderCwd(currentProject?.cwd ?? "");
    const sourceCounts = countSessionsBySource(sessions);
    const sources = Object.entries(sourceCounts).sort((a, b) => b[1] - a[1]);

    return (
        <TooltipProvider>
            <Dialog open={isOpen} onOpenChange={onOpenChange}>
                <DialogContent className="sm:max-w-md">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <FolderOpen className="h-5 w-5 shrink-0" />
                            <TruncatedText
                                text={displayName ?? "Unknown"}
                                maxLength={30}
                                className="text-lg"
                            />
                            {/* 多来源指示器 (逻辑项目视图) */}
                            {isLogicalView && logicalProject && logicalProject.project_count > 1 && (
                                <span className="text-xs px-1.5 py-0.5 rounded bg-primary/10 text-primary">
                                    {t("project.multiSource", { count: logicalProject.project_count })}
                                </span>
                            )}
                        </DialogTitle>
                        <DialogDescription className="sr-only">
                            {t("projectInfo.description", "项目详细信息")}
                        </DialogDescription>
                    </DialogHeader>

                    <div className="divide-y divide-border">
                        {/* Story 1.12: 项目路径列表 - 多路径支持 */}
                        <div className="flex items-start gap-3 py-2">
                            <MapPin className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
                            <div className="flex-1 min-w-0">
                                <div className="text-xs text-muted-foreground mb-1 flex items-center justify-between">
                                    <span className="flex items-center gap-1">
                                        {t("projectInfo.paths", "项目路径")}
                                        {isPlaceholder && (
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <AlertTriangle className="h-3 w-3 text-yellow-500" />
                                                </TooltipTrigger>
                                                <TooltipContent side="right" className="max-w-xs">
                                                    {t(
                                                        "projectInfo.invalidCwdWarning",
                                                        "无法识别的路径格式，请手动设置正确的工作目录"
                                                    )}
                                                </TooltipContent>
                                            </Tooltip>
                                        )}
                                    </span>
                                    {/* Story 1.12 fix: 仅对已关联的项目显示添加路径按钮 */}
                                    {!isPlaceholder && (
                                        <Button
                                            variant="ghost"
                                            size="icon-sm"
                                            onClick={handleAddPath}
                                            disabled={isAddingPath}
                                            title={t("projectInfo.addPath", "添加路径")}
                                            className="h-5 w-5"
                                        >
                                            {isAddingPath ? (
                                                <Loader2 className="h-3 w-3 animate-spin" />
                                            ) : (
                                                <Plus className="h-3 w-3" />
                                            )}
                                        </Button>
                                    )}
                                </div>
                                {/* 路径列表 */}
                                <div className="space-y-1">
                                    {/* Story 1.12 fix: 对于 needs_association 项目，显示关联真实路径按钮 */}
                                    {isPlaceholder ? (
                                        <div className="space-y-2">
                                            {/* 显示当前虚拟路径 */}
                                            <div className="flex items-center gap-2 text-sm text-muted-foreground">
                                                <TruncatedText
                                                    text={logicalProject?.physical_path ?? currentProject?.cwd ?? ""}
                                                    maxLength={40}
                                                    mono
                                                    className="opacity-60"
                                                />
                                            </div>
                                            {/* 关联真实路径按钮 */}
                                            <Button
                                                variant="outline"
                                                size="sm"
                                                onClick={handleAssociatePath}
                                                disabled={isAddingPath}
                                                className="w-full"
                                            >
                                                {isAddingPath ? (
                                                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                                                ) : (
                                                    <Link2 className="h-4 w-4 mr-2" />
                                                )}
                                                {t("projectInfo.associatePath", "关联真实路径")}
                                            </Button>
                                        </div>
                                    ) : paths.length > 0 ? (
                                        paths.map((pathItem) => (
                                            <div
                                                key={pathItem.id}
                                                className="flex items-center gap-1 group"
                                            >
                                                {pathItem.is_primary ? (
                                                    <Tooltip>
                                                        <TooltipTrigger asChild>
                                                            <Star className="h-3 w-3 text-yellow-500 shrink-0" />
                                                        </TooltipTrigger>
                                                        <TooltipContent>
                                                            {t("projectInfo.primaryPath", "主路径")}
                                                        </TooltipContent>
                                                    </Tooltip>
                                                ) : (
                                                    <Button
                                                        variant="ghost"
                                                        size="icon-sm"
                                                        onClick={() => handleSetPrimaryPath(pathItem.path)}
                                                        disabled={isUpdatingCwd}
                                                        title={t("projectInfo.setAsPrimary", "设为主路径")}
                                                        className="h-4 w-4 opacity-0 group-hover:opacity-100 transition-opacity"
                                                    >
                                                        <Star className="h-3 w-3" />
                                                    </Button>
                                                )}
                                                <TruncatedText
                                                    text={pathItem.path}
                                                    maxLength={40}
                                                    mono
                                                    className="text-sm flex-1"
                                                />
                                                {!pathItem.is_primary && (
                                                    <Button
                                                        variant="ghost"
                                                        size="icon-sm"
                                                        onClick={() => handleRemovePath(pathItem.id)}
                                                        disabled={removingPathId === pathItem.id}
                                                        title={t("projectInfo.removePath", "移除路径")}
                                                        className="h-4 w-4 opacity-0 group-hover:opacity-100 transition-opacity text-destructive hover:text-destructive"
                                                    >
                                                        {removingPathId === pathItem.id ? (
                                                            <Loader2 className="h-3 w-3 animate-spin" />
                                                        ) : (
                                                            <Trash2 className="h-3 w-3" />
                                                        )}
                                                    </Button>
                                                )}
                                            </div>
                                        ))
                                    ) : currentProject ? (
                                        // 如果没有 paths 记录，显示 cwd 作为兼容
                                        <div className="flex items-center gap-2">
                                            <TruncatedText
                                                text={currentProject.cwd}
                                                maxLength={40}
                                                mono
                                                className="text-sm flex-1"
                                            />
                                            <Button
                                                variant="ghost"
                                                size="icon-sm"
                                                onClick={handleSetCwd}
                                                disabled={isUpdatingCwd}
                                                title={t("projectInfo.setCwd", "设置工作目录")}
                                                className="shrink-0"
                                            >
                                                {isUpdatingCwd ? (
                                                    <Loader2 className="h-4 w-4 animate-spin" />
                                                ) : (
                                                    <FolderEdit className="h-4 w-4" />
                                                )}
                                            </Button>
                                        </div>
                                    ) : logicalProject ? (
                                        // 逻辑项目视图：显示物理路径
                                        <div className="flex items-center gap-2">
                                            <TruncatedText
                                                text={logicalProject.physical_path}
                                                maxLength={40}
                                                mono
                                                className="text-sm flex-1"
                                            />
                                        </div>
                                    ) : null}
                                </div>
                            </div>
                        </div>

                        {/* Git Remote URL (如果有) - Story 1.9 */}
                        {currentProject?.git_remote_url && (
                            <InfoRow
                                icon={Link2}
                                label={t("projectInfo.gitRemoteUrl", "Git 仓库 URL")}
                                value={currentProject.git_remote_url}
                                mono
                            />
                        )}

                        {/* 会话数量 - 按来源分组 */}
                        <div className="flex items-start gap-3 py-2">
                            <Hash className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
                            <div className="flex-1 min-w-0">
                                <div className="text-xs text-muted-foreground mb-1">
                                    {t("projectInfo.sessionCount", "会话数量")}
                                </div>
                                {isLoading ? (
                                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                                        <Loader2 className="h-3 w-3 animate-spin" />
                                        {t("common.loading", "加载中")}...
                                    </div>
                                ) : sources.length > 0 ? (
                                    <div className="flex flex-wrap gap-3">
                                        {sources.map(([source, count]) => (
                                            <div
                                                key={source}
                                                className="flex items-center gap-1.5 text-sm"
                                            >
                                                <SourceIcon source={source} className="h-4 w-4" />
                                                <span className="text-muted-foreground">{getSourceLabel(source)}:</span>
                                                <span className="font-medium">{count}</span>
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <div className="text-sm text-muted-foreground">
                                        {isLogicalView ? logicalProject?.total_sessions : currentProject?.session_count}
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* 创建时间 - 仅存储层模式 */}
                        {currentProject && (
                            <InfoRow
                                icon={Calendar}
                                label={t("projectInfo.createdAt", "创建时间")}
                                value={formatDateTime(currentProject.created_at, i18n.language)}
                            />
                        )}

                        {/* 最后活动时间 */}
                        <InfoRow
                            icon={Clock}
                            label={t("projectInfo.lastActivity", "最后活动")}
                            value={formatDateTime(
                                isLogicalView && logicalProject
                                    ? logicalProject.last_activity
                                    : currentProject?.last_activity ?? new Date().toISOString(),
                                i18n.language
                            )}
                        />

                        {/* Git 仓库路径 - 仅当与 cwd 不同时显示（cwd 是子目录的情况） */}
                        {currentProject?.has_git_repo &&
                            currentProject?.git_repo_path &&
                            currentProject?.git_repo_path !== currentProject?.cwd && (
                                <InfoRow
                                    icon={GitBranch}
                                    label={t("projectInfo.gitPath", "Git 仓库根目录")}
                                    value={currentProject.git_repo_path}
                                    mono
                                />
                            )}
                        
                        {/* 逻辑项目 Git 状态 */}
                        {isLogicalView && logicalProject?.has_git_repo && (
                            <InfoRow
                                icon={GitBranch}
                                label={t("projectInfo.gitStatus", "Git 状态")}
                                value={t("projectInfo.hasGitRepo", "已检测到 Git 仓库")}
                            />
                        )}
                    </div>

                    {/* 无效 CWD 提示 */}
                    {isPlaceholder && (
                        <div className="mt-4 p-3 rounded-md bg-yellow-500/10 border border-yellow-500/20">
                            <div className="flex items-start gap-2">
                                <AlertTriangle className="h-4 w-4 text-yellow-500 mt-0.5 shrink-0" />
                                <div className="text-sm text-yellow-600 dark:text-yellow-400">
                                    <p className="font-medium mb-1">
                                        {t("projectInfo.invalidCwdTitle", "无法识别的路径")}
                                    </p>
                                    <p className="text-xs opacity-80">
                                        {t(
                                            "projectInfo.invalidCwdDescription",
                                            "此项目的路径可能是占位符格式（如 Gemini CLI 生成）。请点击上方的编辑按钮设置正确的工作目录，以便系统正确识别项目位置并启用 Git 相关功能。"
                                        )}
                                    </p>
                                </div>
                            </div>
                        </div>
                    )}
                </DialogContent>
            </Dialog>
        </TooltipProvider>
    );
}
