/**
 * SnapshotBadge - 单元测试
 * Story 2.14: Task 1.5
 */

import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { SnapshotBadge } from "./SnapshotBadge";

describe("SnapshotBadge", () => {
    describe("icon mode", () => {
        it("renders snapshot icon with blue color", () => {
            render(<SnapshotBadge type="snapshot" mode="icon" />);

            const icon = screen.getByTestId("snapshot-badge-icon-snapshot");
            expect(icon).toBeInTheDocument();
            expect(icon).toHaveClass("text-blue-500");
        });

        it("renders git-history icon with amber color", () => {
            render(<SnapshotBadge type="git-history" mode="icon" />);

            const icon = screen.getByTestId("snapshot-badge-icon-git-history");
            expect(icon).toBeInTheDocument();
            expect(icon).toHaveClass("text-amber-500");
        });

        it("applies custom className", () => {
            render(<SnapshotBadge type="snapshot" mode="icon" className="custom-class" />);

            const icon = screen.getByTestId("snapshot-badge-icon-snapshot");
            expect(icon).toHaveClass("custom-class");
        });
    });

    describe("pill mode - snapshot", () => {
        it("renders snapshot pill with timestamp formatted as HH:MM", () => {
            // 2024-01-15 10:32:00
            const timestamp = new Date("2024-01-15T10:32:00").getTime();
            render(<SnapshotBadge type="snapshot" mode="pill" timestamp={timestamp} />);

            const pill = screen.getByTestId("snapshot-badge-pill-snapshot");
            expect(pill).toBeInTheDocument();
            expect(pill).toHaveClass("bg-blue-500/10", "text-blue-500");
            expect(pill).toHaveTextContent("10:32");
        });

        it("returns null when snapshot pill has no timestamp", () => {
            const { container } = render(<SnapshotBadge type="snapshot" mode="pill" />);
            expect(container.firstChild).toBeNull();
        });
    });

    describe("pill mode - git-history", () => {
        it("renders git-history pill with commit hash", () => {
            render(
                <SnapshotBadge
                    type="git-history"
                    mode="pill"
                    commitHash="abc1234567890"
                />
            );

            const pill = screen.getByTestId("snapshot-badge-pill-git-history");
            expect(pill).toBeInTheDocument();
            expect(pill).toHaveClass("bg-amber-500/10", "text-amber-500");
            expect(pill).toHaveTextContent("abc1234");
        });

        it("renders git-history pill with commit hash and relative time", () => {
            render(
                <SnapshotBadge
                    type="git-history"
                    mode="pill"
                    commitHash="abc1234567890"
                    relativeTime="3天前"
                />
            );

            const pill = screen.getByTestId("snapshot-badge-pill-git-history");
            expect(pill).toHaveTextContent("abc1234 · 3天前");
        });

        it("returns null when git-history pill has no commitHash", () => {
            const { container } = render(<SnapshotBadge type="git-history" mode="pill" />);
            expect(container.firstChild).toBeNull();
        });
    });
});
