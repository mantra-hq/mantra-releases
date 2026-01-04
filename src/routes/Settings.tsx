/**
 * Settings Page - 设置页面
 * Story 3-3: Task 5 - AC #1
 */

import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { ArrowLeft, Settings as SettingsIcon } from 'lucide-react';
import { RuleList } from '@/components/settings/RuleList';
import { RuleTestPanel } from '@/components/settings/RuleTestPanel';
import { useSanitizationRulesStore } from '@/stores/useSanitizationRulesStore';
import { exportRules, importRules } from '@/lib/rule-io';
import { feedback } from '@/lib/feedback';

export function Settings() {
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
                feedback.imported(imported.length, "规则");
            }
        } catch (err) {
            feedback.error("导入", (err as Error).message);
        } finally {
            setIsImporting(false);
        }
    }, [storeImportRules]);

    const handleExport = useCallback(async () => {
        setIsExporting(true);
        try {
            const success = await exportRules(rules);
            if (success) {
                feedback.exported(rules.length, "规则");
            }
        } catch (err) {
            feedback.error("导出", (err as Error).message);
        } finally {
            setIsExporting(false);
        }
    }, [rules]);


    return (
        <div className="min-h-screen bg-background">
            {/* Header */}
            <header className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div className="container flex h-14 items-center px-4">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => navigate(-1)}
                        aria-label="返回"
                        data-testid="back-button"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                    <div className="flex items-center gap-2 ml-2">
                        <SettingsIcon className="h-5 w-5" />
                        <h1 className="text-lg font-semibold">设置</h1>
                    </div>
                </div>
            </header>

            {/* Content */}
            <main className="container px-4 py-6 max-w-4xl">
                <div className="space-y-8">
                    {/* 规则列表 */}
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
                </div>
            </main>
        </div>
    );
}

export default Settings;
