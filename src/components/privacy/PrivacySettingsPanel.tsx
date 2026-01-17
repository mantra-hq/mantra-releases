/**
 * PrivacySettingsPanel Component - 隐私规则设置面板
 * Story 3.10: Task 4.1
 *
 * 显示所有隐私规则（内置 + 自定义），支持启用/禁用和添加自定义规则
 */

import { useState, useEffect, useCallback, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, ChevronDown, ChevronUp, Loader2, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { getPrivacyRules, updatePrivacyRules, getBuiltinRules } from '@/lib/ipc/sanitizer-ipc';
import type { SanitizationRule, PrivacyRulesConfig, SensitiveType } from '@/components/sanitizer/types';
import { SENSITIVE_TYPE_LABELS } from '@/components/sanitizer/types';
import { RuleListItem } from './RuleListItem';
import { AddRuleDialog } from './AddRuleDialog';

export interface PrivacySettingsPanelProps {
    /** 外部样式 */
    className?: string;
}

/** 按敏感类型分组规则 */
interface RuleGroup {
    type: SensitiveType;
    label: string;
    rules: SanitizationRule[];
    isBuiltin: boolean;
}

export function PrivacySettingsPanel({ className }: PrivacySettingsPanelProps) {
    const { t } = useTranslation();
    const [rules, setRules] = useState<SanitizationRule[]>([]);
    const [builtinRuleIds, setBuiltinRuleIds] = useState<Set<string>>(new Set());
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());
    const [addDialogOpen, setAddDialogOpen] = useState(false);
    const [deleteTarget, setDeleteTarget] = useState<SanitizationRule | null>(null);

    // 加载规则
    const loadRules = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            // 并行加载合并规则和内置规则列表
            const [mergedRules, builtinRules] = await Promise.all([
                getPrivacyRules(),
                getBuiltinRules(),
            ]);

            setRules(mergedRules);
            setBuiltinRuleIds(new Set(builtinRules.map(r => r.id)));
        } catch (err) {
            setError(t('privacy.rules.errors.loadFailed'));
            console.error('Failed to load privacy rules:', err);
        } finally {
            setLoading(false);
        }
    }, [t]);

    useEffect(() => {
        loadRules();
    }, [loadRules]);

    // 保存规则配置
    const saveRules = useCallback(async (newRules: SanitizationRule[]) => {
        try {
            setSaving(true);
            setError(null);

            // 构建配置对象
            const config: PrivacyRulesConfig = {
                builtin_enabled: {},
                custom_rules: [],
            };

            for (const rule of newRules) {
                if (builtinRuleIds.has(rule.id)) {
                    // 内置规则：只保存启用状态
                    config.builtin_enabled[rule.id] = rule.enabled;
                } else {
                    // 自定义规则：保存完整规则
                    config.custom_rules.push(rule);
                }
            }

            await updatePrivacyRules(config);
            setRules(newRules);
        } catch (err) {
            setError(t('privacy.rules.errors.saveFailed'));
            console.error('Failed to save privacy rules:', err);
        } finally {
            setSaving(false);
        }
    }, [builtinRuleIds, t]);

    // 切换规则启用状态
    const handleToggle = useCallback((ruleId: string, enabled: boolean) => {
        const newRules = rules.map(r =>
            r.id === ruleId ? { ...r, enabled } : r
        );
        saveRules(newRules);
    }, [rules, saveRules]);

    // 请求删除自定义规则（显示确认对话框）
    const handleDeleteRequest = useCallback((rule: SanitizationRule) => {
        setDeleteTarget(rule);
    }, []);

    // 确认删除自定义规则
    const handleDeleteConfirm = useCallback(() => {
        if (deleteTarget) {
            const newRules = rules.filter(r => r.id !== deleteTarget.id);
            saveRules(newRules);
            setDeleteTarget(null);
        }
    }, [deleteTarget, rules, saveRules]);

    // 添加自定义规则
    const handleAddRule = useCallback((newRule: SanitizationRule) => {
        const newRules = [...rules, newRule];
        saveRules(newRules);
    }, [rules, saveRules]);

    // 按敏感类型分组
    const ruleGroups = useMemo((): RuleGroup[] => {
        const groups: Map<string, RuleGroup> = new Map();

        // 先添加内置规则分组
        for (const rule of rules) {
            if (builtinRuleIds.has(rule.id)) {
                const key = rule.sensitive_type;
                if (!groups.has(key)) {
                    groups.set(key, {
                        type: key as SensitiveType,
                        label: SENSITIVE_TYPE_LABELS[key as SensitiveType] || key,
                        rules: [],
                        isBuiltin: true,
                    });
                }
                groups.get(key)!.rules.push(rule);
            }
        }

        // 自定义规则单独分组
        const customRules = rules.filter(r => !builtinRuleIds.has(r.id));
        if (customRules.length > 0) {
            groups.set('custom', {
                type: 'custom' as SensitiveType,
                label: t('privacy.rules.customGroup'),
                rules: customRules,
                isBuiltin: false,
            });
        }

        return Array.from(groups.values());
    }, [rules, builtinRuleIds, t]);

    // 切换分组折叠状态
    const toggleGroup = useCallback((groupType: string) => {
        setCollapsedGroups(prev => {
            const next = new Set(prev);
            if (next.has(groupType)) {
                next.delete(groupType);
            } else {
                next.add(groupType);
            }
            return next;
        });
    }, []);

    if (loading) {
        return (
            <div className={`flex items-center justify-center p-8 ${className}`}>
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
        );
    }

    return (
        <div className={className} data-testid="privacy-settings-panel">
            {/* 头部 */}
            <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold">{t('privacy.rules.title')}</h3>
                <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setAddDialogOpen(true)}
                    disabled={saving}
                    data-testid="add-rule-button"
                >
                    <Plus className="h-4 w-4 mr-1" />
                    {t('privacy.rules.add')}
                </Button>
            </div>

            {/* 错误提示 */}
            {error && (
                <div className="flex items-center gap-2 p-3 mb-4 rounded-lg bg-destructive/10 text-destructive">
                    <AlertCircle className="h-4 w-4" />
                    <span className="text-sm">{error}</span>
                </div>
            )}

            {/* 规则分组列表 */}
            <div className="space-y-2">
                {ruleGroups.map(group => {
                    const isCollapsed = collapsedGroups.has(group.type);
                    const enabledCount = group.rules.filter(r => r.enabled).length;

                    return (
                        <div
                            key={group.type}
                            className="rounded-lg border bg-card"
                            data-testid={`rule-group-${group.type}`}
                        >
                            {/* 分组头部 */}
                            <button
                                className="flex items-center justify-between w-full p-3 hover:bg-accent/30 transition-colors"
                                onClick={() => toggleGroup(group.type)}
                                data-testid={`rule-group-toggle-${group.type}`}
                            >
                                <div className="flex items-center gap-2">
                                    {isCollapsed ? (
                                        <ChevronDown className="h-4 w-4" />
                                    ) : (
                                        <ChevronUp className="h-4 w-4" />
                                    )}
                                    <span className="font-medium">{group.label}</span>
                                    <span className="text-xs text-muted-foreground">
                                        ({enabledCount}/{group.rules.length})
                                    </span>
                                </div>
                            </button>

                            {/* 分组内规则 */}
                            {!isCollapsed && (
                                <div className="border-t px-1 py-1">
                                    {group.rules.map(rule => (
                                        <RuleListItem
                                            key={rule.id}
                                            rule={rule}
                                            isBuiltin={group.isBuiltin}
                                            onToggle={(enabled) => handleToggle(rule.id, enabled)}
                                            onDelete={!group.isBuiltin ? () => handleDeleteRequest(rule) : undefined}
                                        />
                                    ))}
                                </div>
                            )}
                        </div>
                    );
                })}

                {ruleGroups.length === 0 && (
                    <div className="text-center py-8 text-muted-foreground">
                        {t('privacy.rules.empty')}
                    </div>
                )}
            </div>

            {/* 保存状态指示 */}
            {saving && (
                <div className="fixed bottom-4 right-4 flex items-center gap-2 px-3 py-2 rounded-lg bg-muted">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span className="text-sm">{t('common.saving')}</span>
                </div>
            )}

            {/* 添加规则对话框 */}
            <AddRuleDialog
                open={addDialogOpen}
                onOpenChange={setAddDialogOpen}
                onAdd={handleAddRule}
                existingRuleIds={rules.map(r => r.id)}
                existingRuleNames={rules.map(r => r.name)}
            />

            {/* 删除确认对话框 */}
            <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>{t('privacy.rules.deleteConfirmTitle')}</AlertDialogTitle>
                        <AlertDialogDescription>
                            {t('privacy.rules.deleteConfirmDesc', { name: deleteTarget?.name })}
                        </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
                        <AlertDialogAction onClick={handleDeleteConfirm} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
                            {t('common.delete')}
                        </AlertDialogAction>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>
        </div>
    );
}

export default PrivacySettingsPanel;
