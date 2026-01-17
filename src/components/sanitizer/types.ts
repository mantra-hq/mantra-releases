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
    | 'phone'
    | 'id_card'
    | 'private_key'
    | 'password'
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
    phone: '电话号码',
    id_card: '身份证号',
    private_key: '私钥',
    password: '密码赋值',
    custom: '自定义规则',
};

/** 严重程度 (Story 3-6) */
export type Severity = 'critical' | 'warning' | 'info';

/** 严重程度显示标签 */
export const SEVERITY_LABELS: Record<Severity, string> = {
    critical: '严重',
    warning: '警告',
    info: '提示',
};

/** 严重程度颜色 */
export const SEVERITY_COLORS: Record<Severity, string> = {
    critical: 'text-red-500',
    warning: 'text-yellow-500',
    info: 'text-blue-500',
};

/** 脱敏统计 */
export interface SanitizationStats {
    counts: Partial<Record<SensitiveType, number>>;
    total: number;
}

/** 自定义脱敏规则 */
export interface SanitizationRule {
    /** 规则 ID (必需，与 Rust 端一致) */
    id: string;
    name: string;
    pattern: string;
    replacement?: string;
    /** 敏感信息类型 */
    sensitive_type: SensitiveType;
    /** 严重程度 */
    severity: Severity;
    /** 是否启用 */
    enabled: boolean;
}

/** 脱敏结果 (来自 Rust IPC) */
export interface SanitizationResult {
    sanitized_text: string;
    stats: SanitizationStats;
    has_matches: boolean;
}

// ============================================================
// Story 3-6: 隐私扫描器类型
// ============================================================

/** 扫描匹配结果 */
export interface ScanMatch {
    /** 规则 ID */
    rule_id: string;
    /** 敏感信息类型 */
    sensitive_type: SensitiveType;
    /** 严重程度 */
    severity: Severity;
    /**
     * 行号 (1-based)
     * @remarks Rust 端类型为 usize，JS 端使用 number。
     * 在极端情况下 (>2^53-1) 可能有精度损失，但行号不会达到此范围。
     */
    line: number;
    /**
     * 列号 (1-based)
     * @remarks 同 line 字段说明
     */
    column: number;
    /** 原始匹配文本 */
    matched_text: string;
    /** 脱敏显示文本 */
    masked_text: string;
    /** 上下文片段 */
    context: string;
}

/** 扫描统计 */
export interface ScanStats {
    /** Critical 数量 */
    critical_count: number;
    /** Warning 数量 */
    warning_count: number;
    /** Info 数量 */
    info_count: number;
    /** 总匹配数 */
    total: number;
    /** 按类型统计 */
    by_type: Record<string, number>;
}

/** 扫描结果 */
export interface ScanResult {
    /** 所有匹配项 */
    matches: ScanMatch[];
    /** 是否包含 Critical 级别匹配 */
    has_critical: boolean;
    /** 是否包含 Warning 级别匹配 */
    has_warning: boolean;
    /** 扫描耗时 (毫秒) */
    scan_time_ms: number;
    /** 统计信息 */
    stats: ScanStats;
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

// ============================================================
// Story 3.7: 拦截记录存储类型
// ============================================================

/** 用户操作类型 */
export type UserAction = 'redacted' | 'ignored' | 'cancelled' | 'rule_disabled';

/** 用户操作显示标签 */
export const USER_ACTION_LABELS: Record<UserAction, string> = {
    redacted: '已脱敏',
    ignored: '已忽略',
    cancelled: '已取消',
    rule_disabled: '禁用规则',
};

/** 拦截来源类型 */
export type InterceptionSourceType = 'pre_upload' | 'claude_code_hook' | 'external_hook';

/** 拦截来源 */
export interface InterceptionSource {
    type: InterceptionSourceType;
    session_id?: string;
    tool_name?: string;
}

/** 拦截记录 */
export interface InterceptionRecord {
    /** 记录 ID (UUID) */
    id: string;
    /** 时间戳 (ISO 8601) */
    timestamp: string;
    /** 拦截来源 */
    source: InterceptionSource;
    /** 匹配结果列表 */
    matches: ScanMatch[];
    /** 用户操作 */
    user_action: UserAction;
    /** 原文哈希 */
    original_text_hash: string;
    /** 项目名称 (可选) */
    project_name?: string;
}

/** 拦截统计 */
export interface InterceptionStats {
    /** 总拦截数 */
    total_interceptions: number;
    /** 按敏感类型分组统计 */
    by_type: Record<string, number>;
    /** 按严重程度分组统计 */
    by_severity: Record<string, number>;
    /** 按用户操作分组统计 */
    by_action: Record<string, number>;
    /** 最近 7 天拦截数 */
    recent_7_days: number;
}

/** 分页记录结果 */
export interface PaginatedRecords {
    /** 记录列表 */
    records: InterceptionRecord[];
    /** 总记录数 */
    total: number;
    /** 当前页码 (1-based) */
    page: number;
    /** 每页记录数 */
    per_page: number;
}
