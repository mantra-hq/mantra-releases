/**
 * InterceptionRecordItem Test - 单条记录组件测试
 * Story 3-8: Task 4.6 - 单元测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { InterceptionRecordItem, type InterceptionRecordItemProps } from './InterceptionRecordItem';
import type { InterceptionRecord } from '@/components/sanitizer/types';

// Mock react-i18next
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string, params?: Record<string, unknown>) => {
            const translations: Record<string, string> = {
                'common.select': 'Select',
                'privacy.records.list.collapseDetails': 'Collapse',
                'privacy.records.list.expandDetails': 'Expand',
                'privacy.records.list.project': 'Project',
                'privacy.records.list.detectedItems': `Detected ${params?.count || 0} items`,
                'privacy.records.list.line': `Line ${params?.line || 0}`,
                'privacy.records.source.preUpload': 'Pre-upload Check',
                'privacy.records.source.claudeCodeHook': 'Claude Code Hook',
            };
            return translations[key] || key;
        },
        i18n: {
            language: 'en',
        },
    }),
}));

describe('InterceptionRecordItem', () => {
    const mockRecord: InterceptionRecord = {
        id: 'test-id-1',
        timestamp: '2026-01-17T10:30:00Z',
        source: { type: 'pre_upload' },
        matches: [
            {
                rule_id: 'rule-1',
                sensitive_type: 'api_key',
                severity: 'critical',
                line: 45,
                column: 10,
                matched_text: 'sk-1234567890',
                masked_text: 'sk-****',
                context: 'const key = "sk-****";',
            },
            {
                rule_id: 'rule-2',
                sensitive_type: 'password',
                severity: 'warning',
                line: 100,
                column: 5,
                matched_text: 'password123',
                masked_text: '****',
                context: 'password: "****"',
            },
        ],
        user_action: 'redacted',
        original_text_hash: 'abc123',
        project_name: 'mantra',
    };

    const defaultProps: InterceptionRecordItemProps = {
        record: mockRecord,
        selected: false,
        onSelectionChange: vi.fn(),
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should render record with correct timestamp', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        expect(screen.getByText(/2026-01-17/)).toBeInTheDocument();
    });

    it('should render record source', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        expect(screen.getByText('Pre-upload Check')).toBeInTheDocument();
    });

    it('should render user action', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        expect(screen.getByText('已脱敏')).toBeInTheDocument();
    });

    it('should render sensitive types summary', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        expect(screen.getByText(/API Key/)).toBeInTheDocument();
    });

    it('should render checkbox', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        const checkbox = screen.getByTestId(`record-checkbox-${mockRecord.id}`);
        expect(checkbox).toBeInTheDocument();
    });

    it('should call onSelectionChange when checkbox is clicked', () => {
        const onSelectionChange = vi.fn();
        render(<InterceptionRecordItem {...defaultProps} onSelectionChange={onSelectionChange} />);

        const checkbox = screen.getByTestId(`record-checkbox-${mockRecord.id}`);
        fireEvent.click(checkbox);

        expect(onSelectionChange).toHaveBeenCalledWith(true);
    });

    it('should show checked checkbox when selected', () => {
        render(<InterceptionRecordItem {...defaultProps} selected={true} />);

        const checkbox = screen.getByTestId(`record-checkbox-${mockRecord.id}`);
        expect(checkbox).toHaveAttribute('data-state', 'checked');
    });

    it('should expand details when toggle button is clicked', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        // Details should not be visible initially
        expect(screen.queryByTestId(`record-details-${mockRecord.id}`)).not.toBeInTheDocument();

        // Click expand button
        const toggleButton = screen.getByTestId(`record-toggle-${mockRecord.id}`);
        fireEvent.click(toggleButton);

        // Details should now be visible
        expect(screen.getByTestId(`record-details-${mockRecord.id}`)).toBeInTheDocument();
    });

    it('should show project name in expanded details', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        // Expand
        const toggleButton = screen.getByTestId(`record-toggle-${mockRecord.id}`);
        fireEvent.click(toggleButton);

        expect(screen.getByText('mantra')).toBeInTheDocument();
    });

    it('should show match details in expanded view', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        // Expand
        const toggleButton = screen.getByTestId(`record-toggle-${mockRecord.id}`);
        fireEvent.click(toggleButton);

        // Should show masked text
        expect(screen.getByText('sk-****')).toBeInTheDocument();
        expect(screen.getByText('****')).toBeInTheDocument();
    });

    it('should collapse details when toggle button is clicked again', () => {
        render(<InterceptionRecordItem {...defaultProps} />);

        const toggleButton = screen.getByTestId(`record-toggle-${mockRecord.id}`);

        // Expand
        fireEvent.click(toggleButton);
        expect(screen.getByTestId(`record-details-${mockRecord.id}`)).toBeInTheDocument();

        // Collapse
        fireEvent.click(toggleButton);
        expect(screen.queryByTestId(`record-details-${mockRecord.id}`)).not.toBeInTheDocument();
    });
});
