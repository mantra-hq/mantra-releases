/**
 * SanitizationSummary 组件 - Story 3-2 Task 5
 * 显示脱敏统计摘要面板
 */

import { useTranslation } from 'react-i18next';
import { AlertTriangle, ShieldCheck } from 'lucide-react';
import type { SanitizationSummaryProps, SensitiveType } from './types';
import { SENSITIVE_TYPE_LABELS } from './types';

export function SanitizationSummary({ stats }: SanitizationSummaryProps) {
    const { t } = useTranslation();
    const entries = Object.entries(stats.counts).filter(
        ([, count]) => count !== undefined && count > 0
    ) as [SensitiveType, number][];

    // 无敏感信息的情况
    if (stats.total === 0) {
        return (
            <div className="border-b p-4 bg-muted/50">
                <div className="flex items-center gap-2">
                    <ShieldCheck className="h-4 w-4 text-green-500" />
                    <span className="font-medium">{t('sanitizer.noSensitiveFound')}</span>
                </div>
                <p className="text-sm text-muted-foreground mt-1">
                    {t('sanitizer.safeToShare')}
                </p>
            </div>
        );
    }

    return (
        <div className="border-b p-4 bg-muted/50">
            <div className="flex items-center gap-2 mb-2">
                <AlertTriangle className="h-4 w-4 text-yellow-500" />
                <span className="font-medium">
                    {t('sanitizer.detectedSensitive', { count: stats.total })}
                </span>
            </div>
            <div className="flex flex-wrap gap-2">
                {entries.map(([type, count]) => (
                    <span
                        key={type}
                        className="px-2 py-0.5 rounded-full bg-background text-xs border"
                    >
                        {SENSITIVE_TYPE_LABELS[type]}: {count}
                    </span>
                ))}
            </div>
        </div>
    );
}
