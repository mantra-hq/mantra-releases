/**
 * AddRuleSheet Component Tests
 * Story 3.10: Task 7.2 - 前端组件测试
 * Story 12.2: Dialog → Sheet 改造
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { AddRuleSheet } from './AddRuleSheet';

// Mock i18n
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string) => {
            const translations: Record<string, string> = {
                'privacy.rules.addTitle': 'Add Custom Rule',
                'privacy.rules.addDescription': 'Create custom regex rules',
                'privacy.rules.name': 'Rule Name',
                'privacy.rules.namePlaceholder': 'e.g.: Company domain',
                'privacy.rules.pattern': 'Regular Expression',
                'privacy.rules.patternPlaceholder': 'e.g.: @company\\.com',
                'privacy.rules.severity': 'Severity',
                'privacy.rules.errors.emptyName': 'Rule name cannot be empty',
                'privacy.rules.errors.emptyPattern': 'Regex pattern cannot be empty',
                'privacy.rules.errors.invalidRegex': 'Invalid regular expression',
                'privacy.rules.errors.duplicateName': 'Rule name already exists',
                'privacy.severity.critical': 'Critical',
                'privacy.severity.warning': 'Warning',
                'privacy.severity.info': 'Info',
                'common.cancel': 'Cancel',
                'common.add': 'Add',
                'common.validating': 'Validating...',
            };
            return translations[key] || key;
        },
    }),
}));

// Mock IPC
vi.mock('@/lib/ipc/sanitizer-ipc', () => ({
    validateRegexV2: vi.fn(() => Promise.resolve({ valid: true, error: null })),
}));

const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    onAdd: vi.fn(),
    existingRuleIds: [],
    existingRuleNames: [],
};

describe('AddRuleSheet', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('渲染测试', () => {
        it('应该正确渲染对话框', () => {
            render(<AddRuleSheet {...defaultProps} />);
            expect(screen.getByTestId('add-rule-sheet')).toBeInTheDocument();
        });

        it('open 为 false 时不应该渲染', () => {
            render(<AddRuleSheet {...defaultProps} open={false} />);
            expect(screen.queryByTestId('add-rule-sheet')).not.toBeInTheDocument();
        });

        it('应该显示标题和表单字段', () => {
            render(<AddRuleSheet {...defaultProps} />);
            expect(screen.getByText('Add Custom Rule')).toBeInTheDocument();
            expect(screen.getByTestId('rule-name-input')).toBeInTheDocument();
            expect(screen.getByTestId('rule-pattern-input')).toBeInTheDocument();
            expect(screen.getByTestId('rule-severity-select')).toBeInTheDocument();
        });
    });

    describe('表单验证', () => {
        it('名称为空时应该显示错误', async () => {
            render(<AddRuleSheet {...defaultProps} />);

            const submitButton = screen.getByTestId('add-rule-submit');
            await userEvent.click(submitButton);

            await waitFor(() => {
                expect(screen.getByTestId('add-rule-error')).toHaveTextContent('Rule name cannot be empty');
            });
        });

        it('正则为空时应该显示错误', async () => {
            render(<AddRuleSheet {...defaultProps} />);

            const nameInput = screen.getByTestId('rule-name-input');
            await userEvent.type(nameInput, 'My Rule');

            const submitButton = screen.getByTestId('add-rule-submit');
            await userEvent.click(submitButton);

            await waitFor(() => {
                expect(screen.getByTestId('add-rule-error')).toHaveTextContent('Regex pattern cannot be empty');
            });
        });
    });

    describe('名称唯一性验证 (AC4)', () => {
        it('名称重复时应该显示错误', async () => {
            render(
                <AddRuleSheet
                    {...defaultProps}
                    existingRuleNames={['Existing Rule']}
                />
            );

            const nameInput = screen.getByTestId('rule-name-input');
            await userEvent.type(nameInput, 'Existing Rule');

            const patternInput = screen.getByTestId('rule-pattern-input');
            await userEvent.type(patternInput, '\\btest\\b');

            const submitButton = screen.getByTestId('add-rule-submit');
            await userEvent.click(submitButton);

            await waitFor(() => {
                expect(screen.getByTestId('add-rule-error')).toHaveTextContent('Rule name already exists');
            });
        });

        it('名称唯一性检查应该不区分大小写', async () => {
            render(
                <AddRuleSheet
                    {...defaultProps}
                    existingRuleNames={['Existing Rule']}
                />
            );

            const nameInput = screen.getByTestId('rule-name-input');
            await userEvent.type(nameInput, 'existing rule'); // lowercase

            const patternInput = screen.getByTestId('rule-pattern-input');
            await userEvent.type(patternInput, '\\btest\\b');

            const submitButton = screen.getByTestId('add-rule-submit');
            await userEvent.click(submitButton);

            await waitFor(() => {
                expect(screen.getByTestId('add-rule-error')).toHaveTextContent('Rule name already exists');
            });
        });
    });

    describe('成功添加规则', () => {
        it('添加成功应该调用 onAdd', async () => {
            const onAdd = vi.fn();
            render(<AddRuleSheet {...defaultProps} onAdd={onAdd} />);

            const nameInput = screen.getByTestId('rule-name-input');
            await userEvent.type(nameInput, 'New Rule');

            const patternInput = screen.getByTestId('rule-pattern-input');
            await userEvent.type(patternInput, '\\btest\\b');

            const submitButton = screen.getByTestId('add-rule-submit');
            await userEvent.click(submitButton);

            await waitFor(() => {
                expect(onAdd).toHaveBeenCalledWith(
                    expect.objectContaining({
                        name: 'New Rule',
                        pattern: '\\btest\\b',
                        sensitive_type: 'custom',
                        enabled: true,
                    })
                );
            });
        });
    });

    describe('取消操作', () => {
        it('点击取消应该调用 onOpenChange(false)', async () => {
            const onOpenChange = vi.fn();
            render(<AddRuleSheet {...defaultProps} onOpenChange={onOpenChange} />);

            const cancelButton = screen.getByTestId('add-rule-cancel');
            await userEvent.click(cancelButton);

            expect(onOpenChange).toHaveBeenCalledWith(false);
        });
    });
});
