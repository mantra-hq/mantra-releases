/**
 * SystemRuleList - 系统预设规则列表组件
 * Story 3-5: Task 3 - AC #1, #2, #3
 * Story 3-10: 支持启用/禁用内置规则 + 整体折叠
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Lock, ChevronDown, ChevronRight, ShieldCheck, Loader2 } from 'lucide-react';
import { Switch } from '@/components/ui/switch';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { getPrivacyRules, updatePrivacyRules } from '@/lib/ipc/sanitizer-ipc';
import type { SanitizationRule } from '@/components/sanitizer/types';

export interface SystemRuleListProps {
    className?: string;
    /** 默认折叠整个区域 */
    defaultCollapsed?: boolean;
}

export function SystemRuleList({ className, defaultCollapsed = false }: SystemRuleListProps) {
    const { t } = useTranslation();
    const [rules, setRules] = useState<SanitizationRule[]>([]);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());
    const [isOpen, setIsOpen] = useState(!defaultCollapsed);

    // 加载规则（内置 + 自定义合并后的列表）
    useEffect(() => {
        async function loadRules() {
            try {
                setLoading(true);
                // 获取合并后的规则，包含内置规则的启用状态
                const mergedRules = await getPrivacyRules();
                // 只显示非自定义规则（内置规则）
                const builtinRules = mergedRules.filter(
                    r => (r as unknown as { sensitive_type: string }).sensitive_type !== 'custom'
                );
                setRules(builtinRules);
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

    // 切换规则启用/禁用状态
    const handleToggleRule = useCallback(async (ruleId: string, enabled: boolean) => {
        // 乐观更新 UI
        setRules(prev => prev.map(r =>
            r.id === ruleId ? { ...r, enabled } : r
        ));

        setSaving(true);
        try {
            // 构建内置规则状态 map
            const currentRules = rules.map(r =>
                r.id === ruleId ? { ...r, enabled } : r
            );
            const builtinEnabled: Record<string, boolean> = {};
            for (const rule of currentRules) {
                builtinEnabled[rule.id] = rule.enabled;
            }

            // 保存配置 (保留现有自定义规则)
            await updatePrivacyRules({
                builtin_enabled: builtinEnabled,
                custom_rules: [], // 保持现有自定义规则不变
            });
        } catch (err) {
            // 回滚
            setRules(prev => prev.map(r =>
                r.id === ruleId ? { ...r, enabled: !enabled } : r
            ));
            console.error('Failed to toggle rule:', err);
        } finally {
            setSaving(false);
        }
    }, [rules]);

    // 统计启用的规则数量
    const enabledCount = useMemo(() =>
        rules.filter(r => r.enabled).length,
        [rules]
    );

    return (
        <Collapsible open={isOpen} onOpenChange={setIsOpen} className={className} data-testid="system-rule-list">
            <CollapsibleTrigger asChild>
                <button
                    type="button"
                    className="flex items-center gap-2 w-full text-left -m-4 p-4 rounded-lg transition-all duration-200 hover:bg-accent hover:shadow-sm active:scale-[0.99] cursor-pointer group"
                >
                    {isOpen ? (
                        <ChevronDown className="h-5 w-5 text-muted-foreground group-hover:text-foreground transition-colors" />
                    ) : (
                        <ChevronRight className="h-5 w-5 text-muted-foreground group-hover:text-foreground transition-colors" />
                    )}
                    <ShieldCheck className="h-5 w-5 text-muted-foreground group-hover:text-foreground transition-colors" />
                    <h3 className="text-lg font-medium">{t('settings.builtinRules')}</h3>
                    <span className="text-xs text-muted-foreground ml-auto mr-4">
                        {loading ? '...' : `${enabledCount}/${rules.length} ${t('common.enabled', { defaultValue: 'enabled' })}`}
                    </span>
                    {saving && <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />}
                </button>
            </CollapsibleTrigger>

            <CollapsibleContent className="pt-4">
                {loading && (
                    <div className="text-sm text-muted-foreground py-4 text-center">
                        {t('common.loading')}
                    </div>
                )}

                {error && (
                    <div className="text-sm text-destructive py-4 text-center">
                        {t('common.loadFailed')}: {error}
                    </div>
                )}

                {!loading && !error && (
                    <div className="space-y-2">
                        {Object.entries(groupedRules).map(([type, typeRules]) => {
                            const isExpanded = expandedGroups.has(type);
                            const typeEnabledCount = typeRules.filter(r => r.enabled).length;
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
                                            {typeEnabledCount}/{typeRules.length}
                                        </span>
                                    </button>

                                    {isExpanded && (
                                        <div className="space-y-1 p-2 pt-1">
                                            {typeRules.map((rule) => {
                                                const ruleData = rule as unknown as { name: string; pattern: string };
                                                return (
                                                    <div
                                                        key={rule.id}
                                                        className="flex items-center gap-2 p-2 rounded-md bg-muted/30"
                                                        data-testid={`rule-${rule.id}`}
                                                    >
                                                        <Lock className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
                                                        <div className="flex-1 min-w-0">
                                                            <p className={`text-sm ${!rule.enabled ? 'text-muted-foreground/50 line-through' : 'text-muted-foreground'}`}>
                                                                {ruleData.name}
                                                            </p>
                                                            <p className="text-xs font-mono text-muted-foreground/70 truncate">
                                                                {ruleData.pattern}
                                                            </p>
                                                        </div>
                                                        <Switch
                                                            checked={rule.enabled}
                                                            onCheckedChange={(checked) => handleToggleRule(rule.id, checked)}
                                                            aria-label={t('privacy.rules.toggle', { name: ruleData.name })}
                                                            data-testid={`toggle-${rule.id}`}
                                                        />
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                )}
            </CollapsibleContent>
        </Collapsible>
    );
}

export default SystemRuleList;
