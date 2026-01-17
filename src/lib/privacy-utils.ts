/**
 * Privacy Utils - 脱敏处理工具函数
 * Story 3-9: Task 2 - AC #3
 *
 * 提供将扫描结果应用到内容的脱敏处理功能
 */

import type { ScanMatch } from '@/components/sanitizer/types';

/**
 * 脱敏处理选项
 */
export interface RedactionOptions {
    /**
     * 是否验证 matched_text 与实际内容一致
     * 如果不一致，会在控制台打印警告但仍然执行替换
     * @default false
     */
    validateContent?: boolean;
}

/**
 * 应用脱敏处理
 *
 * 根据 ScanMatch 的 matched_text 和 masked_text 进行替换。
 * 注意：从后往前替换，避免位置偏移问题。
 *
 * @param content 原始内容
 * @param matches 扫描匹配结果列表
 * @param options 可选配置
 * @returns 脱敏后的内容
 */
export function applyRedaction(
    content: string,
    matches: ScanMatch[],
    options: RedactionOptions = {}
): string {
    const { validateContent = false } = options;

    if (matches.length === 0) {
        return content;
    }

    // 按位置从后往前排序（先按行号降序，再按列号降序）
    const sortedMatches = [...matches].sort((a, b) => {
        if (b.line !== a.line) return b.line - a.line;
        return b.column - a.column;
    });

    const lines = content.split('\n');

    for (const match of sortedMatches) {
        const lineIndex = match.line - 1; // 行号从 1 开始转为 0-based

        // 跳过行号超出范围的匹配
        if (lineIndex < 0 || lineIndex >= lines.length) {
            continue;
        }

        const line = lines[lineIndex];
        const startCol = match.column - 1; // 列号从 1 开始转为 0-based

        // 跳过列号超出范围的匹配
        if (startCol < 0 || startCol >= line.length) {
            continue;
        }

        const endCol = startCol + match.matched_text.length;

        // 可选：验证内容是否匹配
        if (validateContent) {
            const actualText = line.substring(startCol, endCol);
            if (actualText !== match.matched_text) {
                console.warn(
                    `[applyRedaction] Content mismatch at line ${match.line}, col ${match.column}: ` +
                    `expected "${match.matched_text}", found "${actualText}". ` +
                    `The file may have been modified since scanning.`
                );
            }
        }

        // 替换匹配文本
        lines[lineIndex] =
            line.substring(0, startCol) +
            match.masked_text +
            line.substring(endCol);
    }

    return lines.join('\n');
}
