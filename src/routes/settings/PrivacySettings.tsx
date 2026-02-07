/**
 * PrivacySettings - 隐私与安全设置页面
 * Story 2-35: Task 3.3
 *
 * 包含 SystemRuleList + RuleList + RuleTestPanel + 隐私记录入口链接
 */

import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Shield, ChevronRight } from 'lucide-react';
import { RuleList } from '@/components/settings/RuleList';
import { RuleTestPanel } from '@/components/settings/RuleTestPanel';
import { SystemRuleList } from '@/components/settings/SystemRuleList';
import { useSanitizationRulesStore } from '@/stores/useSanitizationRulesStore';
import { exportRules, importRules } from '@/lib/rule-io';
import { feedback } from '@/lib/feedback';

export function PrivacySettings() {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const { rules, importRules: storeImportRules } = useSanitizationRulesStore();
    const [isImporting, setIsImporting] = useState(false);
    const [isExporting, setIsExporting] = useState(false);

    const handleImport = useCallback(async () => {
        setIsImporting(true);
        try {
            const imported = await importRules();
            if (imported && imported.length > 0) {
                storeImportRules(imported);
                feedback.imported(imported.length, t("settings.rules"));
            }
        } catch (err) {
            feedback.error(t("settings.import"), (err as Error).message);
        } finally {
            setIsImporting(false);
        }
    }, [storeImportRules, t]);

    const handleExport = useCallback(async () => {
        setIsExporting(true);
        try {
            const success = await exportRules(rules);
            if (success) {
                feedback.exported(rules.length, t("settings.rules"));
            }
        } catch (err) {
            feedback.error(t("settings.export"), (err as Error).message);
        } finally {
            setIsExporting(false);
        }
    }, [rules, t]);

    return (
        <div className="space-y-8">
            {/* 系统预设规则 (默认折叠) */}
            <section className="rounded-lg border bg-card p-4">
                <SystemRuleList defaultCollapsed={true} />
            </section>

            {/* 自定义规则列表 */}
            <section>
                <RuleList
                    onImport={isImporting ? undefined : handleImport}
                    onExport={isExporting ? undefined : handleExport}
                />
            </section>

            {/* 规则测试 */}
            <section>
                <RuleTestPanel />
            </section>

            {/* 隐私保护记录入口 */}
            <section className="rounded-lg border bg-card p-4">
                <button
                    onClick={() => navigate('/privacy-records')}
                    className="w-full flex items-center justify-between hover:bg-accent/50 -m-4 p-4 rounded-lg transition-colors"
                    data-testid="privacy-records-link"
                >
                    <div className="flex items-center gap-3">
                        <div className="p-2 rounded-md bg-emerald-500/10">
                            <Shield className="h-5 w-5 text-emerald-500" />
                        </div>
                        <div className="text-left">
                            <p className="text-sm font-medium">
                                {t("privacy.records.title")}
                            </p>
                            <p className="text-xs text-muted-foreground">
                                {t("settings.privacyRecordsDesc")}
                            </p>
                        </div>
                    </div>
                    <ChevronRight className="h-5 w-5 text-muted-foreground" />
                </button>
            </section>
        </div>
    );
}

export default PrivacySettings;
