/**
 * DiffModeToggle - Diff 模式切换组件
 * Story 2.13: 增强功能 - VSCode 风格 Diff 体验
 *
 * 提供 inline 和 side-by-side 两种 Diff 显示模式的切换
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import { SplitSquareHorizontal, AlignLeft } from "lucide-react";
import { useEditorStore, type DiffMode } from "@/stores/useEditorStore";

export interface DiffModeToggleProps {
    /** 自定义类名 */
    className?: string;
    /** 是否显示 (仅在有 previousCode 时显示) */
    visible?: boolean;
}

/**
 * Diff 模式切换按钮组
 *
 * 功能:
 * - inline: 行内差异模式 (类似 git diff --color-words)
 * - side-by-side: 并排对比模式 (双列视图)
 */
export function DiffModeToggle({ className, visible = true }: DiffModeToggleProps) {
    // 使用独立的选择器确保引用稳定
    const diffMode = useEditorStore((state) => state.diffMode);
    const setDiffMode = useEditorStore((state) => state.setDiffMode);

    if (!visible) return null;

    const modes: Array<{
        value: DiffMode;
        label: string;
        icon: React.ElementType;
        tooltip: string;
    }> = [
        {
            value: "inline",
            label: "行内",
            icon: AlignLeft,
            tooltip: "行内差异模式 (Inline Diff)",
        },
        {
            value: "side-by-side",
            label: "并排",
            icon: SplitSquareHorizontal,
            tooltip: "并排对比模式 (Side by Side)",
        },
    ];

    return (
        <div
            className={cn(
                "flex items-center gap-0.5 rounded-md border border-border bg-muted/50 p-0.5",
                className
            )}
            role="radiogroup"
            aria-label="Diff 显示模式"
        >
            {modes.map(({ value, icon: Icon, tooltip }) => (
                <Tooltip key={value}>
                    <TooltipTrigger asChild>
                        <Button
                            variant="ghost"
                            size="sm"
                            className={cn(
                                "h-6 w-6 p-0 rounded-sm",
                                diffMode === value
                                    ? "bg-background shadow-sm text-foreground"
                                    : "text-muted-foreground hover:text-foreground"
                            )}
                            onClick={() => setDiffMode(value)}
                            role="radio"
                            aria-checked={diffMode === value}
                        >
                            <Icon className="h-3.5 w-3.5" />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent side="bottom">
                        <p>{tooltip}</p>
                    </TooltipContent>
                </Tooltip>
            ))}
        </div>
    );
}

export default DiffModeToggle;
