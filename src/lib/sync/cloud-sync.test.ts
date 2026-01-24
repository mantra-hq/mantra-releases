/**
 * Cloud Sync Service Tests
 * Story 3-9: Task 4 - 单元测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { uploadSession, type UploadSessionOptions, type _UploadSessionResult } from './cloud-sync';
import type { UserAction } from '@/components/sanitizer/types';

// Mock privacy-check
vi.mock('./privacy-check', () => ({
    performPreUploadScan: vi.fn(),
}));

import { performPreUploadScan } from './privacy-check';

const createMockOptions = (overrides?: Partial<UploadSessionOptions>): UploadSessionOptions => ({
    sessionId: 'test-session-123',
    content: 'test content',
    projectName: 'TestProject',
    showPrivacyReport: vi.fn().mockResolvedValue({ action: 'ignored' as UserAction }),
    ...overrides,
});

describe('uploadSession', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('隐私检查集成', () => {
        it('应该调用 performPreUploadScan 进行隐私检查', async () => {
            vi.mocked(performPreUploadScan).mockResolvedValue({
                shouldProceed: true,
                content: 'test content',
            });

            const options = createMockOptions();
            await uploadSession(options);

            expect(performPreUploadScan).toHaveBeenCalledWith(
                options.sessionId,
                options.content,
                options.projectName,
                options.showPrivacyReport
            );
        });

        it('用户取消时应该返回失败', async () => {
            vi.mocked(performPreUploadScan).mockResolvedValue({
                shouldProceed: false,
                content: 'test content',
                userAction: 'cancelled',
            });

            const result = await uploadSession(createMockOptions());

            expect(result.success).toBe(false);
            expect(result.error).toBe('Upload cancelled by user');
        });

        it('用户选择脱敏时应该使用脱敏后的内容', async () => {
            const redactedContent = 'redacted content';
            vi.mocked(performPreUploadScan).mockResolvedValue({
                shouldProceed: true,
                content: redactedContent,
                userAction: 'redacted',
            });

            const result = await uploadSession(createMockOptions());

            expect(result.success).toBe(true);
            expect(result.privacyCheckResult?.content).toBe(redactedContent);
        });

        it('用户选择忽略时应该使用原始内容', async () => {
            const originalContent = 'original content';
            vi.mocked(performPreUploadScan).mockResolvedValue({
                shouldProceed: true,
                content: originalContent,
                userAction: 'ignored',
            });

            const options = createMockOptions({ content: originalContent });
            const result = await uploadSession(options);

            expect(result.success).toBe(true);
            expect(result.privacyCheckResult?.content).toBe(originalContent);
        });
    });

    describe('进度回调', () => {
        it('应该调用进度回调', async () => {
            vi.mocked(performPreUploadScan).mockResolvedValue({
                shouldProceed: true,
                content: 'content',
            });

            const onProgress = vi.fn();
            await uploadSession(createMockOptions({ onProgress }));

            expect(onProgress).toHaveBeenCalledWith(10);
            expect(onProgress).toHaveBeenCalledWith(30);
            expect(onProgress).toHaveBeenCalledWith(100);
        });
    });

    describe('返回结果', () => {
        it('成功时应该返回云端 URL', async () => {
            vi.mocked(performPreUploadScan).mockResolvedValue({
                shouldProceed: true,
                content: 'content',
            });

            const options = createMockOptions({ sessionId: 'abc-123' });
            const result = await uploadSession(options);

            expect(result.success).toBe(true);
            expect(result.cloudUrl).toBe('https://mantra.gonewx.com/s/abc-123');
        });

        it('应该返回隐私检查结果', async () => {
            const privacyResult = {
                shouldProceed: true,
                content: 'content',
                userAction: 'redacted' as UserAction,
            };
            vi.mocked(performPreUploadScan).mockResolvedValue(privacyResult);

            const result = await uploadSession(createMockOptions());

            expect(result.privacyCheckResult).toEqual(privacyResult);
        });
    });
});
