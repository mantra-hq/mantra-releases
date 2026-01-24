/**
 * InterceptionStats Test - 拦截统计组件测试
 * Story 3-8: Task 2.5 - 单元测试
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { InterceptionStats } from './InterceptionStats';
import * as sanitizerIpc from '@/lib/ipc/sanitizer-ipc';

// Mock react-i18next
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string) => {
            const translations: Record<string, string> = {
                'privacy.records.stats.total': 'Total Interceptions',
                'privacy.records.stats.thisWeek': 'This Week',
                'privacy.records.stats.redacted': 'Redacted',
                'privacy.records.stats.ignored': 'Ignored',
                'common.loadFailed': 'Load Failed',
            };
            return translations[key] || key;
        },
    }),
}));

// Mock IPC
vi.mock('@/lib/ipc/sanitizer-ipc', () => ({
    getInterceptionStats: vi.fn(),
}));

describe('InterceptionStats', () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        consoleErrorSpy.mockClear();
    });

    it('should render loading state initially', async () => {
        vi.mocked(sanitizerIpc.getInterceptionStats).mockImplementation(
            () => new Promise(() => {}) // Never resolves
        );

        render(<InterceptionStats />);

        expect(screen.getByTestId('interception-stats-loading')).toBeInTheDocument();
    });

    it('should render stats cards after loading', async () => {
        const mockStats = {
            total_interceptions: 128,
            recent_7_days: 23,
            by_type: { api_key: 50, password: 30 },
            by_severity: { critical: 40, warning: 30, info: 10 },
            by_action: { redacted: 95, ignored: 33 },
        };

        vi.mocked(sanitizerIpc.getInterceptionStats).mockResolvedValue(mockStats);

        render(<InterceptionStats />);

        await waitFor(() => {
            expect(screen.getByTestId('interception-stats')).toBeInTheDocument();
        });

        // Check all 4 stat cards
        expect(screen.getByTestId('stat-card-total')).toBeInTheDocument();
        expect(screen.getByTestId('stat-card-thisWeek')).toBeInTheDocument();
        expect(screen.getByTestId('stat-card-redacted')).toBeInTheDocument();
        expect(screen.getByTestId('stat-card-ignored')).toBeInTheDocument();

        // Check values
        expect(screen.getByText('128')).toBeInTheDocument(); // total
        expect(screen.getByText('23')).toBeInTheDocument(); // this week
        expect(screen.getByText('95')).toBeInTheDocument(); // redacted
        expect(screen.getByText('33')).toBeInTheDocument(); // ignored
    });

    it('should render labels correctly', async () => {
        const mockStats = {
            total_interceptions: 10,
            recent_7_days: 5,
            by_type: {},
            by_severity: {},
            by_action: { redacted: 8, ignored: 2 },
        };

        vi.mocked(sanitizerIpc.getInterceptionStats).mockResolvedValue(mockStats);

        render(<InterceptionStats />);

        await waitFor(() => {
            expect(screen.getByTestId('interception-stats')).toBeInTheDocument();
        });

        expect(screen.getByText('Total Interceptions')).toBeInTheDocument();
        expect(screen.getByText('This Week')).toBeInTheDocument();
        expect(screen.getByText('Redacted')).toBeInTheDocument();
        expect(screen.getByText('Ignored')).toBeInTheDocument();
    });

    it('should show error state when loading fails', async () => {
        vi.mocked(sanitizerIpc.getInterceptionStats).mockRejectedValue(
            new Error('Network error')
        );

        render(<InterceptionStats />);

        await waitFor(() => {
            expect(screen.getByTestId('interception-stats-error')).toBeInTheDocument();
        });

        expect(screen.getByText(/Load Failed/)).toBeInTheDocument();
        expect(screen.getByText(/Network error/)).toBeInTheDocument();
    });

    it('should handle missing by_action gracefully', async () => {
        const mockStats = {
            total_interceptions: 50,
            recent_7_days: 10,
            by_type: {},
            by_severity: {},
            by_action: {}, // Empty
        };

        vi.mocked(sanitizerIpc.getInterceptionStats).mockResolvedValue(mockStats);

        render(<InterceptionStats />);

        await waitFor(() => {
            expect(screen.getByTestId('interception-stats')).toBeInTheDocument();
        });

        // Should show 0 for redacted and ignored
        const redactedCard = screen.getByTestId('stat-card-redacted');
        const ignoredCard = screen.getByTestId('stat-card-ignored');
        expect(redactedCard).toHaveTextContent('0');
        expect(ignoredCard).toHaveTextContent('0');
    });

    it('should reload data when refreshTrigger changes', async () => {
        const mockStats1 = {
            total_interceptions: 10,
            recent_7_days: 5,
            by_type: {},
            by_severity: {},
            by_action: { redacted: 8, ignored: 2 },
        };

        const mockStats2 = {
            total_interceptions: 20,
            recent_7_days: 10,
            by_type: {},
            by_severity: {},
            by_action: { redacted: 15, ignored: 5 },
        };

        vi.mocked(sanitizerIpc.getInterceptionStats)
            .mockResolvedValueOnce(mockStats1)
            .mockResolvedValueOnce(mockStats2);

        const { rerender } = render(<InterceptionStats refreshTrigger={0} />);

        await waitFor(() => {
            expect(screen.getByText('10')).toBeInTheDocument(); // total
        });

        // Change refreshTrigger
        rerender(<InterceptionStats refreshTrigger={1} />);

        await waitFor(() => {
            expect(screen.getByText('20')).toBeInTheDocument(); // new total
        });

        expect(sanitizerIpc.getInterceptionStats).toHaveBeenCalledTimes(2);
    });
});
