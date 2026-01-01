/**
 * BranchSelector - 单元测试
 * Story 2.14: Task 6.6
 */

import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { BranchSelector } from "./BranchSelector";

describe("BranchSelector", () => {
    it("AC #10: 应该显示当前分支名", () => {
        render(<BranchSelector currentBranch="main" />);

        expect(screen.getByTestId("current-branch")).toHaveTextContent("main");
    });

    it("应该显示自定义分支名", () => {
        render(<BranchSelector currentBranch="feature/story-2-14" />);

        expect(screen.getByTestId("current-branch")).toHaveTextContent("feature/story-2-14");
    });

    it("加载中状态应显示提示文字", () => {
        render(<BranchSelector isLoading />);

        expect(screen.getByTestId("current-branch")).toHaveTextContent("加载中...");
    });

    it("应该渲染分支选择器按钮", () => {
        render(
            <BranchSelector
                currentBranch="main"
                branches={[
                    { name: "main", isCurrent: true },
                    { name: "develop", isCurrent: false },
                ]}
            />
        );

        // 验证按钮存在
        expect(screen.getByTestId("branch-selector")).toBeInTheDocument();
    });

    it("加载中时按钮应禁用", () => {
        render(<BranchSelector currentBranch="main" isLoading />);

        expect(screen.getByTestId("branch-selector")).toBeDisabled();
    });
});
