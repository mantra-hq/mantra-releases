/**
 * RuleEditor - 规则编辑表单组件
 * Story 3-3: Task 2 - AC #1, #2
 */

import { useState, useCallback } from 'react';
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

/** 敏感类型选项 */
const SENSITIVE_TYPE_OPTIONS: { value: SensitiveType; label: string }[] = [
    { value: 'custom', label: '自定义' },
    { value: 'api_key', label: 'API Key' },
    { value: 'secret', label: '密码/Secret' },
    { value: 'ip_address', label: 'IP 地址' },
];

export interface RuleEditorProps {
    initialData?: RuleFormData;
    onSave: (data: RuleFormData) => void;
    onCancel: () => void;
}

export function RuleEditor({ initialData, onSave, onCancel }: RuleEditorProps) {
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
                setValidationError(result.error ?? '无效的正则表达式');
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
    }, []);

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
                <Label htmlFor="rule-name">规则名称</Label>
                <Input
                    id="rule-name"
                    placeholder="例如: 公司内部域名"
                    value={formData.name}
                    onChange={(e) =>
                        setFormData((prev) => ({ ...prev, name: e.target.value }))
                    }
                    data-testid="rule-name-input"
                />
            </div>

            <div className="space-y-2">
                <Label htmlFor="rule-pattern">正则表达式</Label>
                <Textarea
                    id="rule-pattern"
                    placeholder="例如: \\b\\w+@company\\.com\\b"
                    className="font-mono text-sm"
                    value={formData.pattern}
                    onChange={(e) => handlePatternChange(e.target.value)}
                    data-testid="rule-pattern-input"
                />
                {isValidating && (
                    <p className="text-sm text-muted-foreground flex items-center gap-1">
                        <Loader2 className="h-3 w-3 animate-spin" />
                        正在验证...
                    </p>
                )}
                {isValid && !isValidating && formData.pattern.trim() && (
                    <p className="text-sm text-green-500 flex items-center gap-1" data-testid="validation-success">
                        <Check className="h-4 w-4" /> 正则表达式有效
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
                <Label htmlFor="sensitive-type">敏感类型</Label>
                <Select
                    value={formData.sensitiveType}
                    onValueChange={(value: SensitiveType) =>
                        setFormData((prev) => ({ ...prev, sensitiveType: value }))
                    }
                >
                    <SelectTrigger data-testid="sensitive-type-select">
                        <SelectValue placeholder="选择类型" />
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
                    取消
                </Button>
                <Button
                    onClick={handleSubmit}
                    disabled={!isFormValid}
                    data-testid="save-button"
                >
                    保存
                </Button>
            </div>
        </div>
    );
}

export default RuleEditor;
