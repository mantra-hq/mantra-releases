/**
 * StatusBar - 单元测试
 * Story 2.14: Task 5.4
 */

import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatusBar } from "./StatusBar";

describe("StatusBar", () => {
    it("AC #9: 应该渲染状态栏", () => {
        render(<StatusBar />);

        expect(screen.getByTestId("status-bar")).toBeInTheDocument();
    });

    it("AC #12: 应该显示光标位置", () => {
        render(<StatusBar cursorPosition={{ line: 42, column: 8 }} />);

        // i18n key: editor.cursorPosition -> "行 {{line}}, 列 {{col}}"
        expect(screen.getByTestId("cursor-position")).toHaveTextContent("行 42, 列 8");
    });

    it("没有光标位置时不显示", () => {
        render(<StatusBar />);

        expect(screen.queryByTestId("cursor-position")).not.toBeInTheDocument();
    });

    it("应该渲染左侧内容插槽", () => {
        render(
            <StatusBar
                leftContent={<span data-testid="left-content">Branch: main</span>}
            />
        );

        expect(screen.getByTestId("left-content")).toBeInTheDocument();
    });

    it("应该应用自定义类名", () => {
        render(<StatusBar className="custom-class" />);

        expect(screen.getByTestId("status-bar")).toHaveClass("custom-class");
    });
});
