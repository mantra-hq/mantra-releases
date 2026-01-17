/**
 * Sanitizer IPC 封装 - Story 3-2 Task 8
 * Story 9.2: Task 5.4 (使用 IPC 适配器)
 * Story 3-6: 隐私扫描器增强
 * Story 3.7: 拦截记录存储
 *
 * 前端调用 Rust sanitizer 的 IPC 接口
 */

import { invoke } from '@/lib/ipc-adapter';
import type {
    InterceptionRecord,
    InterceptionStats,
    PaginatedRecords,
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

// ============================================================
// Story 3.7: 拦截记录存储 IPC
// ============================================================

/**
 * 保存拦截记录
 * @param record 拦截记录
 */
export async function saveInterceptionRecord(record: InterceptionRecord): Promise<void> {
    return invoke<void>('save_interception_record', { record });
}

/**
 * 获取拦截记录 (分页)
 * @param page 页码 (1-based)
 * @param perPage 每页记录数
 * @param sourceFilter 可选的来源类型筛选
 * @returns 分页结果
 */
export async function getInterceptionRecords(
    page: number,
    perPage: number,
    sourceFilter?: string
): Promise<PaginatedRecords> {
    return invoke<PaginatedRecords>('get_interception_records', {
        page,
        per_page: perPage,
        source_filter: sourceFilter,
    });
}

/**
 * 获取拦截统计
 * @returns 统计数据
 */
export async function getInterceptionStats(): Promise<InterceptionStats> {
    return invoke<InterceptionStats>('get_interception_stats');
}

/**
 * 删除拦截记录
 * @param ids 要删除的记录 ID 列表
 * @returns 删除的记录数
 */
export async function deleteInterceptionRecords(ids: string[]): Promise<number> {
    return invoke<number>('delete_interception_records', { ids });
}
