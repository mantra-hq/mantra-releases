/**
 * RuleTestPanel - 规则测试面板
 * Story 3-3: Task 4 - AC #4
 * Story 2.26: 国际化支持
 */

import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Play, Loader2 } from 'lucide-react';
import { sanitizeText } from '@/lib/ipc/sanitizer-ipc';
import { useSanitizationRulesStore } from '@/stores/useSanitizationRulesStore';
import { SanitizationSummary } from '@/components/sanitizer/SanitizationSummary';
import type { SanitizationResult } from '@/components/sanitizer/types';

export function RuleTestPanel() {
    const { t } = useTranslation();
    const [testText, setTestText] = useState('');
    const [result, setResult] = useState<SanitizationResult | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const { getEnabledRules } = useSanitizationRulesStore();

    const handleTest = async () => {
        if (!testText.trim()) return;

        setIsLoading(true);
        setError(null);
        setResult(null);

        try {
            const enabledRules = getEnabledRules();
            const customPatterns = enabledRules.map((rule) => ({
                name: rule.name,
                pattern: rule.pattern,
                replacement: `[REDACTED:${rule.sensitiveType.toUpperCase()}]`,
            }));

            const sanitizationResult = await sanitizeText(testText, customPatterns);
            setResult(sanitizationResult);
        } catch (err) {
            setError((err as Error).message || t("settings.testFailed"));
        } finally {
            setIsLoading(false);
        }
    };

    const enabledRulesCount = getEnabledRules().length;

    return (
        <div className="space-y-4 border rounded-lg p-4" data-testid="rule-test-panel">
            <h4 className="font-medium">{t("settings.ruleTest")}</h4>

            <div className="space-y-2">
                <Label htmlFor="testText">{t("settings.testText")}</Label>
                <Textarea
                    id="testText"
                    placeholder={t("settings.testInputPlaceholder")}
                    className="h-32 font-mono text-sm"
                    value={testText}
                    onChange={(e) => setTestText(e.target.value)}
                    data-testid="test-text-input"
                />
                <p className="text-xs text-muted-foreground">
                    {t("settings.enabledRulesCount", { count: enabledRulesCount })}
                </p>
            </div>

            <Button
                onClick={handleTest}
                disabled={isLoading || !testText.trim()}
                data-testid="run-test-button"
            >
                {isLoading ? (
                    <>
                        <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                        {t("settings.testing")}
                    </>
                ) : (
                    <>
                        <Play className="h-4 w-4 mr-1" />
                        {t("settings.runTest")}
                    </>
                )}
            </Button>

            {error && (
                <div className="p-3 rounded-lg bg-destructive/10 text-destructive text-sm" data-testid="test-error">
                    {error}
                </div>
            )}

            {result && (
                <div className="space-y-4" data-testid="test-results">
                    <SanitizationSummary stats={result.stats} />
                    <div className="space-y-2">
                        <Label>{t("settings.sanitizedResult")}</Label>
                        <div className="p-3 rounded-lg bg-muted font-mono text-sm whitespace-pre-wrap max-h-48 overflow-auto" data-testid="sanitized-result">
                            {result.sanitized_text}
                        </div>
                    </div>
                    {!result.has_matches && (
                        <p className="text-sm text-muted-foreground">
                            {t("settings.noMatchFound")}
                        </p>
                    )}
                </div>
            )}
        </div>
    );
}

export default RuleTestPanel;
