/**
 * Settings Page - 设置页面
 * Story 3-3: Task 5 - AC #1
 * Story 2-26: i18n 国际化
 * Story 3-8: Task 6 - 添加隐私记录入口
 * Story 3.10: 合并内置规则展示（默认折叠）
 * Story 3.11: Task 4.5 - 本地 API 端口配置
 * Story 11.4: Task 6 - 环境变量管理入口
 */

import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button } from '@/components/ui/button';
import { ArrowLeft, Settings as SettingsIcon, ClipboardCopy, Loader2, Globe, BookOpen, Shield, ChevronRight } from 'lucide-react';
import { RuleList } from '@/components/settings/RuleList';
import { RuleTestPanel } from '@/components/settings/RuleTestPanel';
import { LanguageSwitcher } from '@/components/settings/LanguageSwitcher';
import { SystemRuleList } from '@/components/settings/SystemRuleList';
import { LocalServerConfig } from '@/components/settings/LocalServerConfig';
import { EnvVariableManager } from '@/components/hub/EnvVariableManager';
import { useSanitizationRulesStore } from '@/stores/useSanitizationRulesStore';
import { exportRules, importRules } from '@/lib/rule-io';
import { feedback } from '@/lib/feedback';
import { useLogStore } from '@/stores';

export function Settings() {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const { rules, importRules: storeImportRules } = useSanitizationRulesStore();
    const [isImporting, setIsImporting] = useState(false);
    const [isExporting, setIsExporting] = useState(false);
    const [isCopyingLogs, setIsCopyingLogs] = useState(false);
    const copyLogsToClipboard = useLogStore((state) => state.copyToClipboard);

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

    // Story 2.28: 复制运行日志
    const handleCopyLogs = useCallback(async () => {
        setIsCopyingLogs(true);
        try {
            const success = await copyLogsToClipboard();
            if (success) {
                feedback.copied(t("settings.logsCopied"));
            } else {
                feedback.error(t("common.copy"), t("feedback.copyFailed"));
            }
        } catch (err) {
            feedback.error(t("common.copy"), (err as Error).message);
        } finally {
            setIsCopyingLogs(false);
        }
    }, [copyLogsToClipboard, t]);


    return (
        <div className="min-h-screen bg-background">
            {/* Header */}
            <header className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div className="container flex h-14 items-center px-4">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => navigate(-1)}
                        aria-label={t("common.back")}
                        data-testid="back-button"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                    <div className="flex items-center gap-2 ml-2">
                        <SettingsIcon className="h-5 w-5" />
                        <h1 className="text-lg font-semibold">{t("settings.title")}</h1>
                    </div>
                </div>
            </header>

            {/* Content */}
            <main className="container px-4 py-6 max-w-4xl mx-auto">
                <div className="space-y-8">
                    {/* Story 2-26: 语言设置 (AC #1) */}
                    <section className="rounded-lg border bg-card p-4">
                        <LanguageSwitcher />
                    </section>

                    {/* Story 3.11: 本地 API Server 端口配置 (AC #7) */}
                    <section className="rounded-lg border bg-card p-4">
                        <LocalServerConfig />
                    </section>

                    {/* Story 11.4: 环境变量管理 (Task 6) */}
                    <section className="rounded-lg border bg-card p-4">
                        <EnvVariableManager />
                    </section>

                    {/* Story 2.28: 帮助 - 复制运行日志 (AC #1, AC #3) */}
                    <section className="rounded-lg border bg-card p-4">
                        <h2 className="text-lg font-semibold mb-3">
                            {t("settings.helpSection")}
                        </h2>
                        <div className="space-y-3">
                            {/* 官方网站链接 */}
                            <div className="flex items-center justify-between">
                                <div>
                                    <p className="text-sm font-medium">
                                        {t("settings.officialWebsite")}
                                    </p>
                                    <p className="text-xs text-muted-foreground">
                                        {t("settings.officialWebsiteDesc")}
                                    </p>
                                </div>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => openUrl("https://mantra.gonewx.com")}
                                    className="gap-2"
                                    data-testid="official-website-button"
                                >
                                    <Globe className="h-4 w-4" />
                                    {t("common.open")}
                                </Button>
                            </div>
                            {/* 帮助文档链接 */}
                            <div className="flex items-center justify-between">
                                <div>
                                    <p className="text-sm font-medium">
                                        {t("settings.documentation")}
                                    </p>
                                    <p className="text-xs text-muted-foreground">
                                        {t("settings.documentationDesc")}
                                    </p>
                                </div>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => openUrl("https://docs.mantra.gonewx.com")}
                                    className="gap-2"
                                    data-testid="documentation-button"
                                >
                                    <BookOpen className="h-4 w-4" />
                                    {t("common.open")}
                                </Button>
                            </div>
                            {/* 复制运行日志 */}
                            <div className="flex items-center justify-between">
                                <div>
                                    <p className="text-sm font-medium">
                                        {t("settings.copyLogs")}
                                    </p>
                                    <p className="text-xs text-muted-foreground">
                                        {t("settings.logsDescription")}
                                    </p>
                                </div>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={handleCopyLogs}
                                    disabled={isCopyingLogs}
                                    className="gap-2"
                                    data-testid="copy-logs-button"
                                >
                                    {isCopyingLogs ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <ClipboardCopy className="h-4 w-4" />
                                    )}
                                    {t("common.copy")}
                                </Button>
                            </div>
                        </div>
                    </section>

                    {/* Story 3-5: 系统预设规则 (默认折叠) */}
                    <section className="rounded-lg border bg-card p-4">
                        <SystemRuleList defaultCollapsed={true} />
                    </section>

                    {/* Story 3-3: 自定义规则列表 */}
                    <section>
                        <RuleList
                            onImport={isImporting ? undefined : handleImport}
                            onExport={isExporting ? undefined : handleExport}
                        />
                    </section>

                    {/* Story 3-3: 规则测试 */}
                    <section>
                        <RuleTestPanel />
                    </section>

                    {/* Story 3-8: 隐私保护记录入口 (Task 6) */}
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
            </main>
        </div>
    );
}

export default Settings;
