/**
 * DiffPreview 组件 - Story 3-2 Task 1, 2, 4, 6
 * Diff 预览主组件，包含滚动检测和操作按钮
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { ArrowDown, Loader2 } from 'lucide-react';
import { computeDiff, hasDifferences } from './diff-utils';
import { DiffLineComponent } from './DiffLine';
import { SanitizationSummary } from './SanitizationSummary';
import type { DiffPreviewProps } from './types';

export function DiffPreview({
    originalText,
    sanitizedText,
    stats,
    onConfirm,
    onCancel,
    isLoading = false,
    hideActions = false,
}: DiffPreviewProps) {
    const { t } = useTranslation();
    const [hasScrolledToBottom, setHasScrolledToBottom] = useState(false);
    const bottomRef = useRef<HTMLDivElement>(null);
    const scrollContainerRef = useRef<HTMLDivElement>(null);

    const diffLines = computeDiff(originalText, sanitizedText);
    const hasDiff = hasDifferences(originalText, sanitizedText);

    // IntersectionObserver 检测底部元素可见性
    useEffect(() => {
        // 如果没有差异，自动设置为已滚动到底部
        if (!hasDiff) {
            setHasScrolledToBottom(true);
            return;
        }

        const observer = new IntersectionObserver(
            ([entry]) => {
                if (entry.isIntersecting) {
                    setHasScrolledToBottom(true);
                }
            },
            {
                threshold: 1.0,
                root: scrollContainerRef.current,
            }
        );

        if (bottomRef.current) {
            observer.observe(bottomRef.current);
        }

        return () => observer.disconnect();
    }, [hasDiff]);

    // 滚动到底部的辅助函数
    const scrollToBottom = useCallback(() => {
        bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, []);

    return (
        <div className="flex flex-col h-full" data-testid="diff-preview">
            {/* 统计摘要 */}
            {!hideActions && <SanitizationSummary stats={stats} />}

            {/* Diff 视图 */}
            <div className="flex-1 min-h-0 overflow-hidden">
                <ScrollArea className="h-full" ref={scrollContainerRef}>
                    <div className="font-mono text-sm">
                        {/* 表头 */}
                        <div className="flex px-4 py-2 border-b bg-muted/30 text-xs text-muted-foreground sticky top-0">
                            <span className="w-12 text-right pr-4">{t('diff.original')}</span>
                            <span className="w-12 text-right pr-4">{t('diff.new')}</span>
                            <span className="w-4 text-center"></span>
                            <span className="flex-1">{t('diff.content')}</span>
                        </div>

                        {/* Diff 行 */}
                        {diffLines.map((line, idx) => (
                            <DiffLineComponent key={idx} line={line} />
                        ))}

                        {/* 底部检测元素 */}
                        <div ref={bottomRef} className="h-4" aria-hidden="true" />
                    </div>
                </ScrollArea>
            </div>

            {/* 操作区域 - 可通过 hideActions 隐藏 */}
            {!hideActions && (
                <div className="border-t p-4 space-y-3 bg-background">
                    {/* 滚动提示 */}
                    {!hasScrolledToBottom && hasDiff && (
                        <button
                            onClick={scrollToBottom}
                            className="w-full flex items-center justify-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors py-1"
                        >
                            <ArrowDown className="h-4 w-4 animate-bounce" />
                            {t('diff.scrollToBottom')}
                        </button>
                    )}

                    {/* 确认提示 */}
                    <p
                        className="text-sm text-center text-muted-foreground"
                        data-testid="confirm-message"
                    >
                        {t('sanitizer.willShareSanitized')}
                    </p>

                    {/* 按钮区域 */}
                    <div className="flex gap-2 justify-end">
                        <Button
                            variant="outline"
                            onClick={onCancel}
                            disabled={isLoading}
                            data-testid="cancel-button"
                        >
                            {t('common.cancel')}
                        </Button>
                        <Button
                            onClick={onConfirm}
                            disabled={!hasScrolledToBottom || isLoading}
                            data-testid="confirm-button"
                        >
                            {isLoading ? (
                                <>
                                    <Loader2 className="h-4 w-4 animate-spin mr-2" />
                                    {t('common.processing')}
                                </>
                            ) : (
                                t('sanitizer.confirmShare')
                            )}
                        </Button>
                    </div>
                </div>
            )}
        </div>
    );
}
