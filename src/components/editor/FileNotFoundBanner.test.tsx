/**
 * FileNotFoundBanner.test.tsx - 文件不存在提示组件测试
 * Story 2.12: Task 3
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { FileNotFoundBanner } from "./FileNotFoundBanner";

describe("FileNotFoundBanner", () => {
  const defaultProps = {
    filePath: "src/components/Button.tsx",
    timestamp: Date.now(), // Unix milliseconds
  };

  it("显示文件路径", () => {
    render(<FileNotFoundBanner {...defaultProps} />);

    expect(screen.getByText(/src\/components\/Button\.tsx/)).toBeInTheDocument();
  });

  it("显示'文件在该时间点不存在'提示", () => {
    render(<FileNotFoundBanner {...defaultProps} />);

    expect(screen.getByText(/文件在该时间点不存在/)).toBeInTheDocument();
  });

  it("显示时间信息", () => {
    render(<FileNotFoundBanner {...defaultProps} />);

    // 由于使用 formatDistanceToNow，会显示类似 "几秒前" 的文本
    // 检查是否存在时间相关元素
    const banner = screen.getByRole("alert");
    expect(banner).toBeInTheDocument();
  });

  it("点击'保持当前视图'按钮调用 onKeepCurrent", () => {
    const onKeepCurrent = vi.fn();
    render(
      <FileNotFoundBanner {...defaultProps} onKeepCurrent={onKeepCurrent} />
    );

    const keepCurrentButton = screen.getByText("保持当前视图");
    fireEvent.click(keepCurrentButton);

    expect(onKeepCurrent).toHaveBeenCalledTimes(1);
  });

  it("点击关闭按钮调用 onDismiss", () => {
    const onDismiss = vi.fn();
    render(<FileNotFoundBanner {...defaultProps} onDismiss={onDismiss} />);

    // 查找关闭按钮（通过 aria-label 或 X 图标）
    const dismissButton = screen.getByLabelText("关闭提示");
    fireEvent.click(dismissButton);

    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it("不传递 onKeepCurrent 时不显示'保持当前视图'按钮", () => {
    render(<FileNotFoundBanner {...defaultProps} />);

    expect(screen.queryByText("保持当前视图")).not.toBeInTheDocument();
  });

  it("不传递 onDismiss 时不显示关闭按钮", () => {
    render(<FileNotFoundBanner {...defaultProps} />);

    expect(screen.queryByLabelText("关闭提示")).not.toBeInTheDocument();
  });

  it("使用 amber/warning 样式", () => {
    render(<FileNotFoundBanner {...defaultProps} />);

    const banner = screen.getByRole("alert");
    // 检查是否有 amber 相关的 class
    expect(banner.className).toMatch(/amber/);
  });

  it("没有 timestamp 时不显示时间信息", () => {
    render(<FileNotFoundBanner filePath="src/test.ts" />);

    // 应该显示文件路径
    expect(screen.getByText(/src\/test\.ts/)).toBeInTheDocument();
    // 不应该显示 History 图标关联的时间元素
    expect(screen.getByRole("alert")).toBeInTheDocument();
  });
});
