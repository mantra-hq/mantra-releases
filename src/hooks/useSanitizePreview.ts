/**
 * useSanitizePreview Hook - 会话脱敏预览
 * 管理脱敏预览 Modal 的状态和数据获取
 */

import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { sanitizeSession } from '@/lib/ipc/sanitizer-ipc';
import { useSanitizationRulesStore } from '@/stores/useSanitizationRulesStore';
import type { MantraSession } from '@/lib/session-utils';
import type { SanitizationStats, SanitizationRule } from '@/components/sanitizer/types';

export interface UseSanitizePreviewResult {
    /** Modal 是否打开 */
    isOpen: boolean;
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
    /** 打开预览 */
    openPreview: () => Promise<void>;
    /** 关闭预览 */
    closePreview: () => void;
}

/**
 * useSanitizePreview Hook
 * @param sessionId - 会话 ID
 */
export function useSanitizePreview(sessionId: string | null): UseSanitizePreviewResult {
    const [isOpen, setIsOpen] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [originalText, setOriginalText] = useState('');
    const [sanitizedText, setSanitizedText] = useState('');
    const [stats, setStats] = useState<SanitizationStats>({ counts: {}, total: 0 });
    const [error, setError] = useState<string | null>(null);

    // 获取启用的自定义规则
    const getEnabledRules = useSanitizationRulesStore((state) => state.getEnabledRules);

    /**
     * 将自定义规则转换为 IPC 格式
     */
    const convertToSanitizationRules = useCallback((): SanitizationRule[] => {
        const enabledRules = getEnabledRules();
        return enabledRules.map((rule) => ({
            name: rule.name,
            pattern: rule.pattern,
            replacement: `[REDACTED:${rule.sensitiveType.toUpperCase()}]`,
        }));
    }, [getEnabledRules]);

    /**
     * 打开预览并加载数据
     */
    const openPreview = useCallback(async () => {
        if (!sessionId) {
            setError('没有选中的会话');
            return;
        }

        setIsLoading(true);
        setError(null);
        setIsOpen(true);

        try {
            // 1. 获取原始会话内容
            const session = await invoke<MantraSession | null>('get_session', {
                sessionId,
            });

            if (!session) {
                throw new Error('会话不存在');
            }

            // 序列化为 JSON 作为原始文本
            const original = JSON.stringify(session, null, 2);
            setOriginalText(original);

            // 2. 获取自定义规则
            const customPatterns = convertToSanitizationRules();

            // 3. 调用脱敏 IPC
            const result = await sanitizeSession(sessionId, customPatterns);

            setSanitizedText(result.sanitized_text);
            setStats(result.stats);
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : '脱敏预览失败';
            setError(errorMessage);
            console.error('[useSanitizePreview] Error:', err);
        } finally {
            setIsLoading(false);
        }
    }, [sessionId, convertToSanitizationRules]);

    /**
     * 关闭预览
     */
    const closePreview = useCallback(() => {
        setIsOpen(false);
        // 清理状态
        setOriginalText('');
        setSanitizedText('');
        setStats({ counts: {}, total: 0 });
        setError(null);
    }, []);

    return {
        isOpen,
        isLoading,
        originalText,
        sanitizedText,
        stats,
        error,
        openPreview,
        closePreview,
    };
}
