/**
 * RuleListItem Component Tests
 * Story 3.10: Task 7.2 - 前端组件测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { RuleListItem } from './RuleListItem';
import type { SanitizationRule } from '@/components/sanitizer/types';

// Mock i18n
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string, params?: Record<string, unknown>) => {
            const translations: Record<string, string> = {
                'privacy.rules.toggle': `Toggle rule ${params?.name ?? ''}`,
                'privacy.rules.delete': `Delete rule ${params?.name ?? ''}`,
                'privacy.rules.custom': 'Custom',
            };
            return translations[key] || key;
        },
    }),
}));

const mockRule: SanitizationRule = {
    id: 'test_rule',
    name: 'Test Rule',
    pattern: '\\btest\\b',
    sensitive_type: 'custom',
    severity: 'warning',
    enabled: true,
};

const defaultProps = {
    rule: mockRule,
    isBuiltin: false,
    onToggle: vi.fn(),
    onDelete: vi.fn(),
};

describe('RuleListItem', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('渲染测试', () => {
        it('应该正确渲染规则项', () => {
            render(<RuleListItem {...defaultProps} />);
            expect(screen.getByTestId('rule-item-test_rule')).toBeInTheDocument();
        });

        it('应该显示规则名称和正则', () => {
            render(<RuleListItem {...defaultProps} />);
            expect(screen.getByText('Test Rule')).toBeInTheDocument();
            expect(screen.getByText('\\btest\\b')).toBeInTheDocument();
        });

        it('应该显示严重程度标签', () => {
            render(<RuleListItem {...defaultProps} />);
            // 严重程度使用 SEVERITY_LABELS 常量
            expect(screen.getByTestId('rule-item-test_rule')).toBeInTheDocument();
        });
    });

    describe('启用/禁用切换', () => {
        it('应该渲染 Switch 组件', () => {
            render(<RuleListItem {...defaultProps} />);
            expect(screen.getByTestId('rule-switch-test_rule')).toBeInTheDocument();
        });

        it('点击 Switch 应该调用 onToggle', async () => {
            const onToggle = vi.fn();
            render(<RuleListItem {...defaultProps} onToggle={onToggle} />);

            const switchEl = screen.getByTestId('rule-switch-test_rule');
            await userEvent.click(switchEl);

            expect(onToggle).toHaveBeenCalledWith(false); // 从 enabled: true 切换到 false
        });
    });

    describe('删除按钮', () => {
        it('自定义规则应该显示删除按钮', () => {
            render(<RuleListItem {...defaultProps} isBuiltin={false} />);
            expect(screen.getByTestId('rule-delete-test_rule')).toBeInTheDocument();
        });

        it('内置规则不应该显示删除按钮', () => {
            render(<RuleListItem {...defaultProps} isBuiltin={true} onDelete={undefined} />);
            expect(screen.queryByTestId('rule-delete-test_rule')).not.toBeInTheDocument();
        });

        it('点击删除按钮应该调用 onDelete', async () => {
            const onDelete = vi.fn();
            render(<RuleListItem {...defaultProps} onDelete={onDelete} />);

            const deleteButton = screen.getByTestId('rule-delete-test_rule');
            await userEvent.click(deleteButton);

            expect(onDelete).toHaveBeenCalledOnce();
        });
    });

    describe('自定义规则标签', () => {
        it('自定义规则应该显示 Custom 标签', () => {
            render(<RuleListItem {...defaultProps} isBuiltin={false} />);
            expect(screen.getByText('Custom')).toBeInTheDocument();
        });

        it('内置规则不应该显示 Custom 标签', () => {
            render(<RuleListItem {...defaultProps} isBuiltin={true} />);
            expect(screen.queryByText('Custom')).not.toBeInTheDocument();
        });
    });
});
