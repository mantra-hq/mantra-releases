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
            { id: 'openai_key_1', name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key', enabled: true },
            { id: 'github_token_1', name: 'GitHub Token', pattern: 'ghp_[a-zA-Z0-9]+', sensitive_type: 'github_token', enabled: true },
            { id: 'aws_key_1', name: 'AWS Access Key', pattern: 'AKIA[0-9A-Z]+', sensitive_type: 'aws_key', enabled: true },
        ];

        vi.mocked(sanitizerIpc.getPrivacyRules).mockResolvedValue(mockRules as never);

        render(<SystemRuleList />);

        await waitFor(() => {
            expect(screen.getByTestId('system-rule-list')).toBeInTheDocument();
        });

        // Check header
        expect(screen.getByText('Built-in Rules')).toBeInTheDocument();
        // Component shows "3/3 enabled" format
        expect(screen.getByText(/3\/3/)).toBeInTheDocument();

        // Check groups (first group is expanded by default)
        expect(screen.getByTestId('group-api_key')).toBeInTheDocument();
        expect(screen.getByTestId('group-github_token')).toBeInTheDocument();
        expect(screen.getByTestId('group-aws_key')).toBeInTheDocument();
    });

    it('should expand/collapse groups on click', async () => {
        const mockRules = [
            { id: 'openai_key_1', name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key', enabled: true },
            { id: 'github_token_1', name: 'GitHub Token', pattern: 'ghp_[a-zA-Z0-9]+', sensitive_type: 'github_token', enabled: true },
        ];

        vi.mocked(sanitizerIpc.getPrivacyRules).mockResolvedValue(mockRules as never);

        render(<SystemRuleList />);

        await waitFor(() => {
            expect(screen.getByTestId('system-rule-list')).toBeInTheDocument();
        });

        // Click to expand api_key group (groups are collapsed by default)
        fireEvent.click(screen.getByTestId('group-api_key'));

        await waitFor(() => {
            // Component uses rule-{rule.id} format
            expect(screen.getByTestId('rule-openai_key_1')).toBeInTheDocument();
        });

        // Click to expand github_token group
        fireEvent.click(screen.getByTestId('group-github_token'));

        await waitFor(() => {
            expect(screen.getByTestId('rule-github_token_1')).toBeInTheDocument();
        });
    });

    it('should display rule pattern with monospace font', async () => {
        const mockRules = [
            { id: 'openai_key_1', name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key', enabled: true },
        ];

        vi.mocked(sanitizerIpc.getPrivacyRules).mockResolvedValue(mockRules as never);

        render(<SystemRuleList />);

        await waitFor(() => {
            expect(screen.getByTestId('system-rule-list')).toBeInTheDocument();
        });

        // Click to expand api_key group
        fireEvent.click(screen.getByTestId('group-api_key'));

        await waitFor(() => {
            expect(screen.getByTestId('rule-openai_key_1')).toBeInTheDocument();
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
            { id: 'openai_key_1', name: 'OpenAI API Key', pattern: 'sk-[a-zA-Z0-9]+', sensitive_type: 'api_key', enabled: true },
        ];

        vi.mocked(sanitizerIpc.getPrivacyRules).mockResolvedValue(mockRules as never);

        render(<SystemRuleList />);

        await waitFor(() => {
            expect(screen.getByTestId('system-rule-list')).toBeInTheDocument();
        });

        // Click to expand api_key group
        fireEvent.click(screen.getByTestId('group-api_key'));

        await waitFor(() => {
            const ruleItem = screen.getByTestId('rule-openai_key_1');
            expect(ruleItem).toBeInTheDocument();
            // Lock icon should be present (via Lucide React)
            expect(ruleItem.querySelector('svg')).toBeInTheDocument();
        });
    });
});
