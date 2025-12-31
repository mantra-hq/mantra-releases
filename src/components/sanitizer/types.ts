/**
 * Sanitizer Types - Story 3-2: Diff 预览界面
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
}

/** 统计摘要组件 Props */
export interface SanitizationSummaryProps {
    stats: SanitizationStats;
}

/** 脱敏预览 Modal Props */
export interface SanitizePreviewModalProps {
    isOpen: boolean;
    onClose: () => void;
    originalText: string;
    sanitizedText: string;
    stats: SanitizationStats;
    onConfirm: () => void;
    /** 是否处于加载状态 */
    isLoading?: boolean;
}
