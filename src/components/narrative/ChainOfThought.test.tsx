/**
 * ChainOfThought Component Tests
 * Story 2.4: Task 7
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ChainOfThought } from "./ChainOfThought";

describe("ChainOfThought", () => {
  const testContent = "这是思考过程的内容\n\n1. 第一步\n2. 第二步";

  it("renders the trigger with correct label", () => {
    render(<ChainOfThought content={testContent} />);
    expect(screen.getByText("思考过程")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "思考" })).toBeInTheDocument();
  });

  it("is collapsed by default (AC2)", () => {
    render(<ChainOfThought content={testContent} />);
    const trigger = screen.getByRole("button");
    expect(trigger).toHaveAttribute("aria-expanded", "false");
  });

  it("expands when defaultOpen is true", () => {
    render(<ChainOfThought content={testContent} defaultOpen />);
    const trigger = screen.getByRole("button");
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    // Check for partial content since multiline text is split
    expect(screen.getByText(/这是思考过程的内容/)).toBeInTheDocument();
  });

  it("expands on click (AC2)", async () => {
    const user = userEvent.setup();
    render(<ChainOfThought content={testContent} />);

    const trigger = screen.getByRole("button");
    await user.click(trigger);

    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText(/这是思考过程的内容/)).toBeInTheDocument();
  });

  it("toggles with Enter key (AC6)", async () => {
    const user = userEvent.setup();
    render(<ChainOfThought content={testContent} />);

    const trigger = screen.getByRole("button");
    trigger.focus();
    await user.keyboard("{Enter}");

    expect(trigger).toHaveAttribute("aria-expanded", "true");
  });

  it("toggles with Space key (AC6)", async () => {
    const user = userEvent.setup();
    render(<ChainOfThought content={testContent} />);

    const trigger = screen.getByRole("button");
    trigger.focus();
    await user.keyboard(" ");

    expect(trigger).toHaveAttribute("aria-expanded", "true");
  });

  it("has aria-expanded attribute (AC6)", () => {
    render(<ChainOfThought content={testContent} />);
    const trigger = screen.getByRole("button");
    expect(trigger).toHaveAttribute("aria-expanded");
  });

  it("applies custom className", () => {
    const { container } = render(
      <ChainOfThought content={testContent} className="custom-class" />
    );
    expect(container.firstChild).toHaveClass("custom-class");
  });
});
