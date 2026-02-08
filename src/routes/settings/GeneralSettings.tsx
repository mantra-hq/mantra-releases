/**
 * GeneralSettings - 通用设置页面
 * Story 2-35: Task 3.1
 * Story 14.7: 关于与更新区域
 *
 * 包含 LanguageSwitcher + 帮助与支持区域 + 关于 Mantra 区域
 */

import { useState, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { getVersion } from '@tauri-apps/api/app';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { ClipboardCopy, Loader2, Globe, BookOpen, CheckCircle, RefreshCw, RotateCcw } from 'lucide-react';
import { LanguageSwitcher } from '@/components/settings/LanguageSwitcher';
import { useUpdateChecker } from '@/hooks';
import { feedback } from '@/lib/feedback';
import { useLogStore } from '@/stores';

export function GeneralSettings() {
    const { t } = useTranslation();
    const [isCopyingLogs, setIsCopyingLogs] = useState(false);
    const [appVersion, setAppVersion] = useState<string>('');
    const [hasChecked, setHasChecked] = useState(false);
    const copyLogsToClipboard = useLogStore((state) => state.copyToClipboard);

    const {
        updateAvailable,
        updateInfo,
        downloadProgress,
        updateStatus,
        checkForUpdate,
        restartToUpdate,
    } = useUpdateChecker();

    useEffect(() => {
        getVersion().then(setAppVersion);
    }, []);

    const handleCheckForUpdate = useCallback(async () => {
        await checkForUpdate();
        setHasChecked(true);
    }, [checkForUpdate]);

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

    const isChecking = updateStatus === 'checking';

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

            {/* 关于 Mantra */}
            <section className="rounded-lg border bg-card p-4" data-testid="about-mantra-section">
                <h2 className="text-lg font-semibold mb-3">
                    {t("updater.aboutMantra")}
                </h2>
                <div className="space-y-3">
                    {/* 版本号 + 检查更新按钮 */}
                    <div className="flex items-center justify-between">
                        <div>
                            <p className="text-sm font-medium">
                                {t("updater.currentVersion")}
                            </p>
                            <p className="text-xs text-muted-foreground" data-testid="app-version">
                                v{appVersion}
                            </p>
                        </div>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handleCheckForUpdate}
                            disabled={isChecking}
                            className="gap-2"
                            data-testid="check-update-button"
                        >
                            {isChecking ? (
                                <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                                <RefreshCw className="h-4 w-4" />
                            )}
                            {isChecking ? t("updater.checking") : t("updater.checkForUpdates")}
                        </Button>
                    </div>

                    {/* 状态区域 — 条件渲染 */}
                    {updateStatus === 'idle' && !updateAvailable && hasChecked && (
                        <div className="flex items-center gap-2 text-emerald-500" data-testid="up-to-date-status">
                            <CheckCircle className="h-4 w-4" />
                            <span className="text-sm">{t("updater.upToDate")}</span>
                        </div>
                    )}

                    {updateAvailable && updateStatus !== 'downloading' && updateStatus !== 'ready' && (
                        <div className="text-sm text-blue-500" data-testid="update-available-status">
                            {t("updater.updateAvailable", { version: updateInfo?.version })}
                        </div>
                    )}

                    {updateStatus === 'downloading' && (
                        <div className="space-y-2" data-testid="downloading-status">
                            <Progress value={downloadProgress} className="h-2" />
                            <p className="text-xs text-muted-foreground">
                                {t("updater.downloadProgress", { progress: Math.round(downloadProgress) })}
                            </p>
                        </div>
                    )}

                    {updateStatus === 'ready' && (
                        <div className="flex items-center justify-between" data-testid="ready-status">
                            <span className="text-sm text-blue-500">
                                {t("updater.readyToInstall", { version: updateInfo?.version })}
                            </span>
                            <Button
                                variant="default"
                                size="sm"
                                onClick={restartToUpdate}
                                className="gap-2"
                                data-testid="restart-to-update-button"
                            >
                                <RotateCcw className="h-4 w-4" />
                                {t("updater.restartToUpdate")}
                            </Button>
                        </div>
                    )}

                    {updateStatus === 'error' && (
                        <div className="text-sm text-destructive" data-testid="error-status">
                            {t("updater.checkFailed")}
                        </div>
                    )}
                </div>
            </section>
        </div>
    );
}

export default GeneralSettings;
