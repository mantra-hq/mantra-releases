/**
 * TimberLine 组件测试
 * Story 2.6: TimberLine 时间轴控制器
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TimberLine } from "./TimberLine";
import type { TimelineEvent } from "@/types/timeline";

describe("TimberLine", () => {
    const defaultProps = {
        startTime: 1000,
        endTime: 10000,
        currentTime: 5000,
        events: [] as TimelineEvent[],
        onSeek: vi.fn(),
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe("AC1: 时间轴可视化", () => {
        it("renders horizontal timeline container", () => {
            render(<TimberLine {...defaultProps} />);

            const slider = screen.getByRole("slider", { name: "会话时间轴" });
            expect(slider).toBeInTheDocument();
        });

        it("applies custom className", () => {
            render(<TimberLine {...defaultProps} className="custom-class" />);

            const slider = screen.getByRole("slider");
            expect(slider).toHaveClass("custom-class");
        });
    });

    describe("AC3: 可拖拽滑块", () => {
        it("calls onSeek when clicking on track", () => {
            const onSeek = vi.fn();
            render(<TimberLine {...defaultProps} onSeek={onSeek} />);

            const slider = screen.getByRole("slider");
            fireEvent.mouseDown(slider.querySelector(".bg-muted")!);

            expect(onSeek).toHaveBeenCalled();
        });

        it("does not respond when disabled", () => {
            const onSeek = vi.fn();
            render(<TimberLine {...defaultProps} onSeek={onSeek} disabled />);

            const slider = screen.getByRole("slider");
            fireEvent.mouseDown(slider);

            expect(onSeek).not.toHaveBeenCalled();
        });
    });

    describe("AC4: 当前位置指示器", () => {
        it("displays progress bar at correct position", () => {
            // currentTime = 5000, 50% between 1000 and 10000
            render(<TimberLine {...defaultProps} />);

            const slider = screen.getByRole("slider");
            const progressBar = slider.querySelector(".bg-primary");

            expect(progressBar).toBeInTheDocument();
            expect(progressBar).toHaveStyle({ width: "44.44444444444444%" }); // (5000-1000)/(10000-1000) = 44.44%
        });
    });

    describe("AC6: 键盘导航", () => {
        it("has correct ARIA attributes", () => {
            render(<TimberLine {...defaultProps} />);

            const slider = screen.getByRole("slider");

            expect(slider).toHaveAttribute("aria-valuemin", "1000");
            expect(slider).toHaveAttribute("aria-valuemax", "10000");
            expect(slider).toHaveAttribute("aria-valuenow", "5000");
        });

        it("supports keyboard navigation with Home key", async () => {
            const user = userEvent.setup();
            const onSeek = vi.fn();
            render(<TimberLine {...defaultProps} onSeek={onSeek} />);

            const slider = screen.getByRole("slider");
            slider.focus();
            await user.keyboard("{Home}");

            expect(onSeek).toHaveBeenCalledWith(1000); // startTime
        });

        it("supports keyboard navigation with End key", async () => {
            const user = userEvent.setup();
            const onSeek = vi.fn();
            render(<TimberLine {...defaultProps} onSeek={onSeek} />);

            const slider = screen.getByRole("slider");
            slider.focus();
            await user.keyboard("{End}");

            expect(onSeek).toHaveBeenCalledWith(10000); // endTime
        });

        it("is focusable when not disabled", () => {
            render(<TimberLine {...defaultProps} />);

            const slider = screen.getByRole("slider");
            expect(slider).toHaveAttribute("tabIndex", "0");
        });

        it("is not focusable when disabled", () => {
            render(<TimberLine {...defaultProps} disabled />);

            const slider = screen.getByRole("slider");
            expect(slider).toHaveAttribute("tabIndex", "-1");
        });
    });

    describe("AC2: Tick Marks 关键节点", () => {
        it("renders tick marks for events", () => {
            const events: TimelineEvent[] = [
                { timestamp: 3000, type: "user-message", label: "User message" },
                { timestamp: 7000, type: "git-commit", label: "Git commit" },
            ];

            render(<TimberLine {...defaultProps} events={events} />);

            const userMessageTick = screen.getByRole("button", { name: /user message/i });
            const gitCommitTick = screen.getByRole("button", { name: /git commit/i });

            expect(userMessageTick).toBeInTheDocument();
            expect(gitCommitTick).toBeInTheDocument();
        });
    });

    describe("AC5: 时间戳 Tooltip", () => {
        it("shows tooltip on knob hover", async () => {
            render(<TimberLine {...defaultProps} />);

            const slider = screen.getByRole("slider");
            // 滑块是内部的 div，通过查找带有 border-primary 的元素
            const knob = slider.querySelector(".border-primary");
            expect(knob).toBeInTheDocument();

            // 悬停时应显示 Tooltip
            fireEvent.mouseEnter(knob!);

            // Tooltip 使用 bg-zinc-900 样式，检查是否有 Tooltip 渲染
            // Story 2.32: TimeTooltip 结构变更，普通事件不再使用 font-mono
            const tooltip = slider.querySelector(".bg-zinc-900");
            expect(tooltip).toBeInTheDocument();
        });

        it("shows tooltip during drag", () => {
            render(<TimberLine {...defaultProps} />);

            const slider = screen.getByRole("slider");
            const track = slider.querySelector(".bg-muted")!;

            // 开始拖拽
            fireEvent.mouseDown(track);

            // 拖拽时 tooltip 应该显示
            // Story 2.32: TimeTooltip 结构变更，使用 bg-zinc-900 选择器
            const tooltip = slider.querySelector(".bg-zinc-900");
            expect(tooltip).toBeInTheDocument();
        });
    });

    describe("Touch Events (M3)", () => {
        it("responds to touch events for drag", () => {
            const onSeek = vi.fn();
            render(<TimberLine {...defaultProps} onSeek={onSeek} />);

            const slider = screen.getByRole("slider");
            const track = slider.querySelector(".bg-muted")!;

            // 触发 touchstart
            fireEvent.touchStart(track, {
                touches: [{ clientX: 50 }],
            });

            expect(onSeek).toHaveBeenCalled();
        });
    });

    describe("AC7: 主题兼容", () => {
        it("uses CSS variables for theming", () => {
            render(<TimberLine {...defaultProps} />);

            const slider = screen.getByRole("slider");

            // Check that theme-compatible classes are used
            expect(slider).toHaveClass("bg-muted/50");
            expect(slider).toHaveClass("border-border");
        });
    });
});

describe("Timeline Utility Functions", () => {
    describe("timeToPosition", () => {
        it("calculates correct position", async () => {
            const { timeToPosition } = await import("@/types/timeline");

            expect(timeToPosition(5500, 1000, 10000)).toBeCloseTo(50, 1);
            expect(timeToPosition(1000, 1000, 10000)).toBe(0);
            expect(timeToPosition(10000, 1000, 10000)).toBe(100);
        });

        it("clamps position within bounds", async () => {
            const { timeToPosition } = await import("@/types/timeline");

            expect(timeToPosition(0, 1000, 10000)).toBe(0);
            expect(timeToPosition(20000, 1000, 10000)).toBe(100);
        });

        it("handles edge case when start equals end", async () => {
            const { timeToPosition } = await import("@/types/timeline");

            expect(timeToPosition(5000, 5000, 5000)).toBe(0);
        });
    });

    describe("positionToTime", () => {
        it("calculates correct timestamp", async () => {
            const { positionToTime } = await import("@/types/timeline");

            expect(positionToTime(50, 1000, 10000)).toBe(5500);
            expect(positionToTime(0, 1000, 10000)).toBe(1000);
            expect(positionToTime(100, 1000, 10000)).toBe(10000);
        });
    });

    describe("findNearestEvent", () => {
        const events: TimelineEvent[] = [
            { timestamp: 2000, type: "user-message" },
            { timestamp: 5000, type: "ai-response" },
            { timestamp: 8000, type: "git-commit" },
        ];

        it("finds previous event", async () => {
            const { findNearestEvent } = await import("@/types/timeline");

            const result = findNearestEvent(events, 6000, "prev");
            expect(result?.timestamp).toBe(5000);
        });

        it("finds next event", async () => {
            const { findNearestEvent } = await import("@/types/timeline");

            const result = findNearestEvent(events, 3000, "next");
            expect(result?.timestamp).toBe(5000);
        });

        it("finds nearest event", async () => {
            const { findNearestEvent } = await import("@/types/timeline");

            const result = findNearestEvent(events, 4000, "nearest");
            expect(result?.timestamp).toBe(5000);
        });

        it("returns null for empty events", async () => {
            const { findNearestEvent } = await import("@/types/timeline");

            expect(findNearestEvent([], 5000, "nearest")).toBeNull();
        });
    });
});
