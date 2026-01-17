/**
 * PrivacyScanReport Component - 上传前隐私扫描报告弹窗
 * Story 3-9: Task 1 - AC #2
 *
 * 显示隐私扫描结果，提供脱敏/忽略/取消操作
 */

import { useTranslation } from 'react-i18next';
import { Shield, AlertTriangle, Info, Loader2 } from 'lucide-react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import type { ScanResult, ScanMatch, Severity } from '@/components/sanitizer/types';
import { SENSITIVE_TYPE_LABELS, SEVERITY_LABELS } from '@/components/sanitizer/types';

export interface PrivacyScanReportProps {
    /** 是否打开弹窗 */
    isOpen: boolean;
    /** 扫描结果 */
    scanResult: ScanResult | null;
    /** 是否正在扫描 */
    isScanning: boolean;
    /** 一键脱敏回调 */
    onRedact: () => void;
    /** 忽略并继续回调 */
    onIgnore: () => void;
    /** 取消回调 */
    onCancel: () => void;
}

/** 严重程度图标 */
const SeverityIcon = ({ severity }: { severity: Severity }) => {
    switch (severity) {
        case 'critical':
            return <Shield className="h-4 w-4" />;
        case 'warning':
            return <AlertTriangle className="h-4 w-4" />;
        case 'info':
            return <Info className="h-4 w-4" />;
    }
};

/** 严重程度颜色配置 */
const SEVERITY_STYLES: Record<Severity, { text: string; bg: string }> = {
    critical: { text: 'text-red-500', bg: 'bg-red-500/10' },
    warning: { text: 'text-yellow-500', bg: 'bg-yellow-500/10' },
    info: { text: 'text-blue-500', bg: 'bg-blue-500/10' },
};

/** 扫描结果摘要 */
function ScanSummary({ stats }: { stats: ScanResult['stats'] }) {
    const { t } = useTranslation();

    return (
        <div
            className="flex items-center gap-4 rounded-lg border bg-muted/30 p-3"
            data-testid="scan-summary"
        >
            <div className="flex items-center gap-2">
                <div className="p-1.5 rounded-md bg-red-500/10">
                    <Shield className="h-4 w-4 text-red-500" />
                </div>
                <span className="text-sm text-muted-foreground">
                    {t('privacy.scan.severity.critical')}:
                </span>
                <span
                    className="font-semibold tabular-nums text-red-500"
                    data-testid="severity-critical-count"
                >
                    {stats.critical_count}
                </span>
            </div>

            <div className="flex items-center gap-2">
                <div className="p-1.5 rounded-md bg-yellow-500/10">
                    <AlertTriangle className="h-4 w-4 text-yellow-500" />
                </div>
                <span className="text-sm text-muted-foreground">
                    {t('privacy.scan.severity.warning')}:
                </span>
                <span
                    className="font-semibold tabular-nums text-yellow-500"
                    data-testid="severity-warning-count"
                >
                    {stats.warning_count}
                </span>
            </div>

            <div className="flex items-center gap-2">
                <div className="p-1.5 rounded-md bg-blue-500/10">
                    <Info className="h-4 w-4 text-blue-500" />
                </div>
                <span className="text-sm text-muted-foreground">
                    {t('privacy.scan.severity.info')}:
                </span>
                <span
                    className="font-semibold tabular-nums text-blue-500"
                    data-testid="severity-info-count"
                >
                    {stats.info_count}
                </span>
            </div>
        </div>
    );
}

/** 单个匹配项 */
function MatchItem({ match, index }: { match: ScanMatch; index: number }) {
    const { t } = useTranslation();
    const styles = SEVERITY_STYLES[match.severity];

    return (
        <div
            className="rounded-lg border bg-background p-3 space-y-2"
            data-testid={`scan-match-item-${index}`}
        >
            {/* 头部：类型、严重程度、行号 */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <span
                        className="font-medium text-sm"
                        data-testid={`match-type-${index}`}
                    >
                        {SENSITIVE_TYPE_LABELS[match.sensitive_type] || match.sensitive_type}
                    </span>
                    <span
                        className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs ${styles.text} ${styles.bg}`}
                        data-testid={`match-severity-${index}`}
                    >
                        <SeverityIcon severity={match.severity} />
                        {SEVERITY_LABELS[match.severity]}
                    </span>
                </div>
                <span
                    className="text-xs text-muted-foreground tabular-nums"
                    data-testid={`match-line-${index}`}
                >
                    {t('privacy.scan.line')} {match.line}
                </span>
            </div>

            {/* 脱敏预览 */}
            <div className="space-y-1">
                <div className="text-xs text-muted-foreground">{t('privacy.scan.masked')}:</div>
                <div
                    className="font-mono text-xs bg-muted px-2 py-1 rounded overflow-x-auto"
                    data-testid={`match-masked-${index}`}
                >
                    {match.masked_text}
                </div>
            </div>

            {/* 上下文 */}
            {match.context && (
                <div className="space-y-1">
                    <div className="text-xs text-muted-foreground">{t('privacy.scan.context')}:</div>
                    <div
                        className="font-mono text-xs text-muted-foreground bg-muted/50 px-2 py-1 rounded overflow-x-auto truncate"
                        title={match.context}
                        data-testid={`match-context-${index}`}
                    >
                        {match.context}
                    </div>
                </div>
            )}
        </div>
    );
}

export function PrivacyScanReport({
    isOpen,
    scanResult,
    isScanning,
    onRedact,
    onIgnore,
    onCancel,
}: PrivacyScanReportProps) {
    const { t } = useTranslation();

    if (!isOpen) {
        return null;
    }

    const hasMatches = scanResult && scanResult.matches.length > 0;
    const hasCritical = scanResult?.has_critical ?? false;
    const criticalCount = scanResult?.stats.critical_count ?? 0;
    const warningCount = scanResult?.stats.warning_count ?? 0;

    return (
        <Dialog open={isOpen} onOpenChange={(open) => !open && onCancel()}>
            <DialogContent
                className="sm:max-w-2xl max-h-[85vh] overflow-hidden flex flex-col"
                data-testid="privacy-scan-report-dialog"
                // 防止点击外部意外关闭弹窗（ESC 键仍可取消上传）
                onInteractOutside={(e) => e.preventDefault()}
                onEscapeKeyDown={(e) => {
                    // ESC 键触发取消操作
                    // 这是有意设计：对于隐私敏感操作，默认取消比意外继续更安全
                    onCancel();
                }}
            >
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Shield className="h-5 w-5 text-primary" />
                        {t('privacy.scan.title')}
                    </DialogTitle>
                </DialogHeader>

                {/* 加载状态 */}
                {isScanning && (
                    <div
                        className="flex flex-col items-center justify-center py-12 gap-4"
                        data-testid="scan-loading"
                    >
                        <Loader2 className="h-8 w-8 animate-spin text-primary" />
                        <span className="text-sm text-muted-foreground">
                            {t('privacy.scan.scanning')}
                        </span>
                    </div>
                )}

                {/* 扫描结果 */}
                {!isScanning && scanResult && (
                    <div className="flex-1 overflow-hidden flex flex-col gap-4">
                        {/* 摘要 */}
                        <div className="space-y-3">
                            <h3 className="text-sm font-medium">{t('privacy.scan.summary')}</h3>
                            <ScanSummary stats={scanResult.stats} />
                        </div>

                        {/* 警告信息 */}
                        {hasCritical && (
                            <div
                                className="flex items-center gap-2 rounded-lg bg-red-500/10 border border-red-500/20 p-3 text-sm text-red-500"
                                data-testid="critical-warning-message"
                            >
                                <AlertTriangle className="h-4 w-4 shrink-0" />
                                {t('privacy.scan.criticalWarning', { count: criticalCount })}
                            </div>
                        )}

                        {!hasCritical && warningCount > 0 && (
                            <div
                                className="flex items-center gap-2 rounded-lg bg-yellow-500/10 border border-yellow-500/20 p-3 text-sm text-yellow-500"
                                data-testid="warning-message"
                            >
                                <AlertTriangle className="h-4 w-4 shrink-0" />
                                {t('privacy.scan.warningOnly', { count: warningCount })}
                            </div>
                        )}

                        {/* 无敏感信息 */}
                        {!hasMatches && (
                            <div className="flex items-center gap-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20 p-3 text-sm text-emerald-500">
                                <Shield className="h-4 w-4 shrink-0" />
                                {t('privacy.scan.noIssues')}
                            </div>
                        )}

                        {/* 匹配列表 */}
                        {hasMatches && (
                            <div className="flex-1 overflow-hidden flex flex-col gap-2">
                                <h3 className="text-sm font-medium">
                                    {t('privacy.scan.detected')} ({t('privacy.scan.items', { count: scanResult.matches.length })})
                                </h3>
                                <ScrollArea className="flex-1 max-h-[300px]">
                                    <div
                                        className="space-y-2 pr-4"
                                        data-testid="scan-match-list"
                                    >
                                        {scanResult.matches.map((match, index) => (
                                            <MatchItem
                                                key={`${match.rule_id}-${match.line}-${match.column}`}
                                                match={match}
                                                index={index}
                                            />
                                        ))}
                                    </div>
                                </ScrollArea>
                            </div>
                        )}
                    </div>
                )}

                <DialogFooter className="flex flex-row gap-2 sm:justify-between border-t pt-4">
                    <Button
                        variant="ghost"
                        onClick={onCancel}
                        data-testid="btn-cancel"
                        aria-label={t('privacy.scan.actions.cancel')}
                    >
                        {t('privacy.scan.actions.cancel')}
                    </Button>

                    <div className="flex gap-2">
                        <Button
                            variant="outline"
                            onClick={onIgnore}
                            disabled={isScanning}
                            className="bg-yellow-500/10 hover:bg-yellow-500/20 border-yellow-500/30 text-yellow-600 dark:text-yellow-400"
                            data-testid="btn-ignore"
                            aria-label={t('privacy.scan.actions.ignore')}
                        >
                            {t('privacy.scan.actions.ignore')}
                        </Button>
                        <Button
                            onClick={onRedact}
                            disabled={isScanning || !hasMatches}
                            className="bg-emerald-500 hover:bg-emerald-600 text-white"
                            data-testid="btn-redact"
                            aria-label={t('privacy.scan.actions.redact')}
                        >
                            {t('privacy.scan.actions.redact')}
                        </Button>
                    </div>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

export default PrivacyScanReport;
