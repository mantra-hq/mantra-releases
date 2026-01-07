/**
 * SanitizeStatusBanner 组件单元测试 - Story 3-4 Task 7
 */

import { describe, it, expect, vi, type ReactNode } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SanitizeStatusBanner } from './SanitizeStatusBanner';
import { TooltipProvider } from '@/components/ui/tooltip';
import type { SanitizationStats, SensitiveMatch } from './types';

// 使用全局 i18n 配置（来自 test/setup.ts）

// 封装 TooltipProvider
const renderWithProviders = (ui: ReactNode) => {
    return render(
        <TooltipProvider>
            {ui}
        </TooltipProvider>
    );
};

const mockStatsWithSensitive: SanitizationStats = {
    counts: {
        api_key: 2,
        ip_address: 1,
    },
    total: 3,
};

const mockStatsEmpty: SanitizationStats = {
    counts: {},
    total: 0,
};

const mockMatches: SensitiveMatch[] = [
    {
        id: 'match-1',
        type: 'api_key',
        original: 'sk-***1234',
        sanitized: '[REDACTED:API_KEY]',
        lineNumber: 5,
        context: 'line 5 context',
    },
    {
        id: 'match-2',
        type: 'api_key',
        original: 'sk-***5678',
        sanitized: '[REDACTED:API_KEY]',
        lineNumber: 10,
        context: 'line 10 context',
    },
    {
        id: 'match-3',
        type: 'ip_address',
        original: '10.***0.1',
        sanitized: '[REDACTED:IP]',
        lineNumber: 15,
        context: 'line 15 context',
    },
];

describe('SanitizeStatusBanner', () => {
    describe('有敏感信息状态 (AC2)', () => {
        it('应该显示检测到敏感信息的提示', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsWithSensitive}
                    sensitiveMatches={mockMatches}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            expect(screen.getByText(/检测到 3 处敏感信息/)).toBeInTheDocument();
        });

        it('应该显示分类标签', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsWithSensitive}
                    sensitiveMatches={mockMatches}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            expect(screen.getByText('API Key')).toBeInTheDocument();
            expect(screen.getByText('IP 地址')).toBeInTheDocument();
        });

        it('应该显示操作按钮', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsWithSensitive}
                    sensitiveMatches={mockMatches}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            expect(screen.getByTestId('cancel-button')).toBeInTheDocument();
            expect(screen.getByTestId('confirm-button')).toBeInTheDocument();
        });

        it('应该使用警告色背景 (amber)', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsWithSensitive}
                    sensitiveMatches={mockMatches}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            const banner = screen.getByTestId('sanitize-status-banner');
            expect(banner.className).toContain('bg-amber-500/10');
        });
    });

    describe('无敏感信息状态 (AC2)', () => {
        it('应该显示安全提示', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsEmpty}
                    sensitiveMatches={[]}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            // 使用 i18n key sanitizer.safeToShareFull 对应的中文
            expect(screen.getByText(/此内容可以安全分享/)).toBeInTheDocument();
        });

        it('应该使用安全色背景 (green)', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsEmpty}
                    sensitiveMatches={[]}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            const banner = screen.getByTestId('sanitize-status-banner');
            expect(banner.className).toContain('bg-green-500/10');
        });

        it('不应该显示分类标签', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsEmpty}
                    sensitiveMatches={[]}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            expect(screen.queryByText('API Key')).not.toBeInTheDocument();
        });
    });

    describe('标签跳转功能 (AC3)', () => {
        it('点击标签应该触发跳转回调', async () => {
            const user = userEvent.setup();
            const onJumpToLine = vi.fn();

            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsWithSensitive}
                    sensitiveMatches={mockMatches}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                    onJumpToLine={onJumpToLine}
                />
            );

            // 点击 API Key 标签
            const apiKeyTag = screen.getByText('API Key').closest('button');
            await user.click(apiKeyTag!);

            // 应该跳转到第一个 API Key 的行号
            expect(onJumpToLine).toHaveBeenCalledWith(5);
        });

        it('没有 onJumpToLine 时标签应该禁用', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsWithSensitive}
                    sensitiveMatches={mockMatches}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            const apiKeyTag = screen.getByText('API Key').closest('button');
            expect(apiKeyTag).toBeDisabled();
        });
    });

    describe('操作按钮交互', () => {
        it('点击取消应该触发 onCancel', async () => {
            const user = userEvent.setup();
            const onCancel = vi.fn();

            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsWithSensitive}
                    sensitiveMatches={mockMatches}
                    onCancel={onCancel}
                    onConfirm={vi.fn()}
                />
            );

            await user.click(screen.getByTestId('cancel-button'));
            expect(onCancel).toHaveBeenCalled();
        });

        it('点击确认分享按钮应该禁用（即将上线功能）', async () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsWithSensitive}
                    sensitiveMatches={mockMatches}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            // 确认按钮应该禁用
            const confirmButton = screen.getByTestId('confirm-button');
            expect(confirmButton).toBeDisabled();
        });
    });

    describe('加载状态', () => {
        it('加载中应该显示 loading 指示器', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsEmpty}
                    sensitiveMatches={[]}
                    isLoading={true}
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            expect(screen.getByText(/正在扫描/)).toBeInTheDocument();
        });
    });

    describe('错误状态', () => {
        it('错误时应该显示错误信息', () => {
            renderWithProviders(
                <SanitizeStatusBanner
                    stats={mockStatsEmpty}
                    sensitiveMatches={[]}
                    error="脱敏预览失败"
                    onCancel={vi.fn()}
                    onConfirm={vi.fn()}
                />
            );

            expect(screen.getByText('脱敏预览失败')).toBeInTheDocument();
        });
    });
});
