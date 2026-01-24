/**
 * Privacy Check Service - 上传前隐私扫描服务
 * Story 3-9: Task 3 - AC #1, #6
 *
 * 在上传前执行隐私扫描，根据扫描结果决定是否显示报告弹窗
 */

import { scanTextForPrivacy, saveInterceptionRecord } from '@/lib/ipc/sanitizer-ipc';
import type { ScanResult, UserAction, InterceptionSource } from '@/components/sanitizer/types';

/** 上传前检查结果 */
export interface PreUploadCheckResult {
    /** 是否应该继续上传 */
    shouldProceed: boolean;
    /** 最终内容（可能是脱敏后的） */
    content: string;
    /** 用户操作类型 */
    userAction?: UserAction;
}

/** 显示报告回调返回值 */
export interface ShowReportResult {
    /** 用户操作 */
    action: UserAction;
    /** 脱敏后内容（仅当 action 为 'redacted' 时有效） */
    redactedContent?: string;
}

/** 显示报告弹窗的回调类型 */
export type ShowReportCallback = (result: ScanResult) => Promise<ShowReportResult>;

/**
 * 执行上传前隐私扫描
 *
 * @param sessionId 会话 ID
 * @param content 待上传的内容
 * @param projectName 项目名称（可选）
 * @param showReport 显示扫描报告弹窗的回调
 * @returns 检查结果
 */
export async function performPreUploadScan(
    sessionId: string,
    content: string,
    projectName: string | undefined,
    showReport: ShowReportCallback
): Promise<PreUploadCheckResult> {
    // 1. 执行扫描
    const scanResult = await scanTextForPrivacy(content);

    // 2. 无敏感信息（无 Critical 和 Warning），直接通过
    if (!scanResult.has_critical && !scanResult.has_warning) {
        return { shouldProceed: true, content };
    }

    // 3. 显示报告并等待用户操作
    const { action, redactedContent } = await showReport(scanResult);

    // 4. 保存拦截记录
    const source: InterceptionSource = { type: 'pre_upload', session_id: sessionId };
    await saveInterceptionRecord({
        timestamp: new Date().toISOString(),
        source,
        matches: scanResult.matches,
        user_action: action,
        // original_text_hash 由后端根据需要计算，前端不传，但类型定义可能需要
        original_text_hash: '',
        project_name: projectName,
    });

    // 5. 根据操作返回结果
    switch (action) {
        case 'redacted':
            return {
                shouldProceed: true,
                content: redactedContent ?? content,
                userAction: action,
            };
        case 'ignored':
            return {
                shouldProceed: true,
                content,
                userAction: action,
            };
        case 'cancelled':
            return {
                shouldProceed: false,
                content,
                userAction: action,
            };
        default:
            return {
                shouldProceed: false,
                content,
                userAction: action,
            };
    }
}
