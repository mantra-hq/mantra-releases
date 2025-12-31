/**
 * Diff 计算工具函数 - Story 3-2
 * 使用 diff 库进行文本差异计算
 */

import { diffLines } from 'diff';
import type { DiffLine } from './types';

/**
 * 计算两段文本的行级差异
 * @param original 原始文本
 * @param sanitized 脱敏后文本
 * @returns DiffLine 数组
 */
export function computeDiff(original: string, sanitized: string): DiffLine[] {
    const changes = diffLines(original, sanitized);
    const result: DiffLine[] = [];
    let originalLineNum = 1;
    let sanitizedLineNum = 1;

    for (const change of changes) {
        // 分割成行，保留空行
        const lines = change.value.split('\n');
        // 如果最后一个元素是空字符串（由末尾换行产生），移除它
        if (lines.length > 0 && lines[lines.length - 1] === '') {
            lines.pop();
        }

        for (const line of lines) {
            if (change.added) {
                result.push({
                    type: 'added',
                    content: line,
                    lineNumber: { sanitized: sanitizedLineNum++ },
                });
            } else if (change.removed) {
                result.push({
                    type: 'removed',
                    content: line,
                    lineNumber: { original: originalLineNum++ },
                });
            } else {
                result.push({
                    type: 'unchanged',
                    content: line,
                    lineNumber: {
                        original: originalLineNum++,
                        sanitized: sanitizedLineNum++,
                    },
                });
            }
        }
    }

    return result;
}

/**
 * 检查两段文本是否有差异
 */
export function hasDifferences(original: string, sanitized: string): boolean {
    return original !== sanitized;
}

/**
 * 获取差异统计信息
 */
export function getDiffStats(diffLines: DiffLine[]): {
    added: number;
    removed: number;
    unchanged: number;
} {
    return diffLines.reduce(
        (acc, line) => {
            acc[line.type]++;
            return acc;
        },
        { added: 0, removed: 0, unchanged: 0 }
    );
}
