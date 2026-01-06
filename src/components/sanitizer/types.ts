/**
 * Sanitizer Types - Story 3-2: Diff 预览界面
 * Story 3.4: 主视图原生模式重构
 * 与 Rust 端 sanitizer 模块保持类型一致
 */

/** 敏感信息类型 (与 Rust 端 SensitiveType 保持一致) */
export type SensitiveType =
    | 'api_key'
    | 'aws_key'
    | 'github_token'
    | 'anthropic_key'
    | 'google_cloud_key'
    | 'ip_address'
    | 'bearer_token'
    | 'jwt_token'
    | 'secret'
    | 'email'
    | 'custom';

/** 敏感信息类型的显示标签 */
export const SENSITIVE_TYPE_LABELS: Record<SensitiveType, string> = {
    api_key: 'API Key',
    aws_key: 'AWS Key',
    github_token: 'GitHub Token',
    anthropic_key: 'Anthropic Key',
    google_cloud_key: 'Google Cloud Key',
    ip_address: 'IP 地址',
    bearer_token: 'Bearer Token',
    jwt_token: 'JWT Token',
    secret: '密码/Secret',
    email: '邮箱地址',
    custom: '自定义规则',
};

/** 脱敏统计 */
export interface SanitizationStats {
    counts: Partial<Record<SensitiveType, number>>;
    total: number;
}

/** 自定义脱敏规则 */
export interface SanitizationRule {
    name: string;
    pattern: string;
    replacement: string;
    /** 敏感信息类型 (仅内置规则包含此字段) */
    sensitive_type?: SensitiveType;
}

/** 脱敏结果 (来自 Rust IPC) */
export interface SanitizationResult {
    sanitized_text: string;
    stats: SanitizationStats;
    has_matches: boolean;
}

/** Diff 行类型 */
export type DiffLineType = 'added' | 'removed' | 'unchanged';

/** Diff 行数据 */
export interface DiffLine {
    type: DiffLineType;
    content: string;
    lineNumber: {
        original?: number;
        sanitized?: number;
    };
}

/** Diff 预览组件 Props */
export interface DiffPreviewProps {
    originalText: string;
    sanitizedText: string;
    stats: SanitizationStats;
    onConfirm: () => void;
    onCancel: () => void;
    /** 是否处于加载状态 */
    isLoading?: boolean;
    /** 是否隐藏操作按钮（用于嵌入其他组件） */
    hideActions?: boolean;
}

/** 统计摘要组件 Props */
export interface SanitizationSummaryProps {
    stats: SanitizationStats;
}

// ============================================================
// Story 3.4: 主视图原生模式 - 新增类型
// ============================================================

/** 脱敏预览模式 */
export type SanitizeMode = 'idle' | 'preview';

/** 敏感信息匹配详情 */
export interface SensitiveMatch {
    /** 唯一标识 */
    id: string;
    /** 敏感信息类型 */
    type: SensitiveType;
    /** 原始内容（部分遮盖） */
    original: string;
    /** 脱敏后内容 */
    sanitized: string;
    /** 行号 */
    lineNumber: number;
    /** 上下文（前后几行） */
    context?: string;
}

/** 状态横幅组件 Props */
export interface SanitizeStatusBannerProps {
    /** 脱敏统计 */
    stats: SanitizationStats;
    /** 敏感信息匹配列表 */
    sensitiveMatches: SensitiveMatch[];
    /** 是否正在加载 */
    isLoading?: boolean;
    /** 错误信息 */
    error?: string | null;
    /** 取消回调 */
    onCancel: () => void;
    /** 确认分享回调 */
    onConfirm: () => void;
    /** 标签点击跳转回调 (行号) */
    onJumpToLine?: (lineNumber: number) => void;
}

// ============================================================
// 以下类型已废弃 (Story 3.4 设计变更)
// 保留用于向后兼容，将在后续版本移除
// ============================================================

/** @deprecated 使用 SanitizeMode 替代 */
export type SanitizeStep = 'summary' | 'details' | 'confirm';

/** @deprecated 三步流程已移除 */
export interface SanitizePreviewModalProps {
    isOpen: boolean;
    onClose: () => void;
    originalText: string;
    sanitizedText: string;
    stats: SanitizationStats;
    onConfirm: () => void;
    isLoading?: boolean;
}

/** @deprecated 三步流程已移除 */
export interface ScanResultSummaryProps {
    stats: SanitizationStats;
    onCancel: () => void;
    onViewDetails: () => void;
    onSkipToShare: () => void;
    isLoading?: boolean;
}

/** @deprecated 三步流程已移除 */
export interface SensitiveItemCardProps {
    match: SensitiveMatch;
    currentIndex: number;
    totalCount: number;
    onPrevious: () => void;
    onNext: () => void;
    onBack: () => void;
    onSkipToShare: () => void;
    isExpanded: boolean;
    onToggleExpand: () => void;
}

/** @deprecated 三步流程已移除 */
export interface ShareConfirmationProps {
    stats: SanitizationStats;
    originalText: string;
    sanitizedText: string;
    onCancel: () => void;
    onConfirm: () => void;
    isLoading?: boolean;
}
