/**
 * DiffPreview 组件单元测试 - Story 3-2 Task 9
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DiffPreview } from './DiffPreview';
import type { SanitizationStats } from './types';

// Mock IntersectionObserver
class MockIntersectionObserver implements IntersectionObserver {
    readonly root: Element | Document | null = null;
    readonly rootMargin: string = '';
    readonly thresholds: ReadonlyArray<number> = [];

    constructor(callback: IntersectionObserverCallback) {
        // 立即触发回调，模拟元素可见
        setTimeout(() => {
            callback(
                [{ isIntersecting: true } as IntersectionObserverEntry],
                this
            );
        }, 0);
    }

    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
    takeRecords = vi.fn().mockReturnValue([]);
}

beforeEach(() => {
    vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
});

afterEach(() => {
    vi.unstubAllGlobals();
});

const mockStats: SanitizationStats = {
    counts: {
        api_key: 2,
        ip_address: 3,
    },
    total: 5,
};

const emptyStats: SanitizationStats = {
    counts: {},
    total: 0,
};

describe('DiffPreview', () => {
    describe('渲染测试 (Task 9.1)', () => {
        it('应该渲染 Diff 预览组件', () => {
            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="world"
                    stats={mockStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                />
            );

            expect(screen.getByTestId('diff-preview')).toBeInTheDocument();
        });

        it('应该显示确认提示文案', () => {
            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="world"
                    stats={mockStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                />
            );

            expect(screen.getByTestId('confirm-message')).toHaveTextContent('将以清洗后的版本进行分享');
        });

        it('应该显示取消和确认按钮', () => {
            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="world"
                    stats={mockStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                />
            );

            expect(screen.getByTestId('cancel-button')).toBeInTheDocument();
            expect(screen.getByTestId('confirm-button')).toBeInTheDocument();
        });
    });

    describe('统计显示测试 (Task 9.4)', () => {
        it('应该显示敏感信息统计', () => {
            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="world"
                    stats={mockStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                />
            );

            expect(screen.getByText('共检测到 5 处敏感信息')).toBeInTheDocument();
            expect(screen.getByText('API Key: 2')).toBeInTheDocument();
            expect(screen.getByText('IP 地址: 3')).toBeInTheDocument();
        });

        it('无敏感信息时应该显示安全提示', () => {
            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="hello"
                    stats={emptyStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                />
            );

            expect(screen.getByText('未检测到敏感信息')).toBeInTheDocument();
        });
    });

    describe('按钮状态逻辑测试 (Task 9.5)', () => {
        it('点击取消按钮应该调用 onCancel', async () => {
            const user = userEvent.setup();
            const onCancel = vi.fn();

            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="world"
                    stats={mockStats}
                    onConfirm={vi.fn()}
                    onCancel={onCancel}
                />
            );

            await user.click(screen.getByTestId('cancel-button'));
            expect(onCancel).toHaveBeenCalledTimes(1);
        });

        it('点击确认按钮应该调用 onConfirm (当已滚动到底部时)', async () => {
            const user = userEvent.setup();
            const onConfirm = vi.fn();

            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="world"
                    stats={mockStats}
                    onConfirm={onConfirm}
                    onCancel={vi.fn()}
                />
            );

            await user.click(screen.getByTestId('confirm-button'));
            expect(onConfirm).toHaveBeenCalledTimes(1);
        });

        it('加载状态时按钮应该被禁用', () => {
            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="world"
                    stats={mockStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                    isLoading={true}
                />
            );

            expect(screen.getByTestId('cancel-button')).toBeDisabled();
            expect(screen.getByTestId('confirm-button')).toBeDisabled();
        });

        it('加载状态时应该显示加载文案', () => {
            render(
                <DiffPreview
                    originalText="hello"
                    sanitizedText="world"
                    stats={mockStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                    isLoading={true}
                />
            );

            expect(screen.getByText('处理中...')).toBeInTheDocument();
        });
    });

    describe('Diff 渲染测试', () => {
        it('应该正确渲染变更行', () => {
            render(
                <DiffPreview
                    originalText="const key = 'sk-secret';"
                    sanitizedText="const key = '[REDACTED]';"
                    stats={mockStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                />
            );

            // 验证有删除行 (-) 和新增行 (+)
            expect(screen.getByText('+')).toBeInTheDocument();
            expect(screen.getByText('-')).toBeInTheDocument();
        });

        it('无变更时不应该显示 +/- 符号', () => {
            render(
                <DiffPreview
                    originalText="same content"
                    sanitizedText="same content"
                    stats={emptyStats}
                    onConfirm={vi.fn()}
                    onCancel={vi.fn()}
                />
            );

            expect(screen.queryByText('+')).not.toBeInTheDocument();
            expect(screen.queryByText('-')).not.toBeInTheDocument();
        });
    });
});
