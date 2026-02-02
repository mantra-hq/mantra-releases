/**
 * ProjectInfoSheet Component - 项目元信息 Sheet
 * Story 2.27: Task 1 - 项目元信息查看
 * Story 1.9: Task 8.4-8.8 - 设置工作目录功能
 * Story 1.12: 多路径支持
 * Story 11.9: Task 4 - MCP 上下文集成
 * Story 12.1: Task 1 - Dialog → Sheet 改造
 * Story 12.4: 迁移使用 ActionSheet 统一封装组件
 *
 * 展示项目详细信息：名称、路径、来源、会话数、创建时间等
 * 支持手动设置工作目录（修复 Gemini 等占位符路径问题）
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import {
    ActionSheet,
    ActionSheetContent,
    ActionSheetHeader,
    ActionSheetTitle,
    ActionSheetDescription,
} from "@/components/ui/action-sheet";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
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
    Trash2,
} from "lucide-react";
import type { Project, LogicalProjectStats } from "@/types/project";
import type { SessionSummary } from "@/lib/project-ipc";
import { useProjectPaths, addProjectPath, removeProjectPath, getProjectPaths, getProjectsByPhysicalPath } from "@/hooks/useProjects";
import { SourceIcon } from "@/components/import/SourceIcons";
import type { ProjectPath } from "@/types/project";
import { toast } from "sonner";
// Story 11.9: MCP 上下文卡片
import { McpContextCard } from "@/components/hub";

/**
 * ProjectInfoSheet Props
 * Story 1.12: Phase 5 - 支持逻辑项目视图
 * Story 12.1: Task 1 - 重命名 Props 接口
 */
export interface ProjectInfoSheetProps {
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
 * Story 1.12 Phase 7: 从项目 cwd 推断来源类型
 * 由于 Project 类型没有 source 属性，需要从 cwd 格式推断
 */
function inferSourceFromCwd(cwd: string): string {
    if (cwd.startsWith("gemini-project:")) return "gemini";
    if (cwd.startsWith("cursor-workspace:")) return "cursor";
    if (cwd.startsWith("codex-project:")) return "codex";
    // 默认假设是 Claude Code (本地路径格式)
    return "claude";
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
 * ProjectInfoSheet 组件
 * 显示项目的元信息
 * Story 1.12: 支持逻辑项目视图
 * Story 12.1: Task 1 - Dialog → Sheet 改造
 */
export function ProjectInfoSheet({
    isOpen,
    onOpenChange,
    project,
    getProjectSessions,
    logicalProject,
    getLogicalProjectSessions,
    onProjectUpdated,
}: ProjectInfoSheetProps) {
    const { t, i18n } = useTranslation();
    const navigate = useNavigate();
    const [sessions, setSessions] = React.useState<SessionSummary[]>([]);
    const [isLoading, setIsLoading] = React.useState(false);
    const [currentProject, setCurrentProject] = React.useState<Project | null>(project ?? null);
    const [isAddingPath, setIsAddingPath] = React.useState(false);
    // Story 1.12 Phase 7: 解除关联功能
    const [unlinkingProjectId, setUnlinkingProjectId] = React.useState<string | null>(null);
    const [linkedProjects, setLinkedProjects] = React.useState<Array<{ project: Project; pathId: string | null }>>([]);

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

    // 当 Sheet 打开时加载会话
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

    // Story 1.12 V10: 加载聚合项目列表（单源和多源都加载）
    React.useEffect(() => {
        if (!isOpen || !isLogicalView || !logicalProject) {
            setLinkedProjects([]);
            return;
        }

        // 获取所有关联此物理路径的项目及其 path_id
        const loadLinkedProjects = async () => {
            try {
                const projects = await getProjectsByPhysicalPath(logicalProject.physical_path);
                // 为每个项目查找对应的 path_id
                const projectsWithPathId = await Promise.all(
                    projects.map(async (proj) => {
                        const projectPaths = await getProjectPaths(proj.id);
                        const matchingPath = projectPaths.find(
                            (p: ProjectPath) => p.path === logicalProject.physical_path
                        );
                        return { project: proj, pathId: matchingPath?.id ?? null };
                    })
                );
                setLinkedProjects(projectsWithPathId);
            } catch (error) {
                console.error("Failed to load linked projects:", error);
                setLinkedProjects([]);
            }
        };

        loadLinkedProjects();
    }, [isOpen, isLogicalView, logicalProject]);

    /**
     * Story 1.12 Phase 7: 解除项目关联
     * 删除项目与当前物理路径的关联记录
     * 解除后，项目会恢复到其原始 cwd 路径作为独立逻辑项目显示
     */
    const handleUnlinkProject = async (projectId: string, pathId: string) => {
        try {
            setUnlinkingProjectId(projectId);

            // 获取被解除项目的信息，用于恢复其原始路径
            const projectToUnlink = linkedProjects.find((item) => item.project.id === projectId);
            const originalCwd = projectToUnlink?.project.cwd;

            // 删除当前关联
            await removeProjectPath(pathId);

            // 如果项目有原始 cwd 且与当前物理路径不同，确保它有自己的路径记录
            // 这样解除后项目会作为独立逻辑项目显示
            if (originalCwd && logicalProject && originalCwd !== logicalProject.physical_path) {
                try {
                    // 添加原始 cwd 作为主路径（如果还没有）
                    await addProjectPath(projectId, originalCwd, true);
                } catch {
                    // 如果路径已存在则忽略错误
                }
            }

            // 从本地状态移除
            setLinkedProjects((prev) => prev.filter((item) => item.project.id !== projectId));

            // 刷新数据并关闭详情页
            refetchPaths();
            onProjectUpdated?.();

            // 延迟关闭详情页，确保列表刷新完成
            setTimeout(() => {
                onOpenChange(false);
            }, 100);

            toast.success(t("projectInfo.projectUnlinked", "已解除关联"));
        } catch (error) {
            console.error("Failed to unlink project:", error);
            toast.error(
                t("projectInfo.unlinkFailed", "解除关联失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setUnlinkingProjectId(null);
        }
    };

    /**
     * Story 1.12 fix: 关联真实路径
     * 对于 needs_association 项目，选择目录后直接设置为主路径
     */
    const handleAssociatePath = async () => {
        const projectId = currentProject?.id ?? logicalProject?.project_ids[0];
        console.log("[DEBUG-SHEET] handleAssociatePath: Called. projectId:", projectId);

        if (!projectId) {
            console.warn("[DEBUG-SHEET] handleAssociatePath: projectId is missing, returning.");
            return;
        }

        try {
            console.log("[DEBUG-SHEET] handleAssociatePath: Opening dialog...");
            const selected = await open({
                directory: true,
                multiple: false,
                title: t("projectInfo.associatePath", "关联真实路径"),
            });
            console.log("[DEBUG-SHEET] handleAssociatePath: Dialog result:", selected);

            if (!selected || typeof selected !== "string") {
                console.log("[DEBUG-SHEET] handleAssociatePath: No selection or invalid type");
                return;
            }

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
            // 关联成功后关闭 Sheet
            onOpenChange(false);
        } catch (error) {
            console.error("[DEBUG-SHEET] Failed to associate path:", error);
            toast.error(
                t("projectInfo.associatePathFailed", "关联路径失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setIsAddingPath(false);
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
            <ActionSheet open={isOpen} onOpenChange={onOpenChange}>
                <ActionSheetContent size="lg" className="overflow-y-auto" data-testid="project-info-sheet">
                    <ActionSheetHeader>
                        <ActionSheetTitle className="flex items-center gap-2">
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
                        </ActionSheetTitle>
                        <ActionSheetDescription className="sr-only">
                            {t("projectInfo.description", "项目详细信息")}
                        </ActionSheetDescription>
                    </ActionSheetHeader>

                    <div className="divide-y divide-border px-4">
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
                                    {/* Story 1.12 V10: 移除添加路径按钮，一个项目只能关联一个路径 */}
                                </div>
                                {/* Story 1.12 V10: 简化路径显示，一个项目只能关联一个路径 */}
                                <div className="space-y-1">
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
                                    ) : (
                                        // 已关联：显示单一路径（主路径或 cwd）
                                        <div className="flex items-center gap-2">
                                            <TruncatedText
                                                text={
                                                    paths.find((p) => p.is_primary)?.path ??
                                                    paths[0]?.path ??
                                                    logicalProject?.physical_path ??
                                                    currentProject?.cwd ??
                                                    ""
                                                }
                                                maxLength={40}
                                                mono
                                                className="text-sm flex-1"
                                            />
                                            {/* 更换路径按钮 */}
                                            <Button
                                                variant="ghost"
                                                size="icon-sm"
                                                onClick={handleAssociatePath}
                                                disabled={isAddingPath}
                                                title={t("projectInfo.changePath", "更换路径")}
                                                className="shrink-0"
                                            >
                                                {isAddingPath ? (
                                                    <Loader2 className="h-4 w-4 animate-spin" />
                                                ) : (
                                                    <FolderEdit className="h-4 w-4" />
                                                )}
                                            </Button>
                                        </div>
                                    )}
                                </div>
                            </div>
                        </div>

                        {/* Story 1.12 V10: 聚合来源 - 显示所有来源及会话数（单源和多源都显示） */}
                        {isLogicalView && (
                            <div className="flex items-start gap-3 py-2">
                                <Hash className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
                                <div className="flex-1 min-w-0">
                                    <div className="text-xs text-muted-foreground mb-1">
                                        {t("projectInfo.aggregatedSources", "聚合来源")}
                                    </div>
                                    {isLoading ? (
                                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                                            <Loader2 className="h-3 w-3 animate-spin" />
                                            {t("common.loading", "加载中")}...
                                        </div>
                                    ) : linkedProjects.length > 0 ? (
                                        <div className="space-y-1">
                                            {linkedProjects.map(({ project: proj, pathId }) => {
                                                const source = inferSourceFromCwd(proj.cwd);
                                                return (
                                                <div
                                                    key={proj.id}
                                                    className="flex items-center gap-2 group"
                                                >
                                                    <SourceIcon source={source} className="h-4 w-4 shrink-0" />
                                                    <span className="text-sm flex-1 truncate">
                                                        {getSourceLabel(source)}
                                                    </span>
                                                    <span className="text-xs text-muted-foreground">
                                                        {proj.session_count} {t("projectInfo.sessions", "会话")}
                                                    </span>
                                                    {/* 多源时显示解除关联按钮 */}
                                                    {linkedProjects.length > 1 && pathId && (
                                                        <Button
                                                            variant="ghost"
                                                            size="icon-sm"
                                                            onClick={() => handleUnlinkProject(proj.id, pathId)}
                                                            disabled={unlinkingProjectId === proj.id}
                                                            title={t("projectInfo.unlinkProject", "解除关联")}
                                                            className="h-5 w-5 opacity-0 group-hover:opacity-100 transition-opacity text-destructive hover:text-destructive"
                                                        >
                                                            {unlinkingProjectId === proj.id ? (
                                                                <Loader2 className="h-3 w-3 animate-spin" />
                                                            ) : (
                                                                <Trash2 className="h-3 w-3" />
                                                            )}
                                                        </Button>
                                                    )}
                                                </div>
                                                );
                                            })}
                                        </div>
                                    ) : sources.length > 0 ? (
                                        // 单源项目：从会话统计显示
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
                                            {logicalProject?.total_sessions ?? 0} {t("projectInfo.sessions", "会话")}
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}

                        {/* 存储层视图：显示会话来源统计 */}
                        {!isLogicalView && currentProject && (
                            <div className="flex items-start gap-3 py-2">
                                <Hash className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
                                <div className="flex-1 min-w-0">
                                    <div className="text-xs text-muted-foreground mb-1">
                                        {t("projectInfo.aggregatedSources", "聚合来源")}
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
                                            {currentProject.session_count} {t("projectInfo.sessions", "会话")}
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}

                        {/* Git Remote URL (如果有) - Story 1.9 */}
                        {currentProject?.git_remote_url && (
                            <InfoRow
                                icon={Link2}
                                label={t("projectInfo.gitRemoteUrl", "Git 仓库 URL")}
                                value={currentProject.git_remote_url}
                                mono
                            />
                        )}

                        {/* Story 1.12 V10: 移除会话数量区域，信息已在"聚合来源"中显示 */}

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

                    {/* Story 11.9: MCP 上下文卡片 */}
                    {!isPlaceholder && (currentProject || logicalProject) && (
                        <>
                            <Separator className="my-4" />
                            <div className="px-4">
                                <McpContextCard
                                    projectId={currentProject?.id ?? logicalProject?.project_ids[0] ?? ""}
                                    projectPath={
                                        paths.find((p) => p.is_primary)?.path ??
                                        paths[0]?.path ??
                                        logicalProject?.physical_path ??
                                        currentProject?.cwd
                                    }
                                    onNavigateToHub={(projectId) => {
                                        onOpenChange(false); // 关闭 Sheet
                                        navigate(`/hub?project=${projectId}`);
                                    }}
                                />
                            </div>
                        </>
                    )}

                    {/* 无效 CWD 提示 */}
                    {isPlaceholder && (
                        <div className="mx-4 mt-4 p-3 rounded-md bg-yellow-500/10 border border-yellow-500/20">
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
                </ActionSheetContent>
            </ActionSheet>
        </TooltipProvider>
    );
}
