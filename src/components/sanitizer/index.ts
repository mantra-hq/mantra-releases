/**
 * Sanitizer 组件导出
 * Story 3.4: 主视图原生模式重构
 */

export { DiffPreview } from './DiffPreview';
export { DiffLineComponent } from './DiffLine';
export { SanitizationSummary } from './SanitizationSummary';
export { SanitizeStatusBanner } from './SanitizeStatusBanner';
export { computeDiff, hasDifferences, getDiffStats } from './diff-utils';
export * from './types';

// 以下组件已废弃 (Story 3.4 设计变更)
// - SanitizePreviewModal (已删除，使用主视图原生模式替代)
// - StepIndicator (已删除)
// - ScanResultSummary (已删除)
// - SensitiveItemCard (已删除)
// - ShareConfirmation (已删除)
