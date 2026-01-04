/**
 * RemoveProjectDialog Tests
 * Story 2.19: Task 2.6
 *
 * 测试移除项目确认对话框
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RemoveProjectDialog } from "./RemoveProjectDialog";

// Radix UI PointerEvent polyfill
beforeAll(() => {
  class MockPointerEvent extends MouseEvent {
    constructor(type: string, props: PointerEventInit = {}) {
      super(type, props);
      Object.assign(this, {
        pointerId: props.pointerId ?? 0,
        width: props.width ?? 1,
        height: props.height ?? 1,
        pressure: props.pressure ?? 0,
        tangentialPressure: props.tangentialPressure ?? 0,
        tiltX: props.tiltX ?? 0,
        tiltY: props.tiltY ?? 0,
        twist: props.twist ?? 0,
        pointerType: props.pointerType ?? "mouse",
        isPrimary: props.isPrimary ?? true,
      });
    }
  }
  window.PointerEvent = MockPointerEvent as unknown as typeof PointerEvent;
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn();
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => {
  cleanup();
});

describe("RemoveProjectDialog", () => {
  const defaultProps = {
    isOpen: true,
    onOpenChange: vi.fn(),
    projectName: "test-project",
    onConfirm: vi.fn(),
  };

  it("renders dialog when open (AC13)", async () => {
    render(<RemoveProjectDialog {...defaultProps} />);

    await waitFor(() => {
      expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    });
  });

  it("does not render when closed", () => {
    render(<RemoveProjectDialog {...defaultProps} isOpen={false} />);

    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("shows warning title (AC13)", async () => {
    render(<RemoveProjectDialog {...defaultProps} />);

    await waitFor(() => {
      // i18n key: project.removeFromMantra = "从 Mantra 移除"
      expect(screen.getByText(/从 Mantra 移除/)).toBeInTheDocument();
    });
  });

  it("shows project name in description", async () => {
    render(<RemoveProjectDialog {...defaultProps} projectName="my-project" />);

    await waitFor(() => {
      expect(screen.getByText(/my-project/)).toBeInTheDocument();
    });
  });

  it("explains removal does not affect source project (AC14)", async () => {
    render(<RemoveProjectDialog {...defaultProps} />);

    await waitFor(() => {
      expect(screen.getByText(/不会影响.*原始代码项目/)).toBeInTheDocument();
    });
  });

  it("shows cancel and confirm buttons", async () => {
    render(<RemoveProjectDialog {...defaultProps} />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /取消/ })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /移除项目/ })).toBeInTheDocument();
    });
  });

  it("confirm button has destructive style (AC15)", async () => {
    render(<RemoveProjectDialog {...defaultProps} />);

    await waitFor(() => {
      const confirmButton = screen.getByRole("button", { name: /移除项目/ });
      expect(confirmButton).toHaveClass("bg-destructive");
    });
  });

  it("calls onConfirm when confirm button clicked", async () => {
    const onConfirm = vi.fn();
    const user = userEvent.setup();
    render(<RemoveProjectDialog {...defaultProps} onConfirm={onConfirm} />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /移除项目/ })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: /移除项目/ }));

    expect(onConfirm).toHaveBeenCalled();
  });

  it("calls onOpenChange with false when cancel clicked", async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    render(<RemoveProjectDialog {...defaultProps} onOpenChange={onOpenChange} />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /取消/ })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: /取消/ }));

    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });
});
