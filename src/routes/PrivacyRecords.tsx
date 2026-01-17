/**
 * Privacy Records Page - 隐私保护记录页面
 * Story 3-8: Task 5 - AC #1-5 (页面组装与集成)
 *
 * 展示历史隐私拦截记录，支持筛选、分页和批量删除
 */

import { useState, useCallback, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { ArrowLeft, Shield } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
    InterceptionStats,
    RecordFilters,
    InterceptionRecordList,
    type TimeRange,
} from '@/components/privacy';
import {
    getInterceptionRecords,
    deleteInterceptionRecords,
} from '@/lib/ipc/sanitizer-ipc';
import { feedback } from '@/lib/feedback';
import type {
    PaginatedRecords,
    SensitiveType,
    InterceptionSourceType,
} from '@/components/sanitizer/types';

export function PrivacyRecords() {
    const { t } = useTranslation();
    const navigate = useNavigate();

    // 筛选状态
    const [source, setSource] = useState<InterceptionSourceType | 'all'>('all');
    const [sensitiveType, setSensitiveType] = useState<SensitiveType | 'all'>('all');
    const [timeRange, setTimeRange] = useState<TimeRange>('all');

    // 分页状态
    const [page, setPage] = useState(1);
    const [perPage, setPerPage] = useState(20);

    // 数据状态
    const [data, setData] = useState<PaginatedRecords | null>(null);
    const [loading, setLoading] = useState(true);

    // 刷新触发器 (用于统计组件刷新)
    const [refreshTrigger, setRefreshTrigger] = useState(0);

    // 加载数据
    const loadData = useCallback(async () => {
        setLoading(true);
        try {
            // 目前 IPC 只支持 source 筛选，其他筛选在前端处理
            const sourceFilter = source === 'all' ? undefined : source;
            const result = await getInterceptionRecords(page, perPage, sourceFilter);

            // 前端筛选：敏感类型和时间范围
            let filteredRecords = result.records;

            // 按敏感类型筛选
            if (sensitiveType !== 'all') {
                filteredRecords = filteredRecords.filter((record) =>
                    record.matches.some((m) => m.sensitive_type === sensitiveType)
                );
            }

            // 按时间范围筛选
            if (timeRange !== 'all') {
                const now = new Date();
                let startDate: Date;

                switch (timeRange) {
                    case 'today':
                        startDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());
                        break;
                    case 'thisWeek':
                        startDate = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
                        break;
                    case 'thisMonth':
                        startDate = new Date(now.getFullYear(), now.getMonth(), 1);
                        break;
                    default:
                        startDate = new Date(0);
                }

                filteredRecords = filteredRecords.filter(
                    (record) => new Date(record.timestamp) >= startDate
                );
            }

            setData({
                ...result,
                records: filteredRecords,
                total: filteredRecords.length < result.per_page
                    ? (page - 1) * result.per_page + filteredRecords.length
                    : result.total,
            });
        } catch (err) {
            console.error('Failed to load interception records:', err);
            feedback.error(t('common.loadFailed'), (err as Error).message);
        } finally {
            setLoading(false);
        }
    }, [page, perPage, source, sensitiveType, timeRange, t]);

    // 初始加载和筛选变化时重新加载
    useEffect(() => {
        loadData();
    }, [loadData]);

    // 筛选变化时重置页码
    const handleSourceChange = useCallback((value: InterceptionSourceType | 'all') => {
        setSource(value);
        setPage(1);
    }, []);

    const handleSensitiveTypeChange = useCallback((value: SensitiveType | 'all') => {
        setSensitiveType(value);
        setPage(1);
    }, []);

    const handleTimeRangeChange = useCallback((value: TimeRange) => {
        setTimeRange(value);
        setPage(1);
    }, []);

    // 每页条数变化时重置页码
    const handlePerPageChange = useCallback((value: number) => {
        setPerPage(value);
        setPage(1);
    }, []);

    // 删除记录
    const handleDelete = useCallback(async (ids: string[]) => {
        try {
            const count = await deleteInterceptionRecords(ids);
            feedback.deleted(t('privacy.records.delete.success', { count }));

            // 刷新数据
            await loadData();
            // 触发统计组件刷新
            setRefreshTrigger((prev) => prev + 1);
        } catch (err) {
            console.error('Failed to delete records:', err);
            feedback.error(t('privacy.records.delete.failed'), (err as Error).message);
            throw err;
        }
    }, [loadData, t]);

    return (
        <div className="min-h-screen bg-background">
            {/* Header */}
            <header className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div className="container flex h-14 items-center px-4">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => navigate(-1)}
                        aria-label={t('common.back')}
                        data-testid="back-button"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                    <div className="flex items-center gap-2 ml-2">
                        <Shield className="h-5 w-5 text-emerald-500" />
                        <h1 className="text-lg font-semibold">{t('privacy.records.title')}</h1>
                    </div>
                </div>
            </header>

            {/* Content */}
            <main className="container px-4 py-6 max-w-6xl mx-auto">
                <div className="space-y-6">
                    {/* Task 2: 统计概览 (AC #2) */}
                    <InterceptionStats refreshTrigger={refreshTrigger} />

                    {/* Task 3: 筛选栏 (AC #3) */}
                    <RecordFilters
                        source={source}
                        sensitiveType={sensitiveType}
                        timeRange={timeRange}
                        onSourceChange={handleSourceChange}
                        onSensitiveTypeChange={handleSensitiveTypeChange}
                        onTimeRangeChange={handleTimeRangeChange}
                    />

                    {/* Task 4: 记录列表 (AC #4) */}
                    <InterceptionRecordList
                        data={data}
                        loading={loading}
                        onPageChange={setPage}
                        onPerPageChange={handlePerPageChange}
                        onDelete={handleDelete}
                    />
                </div>
            </main>
        </div>
    );
}

export default PrivacyRecords;
