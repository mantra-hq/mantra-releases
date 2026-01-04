/**
 * RuleEditor - 规则编辑表单组件
 * Story 3-3: Task 2 - AC #1, #2
 * Story 2.26: 国际化支持
 */

import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertCircle, Check, Loader2 } from 'lucide-react';
import { validateRegex } from '@/lib/ipc/sanitizer-ipc';
import type { RuleFormData } from './types';
import type { SensitiveType } from '@/components/sanitizer/types';

export interface RuleEditorProps {
    initialData?: RuleFormData;
    onSave: (data: RuleFormData) => void;
    onCancel: () => void;
}

export function RuleEditor({ initialData, onSave, onCancel }: RuleEditorProps) {
    const { t } = useTranslation();

    /** 敏感类型选项 */
    const SENSITIVE_TYPE_OPTIONS: { value: SensitiveType; label: string }[] = [
        { value: 'custom', label: t("settings.custom") },
        { value: 'api_key', label: t("settings.apiKey") },
        { value: 'secret', label: t("settings.password") },
        { value: 'ip_address', label: t("settings.ipAddress") },
    ];

    const [formData, setFormData] = useState<RuleFormData>(
        initialData ?? {
            name: '',
            pattern: '',
            sensitiveType: 'custom',
        }
    );
    const [validationError, setValidationError] = useState<string | null>(null);
    const [isValidating, setIsValidating] = useState(false);
    const [isValid, setIsValid] = useState(!!initialData?.pattern);

    const handlePatternChange = useCallback(async (pattern: string) => {
        setFormData((prev) => ({ ...prev, pattern }));
        setIsValid(false);
        setValidationError(null);

        if (!pattern.trim()) return;

        setIsValidating(true);
        try {
            const result = await validateRegex(pattern);
            if (result.valid) {
                setIsValid(true);
            } else {
                setValidationError(result.error ?? t("settings.invalidRegex"));
            }
        } catch (err) {
            // 降级到本地验证 (测试环境)
            try {
                new RegExp(pattern);
                setIsValid(true);
            } catch (e) {
                setValidationError((e as Error).message);
            }
        } finally {
            setIsValidating(false);
        }
    }, [t]);

    const handleSubmit = useCallback(() => {
        if (!formData.name.trim() || !formData.pattern.trim() || !isValid) {
            return;
        }
        onSave(formData);
    }, [formData, isValid, onSave]);

    const isFormValid = formData.name.trim() !== '' && isValid;

    return (
        <div className="space-y-4 p-4" data-testid="rule-editor">
            <div className="space-y-2">
                <Label htmlFor="rule-name">{t("settings.ruleName")}</Label>
                <Input
                    id="rule-name"
                    placeholder={t("settings.exampleDomain")}
                    value={formData.name}
                    onChange={(e) =>
                        setFormData((prev) => ({ ...prev, name: e.target.value }))
                    }
                    data-testid="rule-name-input"
                />
            </div>

            <div className="space-y-2">
                <Label htmlFor="rule-pattern">{t("settings.regexPattern")}</Label>
                <Textarea
                    id="rule-pattern"
                    placeholder={t("settings.exampleEmail")}
                    className="font-mono text-sm"
                    value={formData.pattern}
                    onChange={(e) => handlePatternChange(e.target.value)}
                    data-testid="rule-pattern-input"
                />
                {isValidating && (
                    <p className="text-sm text-muted-foreground flex items-center gap-1">
                        <Loader2 className="h-3 w-3 animate-spin" />
                        {t("settings.validating")}
                    </p>
                )}
                {isValid && !isValidating && formData.pattern.trim() && (
                    <p className="text-sm text-green-500 flex items-center gap-1" data-testid="validation-success">
                        <Check className="h-4 w-4" /> {t("settings.regexValid")}
                    </p>
                )}
                {validationError && (
                    <Alert variant="destructive" data-testid="validation-error">
                        <AlertCircle className="h-4 w-4" />
                        <AlertDescription>{validationError}</AlertDescription>
                    </Alert>
                )}
            </div>

            <div className="space-y-2">
                <Label htmlFor="sensitive-type">{t("settings.sensitiveType")}</Label>
                <Select
                    value={formData.sensitiveType}
                    onValueChange={(value: SensitiveType) =>
                        setFormData((prev) => ({ ...prev, sensitiveType: value }))
                    }
                >
                    <SelectTrigger data-testid="sensitive-type-select">
                        <SelectValue placeholder={t("settings.selectType")} />
                    </SelectTrigger>
                    <SelectContent>
                        {SENSITIVE_TYPE_OPTIONS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>
                                {opt.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            <div className="flex justify-end gap-2 pt-4">
                <Button variant="outline" onClick={onCancel} data-testid="cancel-button">
                    {t("common.cancel")}
                </Button>
                <Button
                    onClick={handleSubmit}
                    disabled={!isFormValid}
                    data-testid="save-button"
                >
                    {t("common.save")}
                </Button>
            </div>
        </div>
    );
}

export default RuleEditor;
