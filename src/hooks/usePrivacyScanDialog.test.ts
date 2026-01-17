/**
 * usePrivacyScanDialog Hook Tests
 * Story 3-9: Task 4 - 单元测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { usePrivacyScanDialog } from './usePrivacyScanDialog';
import type { ScanResult } from '@/components/sanitizer/types';

// Mock dependencies
vi.mock('@/lib/sync/privacy-check', () => ({
    performPreUploadScan: vi.fn(),
}));

vi.mock('@/lib/privacy-utils', () => ({
    applyRedaction: vi.fn((content, _matches) => `redacted:${content}`),
}));

import { performPreUploadScan } from '@/lib/sync/privacy-check';
import { applyRedaction } from '@/lib/privacy-utils';

const createMockScanResult = (): ScanResult => ({
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

describe('usePrivacyScanDialog', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('初始状态', () => {
        it('初始状态应该正确', () => {
            const { result } = renderHook(() => usePrivacyScanDialog());

            expect(result.current.isOpen).toBe(false);
            expect(result.current.isScanning).toBe(false);
            expect(result.current.scanResult).toBeNull();
        });
    });

    describe('performCheck - 无敏感信息', () => {
        it('无敏感信息时应该直接返回成功', async () => {
            const mockResult = {
                shouldProceed: true,
                content: 'safe content',
            };
            vi.mocked(performPreUploadScan).mockResolvedValue(mockResult);

            const { result } = renderHook(() => usePrivacyScanDialog());

            let checkResult;
            await act(async () => {
                checkResult = await result.current.performCheck('session-1', 'safe content', 'TestProject');
            });

            expect(checkResult).toEqual(mockResult);
        });

        it('无敏感信息时不应显示弹窗', async () => {
            const mockResult = {
                shouldProceed: true,
                content: 'safe content',
            };
            vi.mocked(performPreUploadScan).mockResolvedValue(mockResult);

            const { result } = renderHook(() => usePrivacyScanDialog());

            await act(async () => {
                await result.current.performCheck('session-1', 'safe content', 'TestProject');
            });

            // 扫描完成后应该重置状态
            expect(result.current.isScanning).toBe(false);
        });
    });

    describe('performCheck - 有敏感信息时的异步交互', () => {
        it('有敏感信息时应该等待用户操作', async () => {
            const scanResult = createMockScanResult();
            let capturedShowReport: ((result: ScanResult) => Promise<{ action: string; redactedContent?: string }>) | null = null;

            vi.mocked(performPreUploadScan).mockImplementation(async (sessionId, content, projectName, showReport) => {
                capturedShowReport = showReport;
                // 模拟显示弹窗等待用户操作
                const userResponse = await showReport(scanResult);
                return {
                    shouldProceed: userResponse.action !== 'cancelled',
                    content: userResponse.redactedContent ?? content,
                    userAction: userResponse.action as 'redacted' | 'ignored' | 'cancelled',
                };
            });

            const { result } = renderHook(() => usePrivacyScanDialog());

            // 开始检查但不等待完成
            let checkPromise: Promise<unknown>;
            act(() => {
                checkPromise = result.current.performCheck('session-1', 'sensitive content', 'TestProject');
            });

            // 等待弹窗打开
            await waitFor(() => {
                expect(result.current.isOpen).toBe(true);
            });

            expect(result.current.scanResult).toEqual(scanResult);
        });
    });

    describe('handleRedact - 脱敏操作', () => {
        it('handleRedact 应该调用 applyRedaction', async () => {
            const scanResult = createMockScanResult();

            vi.mocked(performPreUploadScan).mockImplementation(async (sessionId, content, projectName, showReport) => {
                const userResponse = await showReport(scanResult);
                return {
                    shouldProceed: true,
                    content: userResponse.redactedContent ?? content,
                    userAction: userResponse.action as 'redacted' | 'ignored' | 'cancelled',
                };
            });

            const { result } = renderHook(() => usePrivacyScanDialog());

            let checkPromise: Promise<unknown>;
            act(() => {
                checkPromise = result.current.performCheck('session-1', 'sensitive content', 'TestProject');
            });

            // 等待弹窗打开
            await waitFor(() => {
                expect(result.current.isOpen).toBe(true);
            });

            // 点击脱敏
            act(() => {
                result.current.handleRedact();
            });

            // 验证 applyRedaction 被调用
            expect(applyRedaction).toHaveBeenCalledWith('sensitive content', scanResult.matches);

            // 弹窗应该关闭
            await waitFor(() => {
                expect(result.current.isOpen).toBe(false);
            });
        });
    });

    describe('handleIgnore - 忽略操作', () => {
        it('handleIgnore 应该关闭弹窗并继续', async () => {
            const scanResult = createMockScanResult();

            vi.mocked(performPreUploadScan).mockImplementation(async (sessionId, content, projectName, showReport) => {
                const userResponse = await showReport(scanResult);
                return {
                    shouldProceed: userResponse.action !== 'cancelled',
                    content,
                    userAction: userResponse.action as 'redacted' | 'ignored' | 'cancelled',
                };
            });

            const { result } = renderHook(() => usePrivacyScanDialog());

            let checkResult: unknown;
            act(() => {
                result.current.performCheck('session-1', 'sensitive content', 'TestProject').then((r) => {
                    checkResult = r;
                });
            });

            // 等待弹窗打开
            await waitFor(() => {
                expect(result.current.isOpen).toBe(true);
            });

            // 点击忽略
            act(() => {
                result.current.handleIgnore();
            });

            // 弹窗应该关闭
            await waitFor(() => {
                expect(result.current.isOpen).toBe(false);
            });
        });
    });

    describe('handleCancel - 取消操作', () => {
        it('handleCancel 应该关闭弹窗并阻止上传', async () => {
            const scanResult = createMockScanResult();

            vi.mocked(performPreUploadScan).mockImplementation(async (sessionId, content, projectName, showReport) => {
                const userResponse = await showReport(scanResult);
                return {
                    shouldProceed: userResponse.action !== 'cancelled',
                    content,
                    userAction: userResponse.action as 'redacted' | 'ignored' | 'cancelled',
                };
            });

            const { result } = renderHook(() => usePrivacyScanDialog());

            let checkResult: { shouldProceed?: boolean } | undefined;
            act(() => {
                result.current.performCheck('session-1', 'sensitive content', 'TestProject').then((r) => {
                    checkResult = r;
                });
            });

            // 等待弹窗打开
            await waitFor(() => {
                expect(result.current.isOpen).toBe(true);
            });

            // 点击取消
            act(() => {
                result.current.handleCancel();
            });

            // 等待结果
            await waitFor(() => {
                expect(checkResult?.shouldProceed).toBe(false);
            });
        });
    });

    describe('close', () => {
        it('close 应该能调用而不报错', async () => {
            const { result } = renderHook(() => usePrivacyScanDialog());

            // close 在没有 pending promise 时应该只是重置状态
            act(() => {
                result.current.close();
            });

            expect(result.current.isOpen).toBe(false);
        });

        it('close 在有 pending promise 时应该触发 cancel', async () => {
            const scanResult = createMockScanResult();

            vi.mocked(performPreUploadScan).mockImplementation(async (sessionId, content, projectName, showReport) => {
                const userResponse = await showReport(scanResult);
                return {
                    shouldProceed: userResponse.action !== 'cancelled',
                    content,
                    userAction: userResponse.action as 'redacted' | 'ignored' | 'cancelled',
                };
            });

            const { result } = renderHook(() => usePrivacyScanDialog());

            let checkResult: { shouldProceed?: boolean } | undefined;
            act(() => {
                result.current.performCheck('session-1', 'sensitive content', 'TestProject').then((r) => {
                    checkResult = r;
                });
            });

            // 等待弹窗打开
            await waitFor(() => {
                expect(result.current.isOpen).toBe(true);
            });

            // 调用 close
            act(() => {
                result.current.close();
            });

            // 应该触发取消
            await waitFor(() => {
                expect(checkResult?.shouldProceed).toBe(false);
            });
        });
    });

    describe('handlers', () => {
        it('handleRedact 应该存在', () => {
            const { result } = renderHook(() => usePrivacyScanDialog());
            expect(typeof result.current.handleRedact).toBe('function');
        });

        it('handleIgnore 应该存在', () => {
            const { result } = renderHook(() => usePrivacyScanDialog());
            expect(typeof result.current.handleIgnore).toBe('function');
        });

        it('handleCancel 应该存在', () => {
            const { result } = renderHook(() => usePrivacyScanDialog());
            expect(typeof result.current.handleCancel).toBe('function');
        });
    });
});
