/**
 * useSanitizePreviewStore - 脱敏预览全局状态
 * Story 3.4: 主视图原生模式
 * Story 9.2: Task 5.6 (使用 IPC 适配器)
 *
 * 管理脱敏预览的全局状态，供 TopBarActions 和 CodePanel 共享
 */

import { create } from 'zustand';
import { invoke } from '@/lib/ipc-adapter';
import { sanitizeSession } from '@/lib/ipc/sanitizer-ipc';
import { useSanitizationRulesStore } from '@/stores/useSanitizationRulesStore';
import { useDetailPanelStore } from '@/stores/useDetailPanelStore';
import { feedback } from '@/lib/feedback';
import i18n from '@/i18n';
import type { MantraSession } from '@/lib/session-utils';
import type {
    SanitizeMode,
    SanitizationStats,
    SanitizationRule,
    SensitiveMatch,
    SensitiveType,
} from '@/components/sanitizer/types';

interface SanitizePreviewState {
    /** 预览模式: 'idle' = 正常状态, 'preview' = 脱敏预览模式 */
    mode: SanitizeMode;
    /** 是否正在加载 */
    isLoading: boolean;
    /** 原始文本 */
    originalText: string;
    /** 脱敏后文本 */
    sanitizedText: string;
    /** 脱敏统计 */
    stats: SanitizationStats;
    /** 错误信息 */
    error: string | null;
    /** 敏感信息匹配列表 (用于标签跳转) */
    sensitiveMatches: SensitiveMatch[];
    /** 当前会话 ID */
    sessionId: string | null;
}

interface SanitizePreviewActions {
    /** 设置当前会话 ID */
    setSessionId: (sessionId: string | null) => void;
    /** 进入预览模式 */
    enterPreviewMode: () => Promise<void>;
    /** 退出预览模式 */
    exitPreviewMode: () => void;
    /** 确认分享 (AC6) */
    confirmShare: () => Promise<void>;
    /** 根据类型获取第一个匹配的行号 */
    getFirstLineByType: (type: SensitiveType) => number | null;
    /** 重置状态 */
    reset: () => void;
}

type SanitizePreviewStore = SanitizePreviewState & SanitizePreviewActions;

const initialState: SanitizePreviewState = {
    mode: 'idle',
    isLoading: false,
    originalText: '',
    sanitizedText: '',
    stats: { counts: {}, total: 0 },
    error: null,
    sensitiveMatches: [],
    sessionId: null,
};

/**
 * 从原始文本和脱敏文本中解析敏感信息匹配列表
 */
function parseSensitiveMatches(
    originalText: string,
    sanitizedText: string
): SensitiveMatch[] {
    const matches: SensitiveMatch[] = [];
    const originalLines = originalText.split('\n');
    const sanitizedLines = sanitizedText.split('\n');

    const maxLines = Math.max(originalLines.length, sanitizedLines.length);

    for (let i = 0; i < maxLines; i++) {
        const origLine = originalLines[i] || '';
        const sanLine = sanitizedLines[i] || '';

        if (origLine !== sanLine) {
            let detectedType: SensitiveType = 'custom';

            if (sanLine.includes('[REDACTED:API_KEY]')) {
                detectedType = 'api_key';
            } else if (sanLine.includes('[REDACTED:AWS_KEY]')) {
                detectedType = 'aws_key';
            } else if (sanLine.includes('[REDACTED:GITHUB_TOKEN]')) {
                detectedType = 'github_token';
            } else if (sanLine.includes('[REDACTED:ANTHROPIC_KEY]')) {
                detectedType = 'anthropic_key';
            } else if (sanLine.includes('[REDACTED:GOOGLE_CLOUD_KEY]')) {
                detectedType = 'google_cloud_key';
            } else if (sanLine.includes('[REDACTED:IP]')) {
                detectedType = 'ip_address';
            } else if (sanLine.includes('[REDACTED:BEARER_TOKEN]')) {
                detectedType = 'bearer_token';
            } else if (sanLine.includes('[REDACTED:JWT]')) {
                detectedType = 'jwt_token';
            } else if (sanLine.includes('[REDACTED:SECRET]')) {
                detectedType = 'secret';
            } else if (sanLine.includes('[REDACTED:EMAIL]')) {
                detectedType = 'email';
            }

            const contextStart = Math.max(0, i - 2);
            const contextEnd = Math.min(originalLines.length - 1, i + 2);
            const contextLines = originalLines.slice(contextStart, contextEnd + 1);
            const context = contextLines.join('\n');

            const maskedOriginal = maskSensitiveContent(origLine);

            matches.push({
                id: `match-${i}`,
                type: detectedType,
                original: maskedOriginal,
                sanitized: sanLine,
                lineNumber: i + 1,
                context,
            });
        }
    }

    return matches;
}

function maskSensitiveContent(content: string): string {
    // 边界情况处理：短内容完全掩码
    if (content.length <= 4) {
        return '***';
    }
    if (content.length <= 8) {
        return content.slice(0, 1) + '***' + content.slice(-1);
    }
    if (content.length <= 12) {
        return content.slice(0, 2) + '***' + content.slice(-2);
    }
    return content.slice(0, 4) + '***' + content.slice(-4);
}

export const useSanitizePreviewStore = create<SanitizePreviewStore>((set, get) => ({
    ...initialState,

    setSessionId: (sessionId) => {
        set({ sessionId });
    },

    enterPreviewMode: async () => {
        const { sessionId } = get();

        if (!sessionId) {
            set({ error: '没有选中的会话' });
            return;
        }

        // Story 3.4: 自动切换到代码标签页
        useDetailPanelStore.getState().setActiveRightTab('code');

        set({ isLoading: true, error: null, mode: 'preview' });

        try {
            // 1. 获取原始会话内容
            const session = await invoke<MantraSession | null>('get_session', {
                sessionId,
            });

            if (!session) {
                throw new Error('会话不存在');
            }

            const original = JSON.stringify(session, null, 2);

            // 2. 获取自定义规则
            const getEnabledRules = useSanitizationRulesStore.getState().getEnabledRules;
            const enabledRules = getEnabledRules();
            const customPatterns: SanitizationRule[] = enabledRules.map((rule) => ({
                name: rule.name,
                pattern: rule.pattern,
                replacement: `[REDACTED:${rule.sensitiveType.toUpperCase()}]`,
            }));

            // 3. 调用脱敏 IPC
            const result = await sanitizeSession(sessionId, customPatterns);

            // 4. 解析敏感信息匹配列表
            const matches = parseSensitiveMatches(original, result.sanitized_text);

            set({
                originalText: original,
                sanitizedText: result.sanitized_text,
                stats: result.stats,
                sensitiveMatches: matches,
                isLoading: false,
            });
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : '脱敏预览失败';
            console.error('[SanitizePreview] Error:', err);
            // 显示错误 toast 给用户
            feedback.error(i18n.t('topbar.securityCheck'), errorMessage);
            set({
                error: errorMessage,
                mode: 'idle',
                isLoading: false,
            });
        }
    },

    exitPreviewMode: () => {
        set({
            mode: 'idle',
            originalText: '',
            sanitizedText: '',
            stats: { counts: {}, total: 0 },
            error: null,
            sensitiveMatches: [],
        });
    },

    confirmShare: async () => {
        const { sessionId, stats } = get();

        if (!sessionId) {
            feedback.error(
                i18n.t('sanitizer.confirmShare'),
                i18n.t('sanitizer.noSessionSelected', '没有选中的会话')
            );
            return;
        }

        try {
            // TODO: 调用后端分享 API
            // await invoke('share_session', { sessionId, sanitizedContent: sanitizedText });

            // AC6: 显示分享成功反馈
            const { toast } = await import('sonner');
            toast.success(
                stats.total > 0
                    ? i18n.t('sanitizer.shareSuccessWithSanitize', '已脱敏 {{count}} 处敏感信息并分享成功', { count: stats.total })
                    : i18n.t('sanitizer.shareSuccess', '分享成功')
            );

            // AC6: 退出脱敏预览模式
            get().exitPreviewMode();
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : i18n.t('sanitizer.shareFailed', '分享失败');
            console.error('[SanitizePreview] Share error:', err);
            feedback.error(i18n.t('sanitizer.confirmShare'), errorMessage);
        }
    },

    getFirstLineByType: (type) => {
        const { sensitiveMatches } = get();
        const match = sensitiveMatches.find((m) => m.type === type);
        return match ? match.lineNumber : null;
    },

    reset: () => {
        set(initialState);
    },
}));
