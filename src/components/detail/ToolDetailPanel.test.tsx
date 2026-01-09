/**
 * ToolDetailPanel 测试
 * Story 2.15: Task 4.5
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ToolDetailPanel } from "./ToolDetailPanel";

describe("ToolDetailPanel", () => {
    // Story 8.12: 添加 standardTool 用于渲染器选择
    const defaultProps = {
        toolName: "read_file",
        toolInput: { path: "/src/test.ts" },
        toolOutput: "file content here",
        standardTool: {
            type: "file_read" as const,
            path: "/src/test.ts",
        },
    };

    it("应该渲染详情面板", () => {
        render(<ToolDetailPanel {...defaultProps} />);

        const panel = screen.getByTestId("tool-detail-panel");
        expect(panel).toBeInTheDocument();
    });

    it("应该显示工具名称", () => {
        render(<ToolDetailPanel {...defaultProps} />);

        expect(screen.getByText("read_file")).toBeInTheDocument();
    });

    it("应该显示输入参数", () => {
        render(<ToolDetailPanel {...defaultProps} />);

        expect(screen.getByText("输入参数")).toBeInTheDocument();
        expect(screen.getByText(/\/src\/test.ts/)).toBeInTheDocument();
    });

    it("应该显示输出结果（使用渲染器）", () => {
        render(<ToolDetailPanel {...defaultProps} />);

        expect(screen.getByText("输出结果")).toBeInTheDocument();
        // read_file 使用 FileRenderer (Monaco Editor)
        expect(screen.getByTestId("file-renderer")).toBeInTheDocument();
    });

    it("应该显示耗时", () => {
        render(<ToolDetailPanel {...defaultProps} duration={1.5} />);

        expect(screen.getByText("1.5s")).toBeInTheDocument();
    });

    it("错误状态应该显示红色边框", () => {
        render(<ToolDetailPanel {...defaultProps} isError={true} />);

        const panel = screen.getByTestId("tool-detail-panel");
        expect(panel).toHaveClass("border-l-destructive");
    });

    it("应该调用 onClose 回调", () => {
        const onClose = vi.fn();
        render(<ToolDetailPanel {...defaultProps} onClose={onClose} />);

        const closeButton = screen.getByLabelText("关闭详情面板");
        fireEvent.click(closeButton);

        expect(onClose).toHaveBeenCalled();
    });

    it("没有 onClose 时不显示关闭按钮", () => {
        render(<ToolDetailPanel {...defaultProps} />);

        expect(screen.queryByLabelText("关闭详情面板")).not.toBeInTheDocument();
    });

    it("应该使用自定义渲染器", () => {
        const customRenderer = vi.fn().mockReturnValue(<div>Custom Output</div>);
        render(
            <ToolDetailPanel {...defaultProps} renderOutput={customRenderer} />
        );

        expect(customRenderer).toHaveBeenCalledWith("file content here", "read_file");
        expect(screen.getByText("Custom Output")).toBeInTheDocument();
    });

    it("无内容时显示提示", () => {
        render(<ToolDetailPanel toolName="test" />);

        expect(screen.getByText("暂无详情内容")).toBeInTheDocument();
    });

    it("成功状态应该显示绿色图标", () => {
        render(<ToolDetailPanel {...defaultProps} isError={false} />);

        const panel = screen.getByTestId("tool-detail-panel");
        expect(panel).not.toHaveClass("border-l-destructive");
    });
});
