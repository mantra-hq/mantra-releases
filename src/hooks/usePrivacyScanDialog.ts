/**
 * usePrivacyScanDialog Hook - 管理上传前隐私扫描弹窗
 * Story 3-9: Task 4 - AC #1-6
 *
 * 提供一个可复用的 hook 来管理隐私扫描弹窗的显示和用户交互
 */

import { useState, useCallback, useRef } from 'react';
import { performPreUploadScan, type PreUploadCheckResult, type ShowReportResult } from '@/lib/sync/privacy-check';
import { applyRedaction } from '@/lib/privacy-utils';
import type { ScanResult, UserAction } from '@/components/sanitizer/types';

export interface UsePrivacyScanDialogReturn {
    /** 是否显示弹窗 */
    isOpen: boolean;
    /** 是否正在扫描 */
    isScanning: boolean;
    /** 扫描结果 */
    scanResult: ScanResult | null;
    /** 执行上传前隐私检查 */
    performCheck: (sessionId: string, content: string, projectName?: string) => Promise<PreUploadCheckResult>;
    /** 处理用户点击"一键脱敏" */
    handleRedact: () => void;
    /** 处理用户点击"忽略并继续" */
    handleIgnore: () => void;
    /** 处理用户点击"取消" */
    handleCancel: () => void;
    /** 关闭弹窗 */
    close: () => void;
}

/**
 * 管理上传前隐私扫描弹窗的 hook
 *
 * @example
 * ```tsx
 * function UploadButton() {
 *   const {
 *     isOpen,
 *     isScanning,
 *     scanResult,
 *     performCheck,
 *     handleRedact,
 *     handleIgnore,
 *     handleCancel,
 *   } = usePrivacyScanDialog();
 *
 *   const handleUpload = async () => {
 *     const result = await performCheck('session-123', content, 'MyProject');
 *     if (result.shouldProceed) {
 *       await uploadToCloud(result.content);
 *     }
 *   };
 *
 *   return (
 *     <>
 *       <button onClick={handleUpload}>上传</button>
 *       <PrivacyScanReport
 *         isOpen={isOpen}
 *         isScanning={isScanning}
 *         scanResult={scanResult}
 *         onRedact={handleRedact}
 *         onIgnore={handleIgnore}
 *         onCancel={handleCancel}
 *       />
 *     </>
 *   );
 * }
 * ```
 */
export function usePrivacyScanDialog(): UsePrivacyScanDialogReturn {
    const [isOpen, setIsOpen] = useState(false);
    const [isScanning, setIsScanning] = useState(false);
    const [scanResult, setScanResult] = useState<ScanResult | null>(null);
    const [currentContent, setCurrentContent] = useState<string>('');

    // 使用 ref 存储 resolve 函数，避免闭包问题
    const resolveRef = useRef<((result: ShowReportResult) => void) | null>(null);

    /**
     * 显示报告弹窗并等待用户操作
     */
    const showReport = useCallback((result: ScanResult): Promise<ShowReportResult> => {
        return new Promise((resolve) => {
            setScanResult(result);
            setIsOpen(true);
            setIsScanning(false);
            resolveRef.current = resolve;
        });
    }, []);

    /**
     * 处理用户操作
     */
    const handleUserAction = useCallback((action: UserAction) => {
        const resolve = resolveRef.current;
        if (!resolve) return;

        let result: ShowReportResult = { action };

        // 如果用户选择脱敏，应用脱敏处理
        if (action === 'redacted' && scanResult) {
            const redactedContent = applyRedaction(currentContent, scanResult.matches);
            result = { action, redactedContent };
        }

        resolve(result);
        resolveRef.current = null;
        setIsOpen(false);
        setScanResult(null);
    }, [scanResult, currentContent]);

    const handleRedact = useCallback(() => handleUserAction('redacted'), [handleUserAction]);
    const handleIgnore = useCallback(() => handleUserAction('ignored'), [handleUserAction]);
    const handleCancel = useCallback(() => handleUserAction('cancelled'), [handleUserAction]);

    /**
     * 执行上传前隐私检查
     */
    const performCheck = useCallback(async (
        sessionId: string,
        content: string,
        projectName?: string
    ): Promise<PreUploadCheckResult> => {
        setCurrentContent(content);
        setIsScanning(true);
        setIsOpen(true);
        setScanResult(null);

        try {
            const result = await performPreUploadScan(
                sessionId,
                content,
                projectName,
                showReport
            );
            // 如果无敏感信息（userAction 为 undefined），扫描直接通过，关闭扫描状态弹窗
            if (!result.userAction) {
                setIsOpen(false);
            }
            return result;
        } catch (error) {
            // 发生错误时重置所有状态
            setIsOpen(false);
            setScanResult(null);
            throw error;
        } finally {
            // 如果没有弹窗显示（无敏感信息），重置状态
            setIsScanning(false);
        }
    }, [showReport]);

    /**
     * 关闭弹窗
     */
    const close = useCallback(() => {
        if (resolveRef.current) {
            handleCancel();
        } else {
            setIsOpen(false);
            setScanResult(null);
        }
    }, [handleCancel]);

    return {
        isOpen,
        isScanning,
        scanResult,
        performCheck,
        handleRedact,
        handleIgnore,
        handleCancel,
        close,
    };
}

export default usePrivacyScanDialog;
