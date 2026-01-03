/**
 * DrawerSearch Tests
 * Story 2.18: Task 5.5
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { DrawerSearch, HighlightText } from "./DrawerSearch";

describe("DrawerSearch", () => {
  const defaultProps = {
    value: "",
    onChange: vi.fn(),
    placeholder: "搜索...",
  };

  it("renders search input", () => {
    render(<DrawerSearch {...defaultProps} />);
    expect(screen.getByTestId("drawer-search-input")).toBeInTheDocument();
  });

  it("displays placeholder text", () => {
    render(<DrawerSearch {...defaultProps} placeholder="搜索项目或会话..." />);
    expect(screen.getByPlaceholderText("搜索项目或会话...")).toBeInTheDocument();
  });

  it("calls onChange when input value changes", () => {
    const onChange = vi.fn();
    render(<DrawerSearch {...defaultProps} onChange={onChange} />);

    const input = screen.getByTestId("drawer-search-input");
    fireEvent.change(input, { target: { value: "test" } });

    expect(onChange).toHaveBeenCalledWith("test");
  });

  it("shows clear button when value is not empty", () => {
    render(<DrawerSearch {...defaultProps} value="test" />);
    expect(screen.getByTestId("drawer-search-clear")).toBeInTheDocument();
  });

  it("hides clear button when value is empty", () => {
    render(<DrawerSearch {...defaultProps} value="" />);
    expect(screen.queryByTestId("drawer-search-clear")).not.toBeInTheDocument();
  });

  it("clears value when clear button clicked", () => {
    const onChange = vi.fn();
    render(<DrawerSearch {...defaultProps} value="test" onChange={onChange} />);

    fireEvent.click(screen.getByTestId("drawer-search-clear"));
    expect(onChange).toHaveBeenCalledWith("");
  });
});

describe("HighlightText", () => {
  it("renders text without highlight when no keyword", () => {
    render(<HighlightText text="Hello World" />);
    expect(screen.getByText("Hello World")).toBeInTheDocument();
    expect(document.querySelector("mark")).not.toBeInTheDocument();
  });

  it("renders text without highlight when keyword is empty", () => {
    render(<HighlightText text="Hello World" keyword="" />);
    expect(screen.getByText("Hello World")).toBeInTheDocument();
    expect(document.querySelector("mark")).not.toBeInTheDocument();
  });

  it("highlights matching text", () => {
    render(<HighlightText text="Hello World" keyword="World" />);
    const mark = document.querySelector("mark");
    expect(mark).toBeInTheDocument();
    expect(mark?.textContent).toBe("World");
  });

  it("highlights case-insensitively", () => {
    render(<HighlightText text="Hello World" keyword="world" />);
    const mark = document.querySelector("mark");
    expect(mark).toBeInTheDocument();
    expect(mark?.textContent).toBe("World");
  });

  it("highlights multiple occurrences", () => {
    render(<HighlightText text="test test test" keyword="test" />);
    const marks = document.querySelectorAll("mark");
    expect(marks.length).toBe(3);
  });

  it("escapes regex special characters in keyword", () => {
    render(<HighlightText text="Hello (World)" keyword="(World)" />);
    const mark = document.querySelector("mark");
    expect(mark).toBeInTheDocument();
    expect(mark?.textContent).toBe("(World)");
  });
});
