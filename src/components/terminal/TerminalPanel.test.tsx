/**
 * TerminalPanel 测试
 * Story 2.15: Task 2.5
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";

// Mock modules 必须在导入组件之前
vi.mock("@xterm/xterm", () => ({
    Terminal: class MockTerminal {
        options: Record<string, unknown> = {};
        loadAddon = vi.fn();
        open = vi.fn();
        write = vi.fn();
        dispose = vi.fn();
    },
}));

vi.mock("@xterm/addon-fit", () => ({
    FitAddon: class MockFitAddon {
        fit = vi.fn();
    },
}));

vi.mock("@xterm/addon-web-links", () => ({
    WebLinksAddon: class MockWebLinksAddon { },
}));

// Mock xterm CSS
vi.mock("@xterm/xterm/css/xterm.css", () => ({}));

// Mock ResizeObserver
vi.stubGlobal("ResizeObserver", class MockResizeObserver {
    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
});

// 组件必须在 mock 之后导入
import { TerminalPanel } from "./TerminalPanel";

describe("TerminalPanel", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("应该渲染终端容器", () => {
        render(<TerminalPanel content="Hello World" />);

        const terminal = screen.getByTestId("terminal-panel");
        expect(terminal).toBeInTheDocument();
    });

    it("应该应用自定义 className", () => {
        render(<TerminalPanel content="test" className="custom-class" />);

        const terminal = screen.getByTestId("terminal-panel");
        expect(terminal).toHaveClass("custom-class");
    });

    it("应该设置最小和最大高度", () => {
        render(
            <TerminalPanel content="test" minHeight={150} maxHeight={500} />
        );

        const terminal = screen.getByTestId("terminal-panel");
        expect(terminal).toHaveStyle({ minHeight: "150px", maxHeight: "500px" });
    });

    it("深色模式应该应用 zinc-950 背景", () => {
        render(<TerminalPanel content="test" isDark={true} />);

        const terminal = screen.getByTestId("terminal-panel");
        expect(terminal).toHaveClass("bg-zinc-950");
    });

    it("浅色模式应该应用 white 背景", () => {
        render(<TerminalPanel content="test" isDark={false} />);

        const terminal = screen.getByTestId("terminal-panel");
        expect(terminal).toHaveClass("bg-white");
    });

    it("应该使用默认高度值", () => {
        render(<TerminalPanel content="test" />);

        const terminal = screen.getByTestId("terminal-panel");
        expect(terminal).toHaveStyle({ minHeight: "100px", maxHeight: "400px" });
    });
});
