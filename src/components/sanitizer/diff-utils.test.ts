/**
 * diff-utils 单元测试 - Story 3-2 Task 9.2
 */

import { describe, it, expect } from 'vitest';
import { computeDiff, hasDifferences, getDiffStats } from './diff-utils';

describe('diff-utils', () => {
    describe('computeDiff', () => {
        it('应该正确识别新增行', () => {
            const original = 'line1\nline2';
            const sanitized = 'line1\nline2\nline3';

            const result = computeDiff(original, sanitized);

            expect(result.some((line) => line.type === 'added' && line.content === 'line3')).toBe(true);
        });

        it('应该正确识别删除行', () => {
            const original = 'line1\nline2\nline3';
            const sanitized = 'line1\nline2';

            const result = computeDiff(original, sanitized);

            expect(result.some((line) => line.type === 'removed' && line.content === 'line3')).toBe(true);
        });

        it('应该正确识别未变更行', () => {
            const original = 'line1\nline2';
            const sanitized = 'line1\nline2';

            const result = computeDiff(original, sanitized);

            expect(result.every((line) => line.type === 'unchanged')).toBe(true);
        });

        it('应该正确计算行号', () => {
            const original = 'line1\noldline\nline3';
            const sanitized = 'line1\nnewline\nline3';

            const result = computeDiff(original, sanitized);

            // 未变行应该有两个行号
            const unchangedLines = result.filter((l) => l.type === 'unchanged');
            unchangedLines.forEach((line) => {
                expect(line.lineNumber.original).toBeDefined();
                expect(line.lineNumber.sanitized).toBeDefined();
            });

            // 删除行只有原始行号
            const removedLines = result.filter((l) => l.type === 'removed');
            removedLines.forEach((line) => {
                expect(line.lineNumber.original).toBeDefined();
                expect(line.lineNumber.sanitized).toBeUndefined();
            });

            // 新增行只有脱敏后行号
            const addedLines = result.filter((l) => l.type === 'added');
            addedLines.forEach((line) => {
                expect(line.lineNumber.original).toBeUndefined();
                expect(line.lineNumber.sanitized).toBeDefined();
            });
        });

        it('应该正确处理敏感信息替换场景', () => {
            const original = 'const API_KEY = "sk-1234567890";\nconst host = "192.168.1.100";';
            const sanitized = 'const API_KEY = "[REDACTED:API_KEY]";\nconst host = "[REDACTED:IP_ADDRESS]";';

            const result = computeDiff(original, sanitized);

            // 应该有删除行和新增行
            expect(result.some((line) => line.type === 'removed')).toBe(true);
            expect(result.some((line) => line.type === 'added')).toBe(true);
        });

        it('应该正确处理空文本', () => {
            const result = computeDiff('', '');
            expect(result).toEqual([]);
        });

        it('应该正确处理单行文本', () => {
            const result = computeDiff('hello', 'hello');
            expect(result).toHaveLength(1);
            expect(result[0].type).toBe('unchanged');
            expect(result[0].content).toBe('hello');
        });
    });

    describe('hasDifferences', () => {
        it('相同文本应该返回 false', () => {
            expect(hasDifferences('hello', 'hello')).toBe(false);
        });

        it('不同文本应该返回 true', () => {
            expect(hasDifferences('hello', 'world')).toBe(true);
        });

        it('空文本比较应该返回 false', () => {
            expect(hasDifferences('', '')).toBe(false);
        });
    });

    describe('getDiffStats', () => {
        it('应该正确统计各类型行数', () => {
            const original = 'line1\noldline\nline3';
            const sanitized = 'line1\nnewline\nline3';
            const diffLines = computeDiff(original, sanitized);

            const stats = getDiffStats(diffLines);

            expect(stats.unchanged).toBeGreaterThan(0);
            expect(stats.added).toBeGreaterThan(0);
            expect(stats.removed).toBeGreaterThan(0);
        });

        it('无变更时应该全部是 unchanged', () => {
            const diffLines = computeDiff('a\nb\nc', 'a\nb\nc');
            const stats = getDiffStats(diffLines);

            expect(stats.unchanged).toBe(3);
            expect(stats.added).toBe(0);
            expect(stats.removed).toBe(0);
        });
    });
});
