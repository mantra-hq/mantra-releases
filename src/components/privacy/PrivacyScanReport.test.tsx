/**
 * PrivacyScanReport Component Tests
 * Story 3-9: Task 1.6 - 单元测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PrivacyScanReport, type PrivacyScanReportProps } from './PrivacyScanReport';
import type { ScanResult, ScanMatch } from '@/components/sanitizer/types';

// Mock i18n
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string, params?: Record<string, unknown>) => {
            const translations: Record<string, string> = {
                'privacy.scan.title': 'Privacy Check Found Sensitive Information',
                'privacy.scan.summary': 'Scan Summary',
                'privacy.scan.detected': 'Detected Sensitive Information',
                'privacy.scan.items': `${params?.count ?? 0} items`,
                'privacy.scan.original': 'Original',
                'privacy.scan.masked': 'Masked',
                'privacy.scan.context': 'Context',
                'privacy.scan.line': 'Line',
                'privacy.scan.criticalWarning': `Contains ${params?.count ?? 0} critical sensitive items`,
                'privacy.scan.warningOnly': `Contains ${params?.count ?? 0} warning-level items`,
                'privacy.scan.noIssues': 'No sensitive information found',
                'privacy.scan.scanning': 'Scanning...',
                'privacy.scan.actions.cancel': 'Cancel',
                'privacy.scan.actions.ignore': 'Ignore & Continue',
                'privacy.scan.actions.redact': 'Redact All',
                'privacy.scan.severity.critical': 'Critical',
                'privacy.scan.severity.warning': 'Warning',
                'privacy.scan.severity.info': 'Info',
            };
            return translations[key] || key;
        },
    }),
}));

const createMockMatch = (overrides?: Partial<ScanMatch>): ScanMatch => ({
    rule_id: 'api_key_rule',
    sensitive_type: 'api_key',
    severity: 'critical',
    line: 45,
    column: 10,
    matched_text: 'sk-proj-abcd1234567890',
    masked_text: 'sk-proj-****XXXX',
    context: 'const apiKey = "sk-proj-abcd1234..."',
    ...overrides,
});

const createMockScanResult = (
    matchOverrides?: Partial<ScanMatch>[],
    hasCritical = true,
    hasWarning = true
): ScanResult => {
    const matches = matchOverrides?.map((o) => createMockMatch(o)) ?? [
        createMockMatch(),
        createMockMatch({
            rule_id: 'email_rule',
            sensitive_type: 'email',
            severity: 'warning',
            line: 102,
            column: 5,
            matched_text: 'user@example.com',
            masked_text: 'u***@example.com',
            context: 'const email = "user@example.com"',
        }),
    ];

    return {
        matches,
        has_critical: hasCritical,
        has_warning: hasWarning,
        scan_time_ms: 15,
        stats: {
            critical_count: matches.filter((m) => m.severity === 'critical').length,
            warning_count: matches.filter((m) => m.severity === 'warning').length,
            info_count: matches.filter((m) => m.severity === 'info').length,
            total: matches.length,
            by_type: {},
        },
    };
};

const defaultProps: PrivacyScanReportProps = {
    isOpen: true,
    scanResult: createMockScanResult(),
    isScanning: false,
    onRedact: vi.fn(),
    onIgnore: vi.fn(),
    onCancel: vi.fn(),
};

describe('PrivacyScanReport', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('Task 1.1 - 组件渲染', () => {
        it('应该正确渲染弹窗', () => {
            render(<PrivacyScanReport {...defaultProps} />);
            expect(screen.getByTestId('privacy-scan-report-dialog')).toBeInTheDocument();
        });

        it('isOpen 为 false 时不应该渲染内容', () => {
            render(<PrivacyScanReport {...defaultProps} isOpen={false} />);
            expect(screen.queryByTestId('privacy-scan-report-dialog')).not.toBeInTheDocument();
        });
    });

    describe('Task 1.2 - 扫描结果摘要', () => {
        it('应该显示 Critical/Warning/Info 数量', () => {
            render(<PrivacyScanReport {...defaultProps} />);

            expect(screen.getByTestId('severity-critical-count')).toHaveTextContent('1');
            expect(screen.getByTestId('severity-warning-count')).toHaveTextContent('1');
            expect(screen.getByTestId('severity-info-count')).toHaveTextContent('0');
        });

        it('包含 Critical 时应该显示警告信息', () => {
            render(<PrivacyScanReport {...defaultProps} />);
            expect(screen.getByTestId('critical-warning-message')).toBeInTheDocument();
        });
    });

    describe('Task 1.3 - 敏感信息列表', () => {
        it('应该渲染所有匹配项', () => {
            render(<PrivacyScanReport {...defaultProps} />);

            const items = screen.getAllByTestId(/^scan-match-item-/);
            expect(items).toHaveLength(2);
        });

        it('每个匹配项应该显示类型、严重程度、脱敏预览和上下文', () => {
            render(<PrivacyScanReport {...defaultProps} />);

            // 第一个匹配项 - API Key
            expect(screen.getByTestId('scan-match-item-0')).toBeInTheDocument();
            expect(screen.getByTestId('match-type-0')).toHaveTextContent('API Key');
            expect(screen.getByTestId('match-severity-0')).toBeInTheDocument();
            expect(screen.getByTestId('match-masked-0')).toHaveTextContent('sk-proj-****XXXX');
            expect(screen.getByTestId('match-context-0')).toBeInTheDocument();
        });

        it('应该显示行号', () => {
            render(<PrivacyScanReport {...defaultProps} />);
            expect(screen.getByTestId('match-line-0')).toHaveTextContent('45');
        });
    });

    describe('Task 1.4 - 操作按钮', () => {
        it('应该渲染三个操作按钮', () => {
            render(<PrivacyScanReport {...defaultProps} />);

            expect(screen.getByTestId('btn-redact')).toBeInTheDocument();
            expect(screen.getByTestId('btn-ignore')).toBeInTheDocument();
            expect(screen.getByTestId('btn-cancel')).toBeInTheDocument();
        });

        it('点击"一键脱敏"应该调用 onRedact', async () => {
            const onRedact = vi.fn();
            render(<PrivacyScanReport {...defaultProps} onRedact={onRedact} />);

            await userEvent.click(screen.getByTestId('btn-redact'));
            expect(onRedact).toHaveBeenCalledOnce();
        });

        it('点击"忽略并继续"应该调用 onIgnore', async () => {
            const onIgnore = vi.fn();
            render(<PrivacyScanReport {...defaultProps} onIgnore={onIgnore} />);

            await userEvent.click(screen.getByTestId('btn-ignore'));
            expect(onIgnore).toHaveBeenCalledOnce();
        });

        it('点击"取消"应该调用 onCancel', async () => {
            const onCancel = vi.fn();
            render(<PrivacyScanReport {...defaultProps} onCancel={onCancel} />);

            await userEvent.click(screen.getByTestId('btn-cancel'));
            expect(onCancel).toHaveBeenCalledOnce();
        });
    });

    describe('Task 1.5 - 加载状态', () => {
        it('isScanning 为 true 时应该显示加载状态', () => {
            render(<PrivacyScanReport {...defaultProps} isScanning={true} scanResult={null} />);

            expect(screen.getByTestId('scan-loading')).toBeInTheDocument();
            expect(screen.getByText('Scanning...')).toBeInTheDocument();
        });

        it('加载时不应该渲染扫描结果', () => {
            render(<PrivacyScanReport {...defaultProps} isScanning={true} scanResult={null} />);

            expect(screen.queryByTestId('scan-summary')).not.toBeInTheDocument();
            expect(screen.queryByTestId('scan-match-list')).not.toBeInTheDocument();
        });

        it('加载时应该禁用操作按钮', () => {
            render(<PrivacyScanReport {...defaultProps} isScanning={true} />);

            expect(screen.getByTestId('btn-redact')).toBeDisabled();
            expect(screen.getByTestId('btn-ignore')).toBeDisabled();
            expect(screen.getByTestId('btn-cancel')).not.toBeDisabled();
        });
    });

    describe('严重程度颜色', () => {
        it('Critical 应该使用红色', () => {
            const result = createMockScanResult([{ severity: 'critical' }], true, false);
            render(<PrivacyScanReport {...defaultProps} scanResult={result} />);

            const severityBadge = screen.getByTestId('match-severity-0');
            expect(severityBadge).toHaveClass('text-red-500');
        });

        it('Warning 应该使用黄色', () => {
            const result = createMockScanResult([{ severity: 'warning' }], false, true);
            render(<PrivacyScanReport {...defaultProps} scanResult={result} />);

            const severityBadge = screen.getByTestId('match-severity-0');
            expect(severityBadge).toHaveClass('text-yellow-500');
        });

        it('Info 应该使用蓝色', () => {
            const result = createMockScanResult([{ severity: 'info' }], false, false);
            render(<PrivacyScanReport {...defaultProps} scanResult={result} />);

            const severityBadge = screen.getByTestId('match-severity-0');
            expect(severityBadge).toHaveClass('text-blue-500');
        });
    });

    describe('无敏感信息场景', () => {
        it('无匹配项时应该显示安全信息', () => {
            const emptyResult: ScanResult = {
                matches: [],
                has_critical: false,
                has_warning: false,
                scan_time_ms: 10,
                stats: {
                    critical_count: 0,
                    warning_count: 0,
                    info_count: 0,
                    total: 0,
                    by_type: {},
                },
            };
            render(<PrivacyScanReport {...defaultProps} scanResult={emptyResult} />);

            expect(screen.getByText('No sensitive information found')).toBeInTheDocument();
        });
    });

    describe('可访问性', () => {
        it('弹窗应该有正确的 role', () => {
            render(<PrivacyScanReport {...defaultProps} />);
            expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        it('按钮应该有 aria-label', () => {
            render(<PrivacyScanReport {...defaultProps} />);

            expect(screen.getByTestId('btn-redact')).toHaveAccessibleName();
            expect(screen.getByTestId('btn-ignore')).toHaveAccessibleName();
            expect(screen.getByTestId('btn-cancel')).toHaveAccessibleName();
        });
    });
});
