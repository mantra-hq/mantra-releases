/**
 * SanitizePreviewModal 组件 - Story 3-2 Task 7
 * Modal 容器，集成 DiffPreview
 */

import { useEffect, useCallback } from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
} from '@/components/ui/dialog';
import { DiffPreview } from './DiffPreview';
import type { SanitizePreviewModalProps } from './types';

export function SanitizePreviewModal({
    isOpen,
    onClose,
    originalText,
    sanitizedText,
    stats,
    onConfirm,
    isLoading = false,
}: SanitizePreviewModalProps) {
    // Esc 键关闭 Modal (Dialog 组件已内置支持)
    // 这里添加额外的确认逻辑
    const handleConfirm = useCallback(() => {
        onConfirm();
        // 不在这里关闭 Modal，让调用方控制
    }, [onConfirm]);

    const handleCancel = useCallback(() => {
        if (!isLoading) {
            onClose();
        }
    }, [onClose, isLoading]);

    // 禁止背景滚动
    useEffect(() => {
        if (isOpen) {
            document.body.style.overflow = 'hidden';
        } else {
            document.body.style.overflow = '';
        }
        return () => {
            document.body.style.overflow = '';
        };
    }, [isOpen]);

    return (
        <Dialog open={isOpen} onOpenChange={(open) => !open && handleCancel()}>
            <DialogContent
                className="max-w-4xl h-[80vh] flex flex-col p-0 gap-0"
                showCloseButton={!isLoading}
                data-testid="sanitize-preview-modal"
                onPointerDownOutside={(e) => {
                    // 加载中禁止点击外部关闭
                    if (isLoading) {
                        e.preventDefault();
                    }
                }}
                onEscapeKeyDown={(e) => {
                    // 加载中禁止 Esc 关闭
                    if (isLoading) {
                        e.preventDefault();
                    }
                }}
            >
                {/* Modal 标题 */}
                <DialogHeader className="px-6 py-4 border-b shrink-0">
                    <DialogTitle>脱敏预览</DialogTitle>
                    <DialogDescription>
                        请仔细检查以下变更，确认无误后再分享
                    </DialogDescription>
                </DialogHeader>

                {/* Diff 预览内容 */}
                <div className="flex-1 min-h-0 overflow-hidden">
                    <DiffPreview
                        originalText={originalText}
                        sanitizedText={sanitizedText}
                        stats={stats}
                        onConfirm={handleConfirm}
                        onCancel={handleCancel}
                        isLoading={isLoading}
                    />
                </div>
            </DialogContent>
        </Dialog>
    );
}
