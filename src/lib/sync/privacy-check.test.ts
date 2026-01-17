/**
 * Privacy Check Service Tests
 * Story 3-9: Task 3.5 - 单元测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { performPreUploadScan, type PreUploadCheckResult, type ShowReportCallback } from './privacy-check';
import type { ScanResult, UserAction } from '@/components/sanitizer/types';

// Mock sanitizer-ipc
vi.mock('@/lib/ipc/sanitizer-ipc', () => ({
    scanTextForPrivacy: vi.fn(),
    saveInterceptionRecord: vi.fn(),
}));

import { scanTextForPrivacy, saveInterceptionRecord } from '@/lib/ipc/sanitizer-ipc';

const createMockScanResult = (overrides?: Partial<ScanResult>): ScanResult => ({
    matches: [],
    has_critical: false,
    has_warning: false,
    scan_time_ms: 10,
    stats: {
        critical_count: 0,
        warning_count: 0,
        info_count: 0,
        total: 0,
        by_type: {},
    },
    ...overrides,
});

const createMockScanResultWithMatches = (): ScanResult => ({
    matches: [
        {
            rule_id: 'api_key',
            sensitive_type: 'api_key',
            severity: 'critical',
            line: 1,
            column: 1,
            matched_text: 'sk-test-key',
            masked_text: 'sk-****',
            context: 'key = "sk-test-key"',
        },
    ],
    has_critical: true,
    has_warning: false,
    scan_time_ms: 15,
    stats: {
        critical_count: 1,
        warning_count: 0,
        info_count: 0,
        total: 1,
        by_type: { api_key: 1 },
    },
});

describe('performPreUploadScan', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('Task 3.3 - 调用 IPC 扫描', () => {
        it('应该调用 scanTextForPrivacy 扫描内容', async () => {
            const mockScanResult = createMockScanResult();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn();
            const content = 'test content';

            await performPreUploadScan('session-1', content, 'TestProject', mockShowReport);

            expect(scanTextForPrivacy).toHaveBeenCalledWith(content);
        });
    });

    describe('Task 3.4 - 无敏感信息直接通过', () => {
        it('无 Critical 和 Warning 时应该直接通过不显示弹窗', async () => {
            const mockScanResult = createMockScanResult();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn();

            const result = await performPreUploadScan('session-1', 'safe content', 'TestProject', mockShowReport);

            expect(result.shouldProceed).toBe(true);
            expect(result.content).toBe('safe content');
            expect(mockShowReport).not.toHaveBeenCalled();
            expect(saveInterceptionRecord).not.toHaveBeenCalled();
        });

        it('只有 Info 级别时也应该直接通过', async () => {
            const mockScanResult = createMockScanResult({
                matches: [
                    {
                        rule_id: 'info_rule',
                        sensitive_type: 'email',
                        severity: 'info',
                        line: 1,
                        column: 1,
                        matched_text: 'test',
                        masked_text: '****',
                        context: 'context',
                    },
                ],
                stats: { critical_count: 0, warning_count: 0, info_count: 1, total: 1, by_type: {} },
            });
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn();

            const result = await performPreUploadScan('session-1', 'content', 'TestProject', mockShowReport);

            expect(result.shouldProceed).toBe(true);
            expect(mockShowReport).not.toHaveBeenCalled();
        });
    });

    describe('有敏感信息时显示弹窗', () => {
        it('有 Critical 时应该显示弹窗', async () => {
            const mockScanResult = createMockScanResultWithMatches();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'ignored' as UserAction,
            });

            await performPreUploadScan('session-1', 'sensitive content', 'TestProject', mockShowReport);

            expect(mockShowReport).toHaveBeenCalledWith(mockScanResult);
        });

        it('有 Warning 时应该显示弹窗', async () => {
            const mockScanResult = createMockScanResult({
                has_warning: true,
                matches: [
                    {
                        rule_id: 'warning_rule',
                        sensitive_type: 'email',
                        severity: 'warning',
                        line: 1,
                        column: 1,
                        matched_text: 'test@example.com',
                        masked_text: '****@example.com',
                        context: 'email: test@example.com',
                    },
                ],
                stats: { critical_count: 0, warning_count: 1, info_count: 0, total: 1, by_type: {} },
            });
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'ignored' as UserAction,
            });

            await performPreUploadScan('session-1', 'email content', 'TestProject', mockShowReport);

            expect(mockShowReport).toHaveBeenCalled();
        });
    });

    describe('用户操作处理 - Redacted', () => {
        it('用户选择脱敏时应该返回脱敏内容', async () => {
            const mockScanResult = createMockScanResultWithMatches();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const redactedContent = 'key = "sk-****"';
            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'redacted' as UserAction,
                redactedContent,
            });

            const result = await performPreUploadScan('session-1', 'sensitive', 'TestProject', mockShowReport);

            expect(result.shouldProceed).toBe(true);
            expect(result.content).toBe(redactedContent);
            expect(result.userAction).toBe('redacted');
        });

        it('脱敏时应该保存拦截记录', async () => {
            const mockScanResult = createMockScanResultWithMatches();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'redacted' as UserAction,
                redactedContent: 'redacted',
            });

            await performPreUploadScan('session-1', 'sensitive', 'TestProject', mockShowReport);

            expect(saveInterceptionRecord).toHaveBeenCalledWith(
                expect.objectContaining({
                    source: { type: 'pre_upload', session_id: 'session-1' },
                    user_action: 'redacted',
                    project_name: 'TestProject',
                    matches: mockScanResult.matches,
                })
            );
        });
    });

    describe('用户操作处理 - Ignored', () => {
        it('用户选择忽略时应该返回原始内容', async () => {
            const mockScanResult = createMockScanResultWithMatches();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const originalContent = 'sensitive content';
            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'ignored' as UserAction,
            });

            const result = await performPreUploadScan('session-1', originalContent, 'TestProject', mockShowReport);

            expect(result.shouldProceed).toBe(true);
            expect(result.content).toBe(originalContent);
            expect(result.userAction).toBe('ignored');
        });

        it('忽略时应该保存拦截记录', async () => {
            const mockScanResult = createMockScanResultWithMatches();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'ignored' as UserAction,
            });

            await performPreUploadScan('session-1', 'sensitive', 'TestProject', mockShowReport);

            expect(saveInterceptionRecord).toHaveBeenCalledWith(
                expect.objectContaining({
                    user_action: 'ignored',
                })
            );
        });
    });

    describe('用户操作处理 - Cancelled', () => {
        it('用户选择取消时应该阻止上传', async () => {
            const mockScanResult = createMockScanResultWithMatches();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'cancelled' as UserAction,
            });

            const result = await performPreUploadScan('session-1', 'sensitive', 'TestProject', mockShowReport);

            expect(result.shouldProceed).toBe(false);
            expect(result.userAction).toBe('cancelled');
        });

        it('取消时应该保存拦截记录', async () => {
            const mockScanResult = createMockScanResultWithMatches();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'cancelled' as UserAction,
            });

            await performPreUploadScan('session-1', 'sensitive', 'TestProject', mockShowReport);

            expect(saveInterceptionRecord).toHaveBeenCalledWith(
                expect.objectContaining({
                    user_action: 'cancelled',
                })
            );
        });
    });

    describe('可选参数处理', () => {
        it('projectName 可选，不提供时不应该在记录中包含', async () => {
            const mockScanResult = createMockScanResultWithMatches();
            vi.mocked(scanTextForPrivacy).mockResolvedValue(mockScanResult);

            const mockShowReport: ShowReportCallback = vi.fn().mockResolvedValue({
                action: 'ignored' as UserAction,
            });

            await performPreUploadScan('session-1', 'sensitive', undefined, mockShowReport);

            expect(saveInterceptionRecord).toHaveBeenCalledWith(
                expect.objectContaining({
                    project_name: undefined,
                })
            );
        });
    });
});
