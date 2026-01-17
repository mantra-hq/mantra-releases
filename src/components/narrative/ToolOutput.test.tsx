/**
 * ToolOutput Component Tests
 * Story 2.4: Task 7
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ToolOutput } from "./ToolOutput";

describe("ToolOutput", () => {
  const successContent = "File content here...";
  const errorContent = "Error: File not found";

  it("renders success state with check icon (AC4)", () => {
    const { container } = render(<ToolOutput content={successContent} />);
    const checkIcon = container.querySelector("svg");
    expect(checkIcon).toBeInTheDocument();
  });

  it("renders error state with X icon (AC4)", () => {
    const { container } = render(<ToolOutput content={errorContent} isError />);
    const xIcon = container.querySelector("svg");
    expect(xIcon).toBeInTheDocument();
  });

  it("has success styling for non-error state (AC4)", () => {
    const { container } = render(<ToolOutput content={successContent} />);
    expect(container.firstChild).toHaveClass("border-l-success");
  });

  it("has error styling for error state (AC4)", () => {
    const { container } = render(<ToolOutput content={errorContent} isError />);
    expect(container.firstChild).toHaveClass("border-l-destructive");
  });

  it("is collapsed by default", () => {
    render(<ToolOutput content={successContent} />);
    const trigger = screen.getByRole("button");
    expect(trigger).toHaveAttribute("aria-expanded", "false");
  });

  it("expands to show full content", async () => {
    const user = userEvent.setup();
    render(<ToolOutput content={successContent} />);

    const trigger = screen.getByRole("button");
    await user.click(trigger);

    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText(successContent)).toBeInTheDocument();
  });

  it("shows content preview when collapsed", () => {
    const longContent = "A".repeat(150);
    render(<ToolOutput content={longContent} />);
    // Preview should be truncated
    expect(screen.getByText(/\.\.\.$/)).toBeInTheDocument();
  });

  it("toggles with keyboard (AC6)", async () => {
    const user = userEvent.setup();
    render(<ToolOutput content={successContent} />);

    const trigger = screen.getByRole("button");
    trigger.focus();
    await user.keyboard("{Enter}");

    expect(trigger).toHaveAttribute("aria-expanded", "true");
  });

  it("has aria-label for success state (AC6)", () => {
    const { container } = render(<ToolOutput content={successContent} />);
    expect(container.firstChild).toHaveAttribute(
      "aria-label",
      "工具执行成功"
    );
  });

  it("has aria-label for error state (AC6)", () => {
    const { container } = render(<ToolOutput content={errorContent} isError />);
    expect(container.firstChild).toHaveAttribute(
      "aria-label",
      "工具执行失败"
    );
  });

  it("applies custom className", () => {
    const { container } = render(
      <ToolOutput content={successContent} className="custom-class" />
    );
    expect(container.firstChild).toHaveClass("custom-class");
  });

  // Story 8.12: 防御性检查测试
  it("handles structuredResult with undefined filePath gracefully", () => {
    // 模拟不完整的数据：type 为 file_read 但 file_path 为 undefined
    const incompleteResult = { type: "file_read" as const } as { type: "file_read"; file_path: string };
    
    // 不应崩溃
    expect(() => {
      render(
        <ToolOutput 
          content={successContent} 
          structuredResult={incompleteResult}
        />
      );
    }).not.toThrow();
  });
});
