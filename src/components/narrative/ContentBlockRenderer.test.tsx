/**
 * ContentBlockRenderer Component Tests
 * Story 2.4: Task 7
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ContentBlockRenderer } from "./ContentBlockRenderer";
import type { ContentBlock } from "@/types/message";

describe("ContentBlockRenderer", () => {
  it("renders text block with markdown (AC1)", () => {
    const block: ContentBlock = {
      type: "text",
      content: "**bold** and *italic*",
    };
    render(<ContentBlockRenderer block={block} />);

    // Markdown should be rendered
    expect(screen.getByText("bold")).toBeInTheDocument();
    expect(screen.getByText("and")).toBeInTheDocument();
    expect(screen.getByText("italic")).toBeInTheDocument();
  });

  it("renders thinking block as ChainOfThought (AC2)", () => {
    const block: ContentBlock = {
      type: "thinking",
      content: "思考过程...",
    };
    render(<ContentBlockRenderer block={block} />);

    expect(screen.getByText("思考过程")).toBeInTheDocument();
    expect(screen.getByRole("button")).toHaveAttribute("aria-expanded", "false");
  });

  it("renders tool_use block as ToolCall (AC3)", () => {
    const block: ContentBlock = {
      type: "tool_use",
      content: "",
      toolName: "Read",
      toolInput: { file_path: "/test.ts" },
    };
    render(<ContentBlockRenderer block={block} />);

    expect(screen.getByText("Read")).toBeInTheDocument();
  });

  it("uses 'Unknown Tool' when toolName is missing", () => {
    const block: ContentBlock = {
      type: "tool_use",
      content: "",
    };
    render(<ContentBlockRenderer block={block} />);

    expect(screen.getByText("Unknown Tool")).toBeInTheDocument();
  });

  it("renders tool_result block as ToolOutput - success (AC4)", () => {
    const block: ContentBlock = {
      type: "tool_result",
      content: "Result content",
      isError: false,
    };
    const { container } = render(<ContentBlockRenderer block={block} />);

    expect(container.firstChild).toHaveClass("border-l-success");
  });

  it("renders tool_result block as ToolOutput - error (AC4)", () => {
    const block: ContentBlock = {
      type: "tool_result",
      content: "Error message",
      isError: true,
    };
    const { container } = render(<ContentBlockRenderer block={block} />);

    expect(container.firstChild).toHaveClass("border-l-destructive");
  });

  it("returns null and warns for unknown type", () => {
    const consoleWarn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const block = {
      type: "unknown_type" as ContentBlock["type"],
      content: "test",
    } as ContentBlock;

    const { container } = render(<ContentBlockRenderer block={block} />);

    expect(container.firstChild).toBeNull();
    expect(consoleWarn).toHaveBeenCalledWith(
      expect.stringContaining("Unknown content block type")
    );

    consoleWarn.mockRestore();
  });

  it("applies custom className to text block", () => {
    const block: ContentBlock = {
      type: "text",
      content: "test",
    };
    const { container } = render(
      <ContentBlockRenderer block={block} className="custom-class" />
    );

    expect(container.firstChild).toHaveClass("custom-class");
  });
});
