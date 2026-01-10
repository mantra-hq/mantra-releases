/**
 * sanitizer-ipc 单元测试 - Story 3-2 Task 8.4
 * Story 9.2: 更新 mock 为 IPC 适配器
 *
 * 测试 IPC 封装函数的类型安全和行为
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock IPC 适配器
vi.mock('@/lib/ipc-adapter', () => ({
    invoke: vi.fn(),
}));

import { invoke } from '@/lib/ipc-adapter';
import {
    sanitizeText,
    sanitizeSession,
    createEmptyStats,
    hasChanges,
    getBuiltinRules,
} from './sanitizer-ipc';
import type { SanitizationResult } from '@/components/sanitizer/types';

const mockResult: SanitizationResult = {
    sanitized_text: 'const key = "[REDACTED:API_KEY]";',
    stats: {
        counts: { api_key: 1 },
        total: 1,
    },
    has_matches: true,
};

const mockEmptyResult: SanitizationResult = {
    sanitized_text: 'Hello, World!',
    stats: {
        counts: {},
        total: 0,
    },
    has_matches: false,
};

describe('sanitizer-ipc', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('sanitizeText', () => {
        it('应该调用正确的 IPC 命令和参数', async () => {
            vi.mocked(invoke).mockResolvedValue(mockResult);

            const text = 'const key = "sk-1234567890";';
            await sanitizeText(text);

            expect(invoke).toHaveBeenCalledWith('sanitize_text', {
                text,
                custom_patterns: [],
            });
        });

        it('应该传递自定义规则', async () => {
            vi.mocked(invoke).mockResolvedValue(mockResult);

            const text = 'Phone: 123-456-7890';
            const customPatterns = [
                { name: 'Phone', pattern: '\\d{3}-\\d{3}-\\d{4}', replacement: '[PHONE]' },
            ];

            await sanitizeText(text, customPatterns);

            expect(invoke).toHaveBeenCalledWith('sanitize_text', {
                text,
                custom_patterns: customPatterns,
            });
        });

        it('应该返回脱敏结果', async () => {
            vi.mocked(invoke).mockResolvedValue(mockResult);

            const result = await sanitizeText('test');

            expect(result).toEqual(mockResult);
            expect(result.has_matches).toBe(true);
            expect(result.stats.total).toBe(1);
        });

        it('应该处理无匹配情况', async () => {
            vi.mocked(invoke).mockResolvedValue(mockEmptyResult);

            const result = await sanitizeText('Hello, World!');

            expect(result.has_matches).toBe(false);
            expect(result.stats.total).toBe(0);
        });
    });

    describe('sanitizeSession', () => {
        it('应该调用正确的 IPC 命令和参数', async () => {
            vi.mocked(invoke).mockResolvedValue(mockResult);

            const sessionId = 'session-123';
            await sanitizeSession(sessionId);

            expect(invoke).toHaveBeenCalledWith('sanitize_session', {
                sessionId,
                custom_patterns: [],
            });
        });

        it('应该传递自定义规则', async () => {
            vi.mocked(invoke).mockResolvedValue(mockResult);

            const sessionId = 'session-456';
            const customPatterns = [
                { name: 'Secret', pattern: 'secret_\\w+', replacement: '[SECRET]' },
            ];

            await sanitizeSession(sessionId, customPatterns);

            expect(invoke).toHaveBeenCalledWith('sanitize_session', {
                sessionId,
                custom_patterns: customPatterns,
            });
        });
    });

    describe('createEmptyStats', () => {
        it('应该返回空的统计对象', () => {
            const stats = createEmptyStats();

            expect(stats).toEqual({
                counts: {},
                total: 0,
            });
        });

        it('每次调用应该返回新对象', () => {
            const stats1 = createEmptyStats();
            const stats2 = createEmptyStats();

            expect(stats1).not.toBe(stats2);
            expect(stats1).toEqual(stats2);
        });
    });

    describe('hasChanges', () => {
        it('有匹配时应该返回 true', () => {
            expect(hasChanges(mockResult)).toBe(true);
        });

        it('无匹配时应该返回 false', () => {
            expect(hasChanges(mockEmptyResult)).toBe(false);
        });
    });

    // Story 3-5: Task 2.2 - getBuiltinRules IPC 测试
    describe('getBuiltinRules', () => {
        it('应该调用正确的 IPC 命令', async () => {
            const mockBuiltinRules = [
                { name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key' },
                { name: 'GitHub Token', pattern: 'ghp_[a-zA-Z0-9]+', sensitive_type: 'github_token' },
            ];
            vi.mocked(invoke).mockResolvedValue(mockBuiltinRules);

            await getBuiltinRules();

            expect(invoke).toHaveBeenCalledWith('get_builtin_rules');
        });

        it('应该返回内置规则列表', async () => {
            const mockBuiltinRules = [
                { name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key' },
            ];
            vi.mocked(invoke).mockResolvedValue(mockBuiltinRules);

            const result = await getBuiltinRules();

            expect(result).toEqual(mockBuiltinRules);
            expect(result.length).toBe(1);
            expect(result[0].name).toBe('OpenAI API Key');
        });
    });
});
