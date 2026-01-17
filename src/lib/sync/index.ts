/**
 * Sync Module Index - 同步服务导出
 * Story 3-9: Task 4
 */

export {
    performPreUploadScan,
    type PreUploadCheckResult,
    type ShowReportCallback,
    type ShowReportResult,
} from './privacy-check';

export {
    uploadSession,
    needsPrivacyCheck,
    type UploadSessionOptions,
    type UploadSessionResult,
} from './cloud-sync';
