/**
 * DiffHighlighter - Diff 高亮工具
 * Story 2.7: Task 3 - AC #5
 *
 * 功能:
 * - 计算两段代码的行级差异
 * - 生成 Monaco Editor 装饰器
 * - 支持新增行 (绿色) 和删除行 (红色) 高亮
 * - 自动淡出 (3 秒后渐隐)
 */

import { diffLines } from "diff";

/**
 * Diff 装饰器类型
 */
export type DiffDecorationType = "added" | "removed";

/**
 * Diff 装饰器信息
 */
export interface DiffDecoration {
    /** 起始行 (1-indexed) */
    startLineNumber: number;
    /** 结束行 (1-indexed) */
    endLineNumber: number;
    /** 装饰类型 */
    type: DiffDecorationType;
}

/**
 * Monaco 装饰器选项
 */
export interface MonacoDecorationOptions {
    /** 是否整行 */
    isWholeLine: boolean;
    /** 行 CSS 类名 */
    className: string;
    /** Glyph margin CSS 类名 */
    glyphMarginClassName: string;
    /** Glyph margin 悬停消息 */
    glyphMarginHoverMessage?: { value: string };
}

/**
 * 计算代码差异并生成装饰器信息
 *
 * @param oldCode - 旧代码内容
 * @param newCode - 新代码内容
 * @returns Diff 装饰器列表
 */
export function computeDiffDecorations(
    oldCode: string | null,
    newCode: string | null
): DiffDecoration[] {
    if (!oldCode || !newCode) return [];
    if (oldCode === newCode) return [];

    const changes = diffLines(oldCode, newCode);
    const decorations: DiffDecoration[] = [];
    let lineNumber = 1;

    for (const change of changes) {
        const lineCount = change.count || 0;

        if (change.added) {
            decorations.push({
                startLineNumber: lineNumber,
                endLineNumber: lineNumber + lineCount - 1,
                type: "added",
            });
            lineNumber += lineCount;
        } else if (change.removed) {
            // 删除行标记在当前位置 (会显示为红色标记)
            decorations.push({
                startLineNumber: lineNumber,
                endLineNumber: lineNumber,
                type: "removed",
            });
            // 删除行不增加行号
        } else {
            // 未变化的行
            lineNumber += lineCount;
        }
    }

    return decorations;
}

/**
 * 将 DiffDecoration 转换为 Monaco Editor 装饰器格式
 *
 * @param decorations - Diff 装饰器列表
 * @returns Monaco 装饰器配置数组
 */
export function toMonacoDecorations(
    decorations: DiffDecoration[]
): Array<{
    range: {
        startLineNumber: number;
        startColumn: number;
        endLineNumber: number;
        endColumn: number;
    };
    options: MonacoDecorationOptions;
}> {
    return decorations.map((decoration) => ({
        range: {
            startLineNumber: decoration.startLineNumber,
            startColumn: 1,
            endLineNumber: decoration.endLineNumber,
            endColumn: 1,
        },
        options: getDecorationOptions(decoration.type),
    }));
}

/**
 * 获取装饰器选项
 */
function getDecorationOptions(type: DiffDecorationType): MonacoDecorationOptions {
    if (type === "added") {
        return {
            isWholeLine: true,
            className: "diff-line-added",
            glyphMarginClassName: "diff-glyph-added",
            glyphMarginHoverMessage: { value: "新增行" },
        };
    } else {
        return {
            isWholeLine: true,
            className: "diff-line-removed",
            glyphMarginClassName: "diff-glyph-removed",
            glyphMarginHoverMessage: { value: "删除行" },
        };
    }
}

/**
 * 简化的 Diff 统计
 */
export interface DiffStats {
    /** 新增行数 */
    additions: number;
    /** 删除行数 */
    deletions: number;
    /** 是否有变化 */
    hasChanges: boolean;
}

/**
 * 计算 Diff 统计信息
 *
 * @param oldCode - 旧代码
 * @param newCode - 新代码
 * @returns Diff 统计
 */
export function computeDiffStats(
    oldCode: string | null,
    newCode: string | null
): DiffStats {
    if (!oldCode || !newCode) {
        return { additions: 0, deletions: 0, hasChanges: false };
    }
    if (oldCode === newCode) {
        return { additions: 0, deletions: 0, hasChanges: false };
    }

    const changes = diffLines(oldCode, newCode);
    let additions = 0;
    let deletions = 0;

    for (const change of changes) {
        if (change.added) {
            additions += change.count || 0;
        } else if (change.removed) {
            deletions += change.count || 0;
        }
    }

    return {
        additions,
        deletions,
        hasChanges: additions > 0 || deletions > 0,
    };
}

// Re-export useDiffFadeOut from hooks for backwards compatibility
export { useDiffFadeOut, type UseDiffFadeOutResult } from "@/hooks/useDiffFadeOut";

export default computeDiffDecorations;
