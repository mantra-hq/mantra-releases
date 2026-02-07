/**
 * SkillRenderer - 技能调用渲染器
 *
 * 展示 Skill 调用的详情，包括技能名称、参数和执行结果。
 * 使用 Emerald 配色与普通工具区分。
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Zap, ChevronDown, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import type { StandardToolSkillInvoke } from "@/types/message";

export interface SkillRendererProps {
    /** 工具输出内容 */
    content: string;
    /** 标准化工具信息 */
    standardTool: StandardToolSkillInvoke;
    /** 是否错误 */
    isError?: boolean;
    /** 是否自适应高度填满父容器 */
    autoHeight?: boolean;
    /** 自定义 className */
    className?: string;
}

/**
 * SkillRenderer 组件
 */
export function SkillRenderer({
    content,
    standardTool,
    isError = false,
    autoHeight = true,
    className,
}: SkillRendererProps) {
    const { t } = useTranslation();
    const [isContentExpanded, setIsContentExpanded] = React.useState(false);

    // 内容过长时默认折叠
    const isLongContent = content.length > 500;
    const displayContent = !isLongContent || isContentExpanded
        ? content
        : content.slice(0, 500) + "...";

    return (
        <div
            data-testid="skill-renderer"
            className={cn(
                "flex flex-col gap-3 p-3",
                autoHeight ? "h-full min-h-0 overflow-auto" : "max-h-96 overflow-auto",
                className
            )}
        >
            {/* 技能头部 */}
            <div className="flex items-center gap-2 px-2 py-1.5 rounded-md bg-emerald-500/10 border border-emerald-500/20">
                <Zap className="h-4 w-4 text-emerald-500 shrink-0" />
                <span className="text-sm font-medium text-emerald-400">
                    {standardTool.skill}
                </span>
                {standardTool.args && (
                    <span className="text-xs text-muted-foreground ml-1 truncate">
                        {standardTool.args}
                    </span>
                )}
            </div>

            {/* 执行结果 */}
            <div className="flex flex-col gap-1">
                <span className="text-xs font-medium text-muted-foreground px-1">
                    {t("editor.outputResult")}
                </span>
                <pre
                    className={cn(
                        "font-mono text-xs whitespace-pre-wrap break-all p-3",
                        "bg-muted/30 rounded-md",
                        isError ? "text-destructive" : "text-foreground"
                    )}
                >
                    {displayContent}
                </pre>
                {isLongContent && (
                    <button
                        type="button"
                        onClick={() => setIsContentExpanded(!isContentExpanded)}
                        className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors px-1 self-start"
                    >
                        {isContentExpanded ? (
                            <>
                                <ChevronDown className="h-3 w-3" />
                                {t("common.collapse")}
                            </>
                        ) : (
                            <>
                                <ChevronRight className="h-3 w-3" />
                                {t("common.expand")}
                            </>
                        )}
                    </button>
                )}
            </div>
        </div>
    );
}

export default SkillRenderer;
