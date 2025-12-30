/**
 * ToolCall Component Tests
 * Story 2.4: Task 7
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ToolCall } from "./ToolCall";

describe("ToolCall", () => {
  const testToolName = "Read";
  const testToolInput = {
    file_path: "/src/App.tsx",
    offset: 0,
    limit: 100,
  };

  it("renders tool name (AC3)", () => {
    render(<ToolCall toolName={testToolName} />);
    expect(screen.getByText(testToolName)).toBeInTheDocument();
  });

  it("shows wrench icon (AC3)", () => {
    const { container } = render(<ToolCall toolName={testToolName} />);
    const wrenchIcon = container.querySelector("svg");
    expect(wrenchIcon).toBeInTheDocument();
  });

  it("is collapsed by default", () => {
    render(<ToolCall toolName={testToolName} toolInput={testToolInput} />);
    const trigger = screen.getByRole("button");
    expect(trigger).toHaveAttribute("aria-expanded", "false");
  });

  it("expands to show JSON formatted input (AC3)", async () => {
    const user = userEvent.setup();
    render(<ToolCall toolName={testToolName} toolInput={testToolInput} />);

    const trigger = screen.getByRole("button");
    await user.click(trigger);

    expect(screen.getByText(/file_path/)).toBeInTheDocument();
    expect(screen.getByText(/\/src\/App\.tsx/)).toBeInTheDocument();
  });

  it("does not show expand arrow when no input", () => {
    const { container } = render(<ToolCall toolName={testToolName} />);
    // Only wrench icon, no chevron
    const icons = container.querySelectorAll("svg");
    expect(icons).toHaveLength(1);
  });

  it("shows expand arrow when has input", () => {
    const { container } = render(
      <ToolCall toolName={testToolName} toolInput={testToolInput} />
    );
    // Wrench icon + chevron
    const icons = container.querySelectorAll("svg");
    expect(icons).toHaveLength(2);
  });

  it("toggles with Enter key (AC6)", async () => {
    const user = userEvent.setup();
    render(<ToolCall toolName={testToolName} toolInput={testToolInput} />);

    const trigger = screen.getByRole("button");
    trigger.focus();
    await user.keyboard("{Enter}");

    expect(trigger).toHaveAttribute("aria-expanded", "true");
  });

  it("has aria-expanded attribute (AC6)", () => {
    render(<ToolCall toolName={testToolName} toolInput={testToolInput} />);
    const trigger = screen.getByRole("button");
    expect(trigger).toHaveAttribute("aria-expanded");
  });

  it("applies custom className", () => {
    const { container } = render(
      <ToolCall toolName={testToolName} className="custom-class" />
    );
    expect(container.firstChild).toHaveClass("custom-class");
  });
});
