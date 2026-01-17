/**
 * RecordFilters Component - 记录筛选栏
 * Story 3-8: Task 3 - AC #3
 *
 * 支持按来源、敏感类型和时间范围筛选
 */

import { useTranslation } from 'react-i18next';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { SENSITIVE_TYPE_LABELS, type SensitiveType, type InterceptionSourceType } from '@/components/sanitizer/types';

export type TimeRange = 'all' | 'today' | 'thisWeek' | 'thisMonth';

export interface RecordFiltersProps {
    /** 来源筛选值 */
    source: InterceptionSourceType | 'all';
    /** 敏感类型筛选值 */
    sensitiveType: SensitiveType | 'all';
    /** 时间范围筛选值 */
    timeRange: TimeRange;
    /** 来源变化回调 */
    onSourceChange: (value: InterceptionSourceType | 'all') => void;
    /** 敏感类型变化回调 */
    onSensitiveTypeChange: (value: SensitiveType | 'all') => void;
    /** 时间范围变化回调 */
    onTimeRangeChange: (value: TimeRange) => void;
}

const SOURCE_OPTIONS: Array<{ value: InterceptionSourceType | 'all'; labelKey: string }> = [
    { value: 'all', labelKey: 'privacy.records.filter.all' },
    { value: 'pre_upload', labelKey: 'privacy.records.source.preUpload' },
    { value: 'claude_code_hook', labelKey: 'privacy.records.source.claudeCodeHook' },
    { value: 'external_hook', labelKey: 'privacy.records.source.externalHook' },
];

const TIME_OPTIONS: Array<{ value: TimeRange; labelKey: string }> = [
    { value: 'all', labelKey: 'privacy.records.filter.allTime' },
    { value: 'today', labelKey: 'privacy.records.filter.today' },
    { value: 'thisWeek', labelKey: 'privacy.records.filter.thisWeek' },
    { value: 'thisMonth', labelKey: 'privacy.records.filter.thisMonth' },
];

// 获取常用敏感类型选项
const SENSITIVE_TYPE_OPTIONS: Array<SensitiveType | 'all'> = [
    'all',
    'api_key',
    'password',
    'email',
    'phone',
    'ip_address',
    'jwt_token',
    'private_key',
    'custom',
];

export function RecordFilters({
    source,
    sensitiveType,
    timeRange,
    onSourceChange,
    onSensitiveTypeChange,
    onTimeRangeChange,
}: RecordFiltersProps) {
    const { t } = useTranslation();

    return (
        <div
            className="flex flex-wrap items-center gap-3"
            data-testid="record-filters"
        >
            {/* 来源筛选 */}
            <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">
                    {t('privacy.records.filter.source')}:
                </span>
                <Select
                    value={source}
                    onValueChange={(value) => onSourceChange(value as InterceptionSourceType | 'all')}
                >
                    <SelectTrigger className="w-[150px]" data-testid="filter-source">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {SOURCE_OPTIONS.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                                {t(option.labelKey)}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            {/* 敏感类型筛选 */}
            <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">
                    {t('privacy.records.filter.type')}:
                </span>
                <Select
                    value={sensitiveType}
                    onValueChange={(value) => onSensitiveTypeChange(value as SensitiveType | 'all')}
                >
                    <SelectTrigger className="w-[150px]" data-testid="filter-type">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {SENSITIVE_TYPE_OPTIONS.map((type) => (
                            <SelectItem key={type} value={type}>
                                {type === 'all'
                                    ? t('privacy.records.filter.all')
                                    : SENSITIVE_TYPE_LABELS[type]}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            {/* 时间范围筛选 */}
            <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">
                    {t('privacy.records.filter.time')}:
                </span>
                <Select
                    value={timeRange}
                    onValueChange={(value) => onTimeRangeChange(value as TimeRange)}
                >
                    <SelectTrigger className="w-[130px]" data-testid="filter-time">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {TIME_OPTIONS.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                                {t(option.labelKey)}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
        </div>
    );
}

export default RecordFilters;
