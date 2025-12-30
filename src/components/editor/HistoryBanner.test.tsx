/**
 * HistoryBanner - 历史状态 Banner 测试
 * Story 2.7: Task 4 验证
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { HistoryBanner } from "./HistoryBanner";

describe("HistoryBanner", () => {
    const defaultProps = {
        timestamp: 1735500000000, // 2024-12-29T12:00:00.000Z
        onReturnToCurrent: vi.fn(),
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe("渲染", () => {
        it("应该正确渲染时间戳", () => {
            render(<HistoryBanner {...defaultProps} />);

            // 应该显示"查看历史状态"文本
            expect(screen.getByText("查看历史状态:")).toBeInTheDocument();
        });

        it("应该显示返回按钮", () => {
            render(<HistoryBanner {...defaultProps} />);

            const button = screen.getByRole("button", { name: /返回当前/i });
            expect(button).toBeInTheDocument();
        });

        it("应该显示 Commit Hash", () => {
            render(
                <HistoryBanner
                    {...defaultProps}
                    commitHash="abc1234567890"
                    commitMessage="feat: add login"
                />
            );

            // 应该显示短 hash (前 7 位)
            expect(screen.getByText("abc1234")).toBeInTheDocument();
        });

        it("应该显示 Commit 消息", () => {
            render(
                <HistoryBanner
                    {...defaultProps}
                    commitHash="abc1234"
                    commitMessage="feat: add login"
                />
            );

            expect(screen.getByText(/feat: add login/)).toBeInTheDocument();
        });

        it("应该截断过长的 Commit 消息", () => {
            const longMessage = "This is a very long commit message that should be truncated because it exceeds the maximum length";

            render(
                <HistoryBanner
                    {...defaultProps}
                    commitHash="abc1234"
                    commitMessage={longMessage}
                />
            );

            // 消息应该被截断
            expect(screen.getByText(/\.\.\./)).toBeInTheDocument();
        });
    });

    describe("交互", () => {
        it("点击返回按钮应该调用 onReturnToCurrent", () => {
            const onReturnToCurrent = vi.fn();

            render(
                <HistoryBanner
                    {...defaultProps}
                    onReturnToCurrent={onReturnToCurrent}
                />
            );

            const button = screen.getByRole("button", { name: /返回当前/i });
            fireEvent.click(button);

            expect(onReturnToCurrent).toHaveBeenCalledTimes(1);
        });
    });

    describe("无障碍", () => {
        it("应该有正确的 role", () => {
            render(<HistoryBanner {...defaultProps} />);

            expect(screen.getByRole("banner")).toBeInTheDocument();
        });

        it("应该有 aria-label", () => {
            render(<HistoryBanner {...defaultProps} />);

            expect(
                screen.getByRole("banner", { name: /历史模式提示/i })
            ).toBeInTheDocument();
        });
    });
});
