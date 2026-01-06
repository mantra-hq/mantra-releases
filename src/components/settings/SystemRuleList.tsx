/**
 * SystemRuleList - 系统预设规则列表组件
 * Story 3-5: Task 3 - AC #1, #2, #3
 */

import { useState, useEffect, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Lock, ChevronDown, ChevronRight, ShieldCheck } from 'lucide-react';
import { getBuiltinRules } from '@/lib/ipc/sanitizer-ipc';
import type { SanitizationRule } from '@/components/sanitizer/types';

export interface SystemRuleListProps {
    className?: string;
}

export function SystemRuleList({ className }: SystemRuleListProps) {
    const { t } = useTranslation();
    const [rules, setRules] = useState<SanitizationRule[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());

    // 加载内置规则
    useEffect(() => {
        async function loadRules() {
            try {
                setLoading(true);
                const builtinRules = await getBuiltinRules();
                setRules(builtinRules);
                // 默认展开第一个分组
                if (builtinRules.length > 0) {
                    const firstType = (builtinRules[0] as unknown as { sensitive_type: string }).sensitive_type;
                    if (firstType) {
                        setExpandedGroups(new Set([firstType]));
                    }
                }
            } catch (err) {
                setError((err as Error).message);
            } finally {
                setLoading(false);
            }
        }
        loadRules();
    }, []);

    // 按 sensitive_type 分组
    const groupedRules = useMemo(() => {
        const groups: Record<string, SanitizationRule[]> = {};

        for (const rule of rules) {
            const ruleWithType = rule as unknown as { sensitive_type: string; name: string; pattern: string };
            const type = ruleWithType.sensitive_type || 'unknown';

            if (!groups[type]) {
                groups[type] = [];
            }
            groups[type].push(rule);
        }

        return groups;
    }, [rules]);

    const toggleGroup = (type: string) => {
        setExpandedGroups(prev => {
            const next = new Set(prev);
            if (next.has(type)) {
                next.delete(type);
            } else {
                next.add(type);
            }
            return next;
        });
    };

    if (loading) {
        return (
            <div className={className}>
                <div className="flex items-center gap-2 mb-3">
                    <ShieldCheck className="h-5 w-5 text-muted-foreground" />
                    <h3 className="text-lg font-medium">{t('settings.builtinRules')}</h3>
                </div>
                <div className="text-sm text-muted-foreground py-4 text-center">
                    {t('common.loading')}
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className={className}>
                <div className="flex items-center gap-2 mb-3">
                    <ShieldCheck className="h-5 w-5 text-muted-foreground" />
                    <h3 className="text-lg font-medium">{t('settings.builtinRules')}</h3>
                </div>
                <div className="text-sm text-destructive py-4 text-center">
                    {t('common.loadFailed')}: {error}
                </div>
            </div>
        );
    }

    return (
        <div className={className} data-testid="system-rule-list">
            <div className="flex items-center gap-2 mb-3">
                <ShieldCheck className="h-5 w-5 text-muted-foreground" />
                <h3 className="text-lg font-medium">{t('settings.builtinRules')}</h3>
                <span className="text-xs text-muted-foreground ml-auto">
                    {t('settings.builtinRulesCount', { count: rules.length })}
                </span>
            </div>

            <div className="space-y-2">
                {Object.entries(groupedRules).map(([type, typeRules]) => {
                    const isExpanded = expandedGroups.has(type);
                    return (
                        <div key={type} className="rounded-lg border border-dashed">
                            <button
                                type="button"
                                onClick={() => toggleGroup(type)}
                                className="flex items-center gap-2 w-full p-2 rounded-t-lg bg-muted/50 hover:bg-muted/70 transition-colors text-left"
                                data-testid={`group-${type}`}
                            >
                                {isExpanded ? (
                                    <ChevronDown className="h-4 w-4 text-muted-foreground" />
                                ) : (
                                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                )}
                                <span className="font-medium text-sm">
                                    {t(`settings.sensitiveTypes.${type}`, { defaultValue: type })}
                                </span>
                                <span className="text-xs text-muted-foreground ml-auto">
                                    {typeRules.length}
                                </span>
                            </button>

                            {isExpanded && (
                                <div className="space-y-1 p-2 pt-1">
                                    {typeRules.map((rule, index) => {
                                        const ruleData = rule as unknown as { name: string; pattern: string };
                                        return (
                                            <div
                                                key={`${type}-${index}`}
                                                className="flex items-center gap-2 p-2 rounded-md bg-muted/30"
                                                data-testid={`rule-${type}-${index}`}
                                            >
                                                <Lock className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
                                                <div className="flex-1 min-w-0">
                                                    <p className="text-sm text-muted-foreground">
                                                        {ruleData.name}
                                                    </p>
                                                    <p className="text-xs font-mono text-muted-foreground/70 truncate">
                                                        {ruleData.pattern}
                                                    </p>
                                                </div>
                                            </div>
                                        );
                                    })}
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

export default SystemRuleList;
