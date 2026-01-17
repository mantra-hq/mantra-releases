/**
 * RecordFilters Test - 筛选栏组件测试
 * Story 3-8: Task 3.5 - 单元测试
 *
 * Note: 由于 jsdom 与 Radix Select 组件存在兼容性问题，
 * 这里只测试渲染和基本交互，不测试完整的 dropdown 交互。
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { RecordFilters, type RecordFiltersProps } from './RecordFilters';

// Mock react-i18next
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string) => {
            const translations: Record<string, string> = {
                'privacy.records.filter.source': 'Source',
                'privacy.records.filter.type': 'Type',
                'privacy.records.filter.time': 'Time',
                'privacy.records.filter.all': 'All',
                'privacy.records.filter.allTime': 'All Time',
                'privacy.records.filter.today': 'Today',
                'privacy.records.filter.thisWeek': 'This Week',
                'privacy.records.filter.thisMonth': 'This Month',
                'privacy.records.source.preUpload': 'Pre-upload Check',
                'privacy.records.source.claudeCodeHook': 'Claude Code Hook',
                'privacy.records.source.externalHook': 'External Tool',
            };
            return translations[key] || key;
        },
    }),
}));

describe('RecordFilters', () => {
    const defaultProps: RecordFiltersProps = {
        source: 'all',
        sensitiveType: 'all',
        timeRange: 'all',
        onSourceChange: vi.fn(),
        onSensitiveTypeChange: vi.fn(),
        onTimeRangeChange: vi.fn(),
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should render all filter labels', () => {
        render(<RecordFilters {...defaultProps} />);

        expect(screen.getByText('Source:')).toBeInTheDocument();
        expect(screen.getByText('Type:')).toBeInTheDocument();
        expect(screen.getByText('Time:')).toBeInTheDocument();
    });

    it('should render filter dropdowns with correct test ids', () => {
        render(<RecordFilters {...defaultProps} />);

        expect(screen.getByTestId('filter-source')).toBeInTheDocument();
        expect(screen.getByTestId('filter-type')).toBeInTheDocument();
        expect(screen.getByTestId('filter-time')).toBeInTheDocument();
    });

    it('should render main container with correct test id', () => {
        render(<RecordFilters {...defaultProps} />);

        expect(screen.getByTestId('record-filters')).toBeInTheDocument();
    });

    it('should render with different filter values', () => {
        render(
            <RecordFilters
                {...defaultProps}
                source="pre_upload"
                sensitiveType="api_key"
                timeRange="thisWeek"
            />
        );

        expect(screen.getByTestId('record-filters')).toBeInTheDocument();
        // Verify dropdowns are present
        expect(screen.getByTestId('filter-source')).toBeInTheDocument();
        expect(screen.getByTestId('filter-type')).toBeInTheDocument();
        expect(screen.getByTestId('filter-time')).toBeInTheDocument();
    });

    it('should render dropdowns as buttons (Radix Select)', () => {
        render(<RecordFilters {...defaultProps} />);

        // Radix Select triggers are rendered as buttons
        const sourceDropdown = screen.getByTestId('filter-source');
        const typeDropdown = screen.getByTestId('filter-type');
        const timeDropdown = screen.getByTestId('filter-time');

        expect(sourceDropdown.tagName).toBe('BUTTON');
        expect(typeDropdown.tagName).toBe('BUTTON');
        expect(timeDropdown.tagName).toBe('BUTTON');
    });

    it('should render with correct flex layout', () => {
        render(<RecordFilters {...defaultProps} />);

        const container = screen.getByTestId('record-filters');
        expect(container).toHaveClass('flex', 'items-center', 'gap-3');
    });
});
