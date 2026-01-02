/**
 * FilterStats Tests - 过滤统计信息组件测试
 * Story 2.16: Task 4.3
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { FilterStats } from "./FilterStats";

describe("FilterStats", () => {
    it("should render stats when filtered count differs from total", () => {
        render(<FilterStats filteredCount={10} totalCount={50} />);

        expect(screen.getByRole("status")).toBeInTheDocument();
        expect(screen.getByText("匹配: 10/50 条")).toBeInTheDocument();
    });

    it("should not render when filtered equals total", () => {
        render(<FilterStats filteredCount={50} totalCount={50} />);

        expect(screen.queryByRole("status")).not.toBeInTheDocument();
    });

    it("should show zero filtered count", () => {
        render(<FilterStats filteredCount={0} totalCount={100} />);

        expect(screen.getByText("匹配: 0/100 条")).toBeInTheDocument();
    });

    it("should apply custom className", () => {
        render(
            <FilterStats
                filteredCount={5}
                totalCount={10}
                className="custom-class"
            />
        );

        expect(screen.getByRole("status")).toHaveClass("custom-class");
    });

    it("should have proper aria attributes for accessibility", () => {
        render(<FilterStats filteredCount={20} totalCount={40} />);

        const stats = screen.getByRole("status");
        expect(stats).toHaveAttribute("aria-live", "polite");
    });
});
