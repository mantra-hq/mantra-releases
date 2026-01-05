/**
 * useSanitizePreviewStore 测试
 * Story 3-4: Task 7
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useSanitizePreviewStore } from './useSanitizePreviewStore';

// Mock tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

// Mock sanitizer-ipc
vi.mock('@/lib/ipc/sanitizer-ipc', () => ({
    sanitizeSession: vi.fn(),
}));

// Mock useSanitizationRulesStore
vi.mock('@/stores/useSanitizationRulesStore', () => ({
    useSanitizationRulesStore: {
        getState: () => ({
            getEnabledRules: () => [],
        }),
    },
}));

// Mock useDetailPanelStore
vi.mock('@/stores/useDetailPanelStore', () => ({
    useDetailPanelStore: {
        getState: () => ({
            setActiveRightTab: vi.fn(),
        }),
    },
}));

// Mock sonner
vi.mock('sonner', () => ({
    toast: {
        success: vi.fn(),
        error: vi.fn(),
    },
}));

// Mock feedback
vi.mock('@/lib/feedback', () => ({
    feedback: {
        error: vi.fn(),
    },
}));

// Mock i18n
vi.mock('@/i18n', () => ({
    default: {
        t: (key: string, fallback?: string) => fallback || key,
    },
}));

describe('useSanitizePreviewStore', () => {
    beforeEach(() => {
        // 重置 store 状态
        useSanitizePreviewStore.getState().reset();
        vi.clearAllMocks();
    });

    describe('初始状态', () => {
        it('应该有正确的初始状态', () => {
            const state = useSanitizePreviewStore.getState();

            expect(state.mode).toBe('idle');
            expect(state.isLoading).toBe(false);
            expect(state.originalText).toBe('');
            expect(state.sanitizedText).toBe('');
            expect(state.stats).toEqual({ counts: {}, total: 0 });
            expect(state.error).toBeNull();
            expect(state.sensitiveMatches).toEqual([]);
            expect(state.sessionId).toBeNull();
        });
    });

    describe('setSessionId', () => {
        it('应该设置 sessionId', () => {
            useSanitizePreviewStore.getState().setSessionId('test-session-123');

            expect(useSanitizePreviewStore.getState().sessionId).toBe('test-session-123');
        });

        it('应该能清除 sessionId', () => {
            useSanitizePreviewStore.getState().setSessionId('test-session-123');
            useSanitizePreviewStore.getState().setSessionId(null);

            expect(useSanitizePreviewStore.getState().sessionId).toBeNull();
        });
    });

    describe('exitPreviewMode', () => {
        it('应该重置预览相关状态', () => {
            // 设置一些状态
            useSanitizePreviewStore.setState({
                mode: 'preview',
                originalText: 'original',
                sanitizedText: 'sanitized',
                stats: { counts: { api_key: 1 }, total: 1 },
                sensitiveMatches: [{ id: '1', type: 'api_key', original: 'test', sanitized: '[REDACTED]', lineNumber: 1, context: '' }],
            });

            useSanitizePreviewStore.getState().exitPreviewMode();

            const state = useSanitizePreviewStore.getState();
            expect(state.mode).toBe('idle');
            expect(state.originalText).toBe('');
            expect(state.sanitizedText).toBe('');
            expect(state.stats).toEqual({ counts: {}, total: 0 });
            expect(state.sensitiveMatches).toEqual([]);
        });
    });

    describe('getFirstLineByType', () => {
        it('应该返回指定类型的第一个匹配行号', () => {
            useSanitizePreviewStore.setState({
                sensitiveMatches: [
                    { id: '1', type: 'api_key', original: 'test1', sanitized: '[REDACTED]', lineNumber: 5, context: '' },
                    { id: '2', type: 'api_key', original: 'test2', sanitized: '[REDACTED]', lineNumber: 10, context: '' },
                    { id: '3', type: 'ip_address', original: 'test3', sanitized: '[REDACTED]', lineNumber: 15, context: '' },
                ],
            });

            expect(useSanitizePreviewStore.getState().getFirstLineByType('api_key')).toBe(5);
            expect(useSanitizePreviewStore.getState().getFirstLineByType('ip_address')).toBe(15);
        });

        it('没有匹配时应该返回 null', () => {
            useSanitizePreviewStore.setState({
                sensitiveMatches: [
                    { id: '1', type: 'api_key', original: 'test', sanitized: '[REDACTED]', lineNumber: 5, context: '' },
                ],
            });

            expect(useSanitizePreviewStore.getState().getFirstLineByType('ip_address')).toBeNull();
        });
    });

    describe('reset', () => {
        it('应该重置所有状态到初始值', () => {
            useSanitizePreviewStore.setState({
                mode: 'preview',
                isLoading: true,
                originalText: 'original',
                sanitizedText: 'sanitized',
                stats: { counts: { api_key: 1 }, total: 1 },
                error: 'some error',
                sensitiveMatches: [{ id: '1', type: 'api_key', original: 'test', sanitized: '[REDACTED]', lineNumber: 1, context: '' }],
                sessionId: 'test-session',
            });

            useSanitizePreviewStore.getState().reset();

            const state = useSanitizePreviewStore.getState();
            expect(state.mode).toBe('idle');
            expect(state.isLoading).toBe(false);
            expect(state.originalText).toBe('');
            expect(state.sanitizedText).toBe('');
            expect(state.stats).toEqual({ counts: {}, total: 0 });
            expect(state.error).toBeNull();
            expect(state.sensitiveMatches).toEqual([]);
            expect(state.sessionId).toBeNull();
        });
    });

    describe('enterPreviewMode', () => {
        it('没有 sessionId 时应该设置错误', async () => {
            useSanitizePreviewStore.setState({ sessionId: null });

            await useSanitizePreviewStore.getState().enterPreviewMode();

            expect(useSanitizePreviewStore.getState().error).toBe('没有选中的会话');
        });
    });

    describe('confirmShare', () => {
        it('没有 sessionId 时应该显示错误 toast', async () => {
            const { feedback } = await import('@/lib/feedback');
            useSanitizePreviewStore.setState({ sessionId: null });

            await useSanitizePreviewStore.getState().confirmShare();

            expect(feedback.error).toHaveBeenCalled();
        });

        it('有 sessionId 时应该显示成功 toast 并退出预览', async () => {
            const { toast } = await import('sonner');
            useSanitizePreviewStore.setState({
                sessionId: 'test-session',
                mode: 'preview',
                stats: { counts: {}, total: 0 },
            });

            await useSanitizePreviewStore.getState().confirmShare();

            expect(toast.success).toHaveBeenCalled();
            expect(useSanitizePreviewStore.getState().mode).toBe('idle');
        });
    });
});
