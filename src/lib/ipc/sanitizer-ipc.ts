/**
 * Sanitizer IPC 封装 - Story 3-2 Task 8
 * Story 9.2: Task 5.4 (使用 IPC 适配器)
 * Story 3-6: 隐私扫描器增强
 *
 * 前端调用 Rust sanitizer 的 IPC 接口
 */

import { invoke } from '@/lib/ipc-adapter';
import type {
    SanitizationResult,
    SanitizationRule,
    ScanResult,
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

/**
 * 扫描文本中的隐私信息
 * Story 3-6: 隐私扫描器增强
 *
 * 不修改原文，只返回检测结果供用户决策。
 *
 * @param text 待扫描文本
 * @returns 扫描结果，包含所有匹配项的详细信息（行号、列号、上下文等）
 */
export async function scanTextForPrivacy(text: string): Promise<ScanResult> {
    return invoke<ScanResult>('scan_text_for_privacy', { text });
}
