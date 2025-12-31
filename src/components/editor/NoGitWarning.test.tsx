/**
 * NoGitWarning.test - Git 警告组件测试
 * Story 2.11: Task 7 (AC6, AC7)
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { NoGitWarning } from "./NoGitWarning";

describe("NoGitWarning", () => {
  it("renders warning title", () => {
    render(<NoGitWarning />);
    expect(screen.getByText("未检测到 Git 仓库")).toBeInTheDocument();
  });

  it("renders description text (AC6)", () => {
    render(<NoGitWarning />);
    expect(
      screen.getByText(/此项目未检测到 Git 仓库，无法显示代码快照/)
    ).toBeInTheDocument();
  });

  it("renders conversation time travel note", () => {
    render(<NoGitWarning />);
    expect(
      screen.getByText(/对话时间旅行功能仍可正常使用/)
    ).toBeInTheDocument();
  });

  it("renders help text", () => {
    render(<NoGitWarning />);
    expect(
      screen.getByText(/请确保项目目录包含 .git 文件夹/)
    ).toBeInTheDocument();
  });

  it("renders project path when provided", () => {
    render(<NoGitWarning projectPath="/home/user/project" />);
    expect(screen.getByText("/home/user/project")).toBeInTheDocument();
  });

  it("does not render project path when not provided", () => {
    render(<NoGitWarning />);
    expect(screen.queryByRole("code")).not.toBeInTheDocument();
  });

  it("applies custom className", () => {
    const { container } = render(<NoGitWarning className="custom-class" />);
    expect(container.firstChild).toHaveClass("custom-class");
  });

  // AC7: 了解更多链接测试
  it("renders learn more button (AC7)", () => {
    render(<NoGitWarning />);
    expect(screen.getByText("了解更多")).toBeInTheDocument();
  });

  it("calls onLearnMore callback when button clicked", () => {
    const onLearnMore = vi.fn();
    render(<NoGitWarning onLearnMore={onLearnMore} />);
    fireEvent.click(screen.getByText("了解更多"));
    expect(onLearnMore).toHaveBeenCalledTimes(1);
  });
});
