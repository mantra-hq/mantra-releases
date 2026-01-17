/**
 * InterceptionRecordItem Component - 单条拦截记录
 * Story 3-8: Task 4.2 - AC #4
 *
 * 显示单条拦截记录，支持展开/折叠详情
 */

import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ChevronDown, ChevronUp, FolderOpen } from 'lucide-react';
import { format } from 'date-fns';
import { zhCN, enUS } from 'date-fns/locale';
import { Checkbox } from '@/components/ui/checkbox';
import {
    SENSITIVE_TYPE_LABELS,
    SEVERITY_COLORS,
    SEVERITY_LABELS,
    USER_ACTION_LABELS,
    type InterceptionRecord,
    type UserAction,
} from '@/components/sanitizer/types';

export interface InterceptionRecordItemProps {
    /** 拦截记录 */
    record: InterceptionRecord;
    /** 是否选中 */
    selected: boolean;
    /** 选中状态变化回调 */
    onSelectionChange: (selected: boolean) => void;
}

/** 用户操作对应的颜色 */
const ACTION_COLORS: Record<UserAction, string> = {
    redacted: 'text-emerald-500',
    ignored: 'text-yellow-500',
    cancelled: 'text-zinc-400',
    rule_disabled: 'text-orange-500',
};

/** 来源类型的显示标签 */
const SOURCE_LABELS: Record<string, string> = {
    pre_upload: 'privacy.records.source.preUpload',
    claude_code_hook: 'privacy.records.source.claudeCodeHook',
    external_hook: 'privacy.records.source.externalHook',
};

export function InterceptionRecordItem({
    record,
    selected,
    onSelectionChange,
}: InterceptionRecordItemProps) {
    const { t, i18n } = useTranslation();
    const [expanded, setExpanded] = useState(false);

    const locale = i18n.language.startsWith('zh') ? zhCN : enUS;
    const timestamp = format(new Date(record.timestamp), 'yyyy-MM-dd HH:mm', { locale });

    // 获取敏感类型的汇总
    const typeCounts = record.matches.reduce(
        (acc, match) => {
            acc[match.sensitive_type] = (acc[match.sensitive_type] || 0) + 1;
            return acc;
        },
        {} as Record<string, number>
    );

    const typesSummary = Object.entries(typeCounts)
        .map(([type, count]) => `${SENSITIVE_TYPE_LABELS[type as keyof typeof SENSITIVE_TYPE_LABELS] || type} (${count})`)
        .join(', ');

    return (
        <div
            className="rounded-lg border bg-card transition-colors hover:bg-accent/30"
            data-testid={`record-item-${record.id}`}
        >
            {/* 记录主行 */}
            <div className="flex items-center gap-3 p-4">
                {/* 复选框 */}
                <Checkbox
                    checked={selected}
                    onCheckedChange={(checked) => onSelectionChange(checked === true)}
                    aria-label={t('common.select')}
                    data-testid={`record-checkbox-${record.id}`}
                />

                {/* 时间戳 */}
                <div className="w-36 text-sm text-muted-foreground tabular-nums">
                    {timestamp}
                </div>

                {/* 来源 */}
                <div className="w-32 text-sm">
                    {t(SOURCE_LABELS[record.source.type] || record.source.type)}
                </div>

                {/* 敏感类型摘要 */}
                <div className="flex-1 text-sm truncate" title={typesSummary}>
                    {typesSummary}
                </div>

                {/* 用户操作 */}
                <div className={`w-20 text-sm font-medium ${ACTION_COLORS[record.user_action]}`}>
                    {USER_ACTION_LABELS[record.user_action]}
                </div>

                {/* 展开/折叠按钮 */}
                <button
                    onClick={() => setExpanded(!expanded)}
                    className="p-1 hover:bg-accent rounded transition-colors"
                    aria-label={expanded ? t('privacy.records.list.collapseDetails') : t('privacy.records.list.expandDetails')}
                    data-testid={`record-toggle-${record.id}`}
                >
                    {expanded ? (
                        <ChevronUp className="h-4 w-4" />
                    ) : (
                        <ChevronDown className="h-4 w-4" />
                    )}
                </button>
            </div>

            {/* 展开的详情区域 */}
            {expanded && (
                <div className="border-t px-4 py-3 space-y-3 bg-muted/30" data-testid={`record-details-${record.id}`}>
                    {/* 项目名称 (如果有) */}
                    {record.project_name && (
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <FolderOpen className="h-4 w-4" />
                            <span>{t('privacy.records.list.project')}:</span>
                            <span className="text-foreground">{record.project_name}</span>
                        </div>
                    )}

                    {/* 检测到的敏感信息列表 */}
                    <div className="text-sm text-muted-foreground mb-2">
                        {t('privacy.records.list.detectedItems', { count: record.matches.length })}
                    </div>

                    <div className="space-y-2">
                        {record.matches.map((match, index) => (
                            <div
                                key={`${match.rule_id}-${index}`}
                                className="rounded border bg-background p-3 space-y-1"
                            >
                                <div className="flex items-center gap-2">
                                    <span className="font-medium">
                                        {SENSITIVE_TYPE_LABELS[match.sensitive_type] || match.sensitive_type}
                                    </span>
                                    <span className={`text-xs px-1.5 py-0.5 rounded ${SEVERITY_COLORS[match.severity]}`}>
                                        {SEVERITY_LABELS[match.severity]}
                                    </span>
                                </div>
                                <div className="text-xs text-muted-foreground">
                                    {t('privacy.records.list.line', { line: match.line })}
                                </div>
                                <div className="font-mono text-xs bg-muted px-2 py-1 rounded overflow-x-auto">
                                    {match.masked_text}
                                </div>
                                {match.context && (
                                    <div className="text-xs text-muted-foreground mt-1 truncate" title={match.context}>
                                        {match.context}
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}

export default InterceptionRecordItem;
