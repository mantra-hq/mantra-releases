/**
 * ProjectInfoDialog Component - 项目元信息对话框
 * Story 2.27: Task 1 - 项目元信息查看
 *
 * 展示项目详细信息：名称、路径、来源、会话数、创建时间等
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
} from "@/components/ui/dialog";
import {
    FolderOpen,
    Calendar,
    Clock,
    GitBranch,
    Hash,
    MapPin,
    Loader2,
} from "lucide-react";
import type { Project } from "@/types/project";
import type { SessionSummary } from "@/lib/project-ipc";
import { SourceIcon } from "@/components/import/SourceIcons";

/**
 * ProjectInfoDialog Props
 */
export interface ProjectInfoDialogProps {
    /** 是否打开 */
    isOpen: boolean;
    /** 打开状态变化回调 */
    onOpenChange: (open: boolean) => void;
    /** 项目信息 */
    project: Project | null;
    /** 获取项目会话列表 */
    getProjectSessions: (projectId: string) => Promise<SessionSummary[]>;
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
 * 信息行组件
 */
function InfoRow({
    icon: Icon,
    label,
    value,
    mono = false,
}: {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    value: string | React.ReactNode;
    mono?: boolean;
}) {
    return (
        <div className="flex items-start gap-3 py-2">
            <Icon className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
            <div className="flex-1 min-w-0">
                <div className="text-xs text-muted-foreground mb-0.5">{label}</div>
                <div
                    className={`text-sm ${mono ? "font-mono" : ""} break-all`}
                    title={typeof value === "string" ? value : undefined}
                >
                    {value}
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
 */
export function ProjectInfoDialog({
    isOpen,
    onOpenChange,
    project,
    getProjectSessions,
}: ProjectInfoDialogProps) {
    const { t, i18n } = useTranslation();
    const [sessions, setSessions] = React.useState<SessionSummary[]>([]);
    const [isLoading, setIsLoading] = React.useState(false);

    // 当对话框打开时加载会话
    React.useEffect(() => {
        if (isOpen && project) {
            setIsLoading(true);
            getProjectSessions(project.id)
                .then(setSessions)
                .catch(console.error)
                .finally(() => setIsLoading(false));
        } else {
            setSessions([]);
        }
    }, [isOpen, project, getProjectSessions]);

    if (!project) return null;

    const sourceCounts = countSessionsBySource(sessions);
    const sources = Object.entries(sourceCounts).sort((a, b) => b[1] - a[1]);

    return (
        <Dialog open={isOpen} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <FolderOpen className="h-5 w-5" />
                        {project.name}
                    </DialogTitle>
                    <DialogDescription className="sr-only">
                        {t("projectInfo.description", "项目详细信息")}
                    </DialogDescription>
                </DialogHeader>

                <div className="divide-y divide-border">
                    {/* 项目路径 */}
                    <InfoRow
                        icon={MapPin}
                        label={t("projectInfo.path", "项目路径")}
                        value={project.cwd}
                        mono
                    />

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
                                    {project.session_count}
                                </div>
                            )}
                        </div>
                    </div>

                    {/* 创建时间 */}
                    <InfoRow
                        icon={Calendar}
                        label={t("projectInfo.createdAt", "创建时间")}
                        value={formatDateTime(project.created_at, i18n.language)}
                    />

                    {/* 最后活动时间 */}
                    <InfoRow
                        icon={Clock}
                        label={t("projectInfo.lastActivity", "最后活动")}
                        value={formatDateTime(project.last_activity, i18n.language)}
                    />

                    {/* Git 仓库路径 */}
                    {project.has_git_repo && project.git_repo_path && (
                        <InfoRow
                            icon={GitBranch}
                            label={t("projectInfo.gitPath", "Git 仓库")}
                            value={project.git_repo_path}
                            mono
                        />
                    )}
                </div>
            </DialogContent>
        </Dialog>
    );
}

