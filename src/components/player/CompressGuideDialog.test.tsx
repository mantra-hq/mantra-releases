/**
 * CompressGuideDialog Tests - 压缩模式引导弹窗测试
 * Story 10.1: Task 6.3
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { CompressGuideDialog } from "./CompressGuideDialog";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "player.compressGuide.title": "Compress Mode",
        "player.compressGuide.description":
          "Compress mode allows you to optimize session context:",
        "player.compressGuide.feature1": "Mark unwanted messages for deletion",
        "player.compressGuide.feature2": "Edit message content",
        "player.compressGuide.feature3": "Insert new context information",
        "player.compressGuide.feature4":
          "Real-time preview of refined token statistics",
        "player.compressGuide.benefitsTitle": "This helps you:",
        "player.compressGuide.benefit1": "Reduce token usage costs",
        "player.compressGuide.benefit2": "Improve AI response accuracy",
        "player.compressGuide.benefit3":
          "Reduce risk of automatic context compression",
        "player.compressGuide.dontShowAgain": "Don't show this again",
        "player.compressGuide.getStarted": "Get Started",
      };
      return translations[key] || key;
    },
  }),
}));

describe("CompressGuideDialog", () => {
  const defaultProps = {
    open: true,
    onClose: vi.fn(),
    onDismissForever: vi.fn(),
  };

  it("should render dialog when open is true", () => {
    render(<CompressGuideDialog {...defaultProps} />);

    expect(screen.getByText("Compress Mode")).toBeInTheDocument();
    expect(
      screen.getByText("Compress mode allows you to optimize session context:")
    ).toBeInTheDocument();
  });

  it("should not render dialog when open is false", () => {
    render(<CompressGuideDialog {...defaultProps} open={false} />);

    expect(screen.queryByText("Compress Mode")).not.toBeInTheDocument();
  });

  it("should display all feature items", () => {
    render(<CompressGuideDialog {...defaultProps} />);

    expect(
      screen.getByText("Mark unwanted messages for deletion")
    ).toBeInTheDocument();
    expect(screen.getByText("Edit message content")).toBeInTheDocument();
    expect(
      screen.getByText("Insert new context information")
    ).toBeInTheDocument();
    expect(
      screen.getByText("Real-time preview of refined token statistics")
    ).toBeInTheDocument();
  });

  it("should display benefits section", () => {
    render(<CompressGuideDialog {...defaultProps} />);

    expect(screen.getByText("This helps you:")).toBeInTheDocument();
    expect(screen.getByText("Reduce token usage costs")).toBeInTheDocument();
    expect(screen.getByText("Improve AI response accuracy")).toBeInTheDocument();
    expect(
      screen.getByText("Reduce risk of automatic context compression")
    ).toBeInTheDocument();
  });

  it("should display checkbox and get started button", () => {
    render(<CompressGuideDialog {...defaultProps} />);

    expect(screen.getByText("Don't show this again")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Get Started" })
    ).toBeInTheDocument();
  });

  it("should call onClose when Get Started is clicked without checkbox", async () => {
    const onClose = vi.fn();
    const onDismissForever = vi.fn();
    render(
      <CompressGuideDialog
        {...defaultProps}
        onClose={onClose}
        onDismissForever={onDismissForever}
      />
    );

    const button = screen.getByRole("button", { name: "Get Started" });
    fireEvent.click(button);

    await waitFor(() => {
      expect(onClose).toHaveBeenCalled();
      expect(onDismissForever).not.toHaveBeenCalled();
    });
  });

  it("should call onDismissForever when Get Started is clicked with checkbox checked", async () => {
    const onClose = vi.fn();
    const onDismissForever = vi.fn();
    render(
      <CompressGuideDialog
        {...defaultProps}
        onClose={onClose}
        onDismissForever={onDismissForever}
      />
    );

    // Check the checkbox
    const checkbox = screen.getByRole("checkbox");
    fireEvent.click(checkbox);

    // Click Get Started
    const button = screen.getByRole("button", { name: "Get Started" });
    fireEvent.click(button);

    await waitFor(() => {
      expect(onDismissForever).toHaveBeenCalled();
      expect(onClose).not.toHaveBeenCalled();
    });
  });

  it("should toggle checkbox state", () => {
    render(<CompressGuideDialog {...defaultProps} />);

    const checkbox = screen.getByRole("checkbox");

    expect(checkbox).not.toBeChecked();

    fireEvent.click(checkbox);
    expect(checkbox).toBeChecked();

    fireEvent.click(checkbox);
    expect(checkbox).not.toBeChecked();
  });
});
