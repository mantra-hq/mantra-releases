/**
 * ProjectContextMenu Tests
 * Story 2.19: Task 1.6
 *
 * 测试项目上下文菜单的渲染和交互
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, waitFor, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ProjectContextMenu } from "./ProjectContextMenu";

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

describe("ProjectContextMenu", () => {
  const defaultProps = {
    onSync: vi.fn().mockResolvedValue(undefined),
    onForceSync: vi.fn().mockResolvedValue(undefined),
    onRename: vi.fn(),
    onRemove: vi.fn(),
    onViewInfo: vi.fn(),
  };

  it("renders trigger button", () => {
    render(<ProjectContextMenu {...defaultProps} />);
    expect(screen.getByTestId("project-context-menu-trigger")).toBeInTheDocument();
  });

  it("opens menu on trigger click (AC1)", async () => {
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(screen.getByRole("menu")).toBeInTheDocument();
    });
  });

  it("shows all menu items (AC2)", async () => {
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(screen.getByText("同步更新")).toBeInTheDocument();
      expect(screen.getByText("重命名")).toBeInTheDocument();
      expect(screen.getByText("从 Mantra 移除")).toBeInTheDocument();
    });
  });

  it("shows destructive helper text for remove option (AC3)", async () => {
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(screen.getByText("(不会删除源项目)")).toBeInTheDocument();
    });
  });

  it("calls onSync when sync option clicked", async () => {
    const onSync = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} onSync={onSync} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(screen.getByText("同步更新")).toBeInTheDocument();
    });

    await user.click(screen.getByText("同步更新"));

    await waitFor(() => {
      expect(onSync).toHaveBeenCalled();
    });
  });

  it("calls onRename when rename option clicked", async () => {
    const onRename = vi.fn();
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} onRename={onRename} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(screen.getByText("重命名")).toBeInTheDocument();
    });

    await user.click(screen.getByText("重命名"));

    await waitFor(() => {
      expect(onRename).toHaveBeenCalled();
    });
  });

  it("calls onRemove when remove option clicked", async () => {
    const onRemove = vi.fn();
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} onRemove={onRemove} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(screen.getByText("从 Mantra 移除")).toBeInTheDocument();
    });

    await user.click(screen.getByText("从 Mantra 移除"));

    await waitFor(() => {
      expect(onRemove).toHaveBeenCalled();
    });
  });

  it("calls onOpenChange when menu state changes", async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} onOpenChange={onOpenChange} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(true);
    });
  });

  it("shows sync loading state during sync", async () => {
    let resolveSync: () => void = () => { };
    const onSync = vi.fn().mockImplementation(
      () => new Promise<void>((resolve) => {
        resolveSync = resolve;
      })
    );
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} onSync={onSync} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(screen.getByText("同步更新")).toBeInTheDocument();
    });

    await user.click(screen.getByText("同步更新"));

    // 在同步期间应显示加载状态
    await waitFor(() => {
      expect(screen.getByTestId("sync-loading")).toBeInTheDocument();
    });

    // 完成同步
    await act(async () => {
      resolveSync();
    });
  });

  // Story 2.27 AC1: 测试查看详情点击
  it("calls onViewInfo when view details option clicked (Story 2.27 AC1)", async () => {
    const onViewInfo = vi.fn();
    const user = userEvent.setup();
    render(<ProjectContextMenu {...defaultProps} onViewInfo={onViewInfo} />);

    await user.click(screen.getByTestId("project-context-menu-trigger"));

    await waitFor(() => {
      expect(screen.getByText("查看详情")).toBeInTheDocument();
    });

    await user.click(screen.getByText("查看详情"));

    await waitFor(() => {
      expect(onViewInfo).toHaveBeenCalled();
    });
  });
});
