/**
 * SystemRuleList - 组件测试
 * Story 3-5: Task 6.1 - 系统预设规则列表测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { SystemRuleList } from './SystemRuleList';
import * as sanitizerIpc from '@/lib/ipc/sanitizer-ipc';

// Mock react-i18next
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string, params?: Record<string, unknown>) => {
            if (params?.count !== undefined) {
                return `${params.count} rules`;
            }
            const translations: Record<string, string> = {
                'settings.builtinRules': 'Built-in Rules',
                'settings.builtinRulesCount': '{{count}} rules',
                'common.loading': 'Loading',
                'common.loadFailed': 'Load Failed',
            };
            return translations[key] || key;
        },
    }),
}));

// Mock IPC
vi.mock('@/lib/ipc/sanitizer-ipc', () => ({
    getBuiltinRules: vi.fn(),
    getPrivacyRules: vi.fn(),
    updatePrivacyRules: vi.fn(),
}));

describe('SystemRuleList', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should render loading state initially', async () => {
        vi.mocked(sanitizerIpc.getPrivacyRules).mockImplementation(
            () => new Promise(() => { }) // Never resolves
        );

        render(<SystemRuleList />);

        expect(screen.getByText('Loading')).toBeInTheDocument();
    });

    it('should render rules grouped by type', async () => {
        const mockRules = [
            { name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key' },
            { name: 'GitHub Token', pattern: 'ghp_[a-zA-Z0-9]+', sensitive_type: 'github_token' },
            { name: 'AWS Access Key', pattern: 'AKIA[0-9A-Z]+', sensitive_type: 'aws_key' },
        ];

        vi.mocked(sanitizerIpc.getPrivacyRules).mockResolvedValue(mockRules as never);

        render(<SystemRuleList />);

        await waitFor(() => {
            expect(screen.getByTestId('system-rule-list')).toBeInTheDocument();
        });

        // Check header
        expect(screen.getByText('Built-in Rules')).toBeInTheDocument();
        expect(screen.getByText('3 rules')).toBeInTheDocument();

        // Check groups (first group is expanded by default)
        expect(screen.getByTestId('group-api_key')).toBeInTheDocument();
        expect(screen.getByTestId('group-github_token')).toBeInTheDocument();
        expect(screen.getByTestId('group-aws_key')).toBeInTheDocument();
    });

    it('should expand/collapse groups on click', async () => {
        const mockRules = [
            { name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key' },
            { name: 'GitHub Token', pattern: 'ghp_[a-zA-Z0-9]+', sensitive_type: 'github_token' },
        ];

        vi.mocked(sanitizerIpc.getPrivacyRules).mockResolvedValue(mockRules as never);

        render(<SystemRuleList />);

        await waitFor(() => {
            expect(screen.getByTestId('system-rule-list')).toBeInTheDocument();
        });

        // First group is expanded by default
        expect(screen.getByTestId('rule-api_key-0')).toBeInTheDocument();

        // Click to expand github_token group
        fireEvent.click(screen.getByTestId('group-github_token'));

        await waitFor(() => {
            expect(screen.getByTestId('rule-github_token-0')).toBeInTheDocument();
        });
    });

    it('should display rule pattern with monospace font', async () => {
        const mockRules = [
            { name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key' },
        ];

        vi.mocked(sanitizerIpc.getPrivacyRules).mockResolvedValue(mockRules as never);

        render(<SystemRuleList />);

        await waitFor(() => {
            expect(screen.getByTestId('rule-api_key-0')).toBeInTheDocument();
        });

        // Check pattern is displayed
        expect(screen.getByText('sk-[a-zA-Z0-9]+')).toBeInTheDocument();
    });

    it('should show error state when loading fails', async () => {
        vi.mocked(sanitizerIpc.getPrivacyRules).mockRejectedValue(new Error('Network error'));

        render(<SystemRuleList />);

        await waitFor(() => {
            expect(screen.getByText(/Load Failed/)).toBeInTheDocument();
            expect(screen.getByText(/Network error/)).toBeInTheDocument();
        });
    });

    it('should display lock icon for readonly rules', async () => {
        const mockRules = [
            { name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key' },
        ];

        vi.mocked(sanitizerIpc.getPrivacyRules).mockResolvedValue(mockRules as never);

        render(<SystemRuleList />);

        await waitFor(() => {
            const ruleItem = screen.getByTestId('rule-api_key-0');
            expect(ruleItem).toBeInTheDocument();
            // Lock icon should be present (via Lucide React)
            expect(ruleItem.querySelector('svg')).toBeInTheDocument();
        });
    });
});
