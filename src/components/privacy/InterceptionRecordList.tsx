/**
 * InterceptionRecordList Component - 拦截记录列表
 * Story 3-8: Task 4.1 - AC #4
 *
 * 列表容器，支持分页、批量选择和删除
 */

import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Trash2, FileX, ChevronLeft, ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { InterceptionRecordItem } from './InterceptionRecordItem';
import type { InterceptionRecord, PaginatedRecords } from '@/components/sanitizer/types';

export interface InterceptionRecordListProps {
    /** 分页记录数据 */
    data: PaginatedRecords | null;
    /** 是否正在加载 */
    loading?: boolean;
    /** 页码变化回调 */
    onPageChange: (page: number) => void;
    /** 每页条数变化回调 */
    onPerPageChange: (perPage: number) => void;
    /** 删除记录回调 */
    onDelete: (ids: string[]) => Promise<void>;
}

const PER_PAGE_OPTIONS = [10, 20, 50, 100];

export function InterceptionRecordList({
    data,
    loading,
    onPageChange,
    onPerPageChange,
    onDelete,
}: InterceptionRecordListProps) {
    const { t } = useTranslation();
    const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
    const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
    const [isDeleting, setIsDeleting] = useState(false);

    const records = data?.records ?? [];
    const totalPages = data ? Math.ceil(data.total / data.per_page) : 0;
    const currentPage = data?.page ?? 1;
    const perPage = data?.per_page ?? 20;

    // 切换单条记录选中状态
    const toggleSelection = useCallback((id: string, selected: boolean) => {
        setSelectedIds((prev) => {
            const next = new Set(prev);
            if (selected) {
                next.add(id);
            } else {
                next.delete(id);
            }
            return next;
        });
    }, []);

    // 全选/取消全选
    const toggleSelectAll = useCallback(() => {
        if (selectedIds.size === records.length && records.length > 0) {
            setSelectedIds(new Set());
        } else {
            setSelectedIds(new Set(records.map((r) => r.id)));
        }
    }, [records, selectedIds.size]);

    // 处理删除
    const handleDelete = async () => {
        if (selectedIds.size === 0) return;

        setIsDeleting(true);
        try {
            await onDelete(Array.from(selectedIds));
            setSelectedIds(new Set());
        } finally {
            setIsDeleting(false);
            setDeleteDialogOpen(false);
        }
    };

    // 空状态
    if (!loading && records.length === 0) {
        return (
            <div
                className="flex flex-col items-center justify-center py-16 text-muted-foreground"
                data-testid="record-list-empty"
            >
                <FileX className="h-12 w-12 mb-4 opacity-50" />
                <p className="text-lg font-medium">{t('privacy.records.list.noRecords')}</p>
                <p className="text-sm">{t('privacy.records.list.noRecordsHint')}</p>
            </div>
        );
    }

    const allSelected = records.length > 0 && selectedIds.size === records.length;
    const someSelected = selectedIds.size > 0 && selectedIds.size < records.length;

    return (
        <div className="space-y-4" data-testid="record-list">
            {/* 工具栏 */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    {/* 全选按钮 */}
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={toggleSelectAll}
                        data-testid="select-all-button"
                    >
                        {allSelected ? t('import.clearSelection') : t('import.selectAll')}
                    </Button>

                    {/* 删除按钮 */}
                    <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => setDeleteDialogOpen(true)}
                        disabled={selectedIds.size === 0}
                        data-testid="delete-button"
                    >
                        <Trash2 className="h-4 w-4 mr-1" />
                        {t('privacy.records.delete.selected')} ({selectedIds.size})
                    </Button>
                </div>

                {/* 分页控件 */}
                {data && totalPages > 0 && (
                    <div className="flex items-center gap-4">
                        {/* 每页条数 */}
                        <div className="flex items-center gap-2">
                            <span className="text-sm text-muted-foreground">
                                {t('privacy.records.pagination.perPage')}:
                            </span>
                            <Select
                                value={String(perPage)}
                                onValueChange={(value) => onPerPageChange(Number(value))}
                            >
                                <SelectTrigger className="w-20" data-testid="per-page-select">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {PER_PAGE_OPTIONS.map((option) => (
                                        <SelectItem key={option} value={String(option)}>
                                            {option}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        {/* 页码导航 */}
                        <div className="flex items-center gap-2">
                            <Button
                                variant="outline"
                                size="icon"
                                onClick={() => onPageChange(currentPage - 1)}
                                disabled={currentPage <= 1 || loading}
                                data-testid="prev-page-button"
                            >
                                <ChevronLeft className="h-4 w-4" />
                            </Button>
                            <span className="text-sm text-muted-foreground min-w-[80px] text-center">
                                {t('privacy.records.pagination.page', { page: currentPage })} / {totalPages}
                            </span>
                            <Button
                                variant="outline"
                                size="icon"
                                onClick={() => onPageChange(currentPage + 1)}
                                disabled={currentPage >= totalPages || loading}
                                data-testid="next-page-button"
                            >
                                <ChevronRight className="h-4 w-4" />
                            </Button>
                        </div>
                    </div>
                )}
            </div>

            {/* 表头 */}
            <div className="flex items-center gap-3 px-4 py-2 text-sm font-medium text-muted-foreground border-b">
                <div className="w-6" /> {/* Checkbox 占位 */}
                <div className="w-36">{t('privacy.records.list.time')}</div>
                <div className="w-32">{t('privacy.records.list.source')}</div>
                <div className="flex-1">{t('privacy.records.list.sensitiveType')}</div>
                <div className="w-20">{t('privacy.records.list.userAction')}</div>
                <div className="w-8" /> {/* 展开按钮占位 */}
            </div>

            {/* 记录列表 */}
            <div className="space-y-2">
                {records.map((record) => (
                    <InterceptionRecordItem
                        key={record.id}
                        record={record}
                        selected={selectedIds.has(record.id)}
                        onSelectionChange={(selected) => toggleSelection(record.id, selected)}
                    />
                ))}
            </div>

            {/* 删除确认对话框 */}
            <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>{t('privacy.records.delete.confirm')}</AlertDialogTitle>
                        <AlertDialogDescription>
                            {t('privacy.records.delete.confirmDesc', { count: selectedIds.size })}
                        </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogCancel disabled={isDeleting}>
                            {t('common.cancel')}
                        </AlertDialogCancel>
                        <AlertDialogAction
                            onClick={handleDelete}
                            disabled={isDeleting}
                            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                        >
                            {isDeleting ? t('common.processing') : t('common.delete')}
                        </AlertDialogAction>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>
        </div>
    );
}

export default InterceptionRecordList;
