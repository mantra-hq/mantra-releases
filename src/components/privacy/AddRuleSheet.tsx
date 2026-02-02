/**
 * AddRuleSheet Component - 添加自定义规则 Sheet
 * Story 12.2: 简单表单 Dialog 改造为 Sheet - Task 3
 *
 * 提供表单添加自定义脱敏规则
 */

import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetFooter,
    SheetHeader,
    SheetTitle,
} from '@/components/ui/sheet';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { validateRegexV2 } from '@/lib/ipc/sanitizer-ipc';
import type { SanitizationRule, Severity, SensitiveType } from '@/components/sanitizer/types';

export interface AddRuleSheetProps {
    /** 是否打开 */
    open: boolean;
    /** 关闭回调 */
    onOpenChange: (open: boolean) => void;
    /** 添加规则回调 */
    onAdd: (rule: SanitizationRule) => void;
    /** 现有规则 ID 列表 (用于验证唯一性) */
    existingRuleIds: string[];
    /** 现有规则名称列表 (用于验证名称唯一性) */
    existingRuleNames?: string[];
}

/** 生成唯一的规则 ID */
function generateRuleId(): string {
    return `custom_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
}

export function AddRuleSheet({
    open,
    onOpenChange,
    onAdd,
    existingRuleIds,
    existingRuleNames = [],
}: AddRuleSheetProps) {
    const { t } = useTranslation();
    const [name, setName] = useState('');
    const [pattern, setPattern] = useState('');
    const [severity, setSeverity] = useState<Severity>('warning');
    const [isValidating, setIsValidating] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleClose = useCallback(() => {
        setName('');
        setPattern('');
        setSeverity('warning');
        setError(null);
        onOpenChange(false);
    }, [onOpenChange]);

    const handleSubmit = useCallback(async () => {
        // 验证名称
        if (!name.trim()) {
            setError(t('privacy.rules.errors.emptyName'));
            return;
        }

        // 验证名称唯一性 (AC4)
        const normalizedName = name.trim().toLowerCase();
        if (existingRuleNames.some(n => n.toLowerCase() === normalizedName)) {
            setError(t('privacy.rules.errors.duplicateName'));
            return;
        }

        // 验证正则表达式
        if (!pattern.trim()) {
            setError(t('privacy.rules.errors.emptyPattern'));
            return;
        }

        setIsValidating(true);
        setError(null);

        try {
            const result = await validateRegexV2(pattern);
            if (!result.valid) {
                setError(result.error || t('privacy.rules.errors.invalidRegex'));
                setIsValidating(false);
                return;
            }

            // 创建规则
            const newRule: SanitizationRule = {
                id: generateRuleId(),
                name: name.trim(),
                pattern: pattern.trim(),
                sensitive_type: 'custom' as SensitiveType,
                severity,
                enabled: true,
            };

            // 检查 ID 唯一性 (理论上不会重复，但防御性检查)
            if (existingRuleIds.includes(newRule.id)) {
                setError(t('privacy.rules.errors.duplicateId'));
                setIsValidating(false);
                return;
            }

            onAdd(newRule);
            handleClose();
        } catch (_err) {
            setError(t('privacy.rules.errors.validationFailed'));
        } finally {
            setIsValidating(false);
        }
    }, [name, pattern, severity, existingRuleIds, existingRuleNames, onAdd, handleClose, t]);

    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetContent side="right" className="w-full max-w-md" data-testid="add-rule-sheet">
                <SheetHeader>
                    <SheetTitle>{t('privacy.rules.addTitle')}</SheetTitle>
                    <SheetDescription>
                        {t('privacy.rules.addDescription')}
                    </SheetDescription>
                </SheetHeader>

                <div className="space-y-4 py-4 px-4">
                    {/* 规则名称 */}
                    <div className="space-y-2">
                        <Label htmlFor="rule-name">{t('privacy.rules.name')}</Label>
                        <Input
                            id="rule-name"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            placeholder={t('privacy.rules.namePlaceholder')}
                            data-testid="rule-name-input"
                        />
                    </div>

                    {/* 正则表达式 */}
                    <div className="space-y-2">
                        <Label htmlFor="rule-pattern">{t('privacy.rules.pattern')}</Label>
                        <Input
                            id="rule-pattern"
                            value={pattern}
                            onChange={(e) => setPattern(e.target.value)}
                            placeholder={t('privacy.rules.patternPlaceholder')}
                            className="font-mono"
                            data-testid="rule-pattern-input"
                        />
                    </div>

                    {/* 严重程度 */}
                    <div className="space-y-2">
                        <Label>{t('privacy.rules.severity')}</Label>
                        <Select value={severity} onValueChange={(v) => setSeverity(v as Severity)}>
                            <SelectTrigger data-testid="rule-severity-select">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="critical">{t('privacy.severity.critical')}</SelectItem>
                                <SelectItem value="warning">{t('privacy.severity.warning')}</SelectItem>
                                <SelectItem value="info">{t('privacy.severity.info')}</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>

                    {/* 错误提示 */}
                    {error && (
                        <div className="text-sm text-destructive" data-testid="add-rule-error">
                            {error}
                        </div>
                    )}
                </div>

                <SheetFooter>
                    <Button variant="outline" onClick={handleClose} data-testid="add-rule-cancel">
                        {t('common.cancel')}
                    </Button>
                    <Button
                        onClick={handleSubmit}
                        disabled={isValidating}
                        data-testid="add-rule-submit"
                    >
                        {isValidating ? t('common.validating') : t('common.add')}
                    </Button>
                </SheetFooter>
            </SheetContent>
        </Sheet>
    );
}

export default AddRuleSheet;
