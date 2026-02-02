/**
 * PrivacySettingsPanel Component Tests
 * Story 3.10: Task 7.2 - 前端组件测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PrivacySettingsPanel } from './PrivacySettingsPanel';
import type { SanitizationRule } from '@/components/sanitizer/types';

// Mock i18n
vi.mock('react-i18next', () => {
    const t = (key: string, params?: Record<string, unknown>) => {
        const translations: Record<string, string> = {
            'privacy.rules.title': 'Privacy Detection Rules',
            'privacy.rules.add': 'Add Rule',
            'privacy.rules.empty': 'No rules',
            'privacy.rules.customGroup': 'Custom Rules',
            'privacy.rules.deleteConfirmTitle': 'Confirm Delete Rule',
            'privacy.rules.deleteConfirmDesc': `Are you sure you want to delete rule "${params?.name ?? ''}"?`,
            'common.cancel': 'Cancel',
            'common.delete': 'Delete',
            'common.saving': 'Saving...',
        };
        return translations[key] || key;
    };

    return {
        useTranslation: () => ({
            t,
        }),
    };
});

// Mock IPC functions
const mockRules: SanitizationRule[] = [
    {
        id: 'openai_api_key',
        name: 'OpenAI API Key',
        pattern: 'sk-[a-zA-Z0-9]{20,}',
        sensitive_type: 'api_key',
        severity: 'critical',
        enabled: true,
    },
    {
        id: 'custom_rule_1',
        name: 'My Custom Rule',
        pattern: '@mycompany\\.com',
        sensitive_type: 'custom',
        severity: 'warning',
        enabled: true,
    },
];

const mockBuiltinRules = [mockRules[0]];

vi.mock('@/lib/ipc/sanitizer-ipc', () => ({
    getPrivacyRules: vi.fn(() => Promise.resolve(mockRules)),
    getBuiltinRules: vi.fn(() => Promise.resolve(mockBuiltinRules)),
    updatePrivacyRules: vi.fn(() => Promise.resolve()),
    validateRegex: vi.fn(() => Promise.resolve({ valid: true })),
    validateRegexV2: vi.fn(() => Promise.resolve({ valid: true })),
    saveInterceptionRecord: vi.fn(),
    getInterceptionRecords: vi.fn(() => Promise.resolve({ records: [], total: 0 })),
    getInterceptionStats: vi.fn(() => Promise.resolve({ total_blocked: 0 })),
    deleteInterceptionRecords: vi.fn(),
}));

describe('PrivacySettingsPanel', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('渲染测试', () => {
        it('应该正确渲染面板', async () => {
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByTestId('privacy-settings-panel')).toBeInTheDocument();
            });
        });

        it('应该显示标题和添加按钮', async () => {
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByText('Privacy Detection Rules')).toBeInTheDocument();
                expect(screen.getByTestId('add-rule-button')).toBeInTheDocument();
            });
        });

        it('加载时应该显示加载状态', async () => {
            render(<PrivacySettingsPanel />);
            await waitFor(() => {
                expect(screen.getByRole('status')).toBeInTheDocument();
            });
        });
    });

    describe('规则列表', () => {
        it('应该渲染规则列表', async () => {
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByTestId('rule-item-openai_api_key')).toBeInTheDocument();
                expect(screen.getByTestId('rule-item-custom_rule_1')).toBeInTheDocument();
            });
        });

        it('应该按分组显示规则', async () => {
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByTestId('rule-group-api_key')).toBeInTheDocument();
            });
        });
    });

    describe('删除确认对话框 (AC 4.6)', () => {
        it('点击删除应该显示确认对话框', async () => {
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByTestId('rule-item-custom_rule_1')).toBeInTheDocument();
            });

            const deleteButton = screen.getByTestId('rule-delete-custom_rule_1');
            await userEvent.click(deleteButton);

            await waitFor(() => {
                expect(screen.getByText('Confirm Delete Rule')).toBeInTheDocument();
            });
        });

        it('确认对话框应该显示规则名称', async () => {
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByTestId('rule-item-custom_rule_1')).toBeInTheDocument();
            });

            const deleteButton = screen.getByTestId('rule-delete-custom_rule_1');
            await userEvent.click(deleteButton);

            await waitFor(() => {
                expect(
                    screen.getByText('Are you sure you want to delete rule "My Custom Rule"?')
                ).toBeInTheDocument();
            });
        });

        it('点击取消应该关闭对话框', async () => {
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByTestId('rule-item-custom_rule_1')).toBeInTheDocument();
            });

            const deleteButton = screen.getByTestId('rule-delete-custom_rule_1');
            await userEvent.click(deleteButton);

            await waitFor(() => {
                expect(screen.getByText('Confirm Delete Rule')).toBeInTheDocument();
            });

            const cancelButton = screen.getByRole('button', { name: 'Cancel' });
            await userEvent.click(cancelButton);

            await waitFor(() => {
                expect(screen.queryByText('Confirm Delete Rule')).not.toBeInTheDocument();
            });
        });
    });

    describe('规则切换', () => {
        it('应该能切换规则启用状态', async () => {
            const { updatePrivacyRules } = await import('@/lib/ipc/sanitizer-ipc');
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByTestId('rule-switch-openai_api_key')).toBeInTheDocument();
            });

            const switchEl = screen.getByTestId('rule-switch-openai_api_key');
            await userEvent.click(switchEl);

            await waitFor(() => {
                expect(updatePrivacyRules).toHaveBeenCalled();
            });
        });
    });

    describe('添加规则对话框', () => {
        it('点击添加按钮应该打开对话框', async () => {
            render(<PrivacySettingsPanel />);

            await waitFor(() => {
                expect(screen.getByTestId('add-rule-button')).toBeInTheDocument();
            });

            const addButton = screen.getByTestId('add-rule-button');
            await userEvent.click(addButton);

            await waitFor(() => {
                expect(screen.getByTestId('add-rule-sheet')).toBeInTheDocument();
            });
        });
    });
});
