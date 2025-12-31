/**
 * Sanitizer IPC 封装 - Story 3-2 Task 8
 * 前端调用 Rust sanitizer 的 IPC 接口
 */

import { invoke } from '@tauri-apps/api/core';
import type {
    SanitizationResult,
    SanitizationRule,
} from '@/components/sanitizer/types';

/**
 * 脱敏文本
 * @param text 原始文本
 * @param customPatterns 可选的自定义规则
 * @returns 脱敏结果
 */
export async function sanitizeText(
    text: string,
    customPatterns?: SanitizationRule[]
): Promise<SanitizationResult> {
    return invoke<SanitizationResult>('sanitize_text', {
        text,
        custom_patterns: customPatterns ?? [],
    });
}

/**
 * 脱敏会话内容
 * @param sessionId 会话 ID
 * @param customPatterns 可选的自定义规则
 * @returns 脱敏结果
 */
export async function sanitizeSession(
    sessionId: string,
    customPatterns?: SanitizationRule[]
): Promise<SanitizationResult> {
    return invoke<SanitizationResult>('sanitize_session', {
        sessionId,
        custom_patterns: customPatterns ?? [],
    });
}

/**
 * 脱敏结果的工具函数
 */
export function createEmptyStats(): SanitizationResult['stats'] {
    return {
        counts: {},
        total: 0,
    };
}

/**
 * 检查脱敏结果是否有变更
 */
export function hasChanges(result: SanitizationResult): boolean {
    return result.has_matches;
}
