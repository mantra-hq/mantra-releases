/**
 * SanitizeStatusBanner - 脱敏状态横幅
 * Story 3.4: 主视图原生模式
 *
 * 显示在 CodePanel 顶部的状态横幅，提供脱敏预览的操作入口
 */

import { useTranslation } from 'react-i18next';
import { ShieldCheck, ShieldAlert, Info, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';
import type { SanitizeStatusBannerProps, SensitiveType } from './types';
import { SENSITIVE_TYPE_LABELS } from './types';

/**
 * 脱敏状态横幅组件
 *
 * 功能:
 * - 有敏感信息: 显示警告状态 + 分类标签 + 操作按钮
 * - 无敏感信息: 显示安全状态 + 操作按钮
 * - 标签可点击跳转到对应行
 */
export function SanitizeStatusBanner({
    stats,
    sensitiveMatches,
    isLoading = false,
    error,
    onCancel,
    onConfirm: _onConfirm, // TODO: 启用分享功能后移除下划线
    onJumpToLine,
}: SanitizeStatusBannerProps) {
    const { t } = useTranslation();

    const hasSensitiveInfo = stats.total > 0;

    // 获取分类标签及其第一个匹配行号
    const categoryTags = Object.entries(stats.counts)
        .filter(([, count]) => count !== undefined && count > 0)
        .map(([type, count]) => {
            const firstMatch = sensitiveMatches.find(m => m.type === type);
            return {
                type: type as SensitiveType,
                label: SENSITIVE_TYPE_LABELS[type as SensitiveType],
                count: count!,
                lineNumber: firstMatch?.lineNumber,
            };
        });

    // 处理标签点击
    const handleTagClick = (lineNumber: number | undefined) => {
        if (lineNumber && onJumpToLine) {
            onJumpToLine(lineNumber);
        }
    };

    // 错误状态
    if (error) {
        return (
            <div className="bg-destructive/10 border-b border-destructive/20 px-4 py-3">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 text-destructive">
                        <ShieldAlert className="h-5 w-5" />
                        <span className="font-medium">{error}</span>
                    </div>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={onCancel}
                    >
                        {t('common.close', '关闭')}
                    </Button>
                </div>
            </div>
        );
    }

    // 加载状态
    if (isLoading) {
        return (
            <div className="bg-muted/50 border-b px-4 py-3">
                <div className="flex items-center gap-2 text-muted-foreground">
                    <Loader2 className="h-5 w-5 animate-spin" />
                    <span>{t('sanitizer.scanning', '正在扫描敏感信息...')}</span>
                </div>
            </div>
        );
    }

    return (
        <div
            className={cn(
                'border-b px-4 py-3',
                hasSensitiveInfo ? 'bg-amber-500/10 border-amber-500/20' : 'bg-green-500/10 border-green-500/20'
            )}
            data-testid="sanitize-status-banner"
        >
            {/* 主内容行 */}
            <div className="flex items-center justify-between gap-4">
                {/* 左侧: 图标 + 结论 */}
                <div className="flex items-center gap-3 min-w-0">
                    {hasSensitiveInfo ? (
                        <ShieldAlert className="h-5 w-5 text-amber-500 shrink-0" />
                    ) : (
                        <ShieldCheck className="h-5 w-5 text-green-500 shrink-0" />
                    )}
                    <span className="font-medium truncate">
                        {hasSensitiveInfo
                            ? t('sanitizer.detectedCount', '检测到 {{count}} 处敏感信息', { count: stats.total })
                            : t('sanitizer.safeToShareFull')
                        }
                    </span>
                </div>

                {/* 右侧: 操作按钮 */}
                <div className="flex items-center gap-2 shrink-0">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={onCancel}
                        data-testid="cancel-button"
                    >
                        {t('common.cancel', '取消')}
                    </Button>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <span>
                                <Button
                                    size="sm"
                                    disabled
                                    data-testid="confirm-button"
                                >
                                    {t('sanitizer.confirmShare', '确认分享')}
                                </Button>
                            </span>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>{t('common.comingSoon', '即将上线')}</p>
                        </TooltipContent>
                    </Tooltip>
                </div>
            </div>

            {/* 分类标签行 + 提示 (仅在有敏感信息时显示) */}
            {hasSensitiveInfo && categoryTags.length > 0 && (
                <div className="flex items-center gap-2 mt-2 flex-wrap">
                    {categoryTags.map(({ type, label, count, lineNumber }) => (
                        <button
                            key={type}
                            onClick={() => handleTagClick(lineNumber)}
                            disabled={!lineNumber || !onJumpToLine}
                            className={cn(
                                'inline-flex items-center px-2.5 py-1 rounded-md text-xs transition-colors',
                                'bg-background/80 border border-border',
                                lineNumber && onJumpToLine
                                    ? 'hover:bg-background cursor-pointer hover:border-amber-500/50'
                                    : 'cursor-default'
                            )}
                            title={lineNumber ? t('sanitizer.jumpToLine', '跳转到第 {{line}} 行', { line: lineNumber }) : undefined}
                        >
                            <span className="font-medium">{label}</span>
                            <span className="ml-1.5 text-muted-foreground">{count}</span>
                        </button>
                    ))}
                    {/* 提示文字放在标签后面 */}
                    <span className="inline-flex items-center gap-1 text-xs text-muted-foreground ml-1">
                        <Info className="h-3 w-3 shrink-0" />
                        {t('sanitizer.autoSanitizeHint', '敏感信息将自动脱敏 · 分享后可随时撤回')}
                    </span>
                </div>
            )}
        </div>
    );
}
