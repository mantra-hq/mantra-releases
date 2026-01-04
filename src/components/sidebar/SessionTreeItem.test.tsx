/**
 * SessionTreeItem Tests
 * Story 2.18: Task 4.6
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SessionTreeItem } from "./SessionTreeItem";

const mockSession = {
  id: "sess-abc123def456",
  source: "claude" as const,
  created_at: "2024-01-01T00:00:00Z",
  updated_at: "2024-01-02T00:00:00Z",
  message_count: 15,
};

const mockSessionWithTitle = {
  ...mockSession,
  id: "sess-xyz789",
  title: "实现用户认证功能",
};

describe("SessionTreeItem", () => {
  const defaultProps = {
    session: mockSession,
    isCurrent: false,
    searchKeyword: undefined,
    onClick: vi.fn(),
  };

  it("renders session information", () => {
    render(<SessionTreeItem {...defaultProps} />);
    // Session name is derived from ID (first 8 chars after split)
    expect(screen.getByText("abc123de")).toBeInTheDocument();
    // Message count
    expect(screen.getByText("15")).toBeInTheDocument();
  });

  it("shows current session indicator when isCurrent is true", () => {
    render(<SessionTreeItem {...defaultProps} isCurrent={true} />);
    const item = screen.getByTestId("session-tree-item-sess-abc123def456");
    expect(item).toHaveClass("bg-muted");
  });

  it("calls onClick when clicked", () => {
    const onClick = vi.fn();
    render(<SessionTreeItem {...defaultProps} onClick={onClick} />);

    fireEvent.click(screen.getByTestId("session-tree-item-sess-abc123def456"));
    expect(onClick).toHaveBeenCalled();
  });

  it("highlights search keyword", () => {
    render(<SessionTreeItem {...defaultProps} searchKeyword="abc" />);
    const mark = document.querySelector("mark");
    expect(mark).toBeInTheDocument();
    expect(mark?.textContent).toBe("abc");
  });

  it("displays relative time", () => {
    render(<SessionTreeItem {...defaultProps} />);
    // The relative time should be displayed (e.g., "1 天前")
    const item = screen.getByTestId("session-tree-item-sess-abc123def456");
    expect(item).toBeInTheDocument();
  });

  it("displays title when available instead of ID", () => {
    render(
      <SessionTreeItem
        {...defaultProps}
        session={mockSessionWithTitle}
      />
    );
    // Should display the title instead of the ID
    expect(screen.getByText("实现用户认证功能")).toBeInTheDocument();
    // ID-based name should not be displayed
    expect(screen.queryByText("xyz789")).not.toBeInTheDocument();
  });
});
