/**
 * DiffHighlighter - Diff 高亮工具测试
 * Story 2.7: Task 3 验证
 */

import { describe, it, expect } from "vitest";
import {
    computeDiffDecorations,
    toMonacoDecorations,
    computeDiffStats,
} from "./DiffHighlighter";

describe("DiffHighlighter", () => {
    describe("computeDiffDecorations", () => {
        it("应该处理 null 输入", () => {
            expect(computeDiffDecorations(null, "code")).toEqual([]);
            expect(computeDiffDecorations("code", null)).toEqual([]);
            expect(computeDiffDecorations(null, null)).toEqual([]);
        });

        it("应该处理相同代码 (无变化)", () => {
            const code = "const a = 1;\nconst b = 2;";
            expect(computeDiffDecorations(code, code)).toEqual([]);
        });

        it("应该检测新增行", () => {
            const oldCode = "line 1\nline 2";
            const newCode = "line 1\nline 2\nline 3";

            const decorations = computeDiffDecorations(oldCode, newCode);

            // diff 库可能返回多个装饰器
            expect(decorations.length).toBeGreaterThan(0);
            const hasAdded = decorations.some((d) => d.type === "added");
            expect(hasAdded).toBe(true);
        });

        it("应该检测删除行", () => {
            const oldCode = "line 1\nline 2\nline 3";
            const newCode = "line 1\nline 2";

            const decorations = computeDiffDecorations(oldCode, newCode);

            // diff 库可能返回多个装饰器
            expect(decorations.length).toBeGreaterThan(0);
            const hasRemoved = decorations.some((d) => d.type === "removed");
            expect(hasRemoved).toBe(true);
        });

        it("应该检测多处变化", () => {
            const oldCode = "line 1\nline 2\nline 3";
            const newCode = "line 1\nnew line\nline 3\nline 4";

            const decorations = computeDiffDecorations(oldCode, newCode);

            // 应该有删除和新增
            const addedCount = decorations.filter((d) => d.type === "added").length;
            const removedCount = decorations.filter((d) => d.type === "removed").length;

            expect(addedCount).toBeGreaterThan(0);
            expect(removedCount).toBeGreaterThan(0);
        });
    });

    describe("toMonacoDecorations", () => {
        it("应该转换 added 装饰器", () => {
            const decorations = [
                { startLineNumber: 1, endLineNumber: 2, type: "added" as const },
            ];

            const monaco = toMonacoDecorations(decorations);

            expect(monaco).toHaveLength(1);
            expect(monaco[0].range.startLineNumber).toBe(1);
            expect(monaco[0].range.endLineNumber).toBe(2);
            expect(monaco[0].options.className).toBe("diff-line-added");
            expect(monaco[0].options.glyphMarginClassName).toBe("diff-glyph-added");
            expect(monaco[0].options.isWholeLine).toBe(true);
        });

        it("应该转换 removed 装饰器", () => {
            const decorations = [
                { startLineNumber: 5, endLineNumber: 5, type: "removed" as const },
            ];

            const monaco = toMonacoDecorations(decorations);

            expect(monaco).toHaveLength(1);
            expect(monaco[0].options.className).toBe("diff-line-removed");
            expect(monaco[0].options.glyphMarginClassName).toBe("diff-glyph-removed");
        });
    });

    describe("computeDiffStats", () => {
        it("应该处理 null 输入", () => {
            expect(computeDiffStats(null, "code")).toEqual({
                additions: 0,
                deletions: 0,
                hasChanges: false,
            });
        });

        it("应该返回无变化的统计", () => {
            const code = "line 1\nline 2";
            const stats = computeDiffStats(code, code);

            expect(stats.additions).toBe(0);
            expect(stats.deletions).toBe(0);
            expect(stats.hasChanges).toBe(false);
        });

        it("应该计算新增行数", () => {
            const oldCode = "line 1\n";
            const newCode = "line 1\nline 2\nline 3\n";

            const stats = computeDiffStats(oldCode, newCode);

            // diff 库将 "line 2\nline 3\n" 视为 2 行
            expect(stats.additions).toBeGreaterThan(0);
            expect(stats.hasChanges).toBe(true);
        });

        it("应该计算删除行数", () => {
            const oldCode = "line 1\nline 2\nline 3\n";
            const newCode = "line 1\n";

            const stats = computeDiffStats(oldCode, newCode);

            expect(stats.deletions).toBeGreaterThan(0);
            expect(stats.hasChanges).toBe(true);
        });

        it("应该计算混合变化", () => {
            const oldCode = "line 1\nline 2";
            const newCode = "line 1\nnew line\nnew line 2";

            const stats = computeDiffStats(oldCode, newCode);

            expect(stats.hasChanges).toBe(true);
            expect(stats.additions + stats.deletions).toBeGreaterThan(0);
        });
    });
});
