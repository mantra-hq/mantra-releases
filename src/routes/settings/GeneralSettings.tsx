/**
 * GeneralSettings - 通用设置页面
 * Story 2-35: Task 3.1
 *
 * 包含 LanguageSwitcher + 帮助与支持区域
 */

import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button } from '@/components/ui/button';
import { ClipboardCopy, Loader2, Globe, BookOpen } from 'lucide-react';
import { LanguageSwitcher } from '@/components/settings/LanguageSwitcher';
import { feedback } from '@/lib/feedback';
import { useLogStore } from '@/stores';

export function GeneralSettings() {
    const { t } = useTranslation();
    const [isCopyingLogs, setIsCopyingLogs] = useState(false);
    const copyLogsToClipboard = useLogStore((state) => state.copyToClipboard);

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
        <div className="space-y-8">
            {/* 语言设置 */}
            <section className="rounded-lg border bg-card p-4">
                <LanguageSwitcher />
            </section>

            {/* 帮助与支持 */}
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
        </div>
    );
}

export default GeneralSettings;
