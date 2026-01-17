/**
 * RuleListItem Component - 规则列表项
 * Story 3.10: Task 4.2
 *
 * 显示单条规则，支持启用/禁用切换和删除操作
 */

import { useTranslation } from 'react-i18next';
import { Trash2 } from 'lucide-react';
import { Switch } from '@/components/ui/switch';
import {
    SENSITIVE_TYPE_LABELS,
    SEVERITY_COLORS,
    SEVERITY_LABELS,
    type SanitizationRule,
} from '@/components/sanitizer/types';

export interface RuleListItemProps {
    /** 规则数据 */
    rule: SanitizationRule;
    /** 是否为内置规则 (内置规则不可删除) */
    isBuiltin: boolean;
    /** 启用状态变化回调 */
    onToggle: (enabled: boolean) => void;
    /** 删除回调 (仅自定义规则) */
    onDelete?: () => void;
}

export function RuleListItem({
    rule,
    isBuiltin,
    onToggle,
    onDelete,
}: RuleListItemProps) {
    const { t } = useTranslation();

    return (
        <div
            className="flex items-center gap-3 py-2 px-3 rounded-lg hover:bg-accent/30 transition-colors group"
            data-testid={`rule-item-${rule.id}`}
        >
            {/* 启用/禁用开关 */}
            <Switch
                checked={rule.enabled}
                onCheckedChange={onToggle}
                aria-label={t('privacy.rules.toggle', { name: rule.name })}
                data-testid={`rule-switch-${rule.id}`}
            />

            {/* 规则信息 */}
            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                    <span className="font-medium text-sm truncate">{rule.name}</span>
                    <span className={`text-xs px-1.5 py-0.5 rounded ${SEVERITY_COLORS[rule.severity]}`}>
                        {SEVERITY_LABELS[rule.severity]}
                    </span>
                    {!isBuiltin && (
                        <span className="text-xs px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400">
                            {t('privacy.rules.custom')}
                        </span>
                    )}
                </div>
                <div className="text-xs text-muted-foreground font-mono truncate mt-0.5" title={rule.pattern}>
                    {rule.pattern}
                </div>
            </div>

            {/* 敏感类型标签 */}
            <div className="text-xs text-muted-foreground shrink-0">
                {SENSITIVE_TYPE_LABELS[rule.sensitive_type] || rule.sensitive_type}
            </div>

            {/* 删除按钮 (仅自定义规则) */}
            {!isBuiltin && onDelete && (
                <button
                    onClick={onDelete}
                    className="p-1.5 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded transition-colors opacity-0 group-hover:opacity-100"
                    aria-label={t('privacy.rules.delete', { name: rule.name })}
                    data-testid={`rule-delete-${rule.id}`}
                >
                    <Trash2 className="h-4 w-4" />
                </button>
            )}
        </div>
    );
}

export default RuleListItem;
