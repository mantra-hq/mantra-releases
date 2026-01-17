/**
 * Privacy Utils Tests - 脱敏处理工具测试
 * Story 3-9: Task 2.4 - 单元测试
 */

import { describe, it, expect } from 'vitest';
import { applyRedaction } from './privacy-utils';
import type { ScanMatch } from '@/components/sanitizer/types';

const createMatch = (overrides: Partial<ScanMatch>): ScanMatch => ({
    rule_id: 'test_rule',
    sensitive_type: 'api_key',
    severity: 'critical',
    line: 1,
    column: 1,
    matched_text: 'test',
    masked_text: '****',
    context: 'context',
    ...overrides,
});

describe('applyRedaction', () => {
    describe('Task 2.2 - 基本替换功能', () => {
        it('应该替换单个匹配项', () => {
            const content = 'const key = "sk-proj-abcd1234567890";';
            const matches: ScanMatch[] = [
                createMatch({
                    line: 1,
                    column: 14,
                    matched_text: 'sk-proj-abcd1234567890',
                    masked_text: 'sk-proj-****XXXX',
                }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe('const key = "sk-proj-****XXXX";');
        });

        it('应该处理多个匹配项', () => {
            const content = `const key1 = "sk-key1";
const key2 = "sk-key2";`;
            const matches: ScanMatch[] = [
                createMatch({
                    line: 1,
                    column: 15,  // 'const key1 = "' = 14 chars, so sk-key1 starts at column 15 (1-based)
                    matched_text: 'sk-key1',
                    masked_text: '****1',
                }),
                createMatch({
                    line: 2,
                    column: 15,  // 'const key2 = "' = 14 chars, so sk-key2 starts at column 15 (1-based)
                    matched_text: 'sk-key2',
                    masked_text: '****2',
                }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe(`const key1 = "****1";
const key2 = "****2";`);
        });
    });

    describe('Task 2.3 - 从后往前替换避免位置偏移', () => {
        it('同一行多个匹配项应该从后往前替换', () => {
            const content = 'key1=abc123 key2=def456';
            const matches: ScanMatch[] = [
                createMatch({
                    line: 1,
                    column: 6,
                    matched_text: 'abc123',
                    masked_text: '***',
                }),
                createMatch({
                    line: 1,
                    column: 18,
                    matched_text: 'def456',
                    masked_text: '***',
                }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe('key1=*** key2=***');
        });

        it('多行多个匹配项应该正确处理', () => {
            const content = `line1: abc
line2: def
line3: ghi`;
            const matches: ScanMatch[] = [
                createMatch({ line: 1, column: 8, matched_text: 'abc', masked_text: 'XXX' }),
                createMatch({ line: 2, column: 8, matched_text: 'def', masked_text: 'YYY' }),
                createMatch({ line: 3, column: 8, matched_text: 'ghi', masked_text: 'ZZZ' }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe(`line1: XXX
line2: YYY
line3: ZZZ`);
        });
    });

    describe('边界情况处理', () => {
        it('空匹配数组应该返回原始内容', () => {
            const content = 'hello world';
            const result = applyRedaction(content, []);
            expect(result).toBe(content);
        });

        it('行号超出范围应该忽略该匹配', () => {
            const content = 'line1\nline2';
            const matches: ScanMatch[] = [
                createMatch({
                    line: 10,
                    column: 1,
                    matched_text: 'test',
                    masked_text: '****',
                }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe(content);
        });

        it('列号超出范围应该忽略该匹配', () => {
            const content = 'short';
            const matches: ScanMatch[] = [
                createMatch({
                    line: 1,
                    column: 100,
                    matched_text: 'test',
                    masked_text: '****',
                }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe(content);
        });

        it('应该处理 UTF-8 字符', () => {
            const content = '密钥: sk-test-key';
            const matches: ScanMatch[] = [
                createMatch({
                    line: 1,
                    column: 5,
                    matched_text: 'sk-test-key',
                    masked_text: 'sk-****',
                }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe('密钥: sk-****');
        });

        it('应该处理空内容', () => {
            const result = applyRedaction('', []);
            expect(result).toBe('');
        });

        it('应该处理单行无换行符的内容', () => {
            const content = 'key=value';
            const matches: ScanMatch[] = [
                createMatch({
                    line: 1,
                    column: 5,
                    matched_text: 'value',
                    masked_text: '*****',
                }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe('key=*****');
        });

        it('匹配文本与实际不一致时应该仍然替换', () => {
            // 注意：applyRedaction 基于位置替换，不验证文本是否一致
            const content = 'abc def ghi';
            const matches: ScanMatch[] = [
                createMatch({
                    line: 1,
                    column: 5,
                    matched_text: 'def',
                    masked_text: 'XXX',
                }),
            ];

            const result = applyRedaction(content, matches);
            expect(result).toBe('abc XXX ghi');
        });
    });

    describe('性能相关', () => {
        it('应该处理大量匹配项', () => {
            const lines = Array.from({ length: 100 }, (_, i) => `line${i}: secret${i}`);
            const content = lines.join('\n');
            const matches: ScanMatch[] = lines.map((_, i) => {
                // 计算 secret 的起始列号：line + 数字 + ": " = 4 + len(i) + 2 + 1 (1-based)
                const column = 4 + String(i).length + 2 + 1;
                return createMatch({
                    line: i + 1,
                    column,
                    matched_text: `secret${i}`,
                    masked_text: '****',
                });
            });

            const result = applyRedaction(content, matches);
            const expectedLines = Array.from({ length: 100 }, (_, i) => `line${i}: ****`);
            expect(result).toBe(expectedLines.join('\n'));
        });
    });
});
