/**
 * InterceptionRecordList Test - 记录列表组件测试
 * Story 3-8: Task 4.6 - 单元测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { InterceptionRecordList, type InterceptionRecordListProps } from './InterceptionRecordList';
import type { PaginatedRecords, InterceptionRecord } from '@/components/sanitizer/types';

// Mock react-i18next
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string, params?: Record<string, unknown>) => {
            const translations: Record<string, string> = {
                'import.selectAll': 'Select All',
                'import.clearSelection': 'Clear',
                'privacy.records.delete.selected': `Delete Selected (${params?.count || 0})`,
                'privacy.records.pagination.perPage': 'Per page',
                'privacy.records.pagination.page': `Page ${params?.page || 1}`,
                'privacy.records.list.time': 'Time',
                'privacy.records.list.source': 'Source',
                'privacy.records.list.sensitiveType': 'Type',
                'privacy.records.list.userAction': 'Action',
                'privacy.records.list.noRecords': 'No records',
                'privacy.records.list.noRecordsHint': 'Records will appear here',
                'privacy.records.delete.confirm': 'Confirm Delete',
                'privacy.records.delete.confirmDesc': `Delete ${params?.count || 0} records?`,
                'common.cancel': 'Cancel',
                'common.delete': 'Delete',
                'common.processing': 'Processing',
                'common.select': 'Select',
                'privacy.records.list.expandDetails': 'Expand',
                'privacy.records.list.collapseDetails': 'Collapse',
                'privacy.records.list.detectedItems': `${params?.count || 0} items`,
                'privacy.records.list.line': `Line ${params?.line || 0}`,
                'privacy.records.source.preUpload': 'Pre-upload',
            };
            return translations[key] || key;
        },
        i18n: { language: 'en' },
    }),
}));

describe('InterceptionRecordList', () => {
    const mockRecords: InterceptionRecord[] = [
        {
            id: 'record-1',
            timestamp: '2026-01-17T10:00:00Z',
            source: { type: 'pre_upload' },
            matches: [
                {
                    rule_id: 'r1',
                    sensitive_type: 'api_key',
                    severity: 'critical',
                    line: 10,
                    column: 1,
                    matched_text: 'sk-123',
                    masked_text: 'sk-***',
                    context: '',
                },
            ],
            user_action: 'redacted',
            original_text_hash: 'hash1',
        },
        {
            id: 'record-2',
            timestamp: '2026-01-16T15:00:00Z',
            source: { type: 'pre_upload' },
            matches: [
                {
                    rule_id: 'r2',
                    sensitive_type: 'password',
                    severity: 'warning',
                    line: 20,
                    column: 5,
                    matched_text: 'pass123',
                    masked_text: '***',
                    context: '',
                },
            ],
            user_action: 'ignored',
            original_text_hash: 'hash2',
        },
    ];

    const mockData: PaginatedRecords = {
        records: mockRecords,
        total: 25,
        page: 1,
        per_page: 20,
    };

    const defaultProps: InterceptionRecordListProps = {
        data: mockData,
        loading: false,
        onPageChange: vi.fn(),
        onPerPageChange: vi.fn(),
        onDelete: vi.fn().mockResolvedValue(undefined),
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should render empty state when no records', () => {
        render(
            <InterceptionRecordList
                {...defaultProps}
                data={{ records: [], total: 0, page: 1, per_page: 20 }}
            />
        );

        expect(screen.getByTestId('record-list-empty')).toBeInTheDocument();
        expect(screen.getByText('No records')).toBeInTheDocument();
    });

    it('should render record list', () => {
        render(<InterceptionRecordList {...defaultProps} />);

        expect(screen.getByTestId('record-list')).toBeInTheDocument();
        expect(screen.getByTestId('record-item-record-1')).toBeInTheDocument();
        expect(screen.getByTestId('record-item-record-2')).toBeInTheDocument();
    });

    it('should render table headers', () => {
        render(<InterceptionRecordList {...defaultProps} />);

        expect(screen.getByText('Time')).toBeInTheDocument();
        expect(screen.getByText('Source')).toBeInTheDocument();
        expect(screen.getByText('Type')).toBeInTheDocument();
        expect(screen.getByText('Action')).toBeInTheDocument();
    });

    it('should render select all button', () => {
        render(<InterceptionRecordList {...defaultProps} />);

        expect(screen.getByTestId('select-all-button')).toBeInTheDocument();
        expect(screen.getByText('Select All')).toBeInTheDocument();
    });

    it('should render delete button disabled when nothing selected', () => {
        render(<InterceptionRecordList {...defaultProps} />);

        const deleteButton = screen.getByTestId('delete-button');
        expect(deleteButton).toBeDisabled();
    });

    it('should render pagination controls', () => {
        render(<InterceptionRecordList {...defaultProps} />);

        expect(screen.getByTestId('per-page-select')).toBeInTheDocument();
        expect(screen.getByTestId('prev-page-button')).toBeInTheDocument();
        expect(screen.getByTestId('next-page-button')).toBeInTheDocument();
        expect(screen.getByText(/Page 1/)).toBeInTheDocument();
    });

    it('should select all records when select all button is clicked', () => {
        render(<InterceptionRecordList {...defaultProps} />);

        const selectAllButton = screen.getByTestId('select-all-button');
        fireEvent.click(selectAllButton);

        // After selecting all, the delete button should be enabled
        const deleteButton = screen.getByTestId('delete-button');
        expect(deleteButton).not.toBeDisabled();
    });

    it('should call onPageChange when next page button is clicked', () => {
        const onPageChange = vi.fn();
        render(<InterceptionRecordList {...defaultProps} onPageChange={onPageChange} />);

        const nextButton = screen.getByTestId('next-page-button');
        fireEvent.click(nextButton);

        expect(onPageChange).toHaveBeenCalledWith(2);
    });

    it('should disable prev button on first page', () => {
        render(<InterceptionRecordList {...defaultProps} />);

        const prevButton = screen.getByTestId('prev-page-button');
        expect(prevButton).toBeDisabled();
    });

    it('should open delete confirmation dialog when delete button is clicked', () => {
        render(<InterceptionRecordList {...defaultProps} />);

        // Select all first
        const selectAllButton = screen.getByTestId('select-all-button');
        fireEvent.click(selectAllButton);

        // Click delete
        const deleteButton = screen.getByTestId('delete-button');
        fireEvent.click(deleteButton);

        // Dialog should appear
        expect(screen.getByText('Confirm Delete')).toBeInTheDocument();
    });
});
