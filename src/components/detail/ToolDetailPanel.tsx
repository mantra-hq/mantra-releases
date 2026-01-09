/**
 * ToolDetailPanel - 工具详情面板组件
 * Story 2.15: Task 4
 * Story 2.26: 国际化支持
 * Story 8.12: Task 8 - 使用 standardTool 选择渲染器
 *
 * 在右侧面板显示完整的 Call + Output 详情
 * AC: #7, #10
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { X, Wrench, Clock, CheckCircle2, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { ToolOutputRenderer } from "./renderers";
import type { StandardTool } from "@/types/message";

/** 工具详情面板属性 */
export interface ToolDetailPanelProps {
    /** 工具名称 */
    toolName: string;
    /** 工具输入参数 */
    toolInput?: Record<string, unknown>;
    /** 工具输出内容 */
    toolOutput?: string;
    /** 是否错误输出 */
    isError?: boolean;
    /** 执行耗时 (秒) */
    duration?: number;
    /** 关闭回调 */
    onClose?: () => void;
    /** 自定义渲染器 */
    renderOutput?: (output: string, toolName: string) => React.ReactNode;
    /** 自定义 className */
    className?: string;
    /** Story 8.12: 标准化工具类型，用于选择渲染器 */
    standardTool?: StandardTool;
}

/** 格式化耗时 */
function formatDuration(seconds: number): string {
    if (seconds < 1) {
        return `${Math.round(seconds * 1000)}ms`;
    }
    return `${seconds.toFixed(1)}s`;
}

/**
 * ToolDetailPanel 组件
 *
 * 显示工具调用的完整详情：
 * - 工具名称和状态
 * - 完整的输入参数
 * - 完整的输出结果（根据工具类型使用不同渲染器）
 * - 失败状态显示红色边框
 */
export function ToolDetailPanel({
    toolName,
    toolInput,
    toolOutput,
    isError = false,
    duration,
    onClose,
    renderOutput,
    className,
    standardTool,
}: ToolDetailPanelProps) {
    const { t } = useTranslation();
    const hasInput = toolInput && Object.keys(toolInput).length > 0;
    const hasOutput = Boolean(toolOutput);

    return (
        <div
            data-testid="tool-detail-panel"
            className={cn(
                "flex flex-col h-full",
                "border-l border-border bg-background",
                // 错误状态红色边框
                isError && "border-l-destructive border-l-2",
                className
            )}
        >
            {/* 头部 */}
            <div
                className={cn(
                    "flex items-center gap-2 px-4 py-3",
                    "border-b border-border",
                    "shrink-0"
                )}
            >
                <Wrench className="h-4 w-4 text-muted-foreground" />
                <span className="font-medium text-sm truncate flex-1">{toolName}</span>

                {/* 耗时 */}
                {duration !== undefined && (
                    <span className="flex items-center gap-1 text-xs text-muted-foreground">
                        <Clock className="h-3 w-3" />
                        {formatDuration(duration)}
                    </span>
                )}

                {/* 状态图标 */}
                {isError ? (
                    <XCircle className="h-4 w-4 text-destructive" />
                ) : hasOutput ? (
                    <CheckCircle2 className="h-4 w-4 text-green-500" />
                ) : null}

                {/* 关闭按钮 */}
                {onClose && (
                    <button
                        type="button"
                        onClick={onClose}
                        className={cn(
                            "p-1 rounded hover:bg-muted",
                            "transition-colors"
                        )}
                        aria-label={t("editor.closeDetailPanel")}
                    >
                        <X className="h-4 w-4" />
                    </button>
                )}
            </div>

            {/* 内容区 */}
            <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
                {/* Call 参数 - 固定高度 */}
                {hasInput && (
                    <div className="border-b border-border shrink-0">
                        <div className="px-4 py-2 bg-muted/30">
                            <span className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                                {t("editor.inputParams")}
                            </span>
                        </div>
                        <div className="px-4 py-3 max-h-40 overflow-auto">
                            <pre className="font-mono text-xs whitespace-pre-wrap break-all text-foreground">
                                {JSON.stringify(toolInput, null, 2)}
                            </pre>
                        </div>
                    </div>
                )}

                {/* Output 结果 - 填满剩余空间，内部滚动 */}
                {hasOutput && (
                    <div className="flex-1 flex flex-col min-h-0">
                        <div className="px-4 py-2 bg-muted/30 shrink-0">
                            <span className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                                {t("editor.outputResult")}
                            </span>
                        </div>
                        <div className="flex-1 min-h-0 overflow-auto p-2">
                            {renderOutput ? (
                                renderOutput(toolOutput!, toolName)
                            ) : (
                                <ToolOutputRenderer
                                    content={toolOutput!}
                                    toolName={toolName}
                                    toolInput={toolInput}
                                    isError={isError}
                                    standardTool={standardTool}
                                />
                            )}
                        </div>
                    </div>
                )}

                {/* 无内容提示 */}
                {!hasInput && !hasOutput && (
                    <div className="flex items-center justify-center h-32 text-muted-foreground text-sm">
                        {t("editor.noDetailContent")}
                    </div>
                )}
            </div>
        </div>
    );
}

export default ToolDetailPanel;
