/**
 * ToolCallCard - 工具调用卡片组件
 * Story 2.15: Task 3, Task 7
 * Story 2.26: 国际化支持
 *
 * 单行紧凑布局显示工具调用，支持原位展开和查看详情
 * AC: #1, #2, #3, #4, #11, #12, #13
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import * as Collapsible from "@radix-ui/react-collapsible";
import {
    ChevronDown,
    ChevronUp,
    Wrench,
    FileText,
    Terminal,
    Search,
    FolderOpen,
    CheckCircle2,
    XCircle,
    Clock,
    ExternalLink,
    ListTodo,
    Edit3,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useCollapsible } from "@/hooks/useCollapsible";
import { FloatingCollapseBar } from "@/components/common/FloatingCollapseBar";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import i18n from "@/i18n";
import type { StandardTool } from "@/types/message";

/** 工具调用状态 */
export type ToolCallStatus = "pending" | "success" | "error";

/** 工具调用卡片属性 */
export interface ToolCallCardProps {
    /** 唯一 ID，用于配对链接 */
    toolUseId: string;
    /** 工具名称 */
    toolName: string;
    /** 工具输入参数 */
    toolInput?: Record<string, unknown>;
    /** 执行状态 */
    status?: ToolCallStatus;
    /** 执行耗时 (秒) */
    duration?: number;
    /** 是否当前高亮 (悬停配对时) */
    isHighlighted?: boolean;
    /** 悬停时回调 */
    onHover?: (toolUseId: string | null) => void;
    /** 点击卡片回调 (非按钮区域) */
    onClick?: (toolUseId: string) => void;
    /** 点击查看详情回调 */
    onViewDetail?: (toolUseId: string) => void;
    /** 跳转到配对输出回调 */
    onJumpToOutput?: (toolUseId: string) => void;
    /** 自定义 className */
    className?: string;
    // === Story 8.11: 新增字段 ===
    /** 工具显示名称 (优先于 toolName 显示) */
    displayName?: string;
    /** 工具描述 (悬停 Tooltip 显示) */
    description?: string;
    /** 标准化工具类型 */
    standardTool?: StandardTool;
}

/** 工具摘要模板配置 */
interface SummaryTemplate {
    icon: React.ComponentType<{ className?: string }>;
    format: (input: Record<string, unknown>) => string;
}

/** 工具摘要模板映射 */
const SUMMARY_TEMPLATES: Record<string, SummaryTemplate> = {
    read_file: {
        icon: FileText,
        format: (input) => {
            const path = extractPath(input);
            const lines = typeof input.lines === "number" ? input.lines : null;
            return lines ? `${path} (${i18n.t("message.lines", { count: lines })})` : path;
        },
    },
    Read: {
        icon: FileText,
        format: (input) => {
            const path = extractPath(input);
            return path;
        },
    },
    view_file: {
        icon: FileText,
        format: (input) => {
            const path = extractPath(input);
            return path;
        },
    },
    write_to_file: {
        icon: FileText,
        format: (input) => {
            const path = extractPath(input);
            const added = typeof input.added === "number" ? input.added : "?";
            const removed = typeof input.removed === "number" ? input.removed : "?";
            return `${path} (+${added}, -${removed})`;
        },
    },
    Write: {
        icon: FileText,
        format: (input) => {
            const path = extractPath(input);
            return path;
        },
    },
    Edit: {
        icon: FileText,
        format: (input) => {
            const path = extractPath(input);
            return path;
        },
    },
    replace_file_content: {
        icon: FileText,
        format: (input) => {
            const path = extractPath(input, "TargetFile");
            return `${path} (${i18n.t("message.edit")})`;
        },
    },
    run_command: {
        icon: Terminal,
        format: (input) => {
            const cmd = typeof input.command === "string"
                ? input.command
                : typeof input.CommandLine === "string"
                    ? input.CommandLine
                    : "";
            const truncated = cmd.length > 40 ? cmd.slice(0, 40) + "..." : cmd;
            const exitCode = typeof input.exit_code === "number" ? input.exit_code : null;
            return exitCode !== null ? `$ ${truncated} → ${exitCode}` : `$ ${truncated}`;
        },
    },
    bash: {
        icon: Terminal,
        format: (input) => {
            const cmd = typeof input.command === "string" ? input.command : "";
            const truncated = cmd.length > 40 ? cmd.slice(0, 40) + "..." : cmd;
            return truncated ? `$ ${truncated}` : "";
        },
    },
    Bash: {
        icon: Terminal,
        format: (input) => {
            const cmd = typeof input.command === "string" ? input.command : "";
            const truncated = cmd.length > 40 ? cmd.slice(0, 40) + "..." : cmd;
            return truncated ? `$ ${truncated}` : "";
        },
    },
    shell: {
        icon: Terminal,
        format: (input) => {
            // Codex CLI: command is an array of strings
            const cmdArray = Array.isArray(input.command) ? input.command : [];
            const cmd = cmdArray.join(" ");
            const truncated = cmd.length > 40 ? cmd.slice(0, 40) + "..." : cmd;
            return truncated ? `$ ${truncated}` : "";
        },
    },
    Shell: {
        icon: Terminal,
        format: (input) => {
            const cmdArray = Array.isArray(input.command) ? input.command : [];
            const cmd = cmdArray.join(" ");
            const truncated = cmd.length > 40 ? cmd.slice(0, 40) + "..." : cmd;
            return truncated ? `$ ${truncated}` : "";
        },
    },
    grep_search: {
        icon: Search,
        format: (input) => {
            const query = typeof input.query === "string"
                ? input.query
                : typeof input.Query === "string"
                    ? input.Query
                    : typeof input.pattern === "string"
                        ? input.pattern
                        : "";
            const count = typeof input.matches === "number" ? input.matches : null;
            return count !== null
                ? `"${query}" → ${i18n.t("message.results", { count })}`
                : query ? `"${query}"` : "";
        },
    },
    Grep: {
        icon: Search,
        format: (input) => {
            const pattern = typeof input.pattern === "string" ? input.pattern : "";
            return pattern ? `"${pattern}"` : "";
        },
    },
    find_by_name: {
        icon: Search,
        format: (input) => {
            const pattern = typeof input.pattern === "string"
                ? input.pattern
                : typeof input.Pattern === "string"
                    ? input.Pattern
                    : "";
            return pattern;
        },
    },
    Glob: {
        icon: Search,
        format: (input) => {
            const pattern = typeof input.pattern === "string" ? input.pattern : "";
            return pattern;
        },
    },
    list_dir: {
        icon: FolderOpen,
        format: (input) => {
            const path = extractPath(input, "DirectoryPath");
            const count = typeof input.count === "number" ? input.count : null;
            return count !== null
                ? `${path} (${i18n.t("message.items", { count })})`
                : path;
        },
    },
    TodoWrite: {
        icon: ListTodo,
        format: (input) => {
            const todos = Array.isArray(input.todos) ? input.todos : [];
            const count = todos.length;
            return count > 0 ? i18n.t("message.tasks", { count }) : "";
        },
    },
    update_plan: {
        icon: ListTodo,
        format: (input) => {
            // Codex CLI: plan is an array of { status, step }
            const plan = Array.isArray(input.plan) ? input.plan : [];
            const count = plan.length;
            return count > 0 ? i18n.t("message.tasks", { count }) : "";
        },
    },
};

/** 从输入中提取路径 */
function extractPath(
    input: Record<string, unknown>,
    key?: string
): string {
    const pathKeys = key ? [key] : ["path", "file_path", "filePath", "file", "AbsolutePath", "TargetFile"];
    for (const k of pathKeys) {
        if (typeof input[k] === "string") {
            const fullPath = input[k] as string;
            // 只显示文件名
            const parts = fullPath.split("/");
            return parts[parts.length - 1] || fullPath;
        }
    }
    return "";
}

/** 获取工具摘要 */
function getToolSummary(
    toolName: string,
    input?: Record<string, unknown>
): { icon: React.ComponentType<{ className?: string }>; summary: string } {
    const template = SUMMARY_TEMPLATES[toolName];
    if (template && input) {
        const summary = template.format(input);
        return {
            icon: template.icon,
            summary: summary,
        };
    }
    // 默认：无额外摘要
    return {
        icon: Wrench,
        summary: "",
    };
}

/**
 * Story 8.11: 使用 standardTool 生成更优摘要
 * 
 * 优先级:
 * 1. 使用 standardTool 的结构化信息
 * 2. 回退到现有 SUMMARY_TEMPLATES 逻辑
 */
function getToolSummaryWithStandardTool(
    toolName: string,
    input?: Record<string, unknown>,
    standardTool?: StandardTool
): { icon: React.ComponentType<{ className?: string }>; summary: string } {
    // 如果有 standardTool，优先使用它
    if (standardTool) {
        switch (standardTool.type) {
            case "file_read": {
                const fileName = getFileName(standardTool.path);
                const lineInfo = standardTool.start_line !== undefined && standardTool.end_line !== undefined
                    ? ` L${standardTool.start_line}-L${standardTool.end_line}`
                    : standardTool.start_line !== undefined
                        ? ` L${standardTool.start_line}`
                        : "";
                return { icon: FileText, summary: `${fileName}${lineInfo}` };
            }
            case "file_write": {
                const fileName = getFileName(standardTool.path);
                return { icon: FileText, summary: fileName };
            }
            case "file_edit": {
                const fileName = getFileName(standardTool.path);
                return { icon: Edit3, summary: `${fileName} (${i18n.t("message.edit")})` };
            }
            case "shell_exec": {
                const cmd = standardTool.command;
                const truncated = cmd.length > 50 ? cmd.slice(0, 50) + "..." : cmd;
                return { icon: Terminal, summary: `$ ${truncated}` };
            }
            case "file_search": {
                return { icon: Search, summary: standardTool.pattern };
            }
            case "content_search": {
                return { icon: Search, summary: `"${standardTool.pattern}"` };
            }
            case "other":
                // 回退到现有逻辑
                break;
        }
    }

    // 回退到现有的 SUMMARY_TEMPLATES 逻辑
    return getToolSummary(toolName, input);
}

/** 从路径中提取文件名 */
function getFileName(path: string): string {
    const parts = path.split("/");
    return parts[parts.length - 1] || path;
}

/** 格式化耗时 */
function formatDuration(seconds: number): string {
    if (seconds < 1) {
        return `${Math.round(seconds * 1000)}ms`;
    }
    return `${seconds.toFixed(1)}s`;
}

/**
 * ToolCallCard 组件
 *
 * 单行紧凑布局显示工具调用：
 * - 工具图标 + 工具名 + 智能摘要 + 状态 + 耗时 + 操作按钮
 * - 当摘要为空时只显示工具名
 * - 支持原位展开查看完整 JSON
 * - 提供"查看详情"按钮触发右侧面板
 */
export function ToolCallCard({
    toolUseId,
    toolName,
    toolInput,
    status = "pending",
    duration,
    isHighlighted = false,
    onHover,
    onClick,
    onViewDetail,
    onJumpToOutput,
    className,
    displayName,
    description,
    standardTool,
}: ToolCallCardProps) {
    const { t } = useTranslation();
    // Story 8.11: 使用 standardTool 生成更优摘要，回退到现有逻辑
    const { icon: Icon, summary } = getToolSummaryWithStandardTool(toolName, toolInput, standardTool);
    // Story 8.11: 优先显示 displayName
    const effectiveToolName = displayName || toolName;

    // AC #11, #12, #13: 使用 useCollapsible 管理折叠状态和浮动栏
    const {
        isExpanded,
        toggle,
        collapse,
        showFloatingBar,
        collapseButtonRef,
        contentRef,
        scrollToTop,
    } = useCollapsible();

    const handleMouseEnter = React.useCallback(() => {
        onHover?.(toolUseId);
    }, [onHover, toolUseId]);

    const handleMouseLeave = React.useCallback(() => {
        onHover?.(null);
    }, [onHover]);

    const handleClick = React.useCallback((e: React.MouseEvent) => {
        // 始终阻止冒泡，避免触发父组件的消息选中逻辑
        e.stopPropagation();
        onClick?.(toolUseId);
    }, [onClick, toolUseId]);

    const handleViewDetail = React.useCallback(
        (e: React.MouseEvent) => {
            e.stopPropagation();
            onViewDetail?.(toolUseId);
        },
        [onViewDetail, toolUseId]
    );

    const handleJumpToOutput = React.useCallback(
        (e: React.MouseEvent) => {
            e.stopPropagation();
            onJumpToOutput?.(toolUseId);
        },
        [onJumpToOutput, toolUseId]
    );

    const hasClickHandler = Boolean(onClick);
    const hasDetailHandler = Boolean(onViewDetail);
    const hasJumpHandler = Boolean(onJumpToOutput);
    const hasInput = toolInput && Object.keys(toolInput).length > 0;

    return (
        <div
            data-testid="tool-call-card"
            data-tool-use-id={toolUseId}
            className={cn(
                "rounded-lg border my-1.5 overflow-hidden transition-all duration-150",
                // 状态边框颜色
                status === "error"
                    ? "border-destructive bg-destructive/5"
                    : "border-border bg-muted/30",
                // 高亮状态
                isHighlighted && "ring-2 ring-primary/50",
                // 可点击样式
                hasClickHandler && "cursor-pointer hover:bg-muted/50",
                className
            )}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
            onClick={handleClick}
        >
            {/* 单行紧凑布局 */}
            <div className="flex items-center gap-2 px-3 py-2">
                {/* 工具图标 */}
                <Icon className="h-4 w-4 shrink-0 text-muted-foreground" />

                {/* 工具名称 + 可选 Tooltip (Story 8.11) */}
                {description ? (
                    <TooltipProvider>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <span className="font-medium text-sm text-foreground shrink-0 cursor-help underline decoration-dotted underline-offset-2">
                                    {effectiveToolName}
                                </span>
                            </TooltipTrigger>
                            <TooltipContent side="top" className="max-w-xs">
                                <p className="text-xs">{description}</p>
                            </TooltipContent>
                        </Tooltip>
                    </TooltipProvider>
                ) : (
                    <span className="font-medium text-sm text-foreground shrink-0">
                        {effectiveToolName}
                    </span>
                )}

                {/* 智能摘要 (当有值且不等于工具名时显示) */}
                {summary && summary !== toolName && (
                    <>
                        <span className="text-muted-foreground">·</span>
                        <span className="text-sm text-muted-foreground truncate">
                            {summary}
                        </span>
                    </>
                )}

                {/* 弹性占位 */}
                <div className="flex-1" />

                {/* 状态图标 */}
                {status === "success" && (
                    <CheckCircle2 className="h-3.5 w-3.5 text-green-500 shrink-0" />
                )}
                {status === "error" && (
                    <XCircle className="h-3.5 w-3.5 text-destructive shrink-0" />
                )}

                {/* 耗时 */}
                {duration !== undefined && (
                    <span className="flex items-center gap-0.5 text-xs text-muted-foreground shrink-0">
                        <Clock className="h-3 w-3" />
                        {formatDuration(duration)}
                    </span>
                )}

                {/* 展开/折叠按钮 */}
                {hasInput && (
                    <button
                        type="button"
                        onClick={(e) => {
                            e.stopPropagation();
                            toggle();
                        }}
                        className={cn(
                            "p-1 rounded cursor-pointer hover:bg-muted",
                            "text-muted-foreground hover:text-foreground",
                            "transition-colors"
                        )}
                        title={isExpanded ? t("common.collapse") : t("message.expandRawContent")}
                    >
                        {isExpanded ? (
                            <ChevronUp className="h-3.5 w-3.5" />
                        ) : (
                            <ChevronDown className="h-3.5 w-3.5" />
                        )}
                    </button>
                )}

                {/* 跳转到输出 */}
                {hasJumpHandler && (
                    <button
                        type="button"
                        onClick={handleJumpToOutput}
                        className={cn(
                            "p-1 rounded cursor-pointer hover:bg-muted",
                            "text-muted-foreground hover:text-foreground",
                            "transition-colors"
                        )}
                        title={t("message.jumpToOutput")}
                    >
                        <ExternalLink className="h-3.5 w-3.5" />
                    </button>
                )}

                {/* 查看详情 */}
                {hasDetailHandler && (
                    <button
                        type="button"
                        onClick={handleViewDetail}
                        className={cn(
                            "px-2 py-0.5 rounded text-xs cursor-pointer",
                            "bg-primary/10 text-primary",
                            "hover:bg-primary/20 transition-colors"
                        )}
                    >
                        {t("message.details")}
                    </button>
                )}
            </div>

            {/* 展开的原始内容 */}
            {hasInput && (
                <Collapsible.Root open={isExpanded} onOpenChange={toggle}>
                    <Collapsible.Content
                        className={cn(
                            "overflow-hidden",
                            "data-[state=open]:animate-collapsible-down",
                            "data-[state=closed]:animate-collapsible-up"
                        )}
                    >
                        <div
                            ref={contentRef}
                            className="border-t border-border bg-background px-3 py-2"
                        >
                            <pre className="font-mono text-xs whitespace-pre-wrap break-all text-muted-foreground max-h-60 overflow-auto">
                                {JSON.stringify(toolInput, null, 2)}
                            </pre>

                            {/* AC #11: 底部收起按钮 */}
                            <button
                                ref={collapseButtonRef}
                                type="button"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    collapse();
                                }}
                                className={cn(
                                    "mt-2 w-full flex items-center justify-center gap-1",
                                    "py-1.5 rounded text-xs",
                                    "text-muted-foreground hover:text-foreground",
                                    "hover:bg-muted/50 transition-colors"
                                )}
                            >
                                <ChevronUp className="h-3 w-3" />
                                {t("common.collapse")}
                            </button>
                        </div>
                    </Collapsible.Content>
                </Collapsible.Root>
            )}

            {/* AC #12: 浮动操作栏 */}
            <FloatingCollapseBar
                visible={showFloatingBar && isExpanded}
                onCollapse={collapse}
                onScrollToTop={scrollToTop}
            />
        </div>
    );
}

export default ToolCallCard;
