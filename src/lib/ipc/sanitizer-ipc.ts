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

/** 规则验证结果 */
export interface ValidationResult {
    valid: boolean;
    error?: string;
}

/**
 * 验证正则表达式是否有效
 * @param pattern 正则表达式模式
 * @returns 验证结果
 */
export async function validateRegex(pattern: string): Promise<ValidationResult> {
    return invoke<ValidationResult>('validate_regex', { pattern });
}

/**
 * 获取系统内置脱敏规则
 * Story 3-5: Task 2
 * @returns 内置规则列表
 */
export async function getBuiltinRules(): Promise<SanitizationRule[]> {
    return invoke<SanitizationRule[]>('get_builtin_rules');
}
