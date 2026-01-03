/**
 * ProjectRenameInput Tests
 * Story 2.19: Task 5.5
 *
 * 测试项目重命名输入组件
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ProjectRenameInput } from "./ProjectRenameInput";

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
});

afterEach(() => {
  cleanup();
});

describe("ProjectRenameInput", () => {
  const defaultProps = {
    initialName: "test-project",
    onSave: vi.fn(),
    onCancel: vi.fn(),
  };

  it("renders input with initial name (AC10)", () => {
    render(<ProjectRenameInput {...defaultProps} />);

    const input = screen.getByRole("textbox");
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue("test-project");
  });

  it("focuses input on mount", () => {
    render(<ProjectRenameInput {...defaultProps} />);

    const input = screen.getByRole("textbox");
    expect(input).toHaveFocus();
  });

  it("selects all text on mount", () => {
    render(<ProjectRenameInput {...defaultProps} />);

    const input = screen.getByRole("textbox") as HTMLInputElement;
    expect(input.selectionStart).toBe(0);
    expect(input.selectionEnd).toBe("test-project".length);
  });

  it("calls onSave with new name when Enter pressed (AC11)", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();
    render(<ProjectRenameInput {...defaultProps} onSave={onSave} />);

    const input = screen.getByRole("textbox");

    await user.clear(input);
    await user.type(input, "new-name{Enter}");

    expect(onSave).toHaveBeenCalledWith("new-name");
  });

  it("calls onCancel when Escape pressed (AC11)", async () => {
    const onCancel = vi.fn();
    const user = userEvent.setup();
    render(<ProjectRenameInput {...defaultProps} onCancel={onCancel} />);

    const input = screen.getByRole("textbox");

    await user.type(input, "{Escape}");

    expect(onCancel).toHaveBeenCalled();
  });

  it("calls onSave when blur (click outside)", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();
    render(
      <div>
        <ProjectRenameInput {...defaultProps} onSave={onSave} />
        <button>Outside</button>
      </div>
    );

    const input = screen.getByRole("textbox");

    await user.clear(input);
    await user.type(input, "changed-name");
    await user.click(screen.getByText("Outside"));

    expect(onSave).toHaveBeenCalledWith("changed-name");
  });

  it("does not call onSave if name is empty", async () => {
    const onSave = vi.fn();
    const onCancel = vi.fn();
    const user = userEvent.setup();
    render(<ProjectRenameInput {...defaultProps} onSave={onSave} onCancel={onCancel} />);

    const input = screen.getByRole("textbox");

    await user.clear(input);
    await user.type(input, "{Enter}");

    expect(onSave).not.toHaveBeenCalled();
    expect(onCancel).toHaveBeenCalled();
  });

  it("trims whitespace from name", async () => {
    const onSave = vi.fn();
    const user = userEvent.setup();
    render(<ProjectRenameInput {...defaultProps} onSave={onSave} />);

    const input = screen.getByRole("textbox");

    await user.clear(input);
    await user.type(input, "  trimmed-name  {Enter}");

    expect(onSave).toHaveBeenCalledWith("trimmed-name");
  });

  it("does not call onSave if name unchanged", async () => {
    const onSave = vi.fn();
    const onCancel = vi.fn();
    const user = userEvent.setup();
    render(<ProjectRenameInput {...defaultProps} onSave={onSave} onCancel={onCancel} />);

    await user.type(screen.getByRole("textbox"), "{Enter}");

    expect(onSave).not.toHaveBeenCalled();
    expect(onCancel).toHaveBeenCalled();
  });
});
