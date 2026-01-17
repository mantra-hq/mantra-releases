/**
 * InterceptionStats Component - 拦截统计概览
 * Story 3-8: Task 2 - AC #2
 *
 * 显示 4 个统计卡片：总拦截数、本周拦截数、已脱敏数、已忽略数
 */

import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Shield, ShieldCheck, ShieldAlert, ShieldX, Calendar } from 'lucide-react';
import { getInterceptionStats } from '@/lib/ipc/sanitizer-ipc';
import { Skeleton } from '@/components/ui/skeleton';
import type { InterceptionStats as IInterceptionStats } from '@/components/sanitizer/types';

export interface InterceptionStatsProps {
    /** 刷新触发器 (变化时重新加载数据) */
    refreshTrigger?: number;
}

export function InterceptionStats({ refreshTrigger }: InterceptionStatsProps) {
    const { t } = useTranslation();
    const [stats, setStats] = useState<IInterceptionStats | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const loadStats = async () => {
            setLoading(true);
            setError(null);
            try {
                const data = await getInterceptionStats();
                setStats(data);
            } catch (err) {
                console.error('Failed to load interception stats:', err);
                setError((err as Error).message);
            } finally {
                setLoading(false);
            }
        };

        loadStats();
    }, [refreshTrigger]);

    if (loading) {
        return (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4" data-testid="interception-stats-loading">
                {[...Array(4)].map((_, i) => (
                    <div key={i} className="rounded-lg border bg-card p-4">
                        <Skeleton className="h-4 w-20 mb-2" />
                        <Skeleton className="h-8 w-16" />
                    </div>
                ))}
            </div>
        );
    }

    if (error) {
        return (
            <div className="rounded-lg border bg-destructive/10 p-4 text-destructive" data-testid="interception-stats-error">
                {t("common.loadFailed")}: {error}
            </div>
        );
    }

    if (!stats) {
        return null;
    }

    // 从 by_action 获取各类操作数量
    const redactedCount = stats.by_action?.redacted ?? 0;
    const ignoredCount = stats.by_action?.ignored ?? 0;

    const statCards = [
        {
            key: 'total',
            label: t('privacy.records.stats.total'),
            value: stats.total_interceptions,
            icon: Shield,
            color: 'text-blue-500',
            bgColor: 'bg-blue-500/10',
        },
        {
            key: 'thisWeek',
            label: t('privacy.records.stats.thisWeek'),
            value: stats.recent_7_days,
            icon: Calendar,
            color: 'text-purple-500',
            bgColor: 'bg-purple-500/10',
        },
        {
            key: 'redacted',
            label: t('privacy.records.stats.redacted'),
            value: redactedCount,
            icon: ShieldCheck,
            color: 'text-emerald-500',
            bgColor: 'bg-emerald-500/10',
        },
        {
            key: 'ignored',
            label: t('privacy.records.stats.ignored'),
            value: ignoredCount,
            icon: ShieldX,
            color: 'text-yellow-500',
            bgColor: 'bg-yellow-500/10',
        },
    ];

    return (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4" data-testid="interception-stats">
            {statCards.map((card) => (
                <div
                    key={card.key}
                    className="rounded-lg border bg-card p-4 transition-colors hover:bg-accent/50"
                    data-testid={`stat-card-${card.key}`}
                >
                    <div className="flex items-center gap-2 mb-2">
                        <div className={`p-1.5 rounded-md ${card.bgColor}`}>
                            <card.icon className={`h-4 w-4 ${card.color}`} />
                        </div>
                        <span className="text-sm text-muted-foreground">{card.label}</span>
                    </div>
                    <div className="text-2xl font-bold tabular-nums">{card.value}</div>
                </div>
            ))}
        </div>
    );
}

export default InterceptionStats;
